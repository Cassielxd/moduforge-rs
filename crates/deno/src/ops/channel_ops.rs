//! 通道操作 - 实现 Deno 运行时与 ModuForge 插件系统的异步通信

use deno_core::{op2, OpState, JsValue};
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{DenoError, DenoResult};

/// 请求数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRequest {
    pub request_id: String,
    pub plugin_id: String,
    pub method_name: String,
    pub args: serde_json::Value,
    pub timestamp: u64,
}

/// 响应数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub request_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// 通道管理器 - 管理请求-响应通道
#[derive(Clone)]
pub struct ChannelManager {
    /// 请求发送器 - ModuForge 通过此发送请求到 Deno
    request_sender: mpsc::UnboundedSender<(ChannelRequest, oneshot::Sender<ChannelResponse>)>,

    /// 请求接收器 - Deno 运行时通过此接收请求（需要Arc<Mutex<>>包装）
    /// 注意：这里我们只存储发送器，接收器在 Deno 运行时中持有
}

impl ChannelManager {
    /// 创建新的通道管理器
    pub fn new() -> (Self, mpsc::UnboundedReceiver<(ChannelRequest, oneshot::Sender<ChannelResponse>)>) {
        let (request_sender, request_receiver) = mpsc::unbounded_channel();

        let manager = Self {
            request_sender,
        };

        (manager, request_receiver)
    }

    /// 发送请求并等待响应
    pub async fn send_request(
        &self,
        plugin_id: String,
        method_name: String,
        args: serde_json::Value,
    ) -> DenoResult<ChannelResponse> {
        let request_id = Uuid::new_v4().to_string();
        let (response_sender, response_receiver) = oneshot::channel();

        let request = ChannelRequest {
            request_id: request_id.clone(),
            plugin_id,
            method_name,
            args,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };

        // 发送请求
        self.request_sender.send((request, response_sender))
            .map_err(|_| DenoError::Runtime(anyhow::anyhow!("Failed to send request: channel closed")))?;

        // 等待响应
        response_receiver.await
            .map_err(|_| DenoError::Runtime(anyhow::anyhow!("Failed to receive response: sender dropped")))
    }

    /// 发送请求并设置超时
    pub async fn send_request_with_timeout(
        &self,
        plugin_id: String,
        method_name: String,
        args: serde_json::Value,
        timeout_ms: u64,
    ) -> DenoResult<ChannelResponse> {
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        tokio::time::timeout(
            timeout_duration,
            self.send_request(plugin_id, method_name, args)
        ).await
        .map_err(|_| DenoError::Runtime(anyhow::anyhow!("Request timeout after {}ms", timeout_ms)))?
    }
}

/// 通道状态 - 存储在 OpState 中
pub struct ChannelState {
    /// 当前等待处理的请求
    current_request: Option<(ChannelRequest, oneshot::Sender<ChannelResponse>)>,

    /// 请求接收器
    request_receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<(ChannelRequest, oneshot::Sender<ChannelResponse>)>>>,
}

impl ChannelState {
    pub fn new(request_receiver: mpsc::UnboundedReceiver<(ChannelRequest, oneshot::Sender<ChannelResponse>)>) -> Self {
        Self {
            current_request: None,
            request_receiver: Arc::new(tokio::sync::Mutex::new(request_receiver)),
        }
    }
}

/// Op: 等待下一个请求
/// JavaScript 调用: await Deno.core.ops.op_channel_wait_request()
#[op2(async)]
#[serde]
pub async fn op_channel_wait_request(state: &mut OpState) -> Result<Option<ChannelRequest>, deno_core::error::AnyError> {
    let channel_state = state.try_borrow_mut::<ChannelState>()
        .ok_or_else(|| anyhow::anyhow!("ChannelState not found in OpState"))?;

    // 如果有当前请求，返回错误
    if channel_state.current_request.is_some() {
        return Err(anyhow::anyhow!("Previous request not completed").into());
    }

    let request_receiver = channel_state.request_receiver.clone();
    drop(channel_state); // 释放借用

    // 异步等待请求
    let mut receiver = request_receiver.lock().await;
    if let Some((request, response_sender)) = receiver.recv().await {
        // 重新借用 channel_state 来存储当前请求
        let mut channel_state = state.try_borrow_mut::<ChannelState>()
            .ok_or_else(|| anyhow::anyhow!("ChannelState not found in OpState"))?;

        let request_clone = request.clone();
        channel_state.current_request = Some((request, response_sender));

        Ok(Some(request_clone))
    } else {
        // 通道已关闭
        Ok(None)
    }
}

/// Op: 发送响应
/// JavaScript 调用: Deno.core.ops.op_channel_send_response(response)
#[op2]
#[serde]
pub fn op_channel_send_response(
    state: &mut OpState,
    #[serde] response: ChannelResponse,
) -> Result<(), deno_core::error::AnyError> {
    let mut channel_state = state.try_borrow_mut::<ChannelState>()
        .ok_or_else(|| anyhow::anyhow!("ChannelState not found in OpState"))?;

    if let Some((request, response_sender)) = channel_state.current_request.take() {
        // 验证请求ID匹配
        if request.request_id != response.request_id {
            return Err(anyhow::anyhow!("Request ID mismatch: expected {}, got {}",
                request.request_id, response.request_id).into());
        }

        // 发送响应
        if let Err(_) = response_sender.send(response) {
            tracing::warn!("Failed to send response: receiver dropped");
        }

        Ok(())
    } else {
        Err(anyhow::anyhow!("No current request to respond to").into())
    }
}

/// Op: 获取当前请求信息（用于调试）
/// JavaScript 调用: Deno.core.ops.op_channel_get_current_request()
#[op2]
#[serde]
pub fn op_channel_get_current_request(state: &mut OpState) -> Result<Option<ChannelRequest>, deno_core::error::AnyError> {
    let channel_state = state.try_borrow::<ChannelState>()
        .ok_or_else(|| anyhow::anyhow!("ChannelState not found in OpState"))?;

    Ok(channel_state.current_request.as_ref().map(|(request, _)| request.clone()))
}

/// Op: 发送错误响应的便捷方法
/// JavaScript 调用: Deno.core.ops.op_channel_send_error(request_id, error_message)
#[op2]
pub fn op_channel_send_error(
    state: &mut OpState,
    #[string] request_id: String,
    #[string] error_message: String,
) -> Result<(), deno_core::error::AnyError> {
    let response = ChannelResponse {
        request_id,
        success: false,
        result: None,
        error: Some(error_message),
        execution_time_ms: 0,
    };

    op_channel_send_response(state, response)
}

/// 设置通道状态到 OpState
pub fn set_channel_state_to_opstate(
    op_state: &mut OpState,
    request_receiver: mpsc::UnboundedReceiver<(ChannelRequest, oneshot::Sender<ChannelResponse>)>,
) {
    let channel_state = ChannelState::new(request_receiver);
    op_state.put(channel_state);
}
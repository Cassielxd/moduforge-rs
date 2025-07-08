use std::sync::{Arc, RwLock};
use yrs::{Doc, sync::Awareness};
use yrs_warp::AwarenessRef;

// 协议消息类型常量
pub const MESSAGE_SYNC: u8 = 0;
pub const MESSAGE_AWARENESS: u8 = 1;
pub const MESSAGE_AUTH: u8 = 2;
pub const MESSAGE_QUERY_AWARENESS: u8 = 3;

// 重连超时时间（毫秒）
pub const MESSAGE_RECONNECT_TIMEOUT: u64 = 30000;

// 核心扩展接口 - 只包含需要自定义的部分
pub trait MessageHandlerExt: Send + Sync {
    /// 自定义同步逻辑处理
    fn on_sync_step(
        &self,
        _step: u8,
        _data: &[u8],
        _doc: &Arc<Doc>,
    ) -> SyncHandleResult {
        // 默认实现：使用标准Yjs协议处理
        SyncHandleResult::UseDefault
    }

    /// 自定义连接建立时的初始消息
    fn on_connection_messages(
        &self,
        _doc: &Arc<Doc>,
        _awareness: &AwarenessRef,
    ) -> Vec<Vec<u8>> {
        // 默认实现：返回空，使用标准初始化消息
        Vec::new()
    }

    /// 自定义认证处理
    fn on_auth(
        &self,
        _data: &[u8],
    ) -> Option<Vec<u8>> {
        // 默认实现：不处理认证
        None
    }

    /// 自定义未知消息处理
    fn on_unknown_message(
        &self,
        _msg_type: u8,
        _data: &[u8],
    ) -> Option<Vec<u8>> {
        // 默认实现：不处理未知消息
        None
    }

    /// 连接断开时的自定义处理
    fn on_disconnect(
        &self,
        _doc: &Arc<Doc>,
        _awareness: &AwarenessRef,
    ) {
        // 默认实现：什么都不做
    }
}

// 同步处理结果
#[derive(Debug)]
pub enum SyncHandleResult {
    /// 使用默认的Yjs协议处理
    UseDefault,
    /// 自定义处理，返回响应数据（如果有）
    Custom(Option<Vec<u8>>),
    /// 跳过处理
    Skip,
}

// 完整的消息处理器trait（内部使用）
pub trait MessageHandler: Send + Sync {
    /// 处理同步消息
    fn handle_sync(
        &self,
        step: u8,
        data: &[u8],
        doc: &Arc<Doc>,
    ) -> Option<Vec<u8>>;

    /// 处理awareness消息
    fn handle_awareness(
        &self,
        data: &[u8],
        awareness: &Arc<RwLock<Awareness>>,
    );

    /// 处理认证消息
    fn handle_auth(
        &self,
        data: &[u8],
    ) -> Option<Vec<u8>>;

    /// 处理查询awareness消息
    fn handle_query_awareness(
        &self,
        awareness: &AwarenessRef,
    ) -> Option<Vec<u8>>;

    /// 处理连接建立
    fn on_connected(
        &self,
        doc: &Arc<Doc>,
        awareness: &AwarenessRef,
    ) -> Vec<Vec<u8>>;

    /// 处理连接断开
    fn on_disconnected(
        &self,
        doc: &Arc<Doc>,
        awareness: &AwarenessRef,
    );

    /// 处理未知消息类型
    fn handle_unknown(
        &self,
        msg_type: u8,
        data: &[u8],
    ) -> Option<Vec<u8>>;
}

#[derive(Debug, Clone)]
pub enum ProviderEvent {
    ConnectionClose,
    Status(String),
    ConnectionError(String),
    Sync(bool),
    SyncMessage { step: u8, data_length: usize, preview: String },
    AwarenessMessage { data_length: usize, preview: String },
    AuthMessage { data_length: usize, preview: String },
    MessageHandled { msg_type: u8, success: bool, response_size: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

pub struct WebsocketProviderOptions {
    pub connect: bool,
    pub resync_interval: Option<u64>,
    pub max_backoff_time: u64,
}

impl Default for WebsocketProviderOptions {
    fn default() -> Self {
        Self { connect: true, resync_interval: None, max_backoff_time: 2500 }
    }
}

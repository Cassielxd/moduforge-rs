//! 扩展管理Actor - 基于ractor框架实现
//!
//! 此Actor负责管理扩展和插件系统。

use ractor::{Actor, ActorRef, ActorProcessingErr};
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::{
    debug::debug, error::ForgeResult, extension_manager::ExtensionManager,
    extension::OpFn,
};

use mf_model::schema::Schema;
use mf_state::plugin::Plugin;

use super::ActorSystemResult;

/// 扩展管理消息类型
pub enum ExtensionMessage {
    /// 获取Schema
    GetSchema { reply: oneshot::Sender<Arc<Schema>> },
    /// 获取插件列表
    GetPlugins { reply: oneshot::Sender<Vec<Arc<Plugin>>> },
    /// 获取操作函数列表
    GetOpFns { reply: oneshot::Sender<OpFn> },
    /// 重新加载扩展
    ReloadExtensions { reply: oneshot::Sender<ForgeResult<()>> },
}

// Manual Debug implementation since function pointers don't implement Debug
impl std::fmt::Debug for ExtensionMessage {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ExtensionMessage::GetSchema { .. } => {
                write!(f, "GetSchema {{ .. }}")
            },
            ExtensionMessage::GetPlugins { .. } => {
                write!(f, "GetPlugins {{ .. }}")
            },
            ExtensionMessage::GetOpFns { .. } => write!(f, "GetOpFns {{ .. }}"),
            ExtensionMessage::ReloadExtensions { .. } => {
                write!(f, "ReloadExtensions {{ .. }}")
            },
        }
    }
}

// ExtensionMessage 自动实现 ractor::Message (Debug + Send + 'static)

/// 扩展管理Actor状态
pub struct ExtensionManagerActorState {
    /// 扩展管理器
    extension_manager: ExtensionManager,
}

/// 扩展管理Actor
pub struct ExtensionManagerActor;

#[ractor::async_trait]
impl Actor for ExtensionManagerActor {
    type Msg = ExtensionMessage;
    type State = ExtensionManagerActorState;
    type Arguments = ExtensionManager;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        extension_manager: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        debug!("启动扩展管理Actor");

        Ok(ExtensionManagerActorState { extension_manager })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ExtensionMessage::GetSchema { reply } => {
                let schema = state.extension_manager.get_schema();
                let _ = reply.send(schema);
            },

            ExtensionMessage::GetPlugins { reply } => {
                let plugins = state.extension_manager.get_plugins().clone();
                let _ = reply.send(plugins);
            },

            ExtensionMessage::GetOpFns { reply } => {
                let op_fns = state.extension_manager.get_op_fns().clone();
                let _ = reply.send(op_fns);
            },

            ExtensionMessage::ReloadExtensions { reply } => {
                // 这里可以实现扩展重新加载逻辑
                // 目前先返回成功
                let _ = reply.send(Ok(()));
            },
        }

        Ok(())
    }

    async fn post_stop(
        &self,
        _myself: ActorRef<Self::Msg>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        debug!("停止扩展管理Actor");
        Ok(())
    }
}

/// 扩展管理Actor管理器
pub struct ExtensionManagerActorManager;

impl ExtensionManagerActorManager {
    /// 启动扩展管理Actor
    pub async fn start(
        extension_manager: ExtensionManager
    ) -> ActorSystemResult<ActorRef<ExtensionMessage>> {
        let (actor_ref, _handle) = Actor::spawn(
            Some("ExtensionManagerActor".to_string()),
            ExtensionManagerActor,
            extension_manager,
        )
        .await
        .map_err(|e| super::ActorSystemError::ActorStartupFailed {
            actor_name: "ExtensionManagerActor".to_string(),
            source: e,
        })?;

        debug!("扩展管理Actor启动成功");
        Ok(actor_ref)
    }
}

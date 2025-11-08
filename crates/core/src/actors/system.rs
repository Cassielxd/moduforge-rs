//! ForgeActorSystem - Actor系统管理器
//!
//! 负责协调所有Actor的生命周期和通信。

use ractor::ActorRef;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::{
    config::ForgeConfig,
    debug::debug,
    extension_manager::ExtensionManager,
    history_manager::HistoryManager,
    runtime::sync_flow::FlowEngine,
    types::{RuntimeOptions, HistoryEntryWithMeta},
};

use mf_state::state::State;

use super::{
    event_bus::{EventBusActorManager, EventBusMessage},
    extension_manager::{ExtensionManagerActorManager, ExtensionMessage},
    state_actor::{StateActorManager, StateMessage},
    transaction_processor::{TransactionProcessorManager, TransactionMessage},
    ActorSystemError, ActorSystemResult,
};

/// Actor系统配置
#[derive(Debug, Clone)]
pub struct ActorSystemConfig {
    /// 系统名称
    pub system_name: String,
    /// 是否启用监督
    pub enable_supervision: bool,
    /// Actor关闭超时时间（毫秒）
    pub shutdown_timeout_ms: u64,
    /// 是否启用指标收集
    pub enable_metrics: bool,
}

impl Default for ActorSystemConfig {
    fn default() -> Self {
        Self {
            system_name: "ForgeActorSystem".to_string(),
            enable_supervision: true,
            shutdown_timeout_ms: 5000,
            enable_metrics: true,
        }
    }
}

/// Actor系统句柄
pub struct ForgeActorSystemHandle {
    /// 事务处理Actor
    pub transaction_processor: ActorRef<TransactionMessage>,
    /// 状态管理Actor
    pub state_actor: ActorRef<StateMessage>,
    /// 事件总线Actor
    pub event_bus: ActorRef<EventBusMessage>,
    /// 扩展管理Actor
    pub extension_manager: ActorRef<ExtensionMessage>,
    /// 系统配置
    pub config: ActorSystemConfig,
}

/// Forge Actor系统
pub struct ForgeActorSystem;

impl ForgeActorSystem {
    /// 创建并启动完整的Actor系统
    ///
    /// # 参数
    /// * `runtime_options` - 运行时选项
    /// * `forge_config` - Forge配置
    /// * `system_config` - Actor系统配置
    ///
    /// # 返回值
    /// * `ActorSystemResult<ForgeActorSystemHandle>` - Actor系统句柄
    pub async fn start(
        runtime_options: RuntimeOptions,
        forge_config: ForgeConfig,
        system_config: ActorSystemConfig,
    ) -> ActorSystemResult<ForgeActorSystemHandle> {
        debug!("启动ForgeActorSystem: {}", system_config.system_name);

        // 1. 创建扩展管理器
        let extension_manager =
            Self::create_extension_manager(&runtime_options, &forge_config)?;
        let extension_manager_actor =
            ExtensionManagerActorManager::start(extension_manager).await?;

        // 2. 创建初始状态和历史管理器
        let (initial_state, history_manager) = Self::create_state_and_history(
            &runtime_options,
            &forge_config,
            &extension_manager_actor,
        )
        .await?;

        // 3. 启动状态Actor
        let state_actor =
            StateActorManager::start(initial_state, history_manager).await?;

        // 4. 启动事件总线Actor
        let event_bus =
            EventBusActorManager::start(forge_config.event.clone()).await?;

        // 5. 设置事件处理器
        if !runtime_options.get_event_handlers().is_empty() {
            EventBusActorManager::add_handlers(
                &event_bus,
                runtime_options.get_event_handlers(),
            )
            .await
            .map_err(|e| ActorSystemError::ConfigurationError {
                message: format!("添加事件处理器失败: {e}"),
            })?;
        }

        // 6. 创建流引擎
        let flow_engine = Arc::new(FlowEngine::new().map_err(|e| {
            ActorSystemError::ConfigurationError {
                message: format!("创建流引擎失败: {e}"),
            }
        })?);

        // 7. 启动事务处理Actor
        let transaction_processor = TransactionProcessorManager::start(
            state_actor.clone(),
            event_bus.clone(),
            runtime_options.get_middleware_stack(),
            flow_engine,
            forge_config,
        )
        .await?;

        debug!("ForgeActorSystem启动完成");

        Ok(ForgeActorSystemHandle {
            transaction_processor,
            state_actor,
            event_bus,
            extension_manager: extension_manager_actor,
            config: system_config,
        })
    }

    /// 优雅关闭Actor系统
    ///
    /// # 参数
    /// * `handle` - Actor系统句柄
    ///
    /// # 返回值
    /// * `ActorSystemResult<()>` - 关闭结果
    pub async fn shutdown(
        handle: ForgeActorSystemHandle
    ) -> ActorSystemResult<()> {
        debug!("关闭ForgeActorSystem: {}", handle.config.system_name);

        let shutdown_timeout = tokio::time::Duration::from_millis(
            handle.config.shutdown_timeout_ms,
        );

        // 按依赖关系顺序关闭Actor
        // 1. 首先关闭事务处理器（停止接受新事务）
        let _ = tokio::time::timeout(shutdown_timeout, async {
            handle.transaction_processor.stop(None);
        })
        .await;

        // 2. 关闭事件总线
        let _ = tokio::time::timeout(shutdown_timeout, async {
            handle.event_bus.stop(None);
        })
        .await;

        // 3. 关闭状态Actor
        let _ = tokio::time::timeout(shutdown_timeout, async {
            handle.state_actor.stop(None);
        })
        .await;

        // 4. 最后关闭扩展管理器
        let _ = tokio::time::timeout(shutdown_timeout, async {
            handle.extension_manager.stop(None);
        })
        .await;

        debug!("ForgeActorSystem关闭完成");
        Ok(())
    }

    /// 创建扩展管理器 - 自动处理XML schema配置并合并代码扩展
    fn create_extension_manager(
        runtime_options: &RuntimeOptions,
        forge_config: &ForgeConfig,
    ) -> ActorSystemResult<ExtensionManager> {
        crate::helpers::runtime_common::ExtensionManagerHelper::create_extension_manager(
            runtime_options,
            forge_config,
        )
        .map_err(|e| ActorSystemError::ConfigurationError {
            message: format!("创建扩展管理器失败: {e}"),
        })
    }

    /// 创建初始状态和历史管理器
    async fn create_state_and_history(
        runtime_options: &RuntimeOptions,
        forge_config: &ForgeConfig,
        extension_manager_actor: &ActorRef<ExtensionMessage>,
    ) -> ActorSystemResult<(Arc<State>, HistoryManager<HistoryEntryWithMeta>)>
    {
        // 获取Schema
        let (tx, rx) = oneshot::channel();
        extension_manager_actor
            .send_message(ExtensionMessage::GetSchema { reply: tx })
            .map_err(|e| ActorSystemError::CommunicationFailed {
                message: format!("获取Schema失败: {e}"),
            })?;

        let schema =
            rx.await.map_err(|e| ActorSystemError::CommunicationFailed {
                message: format!("接收Schema失败: {e}"),
            })?;

        // 获取插件
        let (tx, rx) = oneshot::channel();
        extension_manager_actor
            .send_message(ExtensionMessage::GetPlugins { reply: tx })
            .map_err(|e| ActorSystemError::CommunicationFailed {
                message: format!("获取插件失败: {e}"),
            })?;

        let plugins =
            rx.await.map_err(|e| ActorSystemError::CommunicationFailed {
                message: format!("接收插件失败: {e}"),
            })?;
        println!("获取插件: {:?}", plugins.len());
        // 获取操作函数
        let (tx, rx) = oneshot::channel();
        extension_manager_actor
            .send_message(ExtensionMessage::GetOpFns { reply: tx })
            .map_err(|e| ActorSystemError::CommunicationFailed {
                message: format!("获取操作函数失败: {e}"),
            })?;

        let op_fns =
            rx.await.map_err(|e| ActorSystemError::CommunicationFailed {
                message: format!("接收操作函数失败: {e}"),
            })?;

        // 创建全局资源管理器
        let op_state = mf_state::ops::GlobalResourceManager::new();
        for op_fn in &op_fns {
            op_fn(&op_state).map_err(|e| {
                ActorSystemError::ConfigurationError {
                    message: format!("执行操作函数失败: {e}"),
                }
            })?;
        }

        // 创建状态配置
        let mut state_config = mf_state::state::StateConfig {
            schema: Some(schema),
            doc: None,
            stored_marks: None,
            plugins: Some(plugins),
            resource_manager: Some(Arc::new(op_state)),
        };

        // 创建文档
        crate::helpers::create_doc::create_doc(
            &runtime_options.get_content(),
            &mut state_config,
        )
        .await
        .map_err(|e| ActorSystemError::ConfigurationError {
            message: format!("创建文档失败: {e}"),
        })?;

        // 创建状态
        let state = State::create(state_config).await.map_err(|e| {
            ActorSystemError::ConfigurationError {
                message: format!("创建状态失败: {e}"),
            }
        })?;

        let state = Arc::new(state);

        // 创建初始空事务用于历史记录
        let initial_transaction = state.tr();

        // 创建历史管理器
        let history_manager = HistoryManager::with_config(
            HistoryEntryWithMeta::new(
                Arc::new(initial_transaction),
                state.clone(),
                "创建工程项目".to_string(),
                serde_json::Value::Null,
            ),
            forge_config.history.clone(),
        );

        Ok((state, history_manager))
    }
}

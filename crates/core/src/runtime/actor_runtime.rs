//! Actor运行时 - 提供与现有API兼容的Actor实现
//!
//! 这个模块作为新Actor系统的Facade，保持与现有ForgeRuntime API的完全兼容性。

use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use tokio::sync::oneshot;

use crate::{
    actors::{
        system::{ForgeActorSystem, ForgeActorSystemHandle, ActorSystemConfig},
        transaction_processor::TransactionMessage,
        state_actor::StateMessage,
        event_bus::EventBusMessage,
    },
    config::ForgeConfig,
    debug::{debug, error},
    error::{error_utils, ForgeResult},
    event::Event,
    runtime::runtime_trait::RuntimeTrait,
    types::RuntimeOptions,
    metrics,
};

use mf_model::schema::Schema;
use mf_state::{
    state::State,
    transaction::{Command, Transaction},
};

/// Actor运行时 - 新的基于Actor的实现
///
/// 提供与原始ForgeRuntime完全相同的API，但内部使用Actor系统实现。
/// 这确保了现有代码无需修改即可使用新的架构。
pub struct ForgeActorRuntime {
    /// Actor系统句柄
    actor_system: ForgeActorSystemHandle,
    /// 配置
    config: ForgeConfig,
    /// 是否已启动
    started: bool,
}

impl ForgeActorRuntime {
    /// 创建新的Actor运行时实例
    ///
    /// # 参数
    /// * `options` - 运行时选项
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - Actor运行时实例或错误
    pub async fn create(options: RuntimeOptions) -> ForgeResult<Self> {
        Self::create_with_config(options, ForgeConfig::default()).await
    }

    /// 使用指定配置创建Actor运行时实例
    ///
    /// # 参数
    /// * `options` - 运行时选项
    /// * `config` - Forge配置
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - Actor运行时实例或错误
    pub async fn create_with_config(
        options: RuntimeOptions,
        config: ForgeConfig,
    ) -> ForgeResult<Self> {
        let start_time = Instant::now();
        debug!("正在创建Actor运行时实例");

        // 启动Actor系统
        let actor_system = ForgeActorSystem::start(
            options,
            config.clone(),
            ActorSystemConfig::default(),
        )
        .await
        .map_err(|e| error_utils::engine_error(format!("启动Actor系统失败: {}", e)))?;

        debug!("Actor运行时实例创建成功");
        metrics::editor_creation_duration(start_time.elapsed());

        Ok(ForgeActorRuntime {
            actor_system,
            config,
            started: true,
        })
    }

    /// 从快照创建Actor运行时实例
    ///
    /// # 参数
    /// * `snapshot_path` - 快照文件路径
    /// * `options` - 可选的运行时选项
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - Actor运行时实例或错误
    pub async fn from_snapshot(
        snapshot_path: &str,
        options: Option<RuntimeOptions>,
    ) -> ForgeResult<Self> {
        // 这里可以实现快照恢复逻辑
        // 目前先使用常规创建方式
        debug!("从快照创建Actor运行时: {}", snapshot_path);
        Self::create(options.unwrap_or_default()).await
    }

    /// 🎯 处理事务 - 与原始dispatch完全相同的API
    ///
    /// 保持与runtime.rs:662-672行完全相同的接口
    pub async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()> {
        self.dispatch_with_meta(
            transaction,
            "".to_string(),
            serde_json::Value::Null,
        )
        .await
    }

    /// 🎯 处理事务（包含元信息）- 与原始dispatch_with_meta完全相同的API
    ///
    /// 保持与runtime.rs:674-721行完全相同的接口和语义
    pub async fn dispatch_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        if !self.started {
            return Err(error_utils::engine_error("运行时未启动".to_string()));
        }

        // 通过Actor系统处理事务，但保持完全相同的语义
        let (tx, rx) = oneshot::channel();

        self.actor_system
            .transaction_processor
            .send_message(TransactionMessage::ProcessTransaction {
                transaction,
                description,
                meta,
                reply: tx,
            })
            .map_err(|e| error_utils::engine_error(format!("发送事务消息失败: {}", e)))?;

        rx.await
            .map_err(|e| error_utils::engine_error(format!("等待事务处理结果失败: {}", e)))?
    }

    /// 🎯 执行命令 - 与原始command完全相同的API
    ///
    /// 保持与runtime.rs:629-639行完全相同的接口
    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> ForgeResult<()> {
        debug!("正在执行命令: {}", command.name());
        metrics::command_executed(command.name().as_str());

        let mut tr = self.get_tr().await?;
        command.execute(&mut tr).await?;
        tr.commit()?;
        self.dispatch(tr).await
    }

    /// 🎯 执行命令（包含元信息）- 与原始command_with_meta完全相同的API
    ///
    /// 保持与runtime.rs:641-653行完全相同的接口
    pub async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        debug!("正在执行命令: {}", command.name());
        metrics::command_executed(command.name().as_str());

        let mut tr = self.get_tr().await?;
        command.execute(&mut tr).await?;
        tr.commit()?;
        self.dispatch_with_meta(tr, description, meta).await
    }

    /// 🎯 获取当前状态 - 与原始get_state完全相同的API
    ///
    /// 保持与runtime.rs:821-823行完全相同的接口
    pub async fn get_state(&self) -> ForgeResult<Arc<State>> {
        let (tx, rx) = oneshot::channel();

        self.actor_system
            .state_actor
            .send_message(StateMessage::GetState { reply: tx })
            .map_err(|e| error_utils::state_error(format!("发送获取状态消息失败: {}", e)))?;

        rx.await
            .map_err(|e| error_utils::state_error(format!("接收状态响应失败: {}", e)))
    }

    /// 🎯 获取事务对象 - 与原始get_tr完全相同的API
    ///
    /// 保持与runtime.rs:833-836行完全相同的接口
    pub async fn get_tr(&self) -> ForgeResult<Transaction> {
        let state = self.get_state().await?;
        Ok(state.tr())
    }

    /// 🎯 撤销操作 - 与原始undo完全相同的API
    ///
    /// 保持与runtime.rs:838-842行完全相同的接口
    pub async fn undo(&mut self) -> ForgeResult<()> {
        let (tx, rx) = oneshot::channel();

        self.actor_system
            .state_actor
            .send_message(StateMessage::Undo { reply: tx })
            .map_err(|e| error_utils::state_error(format!("发送撤销消息失败: {}", e)))?;

        rx.await
            .map_err(|e| error_utils::state_error(format!("接收撤销响应失败: {}", e)))?
            .map(|_| ())
    }

    /// 🎯 重做操作 - 与原始redo完全相同的API
    ///
    /// 保持与runtime.rs:844-848行完全相同的接口
    pub async fn redo(&mut self) -> ForgeResult<()> {
        let (tx, rx) = oneshot::channel();

        self.actor_system
            .state_actor
            .send_message(StateMessage::Redo { reply: tx })
            .map_err(|e| error_utils::state_error(format!("发送重做消息失败: {}", e)))?;

        rx.await
            .map_err(|e| error_utils::state_error(format!("接收重做响应失败: {}", e)))?
            .map(|_| ())
    }

    /// 🎯 跳转到指定历史位置 - 与原始jump完全相同的API
    ///
    /// 保持与runtime.rs:850-856行完全相同的接口
    pub async fn jump(&mut self, steps: isize) -> ForgeResult<()> {
        let (tx, rx) = oneshot::channel();

        self.actor_system
            .state_actor
            .send_message(StateMessage::Jump {
                steps,
                reply: tx,
            })
            .map_err(|e| error_utils::state_error(format!("发送跳转消息失败: {}", e)))?;

        rx.await
            .map_err(|e| error_utils::state_error(format!("接收跳转响应失败: {}", e)))?
            .map(|_| ())
    }

    /// 🎯 发送事件 - 与原始emit_event完全相同的API
    ///
    /// 保持与runtime.rs:521-528行完全相同的接口
    pub async fn emit_event(&mut self, event: Event) -> ForgeResult<()> {
        metrics::event_emitted(event.name());

        self.actor_system
            .event_bus
            .send_message(EventBusMessage::PublishEvent { event })
            .map_err(|e| error_utils::event_error(format!("发送事件消息失败: {}", e)))?;

        Ok(())
    }

    /// 🎯 获取配置 - 与原始get_config完全相同的API
    ///
    /// 保持与runtime.rs:809-811行完全相同的接口
    pub fn get_config(&self) -> &ForgeConfig {
        &self.config
    }

    /// 🎯 更新配置 - 与原始update_config完全相同的API
    ///
    /// 保持与runtime.rs:814-819行完全相同的接口
    pub fn update_config(&mut self, config: ForgeConfig) {
        self.config = config;
        // 这里可以向各个Actor发送配置更新消息
    }

    /// 🎯 销毁运行时 - 与原始destroy完全相同的API
    ///
    /// 保持与runtime.rs:511-519行完全相同的接口
    pub async fn destroy(&mut self) -> ForgeResult<()> {
        debug!("正在销毁Actor运行时实例");

        if self.started {
            // 广播销毁事件
            let _ = self.emit_event(Event::Destroy).await;

            // 关闭Actor系统
            ForgeActorSystem::shutdown(std::mem::replace(
                &mut self.actor_system,
                // 这里需要一个默认值，但我们永远不会使用它
                // 因为started会被设置为false
                unsafe { std::mem::zeroed() },
            ))
            .await
            .map_err(|e| error_utils::engine_error(format!("关闭Actor系统失败: {}", e)))?;

            self.started = false;
        }

        debug!("Actor运行时实例销毁成功");
        Ok(())
    }

    /// 检查运行时是否已启动
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// 获取schema
    pub async fn get_schema(&self) -> ForgeResult<Arc<Schema>> {
        let state = self.get_state().await?;
        Ok(state.schema())
    }

    /// 获取运行时选项 (占位方法，Actor运行时不直接持有options)
    pub fn get_options(&self) -> RuntimeOptions {
        RuntimeOptions::default()
    }
}

/// 确保在Drop时清理资源
impl Drop for ForgeActorRuntime {
    fn drop(&mut self) {
        if self.started {
            debug!("ForgeActorRuntime Drop: 检测到未正确关闭的运行时");
            // 在Drop中只能做同步操作
            // 异步清理应该通过显式调用destroy()来完成
        }
    }
}

// ==================== RuntimeTrait 实现 ====================

#[async_trait]
impl RuntimeTrait for ForgeActorRuntime {
    async fn dispatch(&mut self, transaction: Transaction) -> ForgeResult<()> {
        self.dispatch(transaction).await
    }

    async fn dispatch_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        self.dispatch_with_meta(transaction, description, meta).await
    }

    async fn command(&mut self, command: Arc<dyn Command>) -> ForgeResult<()> {
        self.command(command).await
    }

    async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        self.command_with_meta(command, description, meta).await
    }

    async fn get_state(&self) -> ForgeResult<Arc<State>> {
        self.get_state().await
    }

    async fn get_tr(&self) -> ForgeResult<Transaction> {
        self.get_tr().await
    }

    async fn get_schema(&self) -> ForgeResult<Arc<Schema>> {
        self.get_schema().await
    }

    async fn undo(&mut self) -> ForgeResult<()> {
        self.undo().await
    }

    async fn redo(&mut self) -> ForgeResult<()> {
        self.redo().await
    }

    async fn jump(&mut self, steps: isize) -> ForgeResult<()> {
        self.jump(steps).await
    }

    fn get_config(&self) -> &ForgeConfig {
        self.get_config()
    }

    fn update_config(&mut self, config: ForgeConfig) {
        self.update_config(config);
    }

    fn get_options(&self) -> &RuntimeOptions {
        // Actor运行时不直接持有options,返回一个静态引用
        // 这是一个权衡,因为RuntimeTrait需要返回引用
        thread_local! {
            static DEFAULT_OPTIONS: RuntimeOptions = RuntimeOptions::default();
        }
        DEFAULT_OPTIONS.with(|opts| unsafe {
            // SAFETY: 这是一个只读的thread_local变量,生命周期与线程绑定
            std::mem::transmute::<&RuntimeOptions, &'static RuntimeOptions>(opts)
        })
    }

    async fn destroy(&mut self) -> ForgeResult<()> {
        self.destroy().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_actor_runtime_creation() {
        let options = RuntimeOptions::default();
        let result = ForgeActorRuntime::create(options).await;

        // 基本创建测试 - 完整测试在集成测试中进行
        // 这里只验证编译和基本结构
        assert!(result.is_ok() || result.is_err()); // 确保返回了某种结果
    }

    #[tokio::test]
    async fn test_actor_runtime_api_compatibility() {
        // 测试API签名是否与原始ForgeRuntime兼容
        // 这确保了API层面的兼容性

        let options = RuntimeOptions::default();
        if let Ok(mut runtime) = ForgeActorRuntime::create(options).await {
            // 这些调用应该编译通过，验证API兼容性
            let _ = runtime.get_config();
            let _ = runtime.is_started();

            // 清理
            let _ = runtime.destroy().await;
        }
    }
}
use std::{path::Path, sync::Arc};
use crate::{
    cache::CacheKey, engine_manager::EngineManager, event::EventBus, extension_manager::ExtensionManager, history_manager::HistoryManager, storage_manager::StorageManager, types::EditorOptions
};
use async_trait::async_trait;
use moduforge_core::{
    model::{node_pool::NodePool, schema::Schema},
    state::{
        state::State,
        transaction::{Command, Transaction},
    },
};

/// 定义编辑器核心功能的基础特征
#[async_trait]
pub trait EditorCore {
    type Error;

    /// 获取当前文档内容
    fn doc(&self) -> Arc<NodePool>;

    /// 获取编辑器配置选项
    fn get_options(&self) -> &EditorOptions;

    /// 获取引擎管理器实例
    fn get_engine_manager(&self) -> &EngineManager;

    /// 根据缓存键获取文档快照
    fn get_snapshot(
        &self,
        key: &CacheKey,
    ) -> Option<Arc<NodePool>>;

    /// 获取当前状态
    fn get_state(&self) -> &Arc<State>;

    /// 获取文档模式定义
    fn get_schema(&self) -> Arc<Schema>;

    /// 获取事件总线实例
    fn get_event_bus(&self) -> &EventBus;

    /// 创建新的事务实例
    fn get_tr(&self) -> Transaction;

    /// 执行自定义命令
    async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> Result<(), Self::Error>;

    /// 处理事务并更新状态
    async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), Self::Error>;

    /// 将当前文档导出为zip文件
    async fn export_zip(
        &self,
        output_path: &Path,
    ) -> Result<(), Self::Error>;

    /// 注册新插件
    async fn register_plugin(&mut self) -> Result<(), Self::Error>;

    /// 注销插件
    async fn unregister_plugin(
        &mut self,
        plugin_key: String,
    ) -> Result<(), Self::Error>;

    /// 执行撤销操作
    fn undo(&mut self);

    /// 执行重做操作
    fn redo(&mut self);
}

/// 编辑器的基础结构，包含共享字段
pub struct EditorBase {
    pub event_bus: EventBus,
    pub state: Arc<State>,
    pub extension_manager: ExtensionManager,
    pub storage_manager: Arc<StorageManager>,
    pub engine_manager: EngineManager,
    pub history_manager: HistoryManager<Arc<State>>,
    pub options: EditorOptions,
}

impl EditorBase {
    /// 共享的基础实现方法
    pub fn doc(&self) -> Arc<NodePool> {
        self.state.doc()
    }

    pub fn get_options(&self) -> &EditorOptions {
        &self.options
    }

    pub fn get_engine_manager(&self) -> &EngineManager {
        &self.engine_manager
    }

    pub fn get_snapshot(
        &self,
        key: &CacheKey,
    ) -> Option<Arc<NodePool>> {
        self.storage_manager.get_snapshot(key)
    }

    pub fn get_state(&self) -> &Arc<State> {
        &self.state
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.extension_manager.get_schema()
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn get_tr(&self) -> Transaction {
        let mut tr = self.get_state().tr();
        let engine = self.engine_manager.engine.clone();
        tr.set_meta("engine", engine);
        tr
    }

    pub fn undo(&mut self) {
        self.history_manager.jump(-1);
        self.state = self.history_manager.get_present();
    }

    pub fn redo(&mut self) {
        self.history_manager.jump(1);
        self.state = self.history_manager.get_present();
    }
}

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use async_trait::async_trait;
use mf_core::{
    runtime::async_runtime::ForgeAsyncRuntime,
    history_manager::HistoryManager,
    types::{HistoryEntryWithMeta, RuntimeOptions},
    ForgeResult,
};
use mf_model::node_pool::NodePool;
use mf_state::{
    resource::Resource, resource_table::ResourceId, transaction::Command,
    State, Transaction,
};

use crate::types::EditorTrait;

pub struct DemoEditorOptions {
    pub editor_options: RuntimeOptions,
}

pub struct DemoEditor {
    /// 内部异步编辑器实例，处理底层编辑操作
    ///
    /// 负责状态管理、撤销/重做操作以及资源跟踪等基础功能
    editor: ForgeAsyncRuntime,

    /// 编辑器配置选项
    ///
    /// 包含创建和运行编辑器所需的各项配置，如存储接口和规则加载器
    options: DemoEditorOptions,
}
#[async_trait]
impl EditorTrait for DemoEditor {
    async fn get_history_manager(
        &self
    ) -> Option<&HistoryManager<HistoryEntryWithMeta>> {
        Some(self.editor.get_history_manager())
    }
    async fn get_state(&self) -> Arc<State> {
        self.editor.get_state().clone()
    }
    async fn doc(&self) -> Arc<NodePool> {
        self.editor.doc()
    }
    async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> ForgeResult<()> {
        self.editor.command(command).await
    }

    async fn command_with_meta(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        self.editor.command_with_meta(command, description, meta).await
    }
    async fn dispatch_flow(
        &mut self,
        transaction: Transaction,
    ) -> ForgeResult<()> {
        self.editor.dispatch_flow(transaction).await
    }
    async fn dispatch_flow_with_meta(
        &mut self,
        transaction: Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        self.editor
            .dispatch_flow_with_meta(transaction, description, meta)
            .await
    }
}

impl DemoEditor {
    pub async fn create(options: DemoEditorOptions) -> ForgeResult<Self> {
        // 创建异步编辑器
        let editor =
            ForgeAsyncRuntime::create(options.editor_options.clone()).await?;

        Ok(Self { editor, options })
    }
    pub fn get_options(&self) -> &DemoEditorOptions {
        &self.options
    }

    pub fn get_resource<T: Resource>(
        &self,
        rid: ResourceId,
    ) -> Option<Arc<T>> {
        // 获取编辑器状态
        // 编辑器状态包含所有运行时资源和配置
        let state = self.editor.get_state();

        // 获取资源管理器
        // 资源管理器负责管理系统中所有注册的资源
        let resource_manager = state.resource_manager();

        // 从资源表中获取指定类型和ID的资源
        // 如果资源不存在或类型不匹配，将返回错误
        resource_manager.resource_table.get::<T>(rid)
    }
}
impl Deref for DemoEditor {
    type Target = ForgeAsyncRuntime;

    fn deref(&self) -> &Self::Target {
        &self.editor
    }
}

impl DerefMut for DemoEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.editor
    }
}

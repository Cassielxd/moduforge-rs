//! 扩展系统的泛型定义
//!
//! 此模块包含 Extension 相关的泛型类型定义。

use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::Deref;

use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::{ops::GlobalResourceManager, plugin::PluginGeneric};

use crate::{types::GlobalAttributeItem, ForgeResult};

/// 操作函数项的内部类型
/// GlobalResourceManager 当前不是泛型的
type OpFnItemInner = Arc<dyn Fn(&GlobalResourceManager) -> ForgeResult<()> + Send + Sync>;

/// 操作函数项（泛型版本）
/// 使用 PhantomData 来携带类型参数，保持 API 的泛型一致性
#[derive(Clone)]
pub struct OpFnItemGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    inner: OpFnItemInner,
    _phantom: PhantomData<(C, S)>,
}

impl<C, S> OpFnItemGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub fn new(f: OpFnItemInner) -> Self {
        Self {
            inner: f,
            _phantom: PhantomData,
        }
    }

    pub fn call(&self, manager: &GlobalResourceManager) -> ForgeResult<()> {
        (self.inner)(manager)
    }
}

/// 实现 Deref 以便可以像函数一样调用
impl<C, S> Deref for OpFnItemGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    type Target = OpFnItemInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// 操作函数列表（泛型版本）
pub type OpFnGeneric<C, S> = Vec<OpFnItemGeneric<C, S>>;

/// 节点转换函数的内部类型
/// 注意：这里的 Node 是 crate::node::Node (schema 构建辅助类型)，不是 mf_model::Node (运行时数据节点)
type NodeTransformFnInner =
    Arc<dyn Fn(&mut crate::node::Node) -> ForgeResult<()> + Send + Sync>;

/// 节点转换函数（泛型版本）
/// 使用 PhantomData 来携带类型参数，保持 API 的泛型一致性
/// 内部函数操作的是 schema 构建时的 Node，不是运行时的数据节点
#[derive(Clone)]
pub struct NodeTransformFnGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    inner: NodeTransformFnInner,
    _phantom: PhantomData<(C, S)>,
}

impl<C, S> NodeTransformFnGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub fn new(f: NodeTransformFnInner) -> Self {
        Self {
            inner: f,
            _phantom: PhantomData,
        }
    }

    pub fn call(&self, node: &mut crate::node::Node) -> ForgeResult<()> {
        (self.inner)(node)
    }

    /// 获取内部函数的引用，用于直接调用
    pub fn inner(&self) -> &NodeTransformFnInner {
        &self.inner
    }
}

/// 实现 Deref 以便可以像函数一样调用
impl<C, S> Deref for NodeTransformFnGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    type Target = NodeTransformFnInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// 扩展实现（泛型版本）
/// 组装全局属性和插件
#[derive(Clone)]
pub struct ExtensionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    global_attributes: Vec<GlobalAttributeItem>,
    plugins: Vec<Arc<PluginGeneric<C, S>>>,
    op_fn: Option<OpFnGeneric<C, S>>,
    node_transform: Option<NodeTransformFnGeneric<C, S>>,
}

impl<C, S> Default for ExtensionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<C, S> ExtensionGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub fn new() -> Self {
        ExtensionGeneric {
            global_attributes: vec![],
            plugins: vec![],
            op_fn: Some(vec![]),
            node_transform: None,
        }
    }

    pub fn add_node_transform(
        &mut self,
        node_fn: NodeTransformFnGeneric<C, S>,
    ) -> &mut Self {
        self.node_transform = Some(node_fn);
        self
    }

    pub fn get_node_transform(&self) -> Option<NodeTransformFnGeneric<C, S>> {
        self.node_transform.clone()
    }

    pub fn add_op_fn(
        &mut self,
        op_fn: OpFnItemGeneric<C, S>,
    ) -> &mut Self {
        self.op_fn.get_or_insert(vec![]).push(op_fn);
        self
    }

    pub fn get_op_fns(&self) -> OpFnGeneric<C, S> {
        self.op_fn.clone().unwrap_or_default()
    }

    pub fn add_global_attribute(
        &mut self,
        item: GlobalAttributeItem,
    ) -> &mut Self {
        self.global_attributes.push(item);
        self
    }

    pub fn get_global_attributes(&self) -> &Vec<GlobalAttributeItem> {
        &self.global_attributes
    }

    pub fn add_plugin(
        &mut self,
        plugin: Arc<PluginGeneric<C, S>>,
    ) -> &mut Self {
        self.plugins.push(plugin);
        self
    }

    pub fn get_plugins(&self) -> &Vec<Arc<PluginGeneric<C, S>>> {
        &self.plugins
    }
}

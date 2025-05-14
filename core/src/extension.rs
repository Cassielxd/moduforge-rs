use std::sync::Arc;

use moduforge_state::{ops::GlobalResourceManager, plugin::Plugin};

use crate::{types::GlobalAttributeItem, EditorResult};
pub type OpFn =
    Vec<Arc<dyn Fn(&GlobalResourceManager) -> EditorResult<()> + Send + Sync>>;
///扩展实现
/// 组装全局属性和插件
#[derive(Clone, Default)]
pub struct Extension {
    global_attributes: Vec<GlobalAttributeItem>,
    plugins: Vec<Arc<Plugin>>,
    op_fn: Option<OpFn>,
}

unsafe impl Send for Extension {}
unsafe impl Sync for Extension {}

impl Extension {
    pub fn new() -> Self {
        Extension {
            global_attributes: vec![],
            plugins: vec![],
            op_fn: Some(vec![]),
        }
    }
    pub fn add_op_fn(
        &mut self,
        op_fn: Arc<
            dyn Fn(&GlobalResourceManager) -> EditorResult<()> + Send + Sync,
        >,
    ) -> &mut Self {
        self.op_fn.get_or_insert(vec![]).push(op_fn);
        self
    }
    pub fn get_op_fns(
        &self
    ) -> Vec<
        Arc<dyn Fn(&GlobalResourceManager) -> EditorResult<()> + Send + Sync>,
    > {
        self.op_fn.clone().unwrap_or(vec![])
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
        plugin: Arc<Plugin>,
    ) -> &mut Self {
        self.plugins.push(plugin);
        self
    }
    pub fn get_plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.plugins
    }
}

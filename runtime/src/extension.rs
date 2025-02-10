use std::sync::Arc;

use moduforge_core::state::plugin::Plugin;

use crate::types::GlobalAttributeItem;

#[derive(Clone, Debug, Default)]
pub struct Extension {
    global_attributes: Vec<GlobalAttributeItem>,
    plugins: Vec<Arc<dyn Plugin>>,
}
impl Extension {
    pub fn new() -> Self {
        Extension {
            global_attributes: vec![],
            plugins: vec![],
        }
    }
    pub fn add_global_attribute(&mut self, item: GlobalAttributeItem) -> &mut Self {
        self.global_attributes.push(item);
        self
    }
    pub fn get_global_attributes(&self) -> &Vec<GlobalAttributeItem> {
        &self.global_attributes
    }
    pub fn add_plugin(&mut self, plugin: Arc<dyn Plugin>) -> &mut Self {
        self.plugins.push(plugin);
        self
    }
    pub fn get_plugins(&self) -> &Vec<Arc<dyn Plugin>> {
        &self.plugins
    }
}

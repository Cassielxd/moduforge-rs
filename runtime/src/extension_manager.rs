use std::{collections::HashMap, sync::Arc};

use moduforge_core::{
    model::schema::Schema,
    state::{plugin::Plugin, transaction::Command},
};

use crate::{
    helpers::get_schema_by_resolved_extensions::get_schema_by_resolved_extensions,
    types::Extensions,
};
/// 扩展管理器
pub struct ExtensionManager {
    plugins: Vec<Arc<Plugin>>,
    extensions: Vec<Extensions>,
    schema: Arc<Schema>,
}
impl ExtensionManager {
    pub fn new(extensions: Vec<Extensions>) -> Self {
        let schema = Arc::new(
            get_schema_by_resolved_extensions(&extensions).unwrap_or_else(|e| {
                panic!("schema 构建失败: {}", e);
            }),
        );
        let mut plugins = vec![];
        for extension in &extensions {
            match extension {
                Extensions::E(extension) => {
                    for plugin in extension.get_plugins() {
                        plugins.push(plugin.clone());
                    }
                }
                _ => {}
            }
        }
        ExtensionManager {
            extensions,
            schema,
            plugins,
        }
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
    pub fn get_plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.plugins
    }
}

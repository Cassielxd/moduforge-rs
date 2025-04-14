use std::sync::Arc;

use moduforge_model::schema::Schema;
use moduforge_state::{ops::OpState, plugin::Plugin};

use crate::{
    helpers::get_schema_by_resolved_extensions::get_schema_by_resolved_extensions,
    types::Extensions, EditorResult,
};
/// 扩展管理器
pub struct ExtensionManager {
    plugins: Vec<Arc<Plugin>>,
    schema: Arc<Schema>,
    op_fns: Vec<Arc<dyn Fn(&mut OpState) -> EditorResult<()>>>,
}
impl ExtensionManager {
    pub fn new(extensions: &Vec<Extensions>) -> Self {
        let schema = Arc::new(
            get_schema_by_resolved_extensions(extensions).unwrap_or_else(|e| {
                panic!("schema 构建失败: {}", e);
            }),
        );
        let mut plugins = vec![];
        let mut op_fns = vec![];
        for extension in extensions {
            if let Extensions::E(extension) = extension {
                for plugin in extension.get_plugins() {
                    plugins.push(plugin.clone());
                }
                for op_fn in extension.get_op_fns() {
                    op_fns.push(op_fn.clone());
                }
            }
        }

        ExtensionManager { schema, plugins, op_fns }
    }
    pub fn get_op_fns(
        &self
    ) -> &Vec<Arc<dyn Fn(&mut OpState) -> EditorResult<()>>> {
        &self.op_fns
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
    pub fn get_plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.plugins
    }
}

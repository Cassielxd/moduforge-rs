use std::sync::Arc;
use std::time::Instant;

use moduforge_model::schema::Schema;
use moduforge_state::{ops::GlobalResourceManager, plugin::Plugin};

use crate::{
    helpers::get_schema_by_resolved_extensions::get_schema_by_resolved_extensions,
    metrics, types::Extensions, ForgeResult,
};
/// 扩展管理器
pub struct ExtensionManager {
    plugins: Vec<Arc<Plugin>>,
    schema: Arc<Schema>,
    op_fns: Vec<
        Arc<dyn Fn(&GlobalResourceManager) -> ForgeResult<()> + Send + Sync>,
    >,
}
impl ExtensionManager {
    pub fn new(extensions: &Vec<Extensions>) -> ForgeResult<Self> {
        let start_time = Instant::now();
        let schema = Arc::new(get_schema_by_resolved_extensions(extensions)?);
        let mut plugins = vec![];
        let mut op_fns = vec![];
        let mut extension_count = 0;
        let mut plugin_count = 0;
        for extension in extensions {
            if let Extensions::E(extension) = extension {
                extension_count += 1;
                for plugin in extension.get_plugins() {
                    plugin_count += 1;
                    plugins.push(plugin.clone());
                }
                for op_fn in extension.get_op_fns() {
                    op_fns.push(op_fn.clone());
                }
            }
        }

        metrics::extensions_loaded(extension_count);
        metrics::plugins_loaded(plugin_count);
        metrics::extension_manager_creation_duration(start_time.elapsed());

        Ok(ExtensionManager { schema, plugins, op_fns })
    }
    pub fn get_op_fns(
        &self
    ) -> &Vec<
        Arc<dyn Fn(&GlobalResourceManager) -> ForgeResult<()> + Send + Sync>,
    > {
        &self.op_fns
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
    pub fn get_plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.plugins
    }
}

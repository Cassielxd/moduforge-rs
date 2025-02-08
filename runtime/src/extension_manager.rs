use std::sync::Arc;

use moduforge_core::model::schema::Schema;

use crate::{
    helpers::get_schema_by_resolved_extensions::get_schema_by_resolved_extensions,
    types::Extensions,
};

pub struct ExtensionManager {
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
        ExtensionManager { extensions, schema }
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
}

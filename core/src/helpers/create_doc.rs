use std::sync::Arc;

use moduforge_model::{node_pool::NodePool, schema::Schema};

use crate::types::Content;

/// 创建文档
pub async fn create_doc(content: &Content) -> Option<Arc<NodePool>> {
    let doc = match content {
        Content::NodePool(node_pool) => Some(Arc::new(node_pool.clone())),
        Content::None => None,
        Content::NodePoolFn(_node_pool_fn_trait) => None,
    };
    if let Some(doc) = &doc {
        if let Err(err) = doc.validate_hierarchy() {
            panic!("{}", err);
        }
    }
    doc
}

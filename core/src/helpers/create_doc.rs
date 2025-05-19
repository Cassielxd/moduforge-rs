use std::sync::Arc;

use moduforge_state::StateConfig;

use crate::types::Content;

/// 创建文档
pub async fn create_doc(
    content: &Content,
    config: &mut StateConfig,
) {
    match content {
        Content::NodePool(node_pool) => {
            config.doc = Some(Arc::new(node_pool.clone()));
        },
        Content::None => {
            config.doc = None;
        },
        Content::NodePoolFn(node_pool_fn_trait) => {
            config.doc =
                Some(Arc::new(node_pool_fn_trait.create(&config).await));
        },
    };
    if let Some(doc) = &config.doc {
        if let Err(err) = doc.validate_hierarchy() {
            panic!("{}", err);
        }
    }
}

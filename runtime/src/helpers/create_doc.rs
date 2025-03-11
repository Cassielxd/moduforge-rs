use std::sync::Arc;

use moduforge_core::model::node_pool::NodePool;
use moduforge_delta::{from_binary, snapshot::FullSnapshot};

use crate::types::Content;

/// 创建文档
pub fn create_doc(content: &Content) -> Option<Arc<NodePool>> {
    let doc = match content {
        Content::NodePoolBinary(items) => {
            if let Ok(node_pool) = from_binary::<NodePool>(items) {
                Some(Arc::new(node_pool))
            } else {
                panic!("NodePoolBinary二进制格式数据异常");
            }
        },
        Content::NodePool(node_pool) => Some(Arc::new(node_pool.clone())),
        Content::Snapshot(items) => {
            if let Ok(full_snapshot) = from_binary::<FullSnapshot>(items) {
                // TODO: 优化 需要判断是否有增量事务 并加载应用
                Some(full_snapshot.node_pool.clone())
            } else {
                panic!("Snapshot二进制格式数据异常");
            }
        },
        Content::None => None,
    };
    if let Some(doc) = &doc {
        if let Err(err) = doc.validate_hierarchy() {
            panic!("{}", err);
        }
    }
    doc
}

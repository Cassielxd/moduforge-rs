use std::sync::Arc;

use crate::{transform_error, TransformResult};

use super::{
    step::{StepGeneric, StepResult},
};

use mf_model::{schema::Schema, tree::Tree, types::NodeId, node_pool::NodePool};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use mf_model::rpds::HashTrieMapSync;

/// 节点属性变更步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttrStep {
    pub id: NodeId,
    pub values: HashTrieMapSync<String, Value>,
}

impl AttrStep {
    pub fn new(
        id: NodeId,
        values: HashTrieMapSync<String, Value>,
    ) -> Self {
        AttrStep { id, values }
    }
}

impl StepGeneric<NodePool, Schema> for AttrStep {
    fn name(&self) -> String {
        "attr_step".to_string()
    }

    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let factory = schema.factory();
        match dart.get_node(&self.id) {
            Some(node) => {
                // 获取节点类型定义，若缺失则返回错误而非 panic
                let node_type = match factory.node_definition(&node.r#type) {
                    Some(nt) => nt,
                    None => {
                        return Err(transform_error(format!(
                            "未知的节点类型: {}",
                            node.r#type
                        )));
                    },
                };
                let attr = &node_type.attrs;
                // 删除 self.values 中 attr中没有定义的属性
                let mut new_values = self.values.clone();
                for (key, _) in self.values.iter() {
                    if !attr.contains_key(key) {
                        new_values.remove_mut(key);
                    }
                }
                let result = dart.attrs(&self.id) + new_values;
                match result {
                    Ok(_) => Ok(StepResult::ok()),
                    Err(e) => Err(transform_error(e.to_string())),
                }
            },
            None => Err(transform_error("节点不存在".to_string())),
        }
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn StepGeneric<NodePool, Schema>>> {
        match dart.get_node(&self.id) {
            Some(node) => {
                // 仅对本次修改过的键生成反向值，避免覆盖无关属性
                let mut revert_values = HashTrieMapSync::new_sync();
                for (changed_key, _) in self.values.iter() {
                    if let Some(old_val) = node.attrs.get_safe(changed_key) {
                        revert_values
                            .insert_mut(changed_key.clone(), old_val.clone());
                    }
                    // 若原先不存在该键，这里不设置（缺少删除语义）；
                    // 如需彻底还原，可扩展支持 unset 语义
                }
                if revert_values.is_empty() {
                    None
                } else {
                    Some(Arc::new(AttrStep::new(
                        self.id.clone(),
                        revert_values,
                    )))
                }
            },
            None => None,
        }
    }
}

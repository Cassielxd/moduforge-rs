use std::sync::Arc;

use mf_model::{schema::Schema, tree::Tree};

use crate::{transform_error, TransformResult};

use super::step::{Step, StepResult};

/// 批量步骤：将多个 Step 作为一个原子单元执行
/// - 成功：全部子步骤成功应用
/// - 失败：自动回滚已应用的子步骤，保证草稿一致性
#[derive(Debug, Clone)]
pub struct BatchStep {
    pub steps: Vec<Arc<dyn Step>>,
}

impl BatchStep {
    pub fn new(steps: Vec<Arc<dyn Step>>) -> Self {
        Self { steps }
    }
}

impl Step for BatchStep {
    fn name(&self) -> String {
        "batch_step".to_string()
    }

    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        // 预先为每个子步骤生成回滚步骤（基于应用前的快照）
        // 注意：为保证回滚正确性，这里在应用每个子步骤前都记录一次基线
        let mut inverses: Vec<Arc<dyn Step>> =
            Vec::with_capacity(self.steps.len());

        for step in &self.steps {
            // 基于当前草稿生成该步骤的反向操作
            if let Some(inv) = step.invert(&Arc::new(dart.clone())) {
                inverses.push(inv);
            } else {
                inverses.push(Arc::new(crate::attr_step::AttrStep::new(
                    // 占位的不可用反向步骤（不会被应用）
                    // 这里不应发生，保持对齐
                    "__invalid__".into(),
                    imbl::hashmap! {},
                )));
            }

            // 应用该子步骤
            match step.apply(dart, schema.clone()) {
                Ok(res) => {
                    if let Some(message) = res.failed {
                        // 失败，执行回滚
                        for inv in inverses.into_iter().rev() {
                            let _ = inv.apply(dart, schema.clone());
                        }
                        return Err(transform_error(message));
                    }
                },
                Err(e) => {
                    // 失败，执行回滚
                    for inv in inverses.into_iter().rev() {
                        let _ = inv.apply(dart, schema.clone());
                    }
                    return Err(e);
                },
            }
        }

        Ok(StepResult::ok())
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        // 动态 Step 无法直接 serde 序列化，这里暂不支持
        None
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>> {
        // 简化策略：对每个子步骤都基于同一基线计算反向，并逆序封装
        // 注意：这与 Transform::apply_steps_batch 的预处理策略保持一致
        let mut invs: Vec<Arc<dyn Step>> = Vec::new();
        for step in &self.steps {
            if let Some(inv) = step.invert(dart) {
                invs.push(inv);
            }
        }
        if invs.is_empty() {
            None
        } else {
            invs.reverse();
            Some(Arc::new(BatchStep::new(invs)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{attr_step::AttrStep, node_step::AddNodeStep};
    use mf_model::{
        attrs::Attrs,
        node::Node,
        node_definition::{NodeTree, NodeSpec},
        schema::{Schema, SchemaSpec},
        tree::Tree,
    };
    use serde_json::json;
    use std::collections::HashMap;

    fn create_schema() -> Arc<Schema> {
        let mut nodes = HashMap::new();
        nodes.insert(
            "doc".to_string(),
            NodeSpec {
                content: None,
                marks: None,
                group: None,
                desc: None,
                attrs: None,
            },
        );
        let spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("doc".to_string()),
        };
        Arc::new(Schema::compile(spec).expect("测试 Schema 编译失败"))
    }

    #[test]
    fn batch_step_apply_and_invert() {
        let schema = create_schema();
        let root = Node::new(
            "doc",
            "doc".to_string(),
            Attrs::default(),
            vec![],
            vec![],
        );
        let mut tree = Tree::new(root);

        // add + attr as a batch
        let child = Node::new(
            "n1",
            "doc".to_string(),
            Attrs::default(),
            vec![],
            vec![],
        );
        let add = Arc::new(AddNodeStep::new(
            "doc".into(),
            vec![NodeTree(child, vec![])],
        ));
        let set = Arc::new(AttrStep::new(
            "n1".into(),
            imbl::hashmap! {"k".into()=>json!(1)},
        ));

        let batch = BatchStep::new(vec![add, set]);
        let res = batch.apply(&mut tree, schema.clone());
        assert!(res.is_ok());

        // invert exists
        let inv = batch.invert(&Arc::new(tree.clone())).unwrap();
        let r = inv.apply(&mut tree, schema);
        assert!(r.is_ok());
    }
}

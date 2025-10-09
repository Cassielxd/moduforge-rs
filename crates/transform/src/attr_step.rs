use std::sync::Arc;

use crate::{transform_error, TransformResult};

use super::{
    step::{Step, StepResult},
};
use imbl::HashMap as ImHashMap;
use mf_model::{schema::Schema, tree::Tree, types::NodeId};
use serde::{Deserialize, Serialize};
use serde_json::{Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttrStep {
    pub id: NodeId,
    pub values: ImHashMap<String, Value>,
}

impl AttrStep {
    pub fn new(
        id: NodeId,
        values: ImHashMap<String, Value>,
    ) -> Self {
        AttrStep { id, values }
    }
}

impl Step for AttrStep {
    fn name(&self) -> String {
        "attr_step".to_string()
    }

    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;
        match dart.get_node(&self.id) {
            Some(node) => {
                // 获取节点类型定义，若缺失则返回错误而非 panic
                let node_type = match schema.nodes.get(&node.r#type) {
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
                        new_values.remove(key);
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
    ) -> Option<Arc<dyn Step>> {
        match dart.get_node(&self.id) {
            Some(node) => {
                // 仅对本次修改过的键生成反向值，避免覆盖无关属性
                let mut revert_values = imbl::hashmap!();
                for (changed_key, _) in self.values.iter() {
                    if let Some(old_val) = node.attrs.get_safe(changed_key) {
                        revert_values
                            .insert(changed_key.clone(), old_val.clone());
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

#[cfg(test)]
mod tests {
    use super::*;
    use mf_model::node::Node;
    use mf_model::attrs::Attrs;
    use mf_model::node_type::NodeSpec;
    use mf_model::schema::{SchemaSpec, AttributeSpec};
    use std::collections::HashMap;
    use std::sync::Arc;
    use serde_json::json;

    fn create_test_node(id: &str) -> Node {
        Node::new(id, "test".to_string(), Attrs::default(), vec![], vec![])
    }

    fn create_test_schema() -> Arc<Schema> {
        let mut nodes = HashMap::new();
        let mut attrs = HashMap::new();
        attrs.insert("name".to_string(), AttributeSpec { default: None });
        attrs.insert("age".to_string(), AttributeSpec { default: None });

        nodes.insert(
            "test".to_string(),
            NodeSpec {
                content: None,
                marks: None,
                group: None,
                desc: Some("Test node".to_string()),
                attrs: Some(attrs),
            },
        );

        let spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("test".to_string()),
        };

        Arc::new(Schema::compile(spec).expect("测试 Schema 编译失败"))
    }

    #[test]
    fn test_attr_step_creation() {
        let mut values = imbl::HashMap::new();
        values.insert("name".to_string(), json!("test"));
        values.insert("age".to_string(), json!(25));

        let step = AttrStep::new("node1".into(), values.clone());
        assert_eq!(step.id, "node1".into());
        assert_eq!(step.values, values);
    }

    #[test]
    fn test_attr_step_apply() {
        // 创建测试节点和树
        let node = create_test_node("node1");
        let mut tree = Tree::new(node);

        // 创建测试 schema
        let schema = create_test_schema();

        // 创建属性步骤
        let mut values = HashMap::new();
        values.insert("name".to_string(), json!("test"));
        values.insert("age".to_string(), json!(25));
        let step = AttrStep::new("node1".into(), values.into());

        // 应用步骤
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());

        // 验证属性是否被正确设置
        let updated_node = tree.get_node(&"node1".into()).unwrap();
        assert_eq!(updated_node.attrs.get("name").unwrap(), &json!("test"));
        assert_eq!(updated_node.attrs.get("age").unwrap(), &json!(25));
    }

    #[test]
    fn test_attr_step_apply_invalid_attrs() {
        // 创建测试节点和树
        let node = create_test_node("node1");
        let mut tree = Tree::new(node);

        // 创建测试 schema
        let schema = create_test_schema();

        // 创建包含无效属性的步骤
        let mut values = HashMap::new();
        values.insert("invalid_attr".to_string(), json!("test"));
        let step = AttrStep::new("node1".into(), values.into());

        // 应用步骤
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());

        // 验证无效属性是否被过滤掉
        let updated_node = tree.get_node(&"node1".into()).unwrap();
        assert!(updated_node.attrs.get("invalid_attr").is_none());
    }

    #[test]
    fn test_attr_step_apply_nonexistent_node() {
        // 创建测试树（不包含目标节点）
        let node: Node = create_test_node("root");
        let mut tree = Tree::new(node);

        // 创建测试 schema
        let schema = create_test_schema();

        // 创建属性步骤
        let mut values = HashMap::new();
        values.insert("name".to_string(), json!("test"));
        let step = AttrStep::new("nonexistent".into(), values.into());

        // 应用步骤
        let result = step.apply(&mut tree, schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_attr_step_serialize() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), json!("test"));
        let step = AttrStep::new("node1".into(), values.into());

        let serialized = Step::serialize(&step);
        assert!(serialized.is_some());

        // 验证序列化后的数据可以反序列化
        let deserialized: AttrStep =
            serde_json::from_slice(&serialized.unwrap()).unwrap();
        assert_eq!(deserialized.id, "node1".into());
        assert_eq!(deserialized.values.get("name").unwrap(), &json!("test"));
    }

    #[test]
    fn test_attr_step_invert() {
        // 创建测试节点和树
        let node = create_test_node("node1");
        let mut tree = Tree::new(node);

        // 创建测试 schema
        let schema = create_test_schema();

        // 设置初始属性
        let mut values = HashMap::new();
        values.insert("name".to_string(), json!("original_name"));
        values.insert("age".to_string(), json!(25));
        let step = AttrStep::new("node1".into(), values.into());
        step.apply(&mut tree, schema.clone()).unwrap();

        // 创建新的属性步骤，修改属性
        let mut new_values = HashMap::new();
        new_values.insert("name".to_string(), json!("modified_name"));
        new_values.insert("age".to_string(), json!(30));
        let new_step = AttrStep::new("node1".into(), new_values.into());

        // 获取反转步骤
        let inverted = new_step.invert(&Arc::new(tree.clone()));
        assert!(inverted.is_some());

        // 应用新步骤
        new_step.apply(&mut tree, schema.clone()).unwrap();
        let node = tree.get_node(&"node1".into()).unwrap();
        assert_eq!(node.attrs.get("name").unwrap(), &json!("modified_name"));
        assert_eq!(node.attrs.get("age").unwrap(), &json!(30));

        // 应用反转步骤
        let inverted_step = inverted.unwrap();
        inverted_step.apply(&mut tree, schema).unwrap();

        // 验证属性是否恢复到原始值
        let node = tree.get_node(&"node1".into()).unwrap();
        assert_eq!(node.attrs.get("name").unwrap(), &json!("original_name"));
        assert_eq!(node.attrs.get("age").unwrap(), &json!(25));
    }
}

use std::sync::Arc;

use crate::{transform_error, TransformResult};

use super::{
    step::{Step, StepResult},
};
use im::HashMap as ImHashMap;
use moduforge_model::{schema::Schema, tree::Tree, types::NodeId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttrStep {
    id: NodeId,
    values: ImHashMap<String, Value>,
}

impl AttrStep {
    pub fn new(
        id: String,
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
                let attr = &schema.nodes.get(&node.r#type).unwrap().attrs;
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
            None => {
                return Err(transform_error("节点不存在".to_string()));
            },
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
                let mut new_values = im::hashmap!();
                for (key, value) in node.attrs.attrs.iter() {
                    new_values.insert(key.clone(), value.clone());
                }
                Some(Arc::new(AttrStep::new(self.id.clone(), new_values)))
            },
            None => {
                return None;
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use moduforge_model::node::Node;
    use moduforge_model::attrs::Attrs;
    use moduforge_model::node_type::NodeSpec;
    use moduforge_model::schema::{SchemaSpec, AttributeSpec};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_test_node(id: &str) -> Node {
        Node::new(
            id,
            "test".to_string(),
            Attrs::default(),
            vec![],
            vec![],
        )
    }

    fn create_test_schema() -> Arc<Schema> {
        let mut nodes = HashMap::new();
        let mut attrs = HashMap::new();
        attrs.insert("name".to_string(), AttributeSpec { default: None });
        attrs.insert("age".to_string(), AttributeSpec { default: None });
        
        nodes.insert("test".to_string(), NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("Test node".to_string()),
            attrs: Some(attrs),
        });

        let spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("test".to_string()),
        };

        Arc::new(Schema::compile(spec).unwrap())
    }

    #[test]
    fn test_attr_step_creation() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), json!("test"));
        values.insert("age".to_string(), json!(25));

        let step = AttrStep::new("node1".to_string(), values.clone().into());
        assert_eq!(step.id, "node1");
        assert_eq!(step.values, values.into());
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
        let step = AttrStep::new("node1".to_string(), values.into());

        // 应用步骤
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());

        // 验证属性是否被正确设置
        let updated_node = tree.get_node(&"node1".to_string()).unwrap();
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
        let step = AttrStep::new("node1".to_string(), values.into());

        // 应用步骤
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());

        // 验证无效属性是否被过滤掉
        let updated_node = tree.get_node(&"node1".to_string()).unwrap();
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
        let step = AttrStep::new("nonexistent".to_string(), values.into());

        // 应用步骤
        let result = step.apply(&mut tree, schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_attr_step_serialize() {
        let mut values = HashMap::new();
        values.insert("name".to_string(), json!("test"));
        let step = AttrStep::new("node1".to_string(), values.into());

        let serialized = Step::serialize(&step);
        assert!(serialized.is_some());
        
        // 验证序列化后的数据可以反序列化
        let deserialized: AttrStep = serde_json::from_slice(&serialized.unwrap()).unwrap();
        assert_eq!(deserialized.id, "node1");
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
        let step = AttrStep::new("node1".to_string(), values.into());
        step.apply(&mut tree, schema.clone()).unwrap();

        // 创建新的属性步骤，修改属性
        let mut new_values = HashMap::new();
        new_values.insert("name".to_string(), json!("modified_name"));
        new_values.insert("age".to_string(), json!(30));
        let new_step = AttrStep::new("node1".to_string(), new_values.into());

        // 获取反转步骤
        let inverted = new_step.invert(&Arc::new(tree.clone()));
        assert!(inverted.is_some());

        // 应用新步骤
        new_step.apply(&mut tree, schema.clone()).unwrap();
        let node = tree.get_node(&"node1".to_string()).unwrap();
        assert_eq!(node.attrs.get("name").unwrap(), &json!("modified_name"));
        assert_eq!(node.attrs.get("age").unwrap(), &json!(30));

        // 应用反转步骤
        let inverted_step = inverted.unwrap();
        inverted_step.apply(&mut tree, schema).unwrap();
        
        // 验证属性是否恢复到原始值
        let node = tree.get_node(&"node1".to_string()).unwrap();
        assert_eq!(node.attrs.get("name").unwrap(), &json!("original_name"));
        assert_eq!(node.attrs.get("age").unwrap(), &json!(25));
    }
}
 
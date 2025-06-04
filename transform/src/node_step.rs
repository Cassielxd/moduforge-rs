use std::sync::Arc;

use moduforge_model::{
    node_type::NodeEnum, schema::Schema, tree::Tree, types::NodeId,
};

use crate::transform_error;

use super::{
    step::{Step, StepResult},
    TransformResult,
};
use serde::{Deserialize, Serialize};
/// 添加节点的步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddNodeStep {
    nodes: NodeEnum,
}
impl AddNodeStep {
    pub fn new(nodes: NodeEnum) -> Self {
        AddNodeStep { nodes }
    }
}
impl Step for AddNodeStep {
    fn name(&self) -> String {
        "add_node_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;
        let result = dart.add(self.nodes.clone());
        match result {
            Ok(_) => Ok(StepResult::ok()),
            Err(e) => Err(transform_error(e.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        _: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>> {
        // 递归收集所有子节点的 id
        fn collect_node_ids(node_enum: &NodeEnum) -> Vec<NodeId> {
            let mut ids: Vec<String> = vec![node_enum.0.id.clone()];
            for child in &node_enum.1 {
                ids.extend(collect_node_ids(child));
            }
            ids
        }
        // 收集所有节点的 id
        let mut node_ids = collect_node_ids(&self.nodes);
        if node_ids.len() > 0 {
            // 排除node_ids 第一个 id
            node_ids.remove(0);
            return Some(Arc::new(RemoveNodeStep::new(
                self.nodes.0.id.clone(),
                node_ids,
            )));
        }
        None
    }
}
/// 删除节点的步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveNodeStep {
    parent_id: NodeId,
    node_ids: Vec<NodeId>,
}
impl RemoveNodeStep {
    pub fn new(
        parent_id: NodeId,
        node_ids: Vec<NodeId>,
    ) -> Self {
        RemoveNodeStep { parent_id, node_ids }
    }
}
impl Step for RemoveNodeStep {
    fn name(&self) -> String {
        "remove_node_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;
        let result = dart.node(&self.parent_id) - self.node_ids.clone();
        match result {
            Ok(_) => Ok(StepResult::ok()),
            Err(e) => Ok(StepResult::fail(e.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>> {
        match dart.all_children(
            &self.parent_id,
            Some(&|node| !self.node_ids.contains(&node.id)),
        ) {
            Some(node_enum) => Some(Arc::new(AddNodeStep::new(node_enum))),
            None => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveNodeStep {
    source_parent_id: NodeId,
    target_parent_id: NodeId,
    node_id: NodeId,
    position: Option<usize>, // 目标位置，None 表示追加到末尾
}

impl MoveNodeStep {
    pub fn new(
        source_parent_id: NodeId,
        target_parent_id: NodeId,
        node_id: NodeId,
        position: Option<usize>,
    ) -> Self {
        MoveNodeStep { source_parent_id, target_parent_id, node_id, position }
    }
}

impl Step for MoveNodeStep {
    fn name(&self) -> String {
        "move_node_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;

        match dart.move_node(
            &self.source_parent_id,
            &self.target_parent_id,
            &self.node_id,
            self.position,
        ) {
            Ok(()) => Ok(StepResult::ok()),
            Err(err) => Err(transform_error(err.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>> {
        match dart.get_node(&self.node_id) {
            Some(_) => Some(Arc::new(MoveNodeStep::new(
                self.target_parent_id.clone(),
                self.source_parent_id.clone(),
                self.node_id.clone(),
                self.position,
            ))),
            None => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use moduforge_model::{
        node::Node,
        node_type::{NodeEnum, NodeType, NodeSpec},
        schema::{Schema, SchemaSpec, AttributeSpec},
        tree::Tree,
        attrs::Attrs,
        mark::Mark,
    };
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
        nodes.insert("test".to_string(), NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("Test node".to_string()),
            attrs: None,
        });

        let spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("test".to_string()),
        };

        Arc::new(Schema::compile(spec).unwrap())
    }

    fn create_test_tree() -> Tree {
        let root = create_test_node("root");
        Tree::new(root)
    }

    #[test]
    fn test_add_node_step() {
        let mut tree = create_test_tree();
        let schema = create_test_schema();
        
        // Create a test node to add
        let node = create_test_node("root");
        let test = create_test_node("test");
        let node_enum = NodeEnum(node, vec![NodeEnum(test, vec![])]);
        let step = AddNodeStep::new(node_enum.clone());
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());
        
        // Verify node was added
        assert!(tree.get_node(&"test".to_string()).is_some());
        
        // Test invert
        let inverted = step.invert(&Arc::new(tree.clone()));
        assert!(inverted.is_some());
        
        // Apply inverted step
        if let Some(inverted_step) = inverted {
            let result = inverted_step.apply(&mut tree, schema);
            assert!(result.is_ok());
            // Verify node was removed
            assert!(tree.get_node(&"test".to_string()).is_none());
        }
    }

    #[test]
    fn test_remove_node_step() {
        let mut tree = create_test_tree();
        let schema = create_test_schema();
        
        // Add a node first
        let node = create_test_node("test");
        tree.add_node(&"root".to_string(), &vec![node]).unwrap();
        
        let step = RemoveNodeStep::new("root".to_string(), vec!["test".to_string()]);
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());
        
        // Verify node was removed
        assert!(tree.get_node(&"test".to_string()).is_none());
        
        // Test invert
        let inverted = step.invert(&Arc::new(tree.clone()));
        assert!(inverted.is_some());
    }

    #[test]
    fn test_move_node_step() {
        let mut tree = create_test_tree();
        let schema = create_test_schema();
        
        // Add source and target nodes
        let source = create_test_node("source");
        let target = create_test_node("target");
        let node = create_test_node("node");
        
        tree.add_node(&"root".to_string(), &vec![source]).unwrap();
        tree.add_node(&"root".to_string(), &vec![target]).unwrap();
        tree.add_node(&"source".to_string(), &vec![node]).unwrap();
        
        let step = MoveNodeStep::new(
            "source".to_string(),
            "target".to_string(),
            "node".to_string(),
            None,
        );
        
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());
        
        // Verify node was moved
        let target_node = tree.get_node(&"target".to_string()).unwrap();
        assert!(target_node.content.contains(&"node".to_string()));
        
        // Test invert
        let inverted = step.invert(&Arc::new(tree.clone()));
        assert!(inverted.is_some());
    }
}

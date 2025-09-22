use std::sync::Arc;

use mf_model::{node_type::NodeEnum, schema::Schema, tree::Tree, types::NodeId};

use crate::transform_error;

use super::{
    step::{Step, StepResult},
    TransformResult,
};
use serde::{Deserialize, Serialize};
/// 添加节点的步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddNodeStep {
    pub parent_id: NodeId,
    pub nodes: Vec<NodeEnum>,
}
impl AddNodeStep {
    pub fn new(
        parent_id: NodeId,
        nodes: Vec<NodeEnum>,
    ) -> Self {
        AddNodeStep { parent_id, nodes }
    }
    // 递归收集单个节点枚举的所有子节点 id
    pub fn collect_node_ids(node_enum: &NodeEnum) -> Vec<NodeId> {
        let mut ids: Vec<NodeId> = vec![node_enum.0.id.clone()];
        for child in &node_enum.1 {
            ids.extend(Self::collect_node_ids(child));
        }
        ids
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
        let result = dart.add(&self.parent_id, self.nodes.clone());
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
        // 收集所有节点的 id（包括顶级节点和所有子节点）
        let mut all_node_ids = Vec::new();
        for node_enum in &self.nodes {
            all_node_ids.extend(Self::collect_node_ids(node_enum));
        }

        if !all_node_ids.is_empty() {
            return Some(Arc::new(RemoveNodeStep::new(
                self.parent_id.clone(),
                all_node_ids,
            )));
        }
        None
    }
}
/// 删除节点的步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveNodeStep {
    pub parent_id: NodeId,
    pub node_ids: Vec<NodeId>,
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
        let result = dart.remove_node(&self.parent_id, self.node_ids.clone());
        match result {
            Ok(_) => Ok(StepResult::ok()),
            Err(e) => Err(transform_error(e.to_string())), // 修复：失败时返回Err保持一致性
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        dart: &Arc<Tree>,
    ) -> Option<Arc<dyn Step>> {
        // 收集所有要删除的节点及其子树
        let mut nodes_to_restore = Vec::new();

        for node_id in &self.node_ids {
            if let Some(node_enum) = dart.all_children(node_id, None) {
                nodes_to_restore.push(node_enum);
            }
        }

        if !nodes_to_restore.is_empty() {
            Some(Arc::new(AddNodeStep::new(
                self.parent_id.clone(),
                nodes_to_restore,
            )))
        } else {
            None
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
        match dart.get_parent_node(&self.node_id) {
            Some(source_parent) => {
                // 反向时需要把节点放回原父节点的原索引
                let original_index = source_parent
                    .content
                    .iter()
                    .position(|id| id == &self.node_id);
                let original_pos = original_index; // Option<usize>
                Some(Arc::new(MoveNodeStep::new(
                    self.target_parent_id.clone(),
                    self.source_parent_id.clone(),
                    self.node_id.clone(),
                    original_pos,
                )))
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mf_model::{
        node::Node,
        node_type::{NodeEnum, NodeSpec},
        schema::{Schema, SchemaSpec},
        tree::Tree,
        attrs::Attrs,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_test_node(id: &str) -> Node {
        Node::new(id, "test".to_string(), Attrs::default(), vec![], vec![])
    }

    fn create_test_schema() -> Arc<Schema> {
        let mut nodes = HashMap::new();
        nodes.insert(
            "test".to_string(),
            NodeSpec {
                content: None,
                marks: None,
                group: None,
                desc: Some("Test node".to_string()),
                attrs: None,
            },
        );

        let spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("test".to_string()),
        };

        Arc::new(Schema::compile(spec).expect("测试 Schema 编译失败"))
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
        let node = create_test_node("child");
        let test = create_test_node("test");
        let node_enum = NodeEnum(node, vec![NodeEnum(test, vec![])]);
        let step = AddNodeStep::new("root".into(), vec![node_enum.clone()]);
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());

        // Verify node was added
        assert!(tree.get_node(&"test".into()).is_some());

        // Test invert
        let inverted = step.invert(&Arc::new(tree.clone()));
        assert!(inverted.is_some());

        // Apply inverted step
        if let Some(inverted_step) = inverted {
            let result = inverted_step.apply(&mut tree, schema);
            if result.is_err() {
                eprintln!("Invert step failed: {:?}", result);
            }
            assert!(result.is_ok());
            // Verify node was removed
            assert!(tree.get_node(&"test".into()).is_none());
        }
    }

    #[test]
    fn test_remove_node_step() {
        let mut tree = create_test_tree();
        let schema = create_test_schema();

        // Add a node first
        let node = create_test_node("test");
        tree.add_node(&"root".into(), &vec![node])
            .expect("测试中添加节点应该成功");

        let step = RemoveNodeStep::new("root".into(), vec!["test".into()]);
        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());

        // Verify node was removed
        assert!(tree.get_node(&"test".into()).is_none());

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

        tree.add_node(&"root".into(), &vec![source])
            .expect("测试中添加源节点应该成功");
        tree.add_node(&"root".into(), &vec![target])
            .expect("测试中添加目标节点应该成功");
        tree.add_node(&"source".into(), &vec![node])
            .expect("测试中添加子节点应该成功");

        let step = MoveNodeStep::new(
            "source".into(),
            "target".into(),
            "node".into(),
            None,
        );

        let result = step.apply(&mut tree, schema.clone());
        assert!(result.is_ok());

        // Verify node was moved
        let target_node =
            tree.get_node(&"target".into()).expect("目标节点应该存在");
        assert!(target_node.content.contains(&"node".into()));

        // Test invert
        let inverted = step.invert(&Arc::new(tree.clone()));
        assert!(inverted.is_some());
    }
}

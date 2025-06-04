use std::sync::Arc;

use moduforge_model::{
    node::Node, node_type::NodeEnum, schema::Schema, tree::Tree, types::NodeId,
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
        let _ = dart.node("key") + self.nodes.clone();
        Ok(StepResult::ok())
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

/// 替换节点
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplaceNodeStep {
    node_id: NodeId,
    nodes: Vec<Node>,
}
impl ReplaceNodeStep {
    pub fn new(
        node_id: NodeId,
        nodes: Vec<Node>,
    ) -> Self {
        ReplaceNodeStep { node_id, nodes }
    }
}
impl Step for ReplaceNodeStep {
    fn name(&self) -> String {
        "replace_node_step".to_string()
    }
    fn apply(
        &self,
        dart: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        let _ = schema;

        match dart.replace_node(self.node_id.clone(), &self.nodes) {
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
            Some(node) => {
                //拿到所有替换之前的节点
                let mut new_nodes = Vec::new();
                for node_id in node.content.iter() {
                    let node = dart.get_node(node_id).unwrap();
                    new_nodes.push(node.as_ref().clone());
                }
                Some(Arc::new(ReplaceNodeStep::new(node.id.clone(), new_nodes)))
            },
            None => None,
        }
    }
}

use std::sync::Arc;

use moduforge_model::{node::Node, schema::Schema, tree::Tree, types::NodeId};

use super::{
    step::{Step, StepResult},
    transform::TransformError,
};
use serde::{Deserialize, Serialize};
/// 添加节点的步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddNodeStep {
    parent_id: NodeId,
    nodes: Vec<Node>,
}
impl AddNodeStep {
    pub fn new(
        parent_id: NodeId,
        nodes: Vec<Node>,
    ) -> Self {
        AddNodeStep { parent_id, nodes }
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
    ) -> Result<StepResult, TransformError> {
        let _ = schema;
        dart.node(&self.parent_id)+self.nodes.clone();
        Ok(StepResult::ok())
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
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
    ) -> Result<StepResult, TransformError> {
        let _ = schema;
        let result = dart.node(&self.parent_id)-self.node_ids.clone();
        match result {
            Ok(_) => Ok(StepResult::ok()),
            Err(e) => Ok(StepResult::fail(e.to_string())),
           }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
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
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.move_node(
            &self.source_parent_id,
            &self.target_parent_id,
            &self.node_id,
            self.position,
        ) {
            Ok(()) => Ok(StepResult::ok()),    
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
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
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.replace_node(self.node_id.clone(), &self.nodes) {
            Ok(()) => Ok(StepResult::ok()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }
    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }
}

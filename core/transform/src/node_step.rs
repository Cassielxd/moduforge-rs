use std::sync::Arc;

use moduforge_model::{node::Node, schema::Schema, types::NodeId};
use crate::draft::Draft;

use super::{
    ConcreteStep,
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
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.add_node(&self.parent_id, &self.nodes) {
            Ok(()) => Ok(dart.commit()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::AddNodeStep(self.clone())
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
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.remove_node(&self.parent_id, self.node_ids.clone()) {
            Ok(()) => Ok(dart.commit()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::RemoveNodeStep(self.clone())
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
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.move_node(
            &self.source_parent_id,
            &self.target_parent_id,
            &self.node_id,
            self.position,
        ) {
            Ok(()) => Ok(dart.commit()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::MoveNodeStep(self.clone())
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
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        let _ = schema;

        match dart.replace_node(self.node_id.clone(), &self.nodes) {
            Ok(()) => Ok(dart.commit()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> super::ConcreteStep {
        ConcreteStep::ReplaceNodeStep(self.clone())
    }
}

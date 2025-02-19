use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::{mark::Mark, node::Node, types::NodeId};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub enum Patch {
    UpdateAttr {
        path: Vec<String>,
        id: NodeId,
        old: HashMap<String, String>,
        new: HashMap<String, String>,
    },
    AddNode {
        path: Vec<String>,
        parent_id: NodeId,
        node: Arc<Node>,
    },
    AddMark {
        path: Vec<String>,
        node_id: NodeId,
        mark: Mark,
    },
    RemoveMark {
        path: Vec<String>,
        parent_id: NodeId,
        marks: Vec<Arc<Mark>>,
    },
    RemoveNode {
        path: Vec<String>,
        parent_id: NodeId,
        nodes: Vec<Arc<Node>>,
    },
}

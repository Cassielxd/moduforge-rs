use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::{mark::Mark, node::Node, types::NodeId};
use std::{collections::HashMap, sync::Arc};

/// 文档补丁枚举
/// 用于描述对文档树的各种修改操作
#[derive(Debug, Clone, Serialize, Deserialize, Decode, Encode)]
pub enum Patch {
  /// 更新节点属性
  UpdateAttr {
    /// 节点路径，用于定位节点位置
    path: Vec<String>,
    /// 目标节点的唯一标识符
    id: NodeId,
    /// 更新前的属性映射
    old: HashMap<String, String>,
    /// 更新后的属性映射
    new: HashMap<String, String>,
  },
  /// 添加新节点
  AddNode {
    /// 新节点的路径
    path: Vec<String>,
    /// 父节点的唯一标识符
    parent_id: NodeId,
    /// 要添加的节点
    node: Arc<Node>,
  },
  /// 添加标记
  AddMark {
    /// 目标节点的路径
    path: Vec<String>,
    /// 目标节点的唯一标识符
    node_id: NodeId,
    /// 要添加的标记
    mark: Mark,
  },
  /// 移除标记
  RemoveMark {
    /// 目标节点的路径
    path: Vec<String>,
    /// 父节点的唯一标识符
    parent_id: NodeId,
    /// 要移除的标记列表
    marks: Vec<Arc<Mark>>,
  },
  /// 移除节点
  RemoveNode {
    /// 目标节点的路径
    path: Vec<String>,
    /// 父节点的唯一标识符
    parent_id: NodeId,
    /// 要移除的节点列表
    nodes: Vec<Arc<Node>>,
  },
}

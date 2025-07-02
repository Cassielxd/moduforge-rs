use serde::{Deserialize, Serialize};

use mf_model::{attrs::Attrs, mark::Mark, node::Node, types::NodeId};

/// 文档补丁枚举
/// 用于描述对文档树的各种修改操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Patch {
    /// 更新节点属性
    UpdateAttr {
        /// 节点路径，用于定位节点位置
        path: Vec<String>,
        /// 目标节点的唯一标识符
        id: NodeId,
        /// 更新前的属性映射
        old: Attrs,
        /// 更新后的属性映射
        new: Attrs,
    },
    /// 添加新节点
    AddNode {
        /// 新节点的路径
        path: Vec<String>,
        /// 父节点的唯一标识符
        parent_id: NodeId,
        /// 要添加的节点
        nodes: Vec<Node>,
    },
    /// 添加标记
    AddMark {
        /// 目标节点的路径
        path: Vec<String>,
        /// 目标节点的唯一标识符
        node_id: NodeId,
        /// 要添加的标记
        marks: Vec<Mark>,
    },
    /// 移除标记
    RemoveMark {
        /// 目标节点的路径
        path: Vec<String>,
        /// 父节点的唯一标识符
        parent_id: NodeId,
        /// 要移除的标记列表
        marks: Vec<Mark>,
    },
    /// 移除节点
    RemoveNode {
        /// 目标节点的路径
        path: Vec<String>,
        /// 父节点的唯一标识符
        parent_id: NodeId,
        /// 要移除的节点列表
        nodes: Vec<Node>,
    },
    /// 移动节点
    MoveNode {
        path: Vec<String>,
        node_id: NodeId,
        source_parent_id: NodeId,
        target_parent_id: NodeId,
        position: Option<usize>,
    },
    /// 排序子节点
    SortChildren {
        /// 目标节点的路径
        path: Vec<String>,
        /// 父节点的唯一标识符
        parent_id: NodeId,
        /// 排序前的子节点列表
        old_children: Vec<NodeId>,
        /// 排序后的子节点列表
        new_children: Vec<NodeId>,
    },
}

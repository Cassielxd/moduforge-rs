use super::{error::PoolError, node::Node, types::NodeId};
use im::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
/// 节点池内部数据结构，实现结构共享和高效克隆
///
/// # 字段
///
/// * `root_id` - 根节点标识符
/// * `nodes` - 节点存储的不可变哈希表（使用结构共享）
/// * `parent_map` - 父子关系映射表
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NodePoolInner {
    pub root_id: NodeId,
    pub nodes: im::HashMap<NodeId, Arc<Node>>, // 节点数据共享
    pub parent_map: im::HashMap<NodeId, NodeId>,
}
impl NodePoolInner {
    /// 更新节点属性（创建新版本的数据结构）
    ///
    /// # 参数
    ///
    /// * `id` - 目标节点ID
    /// * `values` - 要更新的属性键值对
    ///
    /// # 返回值
    ///
    /// 返回包含新节点属性的新版本 `NodePoolInner`
    ///
    /// # 错误
    ///
    /// 当节点不存在时返回 [`PoolError::NodeNotFound`]
    pub fn update_attr(
        &self,
        id: &NodeId,
        values: &HashMap<String, String>,
    ) -> Result<Self, PoolError> {
        if !self.nodes.contains_key(id) {
            return Err(PoolError::NodeNotFound(id.clone()));
        }
        let node = self.nodes.get(id).unwrap();

        let mut cope_node = node.clone().as_ref().clone();
        cope_node.attrs.extend(values.clone());
        let nodes = self.nodes.update(id.clone(), Arc::new(cope_node));
        Ok(NodePoolInner {
            nodes,
            parent_map: self.parent_map.clone(),
            root_id: self.root_id.clone(),
        })
    }
}
/// 线程安全的节点池封装
///
/// 使用 [`Arc`] 实现快速克隆，内部使用不可变数据结构保证线程安全
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NodePool {
    // 使用 Arc 包裹内部结构，实现快速克隆
    pub inner: Arc<NodePoolInner>,
}
unsafe impl Send for NodePool {}
unsafe impl Sync for NodePool {}

impl NodePool {
    /// 获取节点池中节点总数
    pub fn size(&self) -> usize {
        self.inner.nodes.len()
    }

    /// 从节点列表构建节点池
    ///
    /// # 参数
    ///
    /// * `nodes` - 初始节点列表
    /// * `root_id` - 指定根节点ID
    ///
    /// # 注意
    ///
    /// 会自动构建父子关系映射表
    pub fn from(
        nodes: Vec<Node>,
        root_id: NodeId,
    ) -> Self {
        let mut nodes_ref = HashMap::new();
        let mut parent_map_ref = HashMap::new();
        for node in nodes.into_iter() {
            for child_id in &node.content {
                parent_map_ref.insert(child_id.clone(), node.id.clone());
            }
            nodes_ref.insert(node.id.clone(), Arc::new(node));
        }

        NodePool {
            inner: Arc::new(NodePoolInner {
                nodes: nodes_ref,
                parent_map: parent_map_ref,
                root_id,
            }),
        }
    }

    // -- 核心查询方法 --

    /// 根据ID获取节点(immutable)
    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<&Arc<Node>> {
        self.inner.nodes.get(id)
    }

    /// 检查节点是否存在
    pub fn contains_node(
        &self,
        id: &NodeId,
    ) -> bool {
        self.inner.nodes.contains_key(id)
    }

    // -- 层级关系操作 --

    /// 获取直接子节点列表
    pub fn children(
        &self,
        parent_id: &NodeId,
    ) -> Option<&im::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| &n.content)
    }

    /// 递归获取所有子节点（深度优先）
    pub fn descendants(
        &self,
        parent_id: &NodeId,
    ) -> Vec<&Node> {
        let mut result: Vec<&Node> = Vec::new();
        self._collect_descendants(parent_id, &mut result);
        result
    }

    fn _collect_descendants<'a>(
        &'a self,
        parent_id: &NodeId,
        result: &mut Vec<&'a Node>,
    ) {
        if let Some(children) = self.children(parent_id) {
            for child_id in children {
                if let Some(child) = self.get_node(child_id) {
                    result.push(child);
                    self._collect_descendants(child_id, result);
                }
            }
        }
    }

    /// 获取父节点ID
    pub fn parent_id(
        &self,
        child_id: &NodeId,
    ) -> Option<&NodeId> {
        self.inner.parent_map.get(child_id)
    }

    /// 获取完整祖先链
    pub fn ancestors(
        &self,
        child_id: &NodeId,
    ) -> Vec<&Arc<Node>> {
        let mut chain = Vec::new();
        let mut current_id = child_id;
        while let Some(parent_id) = self.parent_id(current_id) {
            if let Some(parent) = self.get_node(parent_id) {
                chain.push(parent);
                current_id = parent_id;
            } else {
                break;
            }
        }
        chain
    }

    /// 验证父子关系一致性
    pub fn validate_hierarchy(&self) -> Result<(), PoolError> {
        for (child_id, parent_id) in &self.inner.parent_map {
            // 验证父节点存在
            if !self.contains_node(parent_id) {
                return Err(PoolError::OrphanNode(child_id.clone()));
            }

            // 验证父节点确实包含该子节点
            if let Some(children) = self.children(parent_id) {
                if !children.contains(child_id) {
                    return Err(PoolError::InvalidParenting {
                        child: child_id.clone(),
                        alleged_parent: parent_id.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    // -- 高级查询 --
    /// 根据类型筛选节点
    pub fn filter_nodes<P>(
        &self,
        predicate: P,
    ) -> Vec<&Arc<Node>>
    where
        P: Fn(&Node) -> bool,
    {
        self.inner.nodes.values().filter(|n| predicate(n)).collect()
    }
    /// 查找第一个匹配节点
    pub fn find_node<P>(
        &self,
        predicate: P,
    ) -> Option<&Arc<Node>>
    where
        P: Fn(&Node) -> bool,
    {
        self.inner.nodes.values().find(|n| predicate(n))
    }

    /// 获取节点在树中的深度
    ///
    /// # 参数
    ///
    /// * `node_id` - 目标节点ID
    ///
    /// # 返回值
    ///
    /// 返回节点的深度，根节点深度为0
    pub fn get_node_depth(
        &self,
        node_id: &NodeId,
    ) -> Option<usize> {
        let mut depth = 0;
        let mut current_id = node_id;

        while let Some(parent_id) = self.parent_id(current_id) {
            depth += 1;
            current_id = parent_id;
        }

        Some(depth)
    }

    /// 获取从根节点到目标节点的完整路径
    ///
    /// # 参数
    ///
    /// * `node_id` - 目标节点ID
    ///
    /// # 返回值
    ///
    /// 返回从根节点到目标节点的节点ID路径
    pub fn get_node_path(
        &self,
        node_id: &NodeId,
    ) -> Vec<NodeId> {
        let mut path = Vec::new();
        let mut current_id = node_id;

        while let Some(parent_id) = self.parent_id(current_id) {
            path.push(current_id.clone());
            current_id = parent_id;
        }
        path.push(current_id.clone());
        path.reverse();

        path
    }

    /// 检查节点是否为叶子节点
    ///
    /// # 参数
    ///
    /// * `node_id` - 目标节点ID
    ///
    /// # 返回值
    ///
    /// 如果节点不存在或没有子节点则返回 true
    pub fn is_leaf(
        &self,
        node_id: &NodeId,
    ) -> bool {
        if let Some(children) = self.children(node_id) {
            children.is_empty()
        } else {
            true
        }
    }

    /// 获取节点的同级节点（具有相同父节点的节点）
    ///
    /// # 参数
    ///
    /// * `node_id` - 目标节点ID
    ///
    /// # 返回值
    ///
    /// 返回同级节点的ID列表
    pub fn get_siblings(
        &self,
        node_id: &NodeId,
    ) -> Vec<NodeId> {
        if let Some(parent_id) = self.parent_id(node_id) {
            if let Some(children) = self.children(parent_id) {
                return children
                    .iter()
                    .filter(|&id| id != node_id)
                    .cloned()
                    .collect();
            }
        }
        Vec::new()
    }

    /// 获取节点的所有兄弟节点（包括自身）
    ///
    /// # 参数
    ///
    /// * `node_id` - 目标节点ID
    ///
    /// # 返回值
    ///
    /// 返回所有兄弟节点的ID列表（包括自身）
    pub fn get_all_siblings(
        &self,
        node_id: &NodeId,
    ) -> Vec<NodeId> {
        if let Some(parent_id) = self.parent_id(node_id) {
            if let Some(children) = self.children(parent_id) {
                return children.iter().cloned().collect();
            }
        }
        Vec::new()
    }

    /// 获取节点的子树大小（包括自身和所有子节点）
    ///
    /// # 参数
    ///
    /// * `node_id` - 目标节点ID
    ///
    /// # 返回值
    ///
    /// 返回子树中的节点总数
    pub fn get_subtree_size(
        &self,
        node_id: &NodeId,
    ) -> usize {
        let mut size = 1; // 包含自身
        if let Some(children) = self.children(node_id) {
            for child_id in children {
                size += self.get_subtree_size(child_id);
            }
        }
        size
    }

    /// 检查一个节点是否是另一个节点的祖先
    ///
    /// # 参数
    ///
    /// * `ancestor_id` - 可能的祖先节点ID
    /// * `descendant_id` - 可能的后代节点ID
    ///
    /// # 返回值
    ///
    /// 如果 ancestor_id 是 descendant_id 的祖先则返回 true
    pub fn is_ancestor(
        &self,
        ancestor_id: &NodeId,
        descendant_id: &NodeId,
    ) -> bool {
        let mut current_id = descendant_id;
        while let Some(parent_id) = self.parent_id(current_id) {
            if parent_id == ancestor_id {
                return true;
            }
            current_id = parent_id;
        }
        false
    }

    /// 获取两个节点的最近公共祖先
    ///
    /// # 参数
    ///
    /// * `node1_id` - 第一个节点ID
    /// * `node2_id` - 第二个节点ID
    ///
    /// # 返回值
    ///
    /// 返回两个节点的最近公共祖先ID
    pub fn get_lowest_common_ancestor(
        &self,
        node1_id: &NodeId,
        node2_id: &NodeId,
    ) -> Option<NodeId> {
        let path1 = self.get_node_path(node1_id);
        let path2 = self.get_node_path(node2_id);

        for ancestor_id in path1.iter().rev() {
            if path2.contains(ancestor_id) {
                return Some(ancestor_id.clone());
            }
        }
        None
    }

    
}

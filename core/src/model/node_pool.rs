use std::sync::Arc;

use bincode::{Decode, Encode};
use im::HashMap;
use serde::{Deserialize, Serialize};

use super::{error::PoolError, node::Node, types::NodeId};
#[derive(Debug, PartialEq, Default,Encode, Decode,Serialize, Deserialize)]
pub struct NodePoolInner {
    #[bincode(with_serde)] nodes: im::HashMap<NodeId, Arc<Node>>,  // 节点数据共享
    #[bincode(with_serde)] parent_map: im::HashMap<NodeId, NodeId>, 
    #[bincode(with_serde)] child_map: im::HashMap<NodeId, im::Vector<NodeId>>, // 新增反向索引
}
impl NodePoolInner {
    pub fn link_child_to_parent(
        mut self,
        child_id: &NodeId,
        parent_id: &NodeId,
    ) -> Result<Self, PoolError> {
        if !self.nodes.contains_key(parent_id) {
            return Err(PoolError::ParentNotFound(parent_id.clone()));
        }
        if !self.nodes.contains_key(child_id) {
            return Err(PoolError::ChildNotFound(child_id.clone()));
        }

        // 更新父节点内容
        let parent = self
            .nodes
            .get(parent_id)
            .ok_or_else(|| PoolError::ParentNotFound(parent_id.clone()))?;
        let mut updated_content = parent.content.clone();
        updated_content.push_back(child_id.clone());
        let parent_node = Arc::try_unwrap(parent.clone()).unwrap_or_else(|arc| (*arc).clone());
        let updated_parent = Node {
            content: updated_content,
            ..parent_node
        };

        // 更新内部状态
        self.nodes.insert(parent_id.clone(), Arc::new(updated_parent));
        self.parent_map.insert(child_id.clone(), parent_id.clone());

        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodePool {
    // 使用 Arc 包裹内部结构，实现快速克隆
    pub inner: Arc<NodePoolInner>,
}
unsafe impl Send for NodePool {}
unsafe impl Sync for NodePool {}

impl NodePool {
    pub fn from(nodes: Vec<Node>) -> Self {
        let mut nodes_ref = HashMap::new();
        let mut parent_map_ref = HashMap::new();
        let mut child_map_ref = HashMap::new();
        for node in nodes.into_iter() {
            for child_id in &node.content {
                parent_map_ref.insert(child_id.clone(), node.id.clone());
            }
            nodes_ref.insert(node.id.clone(), Arc::new(node));
        }
        
        NodePool {
            inner: Arc::new(NodePoolInner{ nodes: nodes_ref, parent_map: parent_map_ref, child_map: child_map_ref })
        }
    }

    // -- 核心查询方法 --

    /// 根据ID获取节点(immutable)
    pub fn get_node(&self, id: &NodeId) -> Option<&Arc<Node>> {
        self.inner.nodes.get(id)
    }

 

    /// 检查节点是否存在
    pub fn contains_node(&self, id: &NodeId) -> bool {
        self.inner.nodes.contains_key(id)
    }

    // -- 层级关系操作 --

    /// 获取直接子节点列表
    pub fn children(&self, parent_id: &NodeId) -> Option<&im::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| &n.content)
    }

    /// 递归获取所有子节点（深度优先）
    pub fn descendants(&self, parent_id: &NodeId) -> Vec<&Node> {
        let mut result: Vec<&Node> = Vec::new();
        self._collect_descendants(parent_id, &mut result);
        result
    }

    fn _collect_descendants<'a>(&'a self, parent_id: &NodeId, result: &mut Vec<&'a Node>) {
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
    pub fn parent_id(&self, child_id: &NodeId) -> Option<&NodeId> {
        self.inner.parent_map.get(child_id)
    }

    /// 获取完整祖先链
    pub fn ancestors(&self, child_id: &NodeId) -> Vec<&Arc<Node>> {
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

    // -- 批量操作 --

   

    // -- 结构验证 --

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
    pub fn filter_nodes<P>(&self, predicate: P) -> Vec<&Arc<Node>>
    where
        P: Fn(&Node) -> bool,
    {
        self.inner.nodes.values().filter(|n| predicate(n)).collect()
    }

    /// 查找第一个匹配节点
    pub fn find_node<P>(&self, predicate: P) -> Option<&Arc<Node>>
    where
        P: Fn(&Node) -> bool,
    {
        self.inner.nodes.values().find(|n| predicate(n))
    }
}



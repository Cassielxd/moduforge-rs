use super::{error::PoolError, node::Node, types::NodeId};
use im::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NodePoolInner {
    pub root_id: NodeId,
    pub nodes: im::HashMap<NodeId, Arc<Node>>, // 节点数据共享
    pub parent_map: im::HashMap<NodeId, NodeId>,
    pub child_map: im::HashMap<NodeId, im::Vector<NodeId>>,
}
impl NodePoolInner {
    pub fn update_attr(
        &self,
        id: &NodeId,
        values: &HashMap<String, serde_json::Value>,
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
            child_map: self.child_map.clone(),
            root_id: self.root_id.clone(),
        })
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
    pub fn update_attr(
        &self,
        id: &NodeId,
        values: &HashMap<String, serde_json::Value>,
    ) -> Result<Self, PoolError> {
        Ok(NodePool {
            inner: Arc::new(self.inner.update_attr(id, values)?),
        })
    }
    pub fn from(nodes: Vec<Node>, root_id: NodeId) -> Self {
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
                child_map: HashMap::new(),
                root_id,
            }),
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

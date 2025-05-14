use std::{ops::Index, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::PoolError,
    mark::Mark,
    node::Node,
    ops::{AttrsRef, MarkRef, NodeRef},
    types::NodeId,
};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tree {
    pub root_id: NodeId,
    pub nodes: im::HashMap<NodeId, Arc<Node>>, // 节点数据共享
    pub parent_map: im::HashMap<NodeId, NodeId>,
}
impl Index<&NodeId> for Tree {
    type Output = Arc<Node>;

    fn index(
        &self,
        index: &NodeId,
    ) -> &Self::Output {
        self.nodes.get(index).unwrap()
    }
}

impl Tree {
    // 获取节点的引用
    pub fn node(
        &mut self,
        key: &str,
    ) -> NodeRef<'_> {
        NodeRef::new(self, key.to_string())
    }
    pub fn mark(
        &mut self,
        key: &str,
    ) -> MarkRef<'_> {
        MarkRef::new(self, key.to_string())
    }
    pub fn attrs(
        &mut self,
        key: &str,
    ) -> AttrsRef<'_> {
        AttrsRef::new(self, key.to_string())
    }

    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<&Arc<Node>> {
        self.nodes.get(id)
    }
    pub fn get_parent_node(
        &self,
        id: &NodeId,
    ) -> Option<&Arc<Node>> {
        self.parent_map.get(id).and_then(|id| self.nodes.get(id))
    }

    pub fn children(
        &self,
        parent_id: &NodeId,
    ) -> Option<&im::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| &n.content)
    }
    pub fn children_node(
        &self,
        parent_id: &NodeId,
    ) -> Option<im::Vector<&Arc<Node>>> {
        self.children(parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.get_node(id)).collect())
    }
    pub fn children_count(
        &self,
        parent_id: &NodeId,
    ) -> usize {
        self.get_node(parent_id).map(|n| n.content.len()).unwrap_or(0)
    }

    pub fn new(root: Node) -> Self {
        Self {
            root_id: root.id.clone(),
            nodes: im::HashMap::from(vec![(root.id.clone(), Arc::new(root))]),
            parent_map: im::HashMap::new(),
        }
    }

    pub fn update_attr(
        &mut self,
        id: &NodeId,
        new_values: im::HashMap<String, Value>,
    ) -> Result<(), PoolError> {
        let node =
            self.get_node(id).ok_or(PoolError::NodeNotFound(id.clone()))?;
        let old_values = node.attrs.clone();

        // 更新节点属性
        let mut new_node = node.as_ref().clone();
        let new_attrs = old_values.update(new_values);
        new_node.attrs = new_attrs.clone();
        self.nodes.insert(id.clone(), Arc::new(new_node));
        Ok(())
    }
    pub fn remove_mark(
        &mut self,
        id: &NodeId,
        mark: Mark,
    ) -> Result<(), PoolError> {
        let mut node = self
            .get_node(id)
            .ok_or(PoolError::NodeNotFound(id.clone()))?
            .as_ref()
            .clone();
        node.marks =
            node.marks.iter().filter(|&m| !m.eq(&mark)).cloned().collect();
        self.nodes.insert(id.clone(), Arc::new(node));

        Ok(())
    }

    pub fn add_mark(
        &mut self,
        id: &NodeId,
        marks: &Vec<Mark>,
    ) -> Result<(), PoolError> {
        let mut node = self
            .get_node(id)
            .ok_or(PoolError::NodeNotFound(id.clone()))?
            .as_ref()
            .clone();

        node.marks.extend(marks.clone());
        self.nodes.insert(id.clone(), Arc::new(node));

        Ok(())
    }
    pub fn add_node(
        &mut self,
        parent_id: &NodeId,
        nodes: &Vec<Node>,
    ) -> Result<(), PoolError> {
        let parent = self
            .get_node(parent_id)
            .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;
        let mut new_parent = parent.as_ref().clone();
        new_parent.content.push_back(nodes[0].id.clone());
        self.nodes.insert(parent_id.clone(), Arc::new(new_parent));
        self.parent_map.insert(nodes[0].id.clone(), parent_id.clone());
        let mut new_nodes = vec![];
        for node in nodes.into_iter() {
            new_nodes.push(node.clone());
            // 更新父节点映射
            for child_id in &node.content {
                self.parent_map.insert(child_id.clone(), node.id.clone());
            }
            // 更新节点池
            self.nodes.insert(node.id.clone(), Arc::new(node.clone()));
        }
        Ok(())
    }

    pub fn replace_node(
        &mut self,
        node_id: NodeId,
        nodes: &Vec<Node>,
    ) -> Result<(), PoolError> {
        // 检查节点是否存在
        let _ = self
            .get_node(&node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
        // 确保新节点ID与原节点ID一致
        if nodes[0].id != node_id {
            return Err(PoolError::InvalidNodeId {
                nodeid: node_id,
                new_node_id: nodes[0].id.clone(),
            });
        }
        let _ = self.add_node(&node_id, nodes)?;
        Ok(())
    }

    pub fn move_node(
        &mut self,
        source_parent_id: &NodeId,
        target_parent_id: &NodeId,
        node_id: &NodeId,
        position: Option<usize>,
    ) -> Result<(), PoolError> {
        // 检查源父节点是否存在
        let source_parent = self
            .get_node(source_parent_id)
            .ok_or(PoolError::ParentNotFound(source_parent_id.clone()))?;
        // 检查目标父节点是否存在
        let target_parent = self
            .get_node(target_parent_id)
            .ok_or(PoolError::ParentNotFound(target_parent_id.clone()))?;
        // 检查要移动的节点是否存在
        let _node = self
            .get_node(node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
        // 检查节点是否是源父节点的子节点
        if !source_parent.content.contains(node_id) {
            return Err(PoolError::InvalidParenting {
                child: node_id.clone(),
                alleged_parent: source_parent_id.clone(),
            });
        }
        // 从源父节点中移除该节点
        let mut new_source_parent = source_parent.as_ref().clone();
        new_source_parent.content = new_source_parent
            .content
            .iter()
            .filter(|&id| id != node_id)
            .cloned()
            .collect();

        // 准备将节点添加到目标父节点
        let mut new_target_parent = target_parent.as_ref().clone();
        // 根据指定位置插入节点
        if let Some(pos) = position {
            if pos <= new_target_parent.content.len() {
                // 在指定位置插入
                let mut new_content = im::Vector::new();
                for (i, child_id) in
                    new_target_parent.content.iter().enumerate()
                {
                    if i == pos {
                        new_content.push_back(node_id.clone());
                    }
                    new_content.push_back(child_id.clone());
                }
                // 如果位置是在最后，需要额外处理
                if pos == new_target_parent.content.len() {
                    new_content.push_back(node_id.clone());
                }
                new_target_parent.content = new_content;
            } else {
                // 如果位置超出范围，添加到末尾
                new_target_parent.content.push_back(node_id.clone());
            }
        } else {
            // 默认添加到末尾
            new_target_parent.content.push_back(node_id.clone());
        }

        self.nodes
            .insert(source_parent_id.clone(), Arc::new(new_source_parent));
        self.nodes
            .insert(target_parent_id.clone(), Arc::new(new_target_parent));
        // 更新父子关系映射
        self.parent_map.insert(node_id.clone(), target_parent_id.clone());

        Ok(())
    }

    pub fn remove_node(
        &mut self,
        parent_id: &NodeId,
        nodes: Vec<NodeId>,
    ) -> Result<(), PoolError> {
        // 检查父节点是否存在
        let parent = self
            .get_node(parent_id)
            .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;

        // 检查是否尝试删除根节点
        if nodes.contains(&self.root_id) {
            return Err(PoolError::CannotRemoveRoot);
        }

        // 验证所有要删除的节点都是父节点的直接子节点
        for node_id in &nodes {
            if !parent.content.contains(node_id) {
                return Err(PoolError::InvalidParenting {
                    child: node_id.clone(),
                    alleged_parent: parent_id.clone(),
                });
            }
        }

        // 使用 HashSet 优化查找性能
        let nodes_to_remove: std::collections::HashSet<_> =
            nodes.iter().collect();

        // 过滤保留的子节点
        let filtered_children: im::Vector<NodeId> = parent
            .as_ref()
            .content
            .iter()
            .filter(|&id| !nodes_to_remove.contains(id))
            .cloned()
            .collect();

        // 更新父节点
        let mut parent_node = parent.as_ref().clone();
        parent_node.content = filtered_children;
        self.nodes.insert(parent_id.clone(), Arc::new(parent_node));
        let mut remove_nodes = Vec::new();
        // 递归删除所有子节点
        for node_id in nodes {
            self.remove_subtree(&node_id, &mut remove_nodes)?;
        }

        Ok(())
    }

    fn remove_subtree(
        &mut self,
        node_id: &NodeId,
        remove_nodes: &mut Vec<Node>,
    ) -> Result<(), PoolError> {
        // 检查是否是根节点
        if node_id == &self.root_id {
            return Err(PoolError::CannotRemoveRoot);
        }

        // 获取要删除的节点
        let _ = self
            .get_node(node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;

        // 递归删除所有子节点
        if let Some(children) = self.children(node_id).cloned() {
            for child_id in children {
                self.remove_subtree(&child_id, remove_nodes)?;
            }
        }

        // 从父节点映射中移除
        self.parent_map.remove(node_id);

        // 从节点池中移除并记录补丁
        if let Some(remove_node) = self.nodes.remove(node_id) {
            remove_nodes.push(remove_node.as_ref().clone());
        }
        Ok(())
    }
}

impl Index<&str> for Tree {
    type Output = Arc<Node>;

    fn index(
        &self,
        index: &str,
    ) -> &Self::Output {
        self.nodes.get(index).expect("Node not found")
    }
}

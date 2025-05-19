use std::{ops::Index, sync::Arc, num::NonZeroUsize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use im::Vector;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::node_type::NodeEnum;
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
    pub nodes: Vector<im::HashMap<NodeId, Arc<Node>>>, // 分片存储节点数据
    pub parent_map: im::HashMap<NodeId, NodeId>,
}

impl Tree {
    pub fn get_shard_index(
        &self,
        id: &NodeId,
    ) -> usize {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        (hasher.finish() as usize) % self.nodes.len()
    }

    pub fn contains_node(
        &self,
        id: &NodeId,
    ) -> bool {
        let shard_index = self.get_shard_index(id);
        self.nodes[shard_index].contains_key(id)
    }

    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<Arc<Node>> {
        let shard_index = self.get_shard_index(id);
        self.nodes[shard_index].get(id).cloned()
    }

    pub fn get_parent_node(
        &self,
        id: &NodeId,
    ) -> Option<Arc<Node>> {
        self.parent_map.get(id).and_then(|parent_id| {
            let shard_index = self.get_shard_index(parent_id);
            self.nodes[shard_index].get(parent_id).cloned()
        })
    }
    pub fn from(nodes: NodeEnum) -> Self {
        let num_shards = std::cmp::max(
            std::thread::available_parallelism()
                .map(NonZeroUsize::get)
                .unwrap_or(2),
            2,
        );
        let mut shards = Vector::from(vec![im::HashMap::new(); num_shards]);
        let mut parent_map = im::HashMap::new();
        let (root_node, children) = nodes.into_parts();
        let root_id = root_node.id.clone();

        let mut hasher = DefaultHasher::new();
        root_id.hash(&mut hasher);
        let shard_index = (hasher.finish() as usize) % num_shards;
        shards[shard_index] =
            shards[shard_index].update(root_id.clone(), Arc::new(root_node));

        fn process_children(
            children: Vec<NodeEnum>,
            parent_id: &NodeId,
            shards: &mut Vector<im::HashMap<NodeId, Arc<Node>>>,
            parent_map: &mut im::HashMap<NodeId, NodeId>,
            num_shards: usize,
        ) {
            for child in children {
                let (node, grand_children) = child.into_parts();
                let node_id = node.id.clone();
                let mut hasher = DefaultHasher::new();
                node_id.hash(&mut hasher);
                let shard_index = (hasher.finish() as usize) % num_shards;
                shards[shard_index] =
                    shards[shard_index].update(node_id.clone(), Arc::new(node));
                parent_map.insert(node_id.clone(), parent_id.clone());

                // Recursively process grand children
                process_children(
                    grand_children,
                    &node_id,
                    shards,
                    parent_map,
                    num_shards,
                );
            }
        }

        process_children(
            children,
            &root_id,
            &mut shards,
            &mut parent_map,
            num_shards,
        );

        Self { root_id, nodes: shards, parent_map }
    }

    pub fn new(root: Node) -> Self {
        let num_shards = std::cmp::max(
            std::thread::available_parallelism()
                .map(NonZeroUsize::get)
                .unwrap_or(2),
            2,
        );
        let mut nodes = Vector::from(vec![im::HashMap::new(); num_shards]);
        let root_id = root.id.clone();
        let mut hasher = DefaultHasher::new();
        root_id.hash(&mut hasher);
        let shard_index = (hasher.finish() as usize) % num_shards;
        nodes[shard_index] =
            nodes[shard_index].update(root_id.clone(), Arc::new(root));
        Self { root_id, nodes, parent_map: im::HashMap::new() }
    }

    pub fn update_attr(
        &mut self,
        id: &NodeId,
        new_values: im::HashMap<String, Value>,
    ) -> Result<(), PoolError> {
        let shard_index = self.get_shard_index(id);
        let node = self.nodes[shard_index]
            .get(id)
            .ok_or(PoolError::NodeNotFound(id.clone()))?;
        let old_values = node.attrs.clone();
        let mut new_node = node.as_ref().clone();
        let new_attrs = old_values.update(new_values);
        new_node.attrs = new_attrs.clone();
        self.nodes[shard_index] =
            self.nodes[shard_index].update(id.clone(), Arc::new(new_node));
        Ok(())
    }

    /// 向树中添加新的节点及其子节点
    ///
    /// # 参数
    /// * `nodes` - 要添加的节点枚举，包含节点本身及其子节点
    ///
    /// # 返回值
    /// * `Result<(), PoolError>` - 如果添加成功返回 Ok(()), 否则返回错误
    ///
    /// # 错误
    /// * `PoolError::ParentNotFound` - 如果父节点不存在
    pub fn add(
        &mut self,
        nodes: NodeEnum,
    ) -> Result<(), PoolError> {
        // 将节点枚举分解为当前节点和子节点
        let (mut node, children) = nodes.into_parts();
        let node_id = node.id.clone();

        // 检查父节点是否存在
        let parent_shard_index = self.get_shard_index(&node_id);
        let _ = self.nodes[parent_shard_index]
            .get(&node_id)
            .ok_or(PoolError::ParentNotFound(node_id.clone()))?;

        // 收集所有子节点的ID并添加到当前节点的content中
        let zenliang: Vector<String> =
            children.iter().map(|n| n.0.id.clone()).collect();
        node.content.extend(zenliang);

        // 更新当前节点
        let shard_index = self.get_shard_index(&node_id);
        self.nodes[shard_index] =
            self.nodes[shard_index].update(node_id.clone(), Arc::new(node));

        // 使用队列进行广度优先遍历，处理所有子节点
        let mut node_queue = Vec::new();
        node_queue.push((children, node_id.clone()));
        while let Some((current_children, parent_id)) = node_queue.pop() {
            for child in current_children {
                // 处理每个子节点
                let (mut child_node, grand_children) = child.into_parts();
                let current_node_id = child_node.id.clone();

                // 收集孙节点的ID并添加到子节点的content中
                let zenliang: Vector<String> =
                    grand_children.iter().map(|n| n.0.id.clone()).collect();
                child_node.content.extend(zenliang);

                // 更新子节点
                let shard_index = self.get_shard_index(&current_node_id);
                self.nodes[shard_index] = self.nodes[shard_index]
                    .update(current_node_id.clone(), Arc::new(child_node));

                // 更新父子关系映射
                self.parent_map
                    .insert(current_node_id.clone(), parent_id.clone());

                // 将孙节点加入队列，以便后续处理
                node_queue.push((grand_children, current_node_id.clone()));
            }
        }
        Ok(())
    }

    pub fn add_node(
        &mut self,
        parent_id: &NodeId,
        nodes: &Vec<Node>,
    ) -> Result<(), PoolError> {
        let parent_shard_index = self.get_shard_index(parent_id);
        let parent = self.nodes[parent_shard_index]
            .get(parent_id)
            .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;
        let mut new_parent = parent.as_ref().clone();
        new_parent.content.push_back(nodes[0].id.clone());
        self.nodes[parent_shard_index] = self.nodes[parent_shard_index]
            .update(parent_id.clone(), Arc::new(new_parent));
        self.parent_map.insert(nodes[0].id.clone(), parent_id.clone());
        for node in nodes {
            let shard_index = self.get_shard_index(&node.id);
            for child_id in &node.content {
                self.parent_map.insert(child_id.clone(), node.id.clone());
            }
            self.nodes[shard_index] = self.nodes[shard_index]
                .update(node.id.clone(), Arc::new(node.clone()));
        }
        Ok(())
    }

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

    pub fn children(
        &self,
        parent_id: &NodeId,
    ) -> Option<im::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| n.content.clone())
    }

    pub fn children_node(
        &self,
        parent_id: &NodeId,
    ) -> Option<im::Vector<Arc<Node>>> {
        self.children(parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.get_node(id)).collect())
    }

    pub fn children_count(
        &self,
        parent_id: &NodeId,
    ) -> usize {
        self.get_node(parent_id).map(|n| n.content.len()).unwrap_or(0)
    }

    pub fn remove_mark(
        &mut self,
        id: &NodeId,
        mark: Mark,
    ) -> Result<(), PoolError> {
        let shard_index = self.get_shard_index(id);
        let node = self.nodes[shard_index]
            .get(id)
            .ok_or(PoolError::NodeNotFound(id.clone()))?;
        let mut new_node = node.as_ref().clone();
        new_node.marks =
            new_node.marks.iter().filter(|&m| !m.eq(&mark)).cloned().collect();
        self.nodes[shard_index] =
            self.nodes[shard_index].update(id.clone(), Arc::new(new_node));
        Ok(())
    }

    pub fn add_mark(
        &mut self,
        id: &NodeId,
        marks: &Vec<Mark>,
    ) -> Result<(), PoolError> {
        let shard_index = self.get_shard_index(id);
        let node = self.nodes[shard_index]
            .get(id)
            .ok_or(PoolError::NodeNotFound(id.clone()))?;
        let mut new_node = node.as_ref().clone();
        new_node.marks.extend(marks.clone());
        self.nodes[shard_index] =
            self.nodes[shard_index].update(id.clone(), Arc::new(new_node));
        Ok(())
    }
    pub fn replace_node(
        &mut self,
        node_id: NodeId,
        nodes: &Vec<Node>,
    ) -> Result<(), PoolError> {
        let shard_index = self.get_shard_index(&node_id);
        let _ = self.nodes[shard_index]
            .get(&node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
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
        let source_shard_index = self.get_shard_index(source_parent_id);
        let target_shard_index = self.get_shard_index(target_parent_id);
        let node_shard_index = self.get_shard_index(node_id);
        let source_parent = self.nodes[source_shard_index]
            .get(source_parent_id)
            .ok_or(PoolError::ParentNotFound(source_parent_id.clone()))?;
        let target_parent = self.nodes[target_shard_index]
            .get(target_parent_id)
            .ok_or(PoolError::ParentNotFound(target_parent_id.clone()))?;
        let _node = self.nodes[node_shard_index]
            .get(node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
        if !source_parent.content.contains(node_id) {
            return Err(PoolError::InvalidParenting {
                child: node_id.clone(),
                alleged_parent: source_parent_id.clone(),
            });
        }
        let mut new_source_parent = source_parent.as_ref().clone();
        new_source_parent.content = new_source_parent
            .content
            .iter()
            .filter(|&id| id != node_id)
            .cloned()
            .collect();
        let mut new_target_parent = target_parent.as_ref().clone();
        if let Some(pos) = position {
            if pos <= new_target_parent.content.len() {
                let mut new_content = im::Vector::new();
                for (i, child_id) in
                    new_target_parent.content.iter().enumerate()
                {
                    if i == pos {
                        new_content.push_back(node_id.clone());
                    }
                    new_content.push_back(child_id.clone());
                }
                if pos == new_target_parent.content.len() {
                    new_content.push_back(node_id.clone());
                }
                new_target_parent.content = new_content;
            } else {
                new_target_parent.content.push_back(node_id.clone());
            }
        } else {
            new_target_parent.content.push_back(node_id.clone());
        }
        self.nodes[source_shard_index] = self.nodes[source_shard_index]
            .update(source_parent_id.clone(), Arc::new(new_source_parent));
        self.nodes[target_shard_index] = self.nodes[target_shard_index]
            .update(target_parent_id.clone(), Arc::new(new_target_parent));
        self.parent_map.insert(node_id.clone(), target_parent_id.clone());
        Ok(())
    }

    pub fn remove_node(
        &mut self,
        parent_id: &NodeId,
        nodes: Vec<NodeId>,
    ) -> Result<(), PoolError> {
        let parent_shard_index = self.get_shard_index(parent_id);
        let parent = self.nodes[parent_shard_index]
            .get(parent_id)
            .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;
        if nodes.contains(&self.root_id) {
            return Err(PoolError::CannotRemoveRoot);
        }
        for node_id in &nodes {
            if !parent.content.contains(node_id) {
                return Err(PoolError::InvalidParenting {
                    child: node_id.clone(),
                    alleged_parent: parent_id.clone(),
                });
            }
        }
        let nodes_to_remove: std::collections::HashSet<_> =
            nodes.iter().collect();
        let filtered_children: im::Vector<NodeId> = parent
            .as_ref()
            .content
            .iter()
            .filter(|&id| !nodes_to_remove.contains(id))
            .cloned()
            .collect();
        let mut parent_node = parent.as_ref().clone();
        parent_node.content = filtered_children;
        self.nodes[parent_shard_index] = self.nodes[parent_shard_index]
            .update(parent_id.clone(), Arc::new(parent_node));
        let mut remove_nodes = Vec::new();
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
        if node_id == &self.root_id {
            return Err(PoolError::CannotRemoveRoot);
        }
        let shard_index = self.get_shard_index(node_id);
        let _ = self.nodes[shard_index]
            .get(node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
        if let Some(children) = self.children(node_id) {
            for child_id in children {
                self.remove_subtree(&child_id, remove_nodes)?;
            }
        }
        self.parent_map.remove(node_id);
        if let Some(remove_node) = self.nodes[shard_index].remove(node_id) {
            remove_nodes.push(remove_node.as_ref().clone());
        }
        Ok(())
    }
}

impl Index<&NodeId> for Tree {
    type Output = Arc<Node>;
    fn index(
        &self,
        index: &NodeId,
    ) -> &Self::Output {
        let shard_index = self.get_shard_index(index);
        self.nodes[shard_index].get(index).expect("Node not found")
    }
}

impl Index<&str> for Tree {
    type Output = Arc<Node>;
    fn index(
        &self,
        index: &str,
    ) -> &Self::Output {
        let node_id = NodeId::from(index);
        let shard_index = self.get_shard_index(&node_id);
        self.nodes[shard_index].get(&node_id).expect("Node not found")
    }
}

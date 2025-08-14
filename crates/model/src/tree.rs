use std::sync::Arc;
use std::{ops::Index, num::NonZeroUsize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use imbl::Vector;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use lru::LruCache;
use std::fmt::{self, Debug};
use crate::error::PoolResult;
use crate::node_type::NodeEnum;
use crate::{
    error::error_helpers,
    mark::Mark,
    node::Node,
    ops::{AttrsRef, MarkRef, NodeRef},
    types::NodeId,
};

// 全局LRU缓存用于存储NodeId到分片索引的映射
static SHARD_INDEX_CACHE: Lazy<RwLock<LruCache<NodeId, usize>>> =
    Lazy::new(|| RwLock::new(LruCache::new(NonZeroUsize::new(10000).expect("cache size > 0"))));

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree {
    pub root_id: NodeId,
    pub nodes: Vector<imbl::HashMap<NodeId, Arc<Node>>>, // 分片存储节点数据
    pub parent_map: imbl::HashMap<NodeId, NodeId>,
    #[serde(skip)]
    num_shards: usize, // 缓存分片数量，避免重复计算
}
impl Debug for Tree {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        //输出的时候 过滤掉空的 nodes 节点
        let nodes = self
            .nodes
            .iter()
            .filter(|node| !node.is_empty())
            .collect::<Vec<_>>();
        f.debug_struct("Tree")
            .field("root_id", &self.root_id)
            .field("nodes", &nodes)
            .field("parent_map", &self.parent_map)
            .field("num_shards", &self.num_shards)
            .finish()
    }
}

impl Tree {
    #[inline]
    pub fn get_shard_index(
        &self,
        id: &NodeId,
    ) -> usize {
        // 先检查缓存
        {
            let cache = SHARD_INDEX_CACHE.read();
            if let Some(&index) = cache.peek(id) {
                return index;
            }
        }

        // 缓存未命中，计算哈希值
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let index = (hasher.finish() as usize) % self.num_shards;

        // 更新缓存
        {
            let mut cache = SHARD_INDEX_CACHE.write();
            cache.put(id.clone(), index);
        }

        index
    }

    #[inline]
    pub fn get_shard_indices(
        &self,
        ids: &[&NodeId],
    ) -> Vec<usize> {
        ids.iter().map(|id| self.get_shard_index(id)).collect()
    }

    // 为批量操作提供优化的哈希计算
    #[inline]
    pub fn get_shard_index_batch<'a>(
        &self,
        ids: &'a [&'a NodeId],
    ) -> Vec<(usize, &'a NodeId)> {
        let mut results = Vec::with_capacity(ids.len());
        let mut cache_misses = Vec::new();

        // 批量检查缓存
        {
            let cache = SHARD_INDEX_CACHE.read();
            for &id in ids {
                if let Some(&index) = cache.peek(id) {
                    results.push((index, id));
                } else {
                    cache_misses.push(id);
                }
            }
        }

        // 批量计算缓存未命中的项
        if !cache_misses.is_empty() {
            let mut cache = SHARD_INDEX_CACHE.write();
            for &id in &cache_misses {
                let mut hasher = DefaultHasher::new();
                id.hash(&mut hasher);
                let index = (hasher.finish() as usize) % self.num_shards;
                cache.put(id.clone(), index);
                results.push((index, id));
            }
        }

        results
    }

    // 清理缓存的方法，用于内存管理
    pub fn clear_shard_cache() {
        let mut cache = SHARD_INDEX_CACHE.write();
        cache.clear();
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
        let mut shards = Vector::from(vec![imbl::HashMap::new(); num_shards]);
        let mut parent_map = imbl::HashMap::new();
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
            shards: &mut Vector<imbl::HashMap<NodeId, Arc<Node>>>,
            parent_map: &mut imbl::HashMap<NodeId, NodeId>,
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

        Self { root_id, nodes: shards, parent_map, num_shards }
    }

    pub fn new(root: Node) -> Self {
        let num_shards = std::cmp::max(
            std::thread::available_parallelism()
                .map(NonZeroUsize::get)
                .unwrap_or(2),
            2,
        );
        let mut nodes = Vector::from(vec![imbl::HashMap::new(); num_shards]);
        let root_id = root.id.clone();
        let mut hasher = DefaultHasher::new();
        root_id.hash(&mut hasher);
        let shard_index = (hasher.finish() as usize) % num_shards;
        nodes[shard_index] =
            nodes[shard_index].update(root_id.clone(), Arc::new(root));
        Self { root_id, nodes, parent_map: imbl::HashMap::new(), num_shards }
    }

    pub fn update_attr(
        &mut self,
        id: &NodeId,
        new_values: imbl::HashMap<String, Value>,
    ) -> PoolResult<()> {
        let shard_index = self.get_shard_index(id);
        let node = self.nodes[shard_index]
            .get(id)
            .ok_or(error_helpers::node_not_found(id.clone()))?;
        let new_node = node.as_ref().update_attr(new_values);
        self.nodes[shard_index] =
            self.nodes[shard_index].update(id.clone(), Arc::new(new_node));
        Ok(())
    }
    pub fn update_node(
        &mut self,
        node: Node,
    ) -> PoolResult<()> {
        let shard_index = self.get_shard_index(&node.id);
        self.nodes[shard_index] =
            self.nodes[shard_index].update(node.id.clone(), Arc::new(node));
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
        parent_id: &NodeId,
        nodes: Vec<NodeEnum>,
    ) -> PoolResult<()> {
        // 检查父节点是否存在
        let parent_shard_index = self.get_shard_index(&parent_id);
        let parent_node = self.nodes[parent_shard_index]
            .get(parent_id)
            .ok_or(error_helpers::parent_not_found(parent_id.clone()))?;
        let mut new_parent = parent_node.as_ref().clone();

        // 收集所有子节点的ID并添加到当前节点的content中
        let zenliang: Vector<NodeId> =
            nodes.iter().map(|n| n.0.id.clone()).collect();
        // 需要判断 new_parent.content 中是否已经存在 zenliang 中的节点
        let mut new_content = imbl::Vector::new();
        for id in zenliang {
            if !new_parent.content.contains(&id) {
                new_content.push_back(id);
            }
        }
        new_parent.content.extend(new_content);

        // 更新当前节点
        self.nodes[parent_shard_index] = self.nodes[parent_shard_index]
            .update(parent_id.clone(), Arc::new(new_parent));

        // 使用队列进行广度优先遍历，处理所有子节点
        let mut node_queue = Vec::new();
        node_queue.push((nodes, parent_id.clone()));
        while let Some((current_children, current_parent_id)) = node_queue.pop()
        {
            for child in current_children {
                // 处理每个子节点
                let (mut child_node, grand_children) = child.into_parts();
                let current_node_id = child_node.id.clone();

                // 收集孙节点的ID并添加到子节点的content中
        let grand_children_ids: Vector<NodeId> =
            grand_children.iter().map(|n| n.0.id.clone()).collect();
                let mut new_content = imbl::Vector::new();
                for id in grand_children_ids {
                    if !child_node.content.contains(&id) {
                        new_content.push_back(id);
                    }
                }
                child_node.content.extend(new_content);

                // 将当前节点存储到对应的分片中
                let shard_index = self.get_shard_index(&current_node_id);
                self.nodes[shard_index] = self.nodes[shard_index]
                    .update(current_node_id.clone(), Arc::new(child_node));

                // 更新父子关系映射
                self.parent_map
                    .insert(current_node_id.clone(), current_parent_id.clone());

                // 将孙节点加入队列，以便后续处理
                node_queue.push((grand_children, current_node_id.clone()));
            }
        }
        Ok(())
    }
    // 添加到下标
    pub fn add_at_index(
        &mut self,
        parent_id: &NodeId,
        index: usize,
        node: &Node,
    ) -> PoolResult<()> {
        //添加到节点到 parent_id 的 content 中
        let parent_shard_index = self.get_shard_index(parent_id);
        let parent = self.nodes[parent_shard_index]
            .get(parent_id)
            .ok_or(error_helpers::parent_not_found(parent_id.clone()))?;
        let new_parent =
            parent.as_ref().insert_content_at_index(index, &node.id);
        //更新父节点
        self.nodes[parent_shard_index] = self.nodes[parent_shard_index]
            .update(parent_id.clone(), Arc::new(new_parent));
        //更新父子关系映射
        self.parent_map.insert(node.id.clone(), parent_id.clone());
        //更新子节点
        let shard_index = self.get_shard_index(&node.id);
        self.nodes[shard_index] = self.nodes[shard_index]
            .update(node.id.clone(), Arc::new(node.clone()));
        Ok(())
    }
    pub fn add_node(
        &mut self,
        parent_id: &NodeId,
        nodes: &Vec<Node>,
    ) -> PoolResult<()> {
        let parent_shard_index = self.get_shard_index(parent_id);
        let parent = self.nodes[parent_shard_index]
            .get(parent_id)
            .ok_or(error_helpers::parent_not_found(parent_id.clone()))?;
        let node_ids = nodes.iter().map(|n| n.id.clone()).collect();
        // 更新父节点 - 添加所有节点的ID到content中
        let new_parent = parent.as_ref().insert_contents(&node_ids);

        // 更新父节点到分片中
        self.nodes[parent_shard_index] = self.nodes[parent_shard_index]
            .update(parent_id.clone(), Arc::new(new_parent));

        // 更新所有子节点
        for node in nodes {
            // 设置当前节点的父子关系映射
            self.parent_map.insert(node.id.clone(), parent_id.clone());

            // 设置当前节点的子节点的父子关系映射
            for child_id in &node.content {
                self.parent_map.insert(child_id.clone(), node.id.clone());
            }

            // 将节点添加到对应的分片中
            let shard_index = self.get_shard_index(&node.id);
            self.nodes[shard_index] = self.nodes[shard_index]
                .update(node.id.clone(), Arc::new(node.clone()));
        }
        Ok(())
    }

    pub fn node(
        &mut self,
        key: &str,
    ) -> NodeRef<'_> {
        NodeRef::new(self, key.into())
    }
    pub fn mark(
        &mut self,
        key: &str,
    ) -> MarkRef<'_> {
        MarkRef::new(self, key.into())
    }
    pub fn attrs(
        &mut self,
        key: &str,
    ) -> AttrsRef<'_> {
        AttrsRef::new(self, key.into())
    }

    pub fn children(
        &self,
        parent_id: &NodeId,
    ) -> Option<imbl::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| n.content.clone())
    }

    pub fn children_node(
        &self,
        parent_id: &NodeId,
    ) -> Option<imbl::Vector<Arc<Node>>> {
        self.children(parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.get_node(id)).collect())
    }
    //递归获取所有子节点 封装成 NodeEnum 返回
    pub fn all_children(
        &self,
        parent_id: &NodeId,
        filter: Option<&dyn Fn(&Node) -> bool>,
    ) -> Option<NodeEnum> {
        if let Some(node) = self.get_node(parent_id) {
            let mut child_enums = Vec::new();
            for child_id in &node.content {
                if let Some(child_node) = self.get_node(child_id) {
                    // 检查子节点是否满足过滤条件
                    if let Some(filter_fn) = filter {
                        if !filter_fn(child_node.as_ref()) {
                            continue; // 跳过不满足条件的子节点
                        }
                    }
                    // 递归处理满足条件的子节点
                    if let Some(child_enum) =
                        self.all_children(child_id, filter)
                    {
                        child_enums.push(child_enum);
                    }
                }
            }
            Some(NodeEnum(node.as_ref().clone(), child_enums))
        } else {
            None
        }
    }

    pub fn children_count(
        &self,
        parent_id: &NodeId,
    ) -> usize {
        self.get_node(parent_id).map(|n| n.content.len()).unwrap_or(0)
    }
    pub fn remove_mark_by_name(
        &mut self,
        id: &NodeId,
        mark_name: &str,
    ) -> PoolResult<()> {
        let shard_index = self.get_shard_index(id);
        let node = self.nodes[shard_index]
            .get(id)
            .ok_or(error_helpers::node_not_found(id.clone()))?;
        let new_node = node.as_ref().remove_mark_by_name(mark_name);
        self.nodes[shard_index] =
            self.nodes[shard_index].update(id.clone(), Arc::new(new_node));
        Ok(())
    }
    pub fn get_marks(
        &self,
        id: &NodeId,
    ) -> Option<imbl::Vector<Mark>> {
        self.get_node(id).map(|n| n.marks.clone())
    }

    pub fn remove_mark(
        &mut self,
        id: &NodeId,
        mark_types: &[String],
    ) -> PoolResult<()> {
        let shard_index = self.get_shard_index(id);
        let node = self.nodes[shard_index]
            .get(id)
            .ok_or(error_helpers::node_not_found(id.clone()))?;
        let new_node = node.as_ref().remove_mark(mark_types);
        self.nodes[shard_index] =
            self.nodes[shard_index].update(id.clone(), Arc::new(new_node));
        Ok(())
    }

    pub fn add_mark(
        &mut self,
        id: &NodeId,
        marks: &Vec<Mark>,
    ) -> PoolResult<()> {
        let shard_index = self.get_shard_index(id);
        let node = self.nodes[shard_index]
            .get(id)
            .ok_or(error_helpers::node_not_found(id.clone()))?;
        let new_node = node.as_ref().add_marks(marks);
        self.nodes[shard_index] =
            self.nodes[shard_index].update(id.clone(), Arc::new(new_node));
        Ok(())
    }

    pub fn move_node(
        &mut self,
        source_parent_id: &NodeId,
        target_parent_id: &NodeId,
        node_id: &NodeId,
        position: Option<usize>,
    ) -> PoolResult<()> {
        let source_shard_index = self.get_shard_index(source_parent_id);
        let target_shard_index = self.get_shard_index(target_parent_id);
        let node_shard_index = self.get_shard_index(node_id);
        let source_parent = self.nodes[source_shard_index]
            .get(source_parent_id)
            .ok_or(error_helpers::parent_not_found(source_parent_id.clone()))?;
        let target_parent = self.nodes[target_shard_index]
            .get(target_parent_id)
            .ok_or(error_helpers::parent_not_found(target_parent_id.clone()))?;
        let _node = self.nodes[node_shard_index]
            .get(node_id)
            .ok_or(error_helpers::node_not_found(node_id.clone()))?;
        if !source_parent.content.contains(node_id) {
            return Err(error_helpers::invalid_parenting(
                node_id.clone(),
                source_parent_id.clone(),
            ));
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
            // 确保position不超过当前content的长度
            let insert_pos = pos.min(new_target_parent.content.len());

            // 在指定位置插入节点
            new_target_parent.content.insert(insert_pos, node_id.clone());
        } else {
            // 没有指定位置，添加到末尾
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
    ) -> PoolResult<()> {
        let parent_shard_index = self.get_shard_index(parent_id);
        let parent = self.nodes[parent_shard_index]
            .get(parent_id)
            .ok_or(error_helpers::parent_not_found(parent_id.clone()))?;
        if nodes.contains(&self.root_id) {
            return Err(error_helpers::cannot_remove_root());
        }
        for node_id in &nodes {
            if !parent.content.contains(node_id) {
                return Err(error_helpers::invalid_parenting(
                    node_id.clone(),
                    parent_id.clone(),
                ));
            }
        }
        let nodes_to_remove: std::collections::HashSet<_> =
            nodes.iter().collect();
        let filtered_children: imbl::Vector<NodeId> = parent
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
    //=删除节点
    pub fn remove_node_by_id(
        &mut self,
        node_id: &NodeId,
    ) -> PoolResult<()> {
        // 检查是否试图删除根节点
        if node_id == &self.root_id {
            return Err(error_helpers::cannot_remove_root());
        }

        let shard_index = self.get_shard_index(node_id);
        let _ = self.nodes[shard_index]
            .get(node_id)
            .ok_or(error_helpers::node_not_found(node_id.clone()))?;

        // 从父节点的content中移除该节点
        if let Some(parent_id) = self.parent_map.get(node_id).cloned() {
            let parent_shard_index = self.get_shard_index(&parent_id);
            if let Some(parent_node) =
                self.nodes[parent_shard_index].get(&parent_id)
            {
                let mut new_parent = parent_node.as_ref().clone();
                new_parent.content = new_parent
                    .content
                    .iter()
                    .filter(|&id| id != node_id)
                    .cloned()
                    .collect();
                self.nodes[parent_shard_index] = self.nodes[parent_shard_index]
                    .update(parent_id.clone(), Arc::new(new_parent));
            }
        }

        // 删除子树（remove_subtree内部已经处理了节点的删除和parent_map的清理）
        let mut remove_nodes = Vec::new();
        self.remove_subtree(node_id, &mut remove_nodes)?;

        // remove_subtree已经删除了所有节点，包括node_id本身，所以这里不需要再次删除
        Ok(())
    }

    ///根据下标删除
    pub fn remove_node_by_index(
        &mut self,
        parent_id: &NodeId,
        index: usize,
    ) -> PoolResult<()> {
        let shard_index = self.get_shard_index(parent_id);
        let parent = self.nodes[shard_index]
            .get(parent_id)
            .ok_or(error_helpers::parent_not_found(parent_id.clone()))?;
        let mut new_parent = parent.as_ref().clone();
        let remove_node_id = new_parent.content.remove(index);
        self.nodes[shard_index] = self.nodes[shard_index]
            .update(parent_id.clone(), Arc::new(new_parent));
        let mut remove_nodes = Vec::new();
        self.remove_subtree(&remove_node_id, &mut remove_nodes)?;
        Ok(())
    }

    //删除子树
    fn remove_subtree(
        &mut self,
        node_id: &NodeId,
        remove_nodes: &mut Vec<Node>,
    ) -> PoolResult<()> {
        if node_id == &self.root_id {
            return Err(error_helpers::cannot_remove_root());
        }
        let shard_index = self.get_shard_index(node_id);
        let _ = self.nodes[shard_index]
            .get(node_id)
            .ok_or(error_helpers::node_not_found(node_id.clone()))?;
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
        self.nodes[shard_index]
            .get(index)
            .expect("Node not found")
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
        self.nodes[shard_index]
            .get(&node_id)
            .expect("Node not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::attrs::Attrs;
    use crate::mark::Mark;
    use imbl::HashMap;
    use serde_json::json;

    fn create_test_node(id: &str) -> Node {
        Node::new(id, "test".to_string(), Attrs::default(), vec![], vec![])
    }

    #[test]
    fn test_tree_creation() {
        let root = create_test_node("root");
        let tree = Tree::new(root.clone());
        assert_eq!(tree.root_id, root.id);
        assert!(tree.contains_node(&root.id));
    }

    #[test]
    fn test_add_node() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let child = create_test_node("child");
        let nodes = vec![child.clone()];

        tree.add_node(&root.id, &nodes).unwrap();
        dbg!(&tree);
        assert!(tree.contains_node(&child.id));
        assert_eq!(tree.children(&root.id).unwrap().len(), 1);
    }

    #[test]
    fn test_remove_node() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let child = create_test_node("child");
        let nodes = vec![child.clone()];

        tree.add_node(&root.id, &nodes).unwrap();
        dbg!(&tree);
        tree.remove_node(&root.id, vec![child.id.clone()]).unwrap();
        dbg!(&tree);
        assert!(!tree.contains_node(&child.id));
        assert_eq!(tree.children(&root.id).unwrap().len(), 0);
    }

    #[test]
    fn test_move_node() {
        // 创建两个父节点
        let parent1 = create_test_node("parent1");
        let parent2 = create_test_node("parent2");
        let mut tree = Tree::new(parent1.clone());

        // 将 parent2 添加为 parent1 的子节点
        tree.add_node(&parent1.id, &vec![parent2.clone()]).unwrap();

        // 创建三个子节点
        let child1 = create_test_node("child1");
        let child2 = create_test_node("child2");
        let child3 = create_test_node("child3");

        // 将所有子节点添加到 parent1 下
        tree.add_node(&parent1.id, &vec![child1.clone()]).unwrap();
        tree.add_node(&parent1.id, &vec![child2.clone()]).unwrap();
        tree.add_node(&parent1.id, &vec![child3.clone()]).unwrap();

        // 验证初始状态
        let parent1_children = tree.children(&parent1.id).unwrap();
        assert_eq!(parent1_children.len(), 4); // parent2 + 3 children
        assert_eq!(parent1_children[0], parent2.id);
        assert_eq!(parent1_children[1], child1.id);
        assert_eq!(parent1_children[2], child2.id);
        assert_eq!(parent1_children[3], child3.id);

        // 将 child1 移动到 parent2 下
        tree.move_node(&parent1.id, &parent2.id, &child1.id, None).unwrap();

        // 验证移动后的状态
        let parent1_children = tree.children(&parent1.id).unwrap();
        let parent2_children = tree.children(&parent2.id).unwrap();
        assert_eq!(parent1_children.len(), 3); // parent2 + 2 children
        assert_eq!(parent2_children.len(), 1); // child1
        assert_eq!(parent2_children[0], child1.id);

        // 将 child2 移动到 parent2 下，放在 child1 后面
        tree.move_node(&parent1.id, &parent2.id, &child2.id, Some(1)).unwrap();

        // 验证最终状态
        let parent1_children = tree.children(&parent1.id).unwrap();
        let parent2_children = tree.children(&parent2.id).unwrap();
        assert_eq!(parent1_children.len(), 2); // parent2 + 1 child
        assert_eq!(parent2_children.len(), 2); // child1 + child2
        assert_eq!(parent2_children[0], child1.id);
        assert_eq!(parent2_children[1], child2.id);

        // 验证父节点关系
        let child1_parent = tree.get_parent_node(&child1.id).unwrap();
        let child2_parent = tree.get_parent_node(&child2.id).unwrap();
        assert_eq!(child1_parent.id, parent2.id);
        assert_eq!(child2_parent.id, parent2.id);
    }

    #[test]
    fn test_update_attr() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let mut attrs = HashMap::new();
        attrs.insert("key".to_string(), json!("value"));

        tree.update_attr(&root.id, attrs).unwrap();

        let node = tree.get_node(&root.id).unwrap();
        dbg!(&node);
        assert_eq!(node.attrs.get("key").unwrap(), &json!("value"));
    }

    #[test]
    fn test_add_mark() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let mark = Mark { r#type: "test".to_string(), attrs: Attrs::default() };
        tree.add_mark(&root.id, &vec![mark.clone()]).unwrap();
        dbg!(&tree);
        let node = tree.get_node(&root.id).unwrap();
        assert!(node.marks.contains(&mark));
    }

    #[test]
    fn test_remove_mark() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let mark = Mark { r#type: "test".to_string(), attrs: Attrs::default() };
        tree.add_mark(&root.id, &vec![mark.clone()]).unwrap();
        dbg!(&tree);
        tree.remove_mark(&root.id, &[mark.r#type.clone()]).unwrap();
        dbg!(&tree);
        let node = tree.get_node(&root.id).unwrap();
        assert!(!node.marks.iter().any(|m| m.r#type == mark.r#type));
    }

    #[test]
    fn test_all_children() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let child1 = create_test_node("child1");
        let child2 = create_test_node("child2");

        tree.add_node(&root.id, &vec![child1.clone()]).unwrap();
        tree.add_node(&root.id, &vec![child2.clone()]).unwrap();
        dbg!(&tree);
        let all_children = tree.all_children(&root.id, None).unwrap();
        assert_eq!(all_children.1.len(), 2);
    }

    #[test]
    fn test_children_count() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let child1 = create_test_node("child1");
        let child2 = create_test_node("child2");

        tree.add_node(&root.id, &vec![child1.clone()]).unwrap();
        tree.add_node(&root.id, &vec![child2.clone()]).unwrap();

        assert_eq!(tree.children_count(&root.id), 2);
    }

    #[test]
    fn test_remove_node_by_id_updates_parent() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let child = create_test_node("child");
        tree.add_node(&root.id, &vec![child.clone()]).unwrap();

        // 验证子节点被添加
        assert_eq!(tree.children_count(&root.id), 1);
        assert!(tree.contains_node(&child.id));

        // 删除子节点
        tree.remove_node_by_id(&child.id).unwrap();

        // 验证子节点被删除且父节点的content被更新
        assert_eq!(tree.children_count(&root.id), 0);
        assert!(!tree.contains_node(&child.id));
    }

    #[test]
    fn test_move_node_position_edge_cases() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let container = create_test_node("container");
        tree.add_node(&root.id, &vec![container.clone()]).unwrap();

        let child1 = create_test_node("child1");
        let child2 = create_test_node("child2");
        let child3 = create_test_node("child3");

        tree.add_node(&root.id, &vec![child1.clone()]).unwrap();
        tree.add_node(&root.id, &vec![child2.clone()]).unwrap();
        tree.add_node(&root.id, &vec![child3.clone()]).unwrap();

        // 测试移动到超出范围的位置（应该插入到末尾）
        tree.move_node(&root.id, &container.id, &child1.id, Some(100)).unwrap();

        let container_children = tree.children(&container.id).unwrap();
        assert_eq!(container_children.len(), 1);
        assert_eq!(container_children[0], child1.id);

        // 测试移动到位置0
        tree.move_node(&root.id, &container.id, &child2.id, Some(0)).unwrap();

        let container_children = tree.children(&container.id).unwrap();
        assert_eq!(container_children.len(), 2);
        assert_eq!(container_children[0], child2.id);
        assert_eq!(container_children[1], child1.id);
    }

    #[test]
    fn test_cannot_remove_root_node() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        // 尝试删除根节点应该失败
        let result = tree.remove_node_by_id(&root.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_parent_node() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root.clone());

        let child = create_test_node("child");
        tree.add_node(&root.id, &vec![child.clone()]).unwrap();

        let parent = tree.get_parent_node(&child.id).unwrap();
        assert_eq!(parent.id, root.id);
    }
}

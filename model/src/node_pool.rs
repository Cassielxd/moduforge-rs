use crate::{node_type::NodeEnum, tree::Tree};

use super::{error::PoolError, node::Node, types::NodeId};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref},
    sync::Arc,
};
use rayon::prelude::*;
use std::marker::Sync;
use std::collections::{HashMap, HashSet};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use once_cell::sync::Lazy;

// 用于生成唯一ID的计数器
static POOL_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 线程安全的节点池封装
///
/// 使用 [`Arc`] 实现快速克隆，内部使用不可变数据结构保证线程安全
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NodePool {
    // 使用 Arc 包裹内部结构，实现快速克隆
    inner: Arc<Tree>,
    // 节点池的唯一标识符
    #[serde(skip)]
    key: String,
}

impl Deref for NodePool {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
unsafe impl Send for NodePool {}
unsafe impl Sync for NodePool {}

impl NodePool {
    pub fn new(inner: Arc<Tree>) -> Arc<NodePool> {
        let id = POOL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pool = Self { inner, key: format!("pool_{}", id) };
        let pool: Arc<NodePool> = Arc::new(pool);
        let pool_clone = pool.clone();
        tokio::task::spawn(async move {
            let config = QueryCacheConfig::default();
            let engine = OptimizedQueryEngine::new(&pool_clone, config);
            ENGINE_CACHE
                .lock()
                .unwrap()
                .put(pool_clone.key.clone(), engine.clone());
        });
        pool
    }

    /// 获取节点池的唯一标识符
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 获取节点池中节点总数
    pub fn size(&self) -> usize {
        self.inner.nodes.len()
    }
    pub fn get_inner(&self) -> &Arc<Tree> {
        &self.inner
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
    pub fn from(nodes: NodeEnum) -> Arc<NodePool> {
        let id = POOL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pool = Self {
            inner: Arc::new(Tree::from(nodes)),
            key: format!("pool_{}", id),
        };
        let pool: Arc<NodePool> = Arc::new(pool);
        pool
    }

    // -- 核心查询方法 --

    /// 根据ID获取节点(immutable)
    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<Arc<Node>> {
        self.inner.get_node(id)
    }

    /// 检查节点是否存在
    pub fn contains_node(
        &self,
        id: &NodeId,
    ) -> bool {
        self.inner.contains_node(id)
    }

    // -- 层级关系操作 --

    /// 获取直接子节点列表
    pub fn children(
        &self,
        parent_id: &NodeId,
    ) -> Option<im::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| n.content.clone())
    }

    /// 递归获取所有子节点（深度优先）
    pub fn descendants(
        &self,
        parent_id: &NodeId,
    ) -> Vec<Arc<Node>> {
        let mut result: Vec<Arc<Node>> = Vec::new();
        self._collect_descendants(parent_id, &mut result);
        result
    }

    fn _collect_descendants(
        &self,
        parent_id: &NodeId,
        result: &mut Vec<Arc<Node>>,
    ) {
        if let Some(children) = self.children(parent_id) {
            for child_id in &children {
                if let Some(child) = self.get_node(child_id) {
                    result.push(child);
                    self._collect_descendants(child_id, result);
                }
            }
        }
    }
    pub fn for_each<F>(
        &self,
        id: &NodeId,
        f: F,
    ) where
        F: Fn(&Arc<Node>),
    {
        if let Some(children) = self.children(id) {
            for child_id in &children {
                if let Some(child) = self.get_node(child_id) {
                    f(&child);
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
    ) -> Vec<Arc<Node>> {
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
    ) -> Vec<Arc<Node>>
    where
        P: Fn(&Node) -> bool,
    {
        self.get_all_nodes().into_iter().filter(|n| predicate(n)).collect()
    }
    /// 查找第一个匹配节点
    pub fn find_node<P>(
        &self,
        predicate: P,
    ) -> Option<Arc<Node>>
    where
        P: Fn(&Node) -> bool,
    {
        self.get_all_nodes().into_iter().find(|n| predicate(n))
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
    /// 获取从根节点到目标节点的完整路径
    pub fn resolve(
        &self,
        node_id: &NodeId,
    ) -> Vec<Arc<Node>> {
        let mut result = Vec::new();
        let mut current_id = node_id;
        while let Some(parent_id) = self.parent_id(current_id) {
            if let Some(node) = self.get_node(current_id) {
                result.push(node);
            }
            current_id = parent_id;
        }
        if let Some(node) = self.get_node(node_id) {
            result.push(node);
        }
        result.reverse();
        result
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

    /// 获取左边的所有节点 根据下标
    pub fn get_left_siblings(
        &self,
        node_id: &NodeId,
    ) -> Vec<NodeId> {
        if let Some(parent_id) = self.parent_id(node_id) {
            if let Some(siblings) = self.children(parent_id) {
                let index =
                    siblings.iter().position(|id| id == node_id).unwrap();
                return siblings.iter().take(index).cloned().collect();
            }
        }
        Vec::new()
    }
    /// 获取右边边的所有节点 根据下标
    pub fn get_right_siblings(
        &self,
        node_id: &NodeId,
    ) -> Vec<NodeId> {
        if let Some(parent_id) = self.parent_id(node_id) {
            if let Some(siblings) = self.children(parent_id) {
                let index =
                    siblings.iter().position(|id| id == node_id).unwrap();
                return siblings.iter().skip(index + 1).cloned().collect();
            }
        }
        Vec::new()
    }
    /// 获取左边的所有节点
    pub fn get_left_nodes(
        &self,
        node_id: &NodeId,
    ) -> Vec<Arc<Node>> {
        let siblings = self.get_left_siblings(node_id);
        let mut result = Vec::new();
        for sibling_id in siblings {
            if let Some(node) = self.get_node(&sibling_id) {
                result.push(node);
            }
        }
        result
    }

    /// 获取右边的所有节点
    pub fn get_right_nodes(
        &self,
        node_id: &NodeId,
    ) -> Vec<Arc<Node>> {
        let siblings = self.get_right_siblings(node_id);
        let mut result = Vec::new();
        for sibling_id in siblings {
            if let Some(node) = self.get_node(&sibling_id) {
                result.push(node);
            }
        }
        result
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
            for child_id in &children {
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

    /// 并行查询节点
    ///
    /// # 参数
    ///
    /// * `predicate` - 查询条件函数
    ///
    /// # 返回值
    ///
    /// 返回所有满足条件的节点
    pub fn parallel_query<P>(
        &self,
        predicate: P,
    ) -> Vec<Arc<Node>>
    where
        P: Fn(&Node) -> bool + Send + Sync,
    {
        self.get_all_nodes()
            .into_par_iter()
            .filter(|node| predicate(node))
            .collect()
    }

    /// 并行批量查询节点
    ///
    /// # 参数
    ///
    /// * `batch_size` - 批处理大小
    /// * `predicate` - 查询条件函数
    ///
    /// # 返回值
    ///
    /// 返回所有满足条件的节点
    pub fn parallel_batch_query<'a, P>(
        &'a self,
        batch_size: usize,
        predicate: P,
    ) -> Vec<Arc<Node>>
    where
        P: Fn(&[Arc<Node>]) -> Vec<Arc<Node>> + Send + Sync,
    {
        let nodes = self.get_all_nodes();
        nodes
            .par_chunks(batch_size)
            .flat_map(|chunk| predicate(chunk))
            .collect()
    }

    /// 并行查询并转换结果
    ///
    /// # 参数
    ///
    /// * `predicate` - 查询条件函数
    /// * `transform` - 结果转换函数
    ///
    /// # 返回值
    ///
    /// 返回转换后的结果列表
    pub fn parallel_query_map<'a, P, T, F>(
        &'a self,
        predicate: P,
        transform: F,
    ) -> Vec<T>
    where
        P: Fn(&Node) -> bool + Send + Sync,
        F: Fn(&Arc<Node>) -> T + Send + Sync,
        T: Send,
    {
        self.get_all_nodes()
            .par_iter()
            .filter(|node| predicate(node))
            .map(transform)
            .collect()
    }

    /// 并行查询并聚合结果
    ///
    /// # 参数
    ///
    /// * `predicate` - 查询条件函数
    /// * `init` - 初始值
    /// * `fold` - 聚合函数
    ///
    /// # 返回值
    ///
    /// 返回聚合后的结果
    pub fn parallel_query_reduce<P, T, F>(
        &self,
        predicate: P,
        init: T,
        fold: F,
    ) -> T
    where
        P: Fn(&Node) -> bool + Send + Sync,
        F: Fn(T, &Arc<Node>) -> T + Send + Sync,
        T: Send + Sync + Clone,
    {
        let dummy_node = Arc::new(Node::new(
            "",
            "".to_string(),
            Default::default(),
            vec![],
            vec![],
        ));
        self.get_all_nodes()
            .into_par_iter()
            .filter(|node| predicate(node))
            .fold(|| init.clone(), |acc, node| fold(acc, &node))
            .reduce(|| init.clone(), |a, b| fold(a, &dummy_node))
    }

    // Add helper method to get all nodes
    fn get_all_nodes(&self) -> Vec<Arc<Node>> {
        let mut result = Vec::new();
        for shard in &self.inner.nodes {
            for node in shard.values() {
                result.push(node.clone());
            }
        }
        result
    }
}

/// 查询条件构建器
pub struct QueryEngine<'a> {
    pool: &'a NodePool,
    conditions: Vec<Box<dyn Fn(&Node) -> bool + Send + Sync + 'a>>,
}

impl<'a> QueryEngine<'a> {
    /// 创建新的查询引擎实例
    pub fn new(pool: &'a NodePool) -> Self {
        Self { pool, conditions: Vec::new() }
    }

    /// 按节点类型查询
    pub fn by_type(
        mut self,
        node_type: &'a str,
    ) -> Self {
        let node_type = node_type.to_string();
        self.conditions.push(Box::new(move |node| node.r#type == node_type));
        self
    }

    /// 按属性值查询
    pub fn by_attr(
        mut self,
        key: &'a str,
        value: &'a serde_json::Value,
    ) -> Self {
        let key = key.to_string();
        let value = value.clone();
        self.conditions.push(Box::new(move |node| {
            node.attrs.get(&key).map_or(false, |v| v == &value)
        }));
        self
    }

    /// 按标记查询
    pub fn by_mark(
        mut self,
        mark_type: &'a str,
    ) -> Self {
        let mark_type = mark_type.to_string();
        self.conditions.push(Box::new(move |node| {
            node.marks.iter().any(|mark| mark.r#type == mark_type)
        }));
        self
    }

    /// 按子节点数量查询
    pub fn by_child_count(
        mut self,
        count: usize,
    ) -> Self {
        self.conditions.push(Box::new(move |node| node.content.len() == count));
        self
    }

    /// 按深度查询
    pub fn by_depth(
        mut self,
        depth: usize,
    ) -> Self {
        let pool = self.pool.clone();
        self.conditions.push(Box::new(move |node| {
            pool.get_node_depth(&node.id).map_or(false, |d| d == depth)
        }));
        self
    }

    /// 按祖先节点类型查询
    pub fn by_ancestor_type(
        mut self,
        ancestor_type: &'a str,
    ) -> Self {
        let pool = self.pool.clone();
        let ancestor_type = ancestor_type.to_string();
        self.conditions.push(Box::new(move |node| {
            pool.ancestors(&node.id)
                .iter()
                .any(|ancestor| ancestor.r#type == ancestor_type)
        }));
        self
    }

    /// 按后代节点类型查询
    pub fn by_descendant_type(
        mut self,
        descendant_type: &'a str,
    ) -> Self {
        let pool = self.pool.clone();
        let descendant_type = descendant_type.to_string();
        self.conditions.push(Box::new(move |node| {
            pool.descendants(&node.id)
                .iter()
                .any(|descendant| descendant.r#type == descendant_type)
        }));
        self
    }

    /// 执行查询并返回所有匹配的节点
    pub fn find_all(&self) -> Vec<Arc<Node>> {
        self.pool
            .get_all_nodes()
            .into_iter()
            .filter(|node| {
                self.conditions.iter().all(|condition| condition(node))
            })
            .collect()
    }

    /// 执行查询并返回第一个匹配的节点
    pub fn find_first(&self) -> Option<Arc<Node>> {
        self.pool.get_all_nodes().into_iter().find(|node| {
            self.conditions.iter().all(|condition| condition(node))
        })
    }

    /// 执行查询并返回匹配的节点数量
    pub fn count(&self) -> usize {
        self.pool
            .get_all_nodes()
            .into_iter()
            .filter(|node| {
                self.conditions.iter().all(|condition| condition(node))
            })
            .count()
    }

    /// 并行执行查询并返回所有匹配的节点
    pub fn parallel_find_all(&self) -> Vec<Arc<Node>> {
        let conditions = &self.conditions;
        self.pool.parallel_query(|node| {
            conditions.iter().all(|condition| condition(node))
        })
    }

    /// 并行执行查询并返回第一个匹配的节点
    pub fn parallel_find_first(&self) -> Option<Arc<Node>> {
        let conditions = &self.conditions;
        self.pool.get_all_nodes().into_par_iter().find_any(move |node| {
            conditions.iter().all(|condition| condition(node))
        })
    }

    /// 并行执行查询并返回匹配的节点数量
    pub fn parallel_count(&self) -> usize {
        let conditions = &self.conditions;
        self.pool
            .get_all_nodes()
            .into_par_iter()
            .filter(move |node| {
                conditions.iter().all(|condition| condition(node))
            })
            .count()
    }
}

impl NodePool {
    /// 创建查询引擎实例
    pub fn query(&self) -> QueryEngine {
        QueryEngine::new(self)
    }
}

/// 查询缓存配置
#[derive(Clone, Debug)]
pub struct QueryCacheConfig {
    /// 缓存大小
    pub capacity: usize,
    /// 是否启用缓存
    pub enabled: bool,
}

impl Default for QueryCacheConfig {
    fn default() -> Self {
        Self { capacity: 1000, enabled: true }
    }
}

/// 全局缓存，用于存储 OptimizedQueryEngine 实例
static ENGINE_CACHE: Lazy<Mutex<LruCache<String, OptimizedQueryEngine>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(10).unwrap())));

/// 优化的查询引擎
pub struct OptimizedQueryEngine {
    pool: Arc<NodePool>,
    cache: Option<LruCache<String, Vec<Arc<Node>>>>,
    type_index: HashMap<String, Vec<Arc<Node>>>,
    depth_index: HashMap<usize, Vec<Arc<Node>>>,
    mark_index: HashMap<String, Vec<Arc<Node>>>,
}

impl OptimizedQueryEngine {
    pub fn new(
        pool: &NodePool,
        config: QueryCacheConfig,
    ) -> Self {
        let mut engine = Self {
            pool: Arc::new(pool.clone()),
            cache: if config.enabled {
                Some(LruCache::new(NonZeroUsize::new(config.capacity).unwrap()))
            } else {
                None
            },
            type_index: HashMap::new(),
            depth_index: HashMap::new(),
            mark_index: HashMap::new(),
        };

        engine.build_indices();
        engine
    }

    /// 构建索引
    fn build_indices(&mut self) {
        use rayon::prelude::*;
        use std::collections::HashMap;
        use std::sync::Mutex;
        use std::sync::atomic::{AtomicUsize, Ordering};

        // 预分配容量
        let node_count = self.pool.inner.nodes.len();
        let type_index = Mutex::new(HashMap::with_capacity(node_count / 5));
        let depth_index = Mutex::new(HashMap::with_capacity(10));
        let mark_index = Mutex::new(HashMap::with_capacity(node_count / 10));

        // 使用原子计数器跟踪进度
        let processed = AtomicUsize::new(0);

        // 获取分片的引用并转换为可并行迭代的形式
        let shards: Vec<_> = self.pool.inner.nodes.iter().collect();

        // 使用分片级别的并行处理
        shards.into_par_iter().for_each(|shard| {
            // 为每个线程创建本地索引，使用预分配的容量
            let mut local_type_index = HashMap::with_capacity(shard.len() / 5);
            let mut local_depth_index = HashMap::with_capacity(5);
            let mut local_mark_index = HashMap::with_capacity(shard.len() / 10);

            // 处理当前分片的所有节点
            for node in shard.values() {
                // 类型索引 - 使用 entry API 避免重复查找
                local_type_index
                    .entry(node.r#type.clone())
                    .or_insert_with(|| Vec::with_capacity(shard.len() / 5))
                    .push(Arc::clone(node));

                // 深度索引 - 使用 entry API
                if let Some(depth) = self.pool.get_node_depth(&node.id) {
                    local_depth_index
                        .entry(depth)
                        .or_insert_with(|| Vec::with_capacity(shard.len() / 10))
                        .push(Arc::clone(node));
                }

                // 标记索引 - 使用 entry API
                for mark in node.marks.iter() {
                    local_mark_index
                        .entry(mark.r#type.clone())
                        .or_insert_with(|| Vec::with_capacity(shard.len() / 10))
                        .push(Arc::clone(node));
                }
            }

            // 批量更新全局索引 - 使用单个锁减少锁竞争
            {
                let mut type_idx = type_index.lock().unwrap();
                for (k, v) in local_type_index {
                    type_idx
                        .entry(k)
                        .or_insert_with(|| Vec::with_capacity(v.len()))
                        .extend(v);
                }
            }
            {
                let mut depth_idx = depth_index.lock().unwrap();
                for (k, v) in local_depth_index {
                    depth_idx
                        .entry(k)
                        .or_insert_with(|| Vec::with_capacity(v.len()))
                        .extend(v);
                }
            }
            {
                let mut mark_idx = mark_index.lock().unwrap();
                for (k, v) in local_mark_index {
                    mark_idx
                        .entry(k)
                        .or_insert_with(|| Vec::with_capacity(v.len()))
                        .extend(v);
                }
            }

            // 更新进度
            processed.fetch_add(shard.len(), Ordering::Relaxed);
        });

        // 将并行构建的索引转移到结构体中
        self.type_index = type_index.into_inner().unwrap();
        self.depth_index = depth_index.into_inner().unwrap();
        self.mark_index = mark_index.into_inner().unwrap();
    }

    /// 按类型查询（使用索引）
    pub fn by_type(
        &self,
        node_type: &str,
    ) -> Vec<Arc<Node>> {
        self.type_index.get(node_type).cloned().unwrap_or_default()
    }

    /// 按深度查询（使用索引）
    pub fn by_depth(
        &self,
        depth: usize,
    ) -> Vec<Arc<Node>> {
        self.depth_index.get(&depth).cloned().unwrap_or_default()
    }

    /// 按标记查询（使用索引）
    pub fn by_mark(
        &self,
        mark_type: &str,
    ) -> Vec<Arc<Node>> {
        self.mark_index.get(mark_type).cloned().unwrap_or_default()
    }

    /// 组合查询（使用索引和缓存）
    pub fn query(
        &mut self,
        conditions: Vec<Box<dyn Fn(&Node) -> bool + Send + Sync>>,
    ) -> Vec<Arc<Node>> {
        // 生成缓存键
        let cache_key = format!("query_{}", conditions.len());

        // 检查缓存
        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.peek(&cache_key) {
                return cached.clone();
            }
        }

        // 使用索引优化查询
        let mut candidates: Option<Vec<Arc<Node>>> = None;

        // 根据条件类型选择最优的索引
        for condition in &conditions {
            if let Some(indexed) = self.get_indexed_nodes(condition) {
                candidates = match candidates {
                    None => Some(indexed),
                    Some(existing) => {
                        Some(self.intersect_nodes(&existing, &indexed))
                    },
                };
            }
        }

        let result: Vec<Arc<Node>> = match candidates {
            Some(nodes) => {
                // 使用索引过滤后的候选节点
                nodes
                    .par_iter()
                    .filter(|node| {
                        conditions.iter().all(|condition| condition(node))
                    })
                    .cloned()
                    .collect()
            },
            None => {
                // 回退到全量查询
                self.pool
                    .parallel_query(|node| {
                        conditions.iter().all(|condition| condition(node))
                    })
                    .into_iter()
                    .collect()
            },
        };

        // 更新缓存
        if let Some(cache) = &mut self.cache {
            cache.put(cache_key, result.clone());
        }

        result
    }

    /// 从索引中获取节点
    fn get_indexed_nodes(
        &self,
        condition: &Box<dyn Fn(&Node) -> bool + Send + Sync>,
    ) -> Option<Vec<Arc<Node>>> {
        // 尝试从类型索引获取
        if let Some(type_nodes) = self.type_index.get("document") {
            if condition(&type_nodes[0]) {
                return Some(type_nodes.clone());
            }
        }

        // 尝试从深度索引获取
        if let Some(depth_nodes) = self.depth_index.get(&0) {
            if condition(&depth_nodes[0]) {
                return Some(depth_nodes.clone());
            }
        }

        // 尝试从标记索引获取
        for (_, mark_nodes) in &self.mark_index {
            if !mark_nodes.is_empty() && condition(&mark_nodes[0]) {
                return Some(mark_nodes.clone());
            }
        }

        None
    }

    /// 计算两个节点集合的交集
    fn intersect_nodes(
        &self,
        nodes1: &[Arc<Node>],
        nodes2: &[Arc<Node>],
    ) -> Vec<Arc<Node>> {
        let set1: HashSet<_> = nodes1.iter().map(|n| n.id.as_str()).collect();
        nodes2
            .iter()
            .filter(|node| set1.contains(node.id.as_str()))
            .cloned()
            .collect()
    }
}

impl NodePool {
    /// 创建优化查询引擎（带缓存）
    pub fn optimized_query(
        &self,
        config: QueryCacheConfig,
    ) -> OptimizedQueryEngine {
        let cache_key = format!("engine_{}", self.inner.nodes.len());

        if let Some(cached) = ENGINE_CACHE.lock().unwrap().get(&cache_key) {
            return cached.clone();
        }

        let engine = OptimizedQueryEngine::new(self, config);
        ENGINE_CACHE.lock().unwrap().put(cache_key, engine.clone());
        engine
    }
}

// 为 OptimizedQueryEngine 实现 Clone
impl Clone for OptimizedQueryEngine {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            cache: self.cache.clone(),
            type_index: self.type_index.clone(),
            depth_index: self.depth_index.clone(),
            mark_index: self.mark_index.clone(),
        }
    }
}

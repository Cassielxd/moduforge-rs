use crate::error::PoolResult;
use crate::{node_type::NodeEnum, tree::Tree};

use super::{error::error_helpers, node::Node, types::NodeId};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{sync::Arc};
use rayon::prelude::*;
use std::marker::Sync;
use std::collections::{HashMap, HashSet};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

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

unsafe impl Send for NodePool {}
unsafe impl Sync for NodePool {}

impl NodePool {
    pub fn new(inner: Arc<Tree>) -> Arc<NodePool> {
        let id = POOL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pool = Self { inner, key: format!("pool_{}", id) };
        let pool: Arc<NodePool> = Arc::new(pool);

        pool
    }

    /// 获取节点池的唯一标识符
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 获取节点池中节点总数
    pub fn size(&self) -> usize {
        self.inner.nodes.iter().map(|i| i.values().len()).sum()
    }

    pub fn root(&self) -> Arc<Node> {
        self.inner[&self.inner.root_id].clone()
    }

    pub fn root_id(&self) -> &NodeId {
        &self.inner.root_id
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
    pub fn get_parent_node(
        &self,
        id: &NodeId,
    ) -> Option<Arc<Node>> {
        self.inner.get_parent_node(id)
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
    pub fn validate_hierarchy(&self) -> PoolResult<()> {
        for (child_id, parent_id) in &self.inner.parent_map {
            // 验证父节点存在
            if !self.contains_node(parent_id) {
                return Err(error_helpers::orphan_node(child_id.clone()));
            }

            // 验证父节点确实包含该子节点
            if let Some(children) = self.children(parent_id) {
                if !children.contains(child_id) {
                    return Err(error_helpers::invalid_parenting(
                        child_id.clone(),
                        parent_id.clone(),
                    ));
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
        
        // 收集从当前节点到根节点的路径
        loop {
            if let Some(node) = self.get_node(current_id) {
                result.push(node);
            }
            
            if let Some(parent_id) = self.parent_id(current_id) {
                current_id = parent_id;
            } else {
                // 已到达根节点
                break;
            }
        }
        
        // 反转以获得从根到目标的路径
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
                if let Some(index) = siblings.iter().position(|id| id == node_id) {
                    return siblings.iter().take(index).cloned().collect();
                } else {
                    // 节点不在父节点的children列表中，可能是数据不一致
                    eprintln!("Warning: Node {:?} not found in parent's children list", node_id);
                }
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
                if let Some(index) = siblings.iter().position(|id| id == node_id) {
                    return siblings.iter().skip(index + 1).cloned().collect();
                } else {
                    // 节点不在父节点的children列表中，可能是数据不一致
                    eprintln!("Warning: Node {:?} not found in parent's children list", node_id);
                }
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
        // 将分片转换为 Vec 以支持并行处理
        let shards: Vec<_> = self.inner.nodes.iter().collect();

        // 对每个分片进行并行处理
        shards
            .into_par_iter()
            .flat_map(|shard| {
                // 在每个分片内部进行顺序处理
                shard
                    .values()
                    .filter(|node| predicate(node))
                    .cloned()
                    .collect::<Vec<_>>()
            })
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
        // 将分片转换为 Vec 以支持并行处理
        let shards: Vec<_> = self.inner.nodes.iter().collect();

        // 对每个分片进行并行处理
        shards
            .into_par_iter()
            .flat_map(|shard| {
                // 将分片中的节点收集到 Vec 中
                let nodes: Vec<_> = shard.values().cloned().collect();

                // 按批次处理节点
                nodes
                    .chunks(batch_size)
                    .flat_map(|chunk| predicate(chunk))
                    .collect::<Vec<_>>()
            })
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
        // 将分片转换为 Vec 以支持并行处理
        let shards: Vec<_> = self.inner.nodes.iter().collect();

        // 对每个分片进行并行处理
        shards
            .into_par_iter()
            .flat_map(|shard| {
                // 在每个分片内部进行顺序处理
                shard
                    .values()
                    .filter(|node| predicate(node))
                    .map(|node| transform(node))
                    .collect::<Vec<T>>()
            })
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

        // 将分片转换为 Vec 以支持并行处理
        let shards: Vec<_> = self.inner.nodes.iter().collect();

        // 对每个分片进行并行处理
        shards
            .into_par_iter()
            .map(|shard| {
                // 在每个分片内部进行顺序处理
                shard
                    .values()
                    .filter(|node| predicate(node))
                    .fold(init.clone(), |acc, node| fold(acc, node))
            })
            // 合并所有分片的结果
            .reduce(|| init.clone(), |a, _b| fold(a, &dummy_node))
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
        let start = Instant::now();
        engine.build_indices();
        let duration = start.elapsed();
        println!("索引构建完成，耗时: {:?}", duration);
        engine
    }

    /// 构建索引
    fn build_indices(&mut self) {
        use rayon::prelude::*;
        use std::collections::HashMap;
        use std::sync::Mutex;

        use std::sync::Arc;
        // 预分配容量
        let node_count = self.pool.size();

        // 使用 Arc 包装索引，避免克隆开销
        let type_index =
            Arc::new(Mutex::new(HashMap::with_capacity(node_count / 5)));
        let depth_index = Arc::new(Mutex::new(HashMap::with_capacity(10)));
        let mark_index =
            Arc::new(Mutex::new(HashMap::with_capacity(node_count / 10)));

        // 优化分片策略：使用更细粒度的分片
        let optimal_shard_size = 1000; // 固定较小的分片大小

        // 重新组织数据为更小的分片
        let mut all_nodes: Vec<_> = self
            .pool
            .inner
            .nodes
            .iter()
            .flat_map(|shard| shard.values().cloned())
            .collect();

        // 按ID排序以确保确定性
        all_nodes.sort_by(|a, b| a.id.cmp(&b.id));

        // 创建更小的分片
        let shards: Vec<_> = all_nodes.chunks(optimal_shard_size).collect();

        // 使用分片级别的并行处理
        shards.into_par_iter().for_each(|shard| {
            // 为每个线程创建本地索引，使用预分配的容量
            let mut local_type_index = HashMap::with_capacity(shard.len() / 5);
            let mut local_depth_index = HashMap::with_capacity(5);
            let mut local_mark_index = HashMap::with_capacity(shard.len() / 10);

            // 预分配向量容量，避免动态扩容
            let mut type_nodes = Vec::with_capacity(shard.len());
            let mut depth_nodes = Vec::with_capacity(shard.len());
            let mut mark_nodes = Vec::with_capacity(shard.len() * 2);

            // 批量收集节点信息，使用引用避免克隆
            for node in shard {
                // 收集类型信息
                type_nodes.push((node.r#type.clone(), Arc::clone(node)));

                // 收集深度信息
                if let Some(depth) = self.pool.get_node_depth(&node.id) {
                    depth_nodes.push((depth, Arc::clone(node)));
                }

                // 收集标记信息
                for mark in &node.marks {
                    mark_nodes.push((mark.r#type.clone(), Arc::clone(node)));
                }
            }

            // 批量更新本地索引，使用预分配的容量
            for (type_name, node) in type_nodes {
                local_type_index
                    .entry(type_name)
                    .or_insert_with(|| Vec::with_capacity(shard.len() / 5))
                    .push(node);
            }

            for (depth, node) in depth_nodes {
                local_depth_index
                    .entry(depth)
                    .or_insert_with(|| Vec::with_capacity(shard.len() / 10))
                    .push(node);
            }

            for (mark_type, node) in mark_nodes {
                local_mark_index
                    .entry(mark_type)
                    .or_insert_with(|| Vec::with_capacity(shard.len() / 10))
                    .push(node);
            }

            // 批量更新全局索引，使用更细粒度的锁
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
        });

        // 将并行构建的索引转移到结构体中
        self.type_index =
            Arc::try_unwrap(type_index).unwrap().into_inner().unwrap();
        self.depth_index =
            Arc::try_unwrap(depth_index).unwrap().into_inner().unwrap();
        self.mark_index =
            Arc::try_unwrap(mark_index).unwrap().into_inner().unwrap();
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
        // 生成更安全的缓存键
        let cache_key = self.generate_query_cache_key(&conditions);

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

    /// 生成查询缓存键
    fn generate_query_cache_key(&self, conditions: &[Box<dyn Fn(&Node) -> bool + Send + Sync>]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // 使用条件数量、池ID和时间戳生成唯一键
        conditions.len().hash(&mut hasher);
        self.pool.key().hash(&mut hasher);
        
        // 添加条件的内存地址作为唯一标识符
        for (i, _condition) in conditions.iter().enumerate() {
            // 使用索引和一个随机值来区分不同的条件
            i.hash(&mut hasher);
            std::ptr::addr_of!(_condition).hash(&mut hasher);
        }
        
        format!("query_{:x}", hasher.finish())
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
        let engine = OptimizedQueryEngine::new(self, config);
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

/// 懒加载查询引擎配置
#[derive(Clone, Debug)]
pub struct LazyQueryConfig {
    /// 缓存大小
    pub cache_capacity: usize,
    /// 索引缓存大小
    pub index_cache_capacity: usize,
    /// 是否启用缓存
    pub cache_enabled: bool,
    /// 索引构建阈值（当查询频率超过此值时才构建索引）
    pub index_build_threshold: usize,
}

impl Default for LazyQueryConfig {
    fn default() -> Self {
        Self {
            cache_capacity: 1000,
            index_cache_capacity: 100,
            cache_enabled: true,
            index_build_threshold: 5,
        }
    }
}

/// 查询统计信息
#[derive(Debug, Clone)]
struct QueryStats {
    /// 查询次数
    count: usize,
    /// 最后查询时间
    last_query: Instant,
}

/// 实时构建的懒加载查询引擎
pub struct LazyQueryEngine {
    pool: Arc<NodePool>,
    
    // 缓存系统
    query_cache: Option<LruCache<String, Vec<Arc<Node>>>>,
    
    // 懒加载索引系统
    type_index_cache: LruCache<String, Vec<Arc<Node>>>,
    depth_index_cache: LruCache<usize, Vec<Arc<Node>>>,
    mark_index_cache: LruCache<String, Vec<Arc<Node>>>,
    
    // 查询统计
    type_query_stats: HashMap<String, QueryStats>,
    depth_query_stats: HashMap<usize, QueryStats>,
    mark_query_stats: HashMap<String, QueryStats>,
    
    // 配置
    config: LazyQueryConfig,
}

// 为 LazyQueryEngine 实现线程安全
unsafe impl Send for LazyQueryEngine {}
unsafe impl Sync for LazyQueryEngine {}

impl LazyQueryEngine {
    pub fn new(pool: &NodePool, config: LazyQueryConfig) -> Self {
        Self {
            pool: Arc::new(pool.clone()),
            query_cache: if config.cache_enabled {
                Some(LruCache::new(NonZeroUsize::new(config.cache_capacity).unwrap()))
            } else {
                None
            },
            type_index_cache: LruCache::new(
                NonZeroUsize::new(config.index_cache_capacity).unwrap()
            ),
            depth_index_cache: LruCache::new(
                NonZeroUsize::new(config.index_cache_capacity).unwrap()
            ),
            mark_index_cache: LruCache::new(
                NonZeroUsize::new(config.index_cache_capacity).unwrap()
            ),
            type_query_stats: HashMap::new(),
            depth_query_stats: HashMap::new(),
            mark_query_stats: HashMap::new(),
            config,
        }
    }

    /// 懒加载类型索引
    pub fn by_type_lazy(&mut self, node_type: &str) -> Vec<Arc<Node>> {
        // 更新查询统计
        self.update_type_stats(node_type);
        
        // 检查索引缓存
        if let Some(cached) = self.type_index_cache.get(node_type) {
            return cached.clone();
        }
        
        // 实时构建索引
        let start = Instant::now();
        let nodes = self.build_type_index(node_type);
        let duration = start.elapsed();
        
        println!("实时构建类型索引 '{}', 耗时: {:?}, 节点数: {}", 
                node_type, duration, nodes.len());
        
        // 缓存索引
        self.type_index_cache.put(node_type.to_string(), nodes.clone());
        
        nodes
    }

    /// 懒加载深度索引
    pub fn by_depth_lazy(&mut self, depth: usize) -> Vec<Arc<Node>> {
        self.update_depth_stats(depth);
        
        if let Some(cached) = self.depth_index_cache.get(&depth) {
            return cached.clone();
        }
        
        let start = Instant::now();
        let nodes = self.build_depth_index(depth);
        let duration = start.elapsed();
        
        println!("实时构建深度索引 {}, 耗时: {:?}, 节点数: {}", 
                depth, duration, nodes.len());
        
        self.depth_index_cache.put(depth, nodes.clone());
        nodes
    }

    /// 懒加载标记索引
    pub fn by_mark_lazy(&mut self, mark_type: &str) -> Vec<Arc<Node>> {
        self.update_mark_stats(mark_type);
        
        if let Some(cached) = self.mark_index_cache.get(mark_type) {
            return cached.clone();
        }
        
        let start = Instant::now();
        let nodes = self.build_mark_index(mark_type);
        let duration = start.elapsed();
        
        println!("实时构建标记索引 '{}', 耗时: {:?}, 节点数: {}", 
                mark_type, duration, nodes.len());
        
        self.mark_index_cache.put(mark_type.to_string(), nodes.clone());
        nodes
    }

    /// 智能查询（根据查询频率决定是否使用索引）
    pub fn smart_query<F>(&mut self, query_name: &str, query_fn: F) -> Vec<Arc<Node>>
    where
        F: Fn() -> Vec<Arc<Node>>,
    {
        // 生成更好的缓存键
        let cache_key = self.generate_cache_key(query_name);
        
        // 检查查询缓存
        if let Some(cache) = &self.query_cache {
            if let Some(cached) = cache.peek(&cache_key) {
                return cached.clone();
            }
        }
        
        // 执行查询
        let start = Instant::now();
        let result = query_fn();
        let duration = start.elapsed();
        
        println!("执行查询 '{}', 耗时: {:?}, 结果数: {}", 
                query_name, duration, result.len());
        
        // 更新缓存
        if let Some(cache) = &mut self.query_cache {
            cache.put(cache_key, result.clone());
        }
        
        result
    }

    /// 组合查询（支持索引优化）
    pub fn combined_query(&mut self, conditions: &[QueryCondition]) -> Vec<Arc<Node>> {
        let cache_key = self.generate_combined_cache_key(conditions);
        
        // 检查缓存
        if let Some(cache) = &self.query_cache {
            if let Some(cached) = cache.peek(&cache_key) {
                return cached.clone();
            }
        }
        
        let mut candidates: Option<Vec<Arc<Node>>> = None;
        
        // 根据条件选择最优索引
        for condition in conditions {
            let indexed_nodes = match condition {
                QueryCondition::ByType(type_name) => {
                    if self.should_use_type_index(type_name) {
                        Some(self.by_type_lazy(type_name))
                    } else {
                        None
                    }
                },
                QueryCondition::ByDepth(depth) => {
                    if self.should_use_depth_index(*depth) {
                        Some(self.by_depth_lazy(*depth))
                    } else {
                        None
                    }
                },
                QueryCondition::ByMark(mark_type) => {
                    if self.should_use_mark_index(mark_type) {
                        Some(self.by_mark_lazy(mark_type))
                    } else {
                        None
                    }
                },
                                 QueryCondition::ByAttr { .. } | 
                 QueryCondition::IsLeaf | 
                 QueryCondition::HasChildren => None,
            };
            
            if let Some(indexed) = indexed_nodes {
                candidates = match candidates {
                    None => Some(indexed),
                    Some(existing) => Some(self.intersect_nodes(&existing, &indexed)),
                };
            }
        }
        
        // 执行最终过滤
        let result = match candidates {
            Some(nodes) => {
                nodes.into_par_iter()
                    .filter(|node| conditions.iter().all(|cond| cond.matches(node)))
                    .collect()
            },
            None => {
                // 回退到全量查询
                self.pool.parallel_query(|node| {
                    conditions.iter().all(|cond| cond.matches(node))
                })
            }
        };
        
        // 更新缓存
        if let Some(cache) = &mut self.query_cache {
            cache.put(cache_key, result.clone());
        }
        
        result
    }

    // 私有辅助方法
    
    fn update_type_stats(&mut self, type_name: &str) {
        let stats = self.type_query_stats.entry(type_name.to_string())
            .or_insert(QueryStats { count: 0, last_query: Instant::now() });
        stats.count += 1;
        stats.last_query = Instant::now();
    }
    
    fn update_depth_stats(&mut self, depth: usize) {
        let stats = self.depth_query_stats.entry(depth)
            .or_insert(QueryStats { count: 0, last_query: Instant::now() });
        stats.count += 1;
        stats.last_query = Instant::now();
    }
    
    fn update_mark_stats(&mut self, mark_type: &str) {
        let stats = self.mark_query_stats.entry(mark_type.to_string())
            .or_insert(QueryStats { count: 0, last_query: Instant::now() });
        stats.count += 1;
        stats.last_query = Instant::now();
    }
    
    fn should_use_type_index(&self, type_name: &str) -> bool {
        self.type_query_stats.get(type_name)
            .map(|stats| stats.count >= self.config.index_build_threshold)
            .unwrap_or(false)
    }
    
    fn should_use_depth_index(&self, depth: usize) -> bool {
        self.depth_query_stats.get(&depth)
            .map(|stats| stats.count >= self.config.index_build_threshold)
            .unwrap_or(false)
    }
    
    fn should_use_mark_index(&self, mark_type: &str) -> bool {
        self.mark_query_stats.get(mark_type)
            .map(|stats| stats.count >= self.config.index_build_threshold)
            .unwrap_or(false)
    }
    
    fn build_type_index(&self, node_type: &str) -> Vec<Arc<Node>> {
        self.pool.parallel_query(|node| node.r#type == node_type)
    }
    
    fn build_depth_index(&self, target_depth: usize) -> Vec<Arc<Node>> {
        self.pool.parallel_query(|node| {
            self.pool.get_node_depth(&node.id)
                .map(|depth| depth == target_depth)
                .unwrap_or(false)
        })
    }
    
    fn build_mark_index(&self, mark_type: &str) -> Vec<Arc<Node>> {
        self.pool.parallel_query(|node| {
            node.marks.iter().any(|mark| mark.r#type == mark_type)
        })
    }
    
    fn generate_cache_key(&self, query_name: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        query_name.hash(&mut hasher);
        self.pool.key().hash(&mut hasher);
        format!("query_{:x}", hasher.finish())
    }
    
    fn generate_combined_cache_key(&self, conditions: &[QueryCondition]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        for condition in conditions {
            condition.cache_key().hash(&mut hasher);
        }
        self.pool.key().hash(&mut hasher);
        format!("combined_{:x}", hasher.finish())
    }
    
    fn intersect_nodes(&self, nodes1: &[Arc<Node>], nodes2: &[Arc<Node>]) -> Vec<Arc<Node>> {
        let set1: HashSet<_> = nodes1.iter().map(|n| n.id.as_str()).collect();
        nodes2
            .iter()
            .filter(|node| set1.contains(node.id.as_str()))
            .cloned()
            .collect()
    }
    
    /// 获取查询统计信息
    pub fn get_query_stats(&self) -> QueryStatsSummary {
        QueryStatsSummary {
            type_queries: self.type_query_stats.clone(),
            depth_queries: self.depth_query_stats.clone(),
            mark_queries: self.mark_query_stats.clone(),
            cache_hit_rates: self.calculate_cache_hit_rates(),
        }
    }
    
    fn calculate_cache_hit_rates(&self) -> CacheHitRates {
        CacheHitRates {
            query_cache_size: self.query_cache.as_ref()
                .map(|c| c.len()).unwrap_or(0),
            type_index_cache_size: self.type_index_cache.len(),
            depth_index_cache_size: self.depth_index_cache.len(),
            mark_index_cache_size: self.mark_index_cache.len(),
        }
    }
}

/// 查询条件枚举
#[derive(Debug, Clone)]
pub enum QueryCondition {
    ByType(String),
    ByDepth(usize),
    ByMark(String),
    ByAttr { key: String, value: serde_json::Value },
    IsLeaf,
    HasChildren,
}

impl QueryCondition {
    pub fn matches(&self, node: &Node) -> bool {
        match self {
            QueryCondition::ByType(type_name) => node.r#type == *type_name,
            QueryCondition::ByDepth(_) => true, // 深度检查在索引中完成
            QueryCondition::ByMark(mark_type) => {
                node.marks.iter().any(|mark| mark.r#type == *mark_type)
            },
            QueryCondition::ByAttr { key, value } => {
                node.attrs.get(key).map_or(false, |v| v == value)
            },
            QueryCondition::IsLeaf => node.content.is_empty(),
            QueryCondition::HasChildren => !node.content.is_empty(),
        }
    }
    
    pub fn cache_key(&self) -> String {
        match self {
            QueryCondition::ByType(t) => format!("type_{}", t),
            QueryCondition::ByDepth(d) => format!("depth_{}", d),
            QueryCondition::ByMark(m) => format!("mark_{}", m),
            QueryCondition::ByAttr { key, value } => {
                format!("attr_{}_{}", key, serde_json::to_string(value).unwrap_or_default())
            },
            QueryCondition::IsLeaf => "is_leaf".to_string(),
            QueryCondition::HasChildren => "has_children".to_string(),
        }
    }
}

/// 查询统计摘要
#[derive(Debug)]
pub struct QueryStatsSummary {
    pub type_queries: HashMap<String, QueryStats>,
    pub depth_queries: HashMap<usize, QueryStats>,
    pub mark_queries: HashMap<String, QueryStats>,
    pub cache_hit_rates: CacheHitRates,
}

/// 缓存命中率统计
#[derive(Debug)]
pub struct CacheHitRates {
    pub query_cache_size: usize,
    pub type_index_cache_size: usize,
    pub depth_index_cache_size: usize,
    pub mark_index_cache_size: usize,
}

impl NodePool {
    /// 创建懒加载查询引擎
    pub fn lazy_query(&self, config: LazyQueryConfig) -> LazyQueryEngine {
        LazyQueryEngine::new(self, config)
    }
}

/*
使用示例：

```rust
use crate::node_pool::{LazyQueryConfig, QueryCondition};

// 1. 创建懒加载查询引擎
let config = LazyQueryConfig {
    cache_capacity: 2000,
    index_cache_capacity: 200,
    cache_enabled: true,
    index_build_threshold: 3, // 查询3次后才构建索引
};

let mut lazy_engine = pool.lazy_query(config);

// 2. 使用懒加载索引查询
// 第一次查询 "document" 类型会触发实时索引构建
let docs = lazy_engine.by_type_lazy("document");
println!("文档节点数: {}", docs.len());

// 第二次查询会直接使用缓存的索引
let docs_again = lazy_engine.by_type_lazy("document");

// 3. 使用智能查询
let complex_result = lazy_engine.smart_query("find_complex_nodes", || {
    pool.parallel_query(|node| {
        node.r#type == "paragraph" && !node.attrs.is_empty()
    })
});

// 4. 使用组合查询
let conditions = vec![
    QueryCondition::ByType("text".to_string()),
    QueryCondition::ByDepth(2),
    QueryCondition::IsLeaf,
];

let filtered_nodes = lazy_engine.combined_query(&conditions);

// 5. 查看统计信息
let stats = lazy_engine.get_query_stats();
println!("查询统计: {:#?}", stats);
```

优势：
1. **按需构建**: 只在实际需要时构建索引，避免预构建的开销
2. **智能缓存**: 根据查询频率智能决定是否使用索引
3. **实时反馈**: 每次索引构建都会输出耗时和结果数量
4. **统计监控**: 提供详细的查询统计信息
5. **内存高效**: LRU缓存自动淘汰不常用的索引

适用场景：
- 大型节点池，但只查询部分类型的节点
- 查询模式不确定的应用
- 需要快速启动的应用（不用等待索引预构建）
- 内存受限的环境
*/

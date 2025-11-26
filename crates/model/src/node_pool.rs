use crate::error::PoolResult;
use crate::{node_definition::NodeTree, tree::Tree};

use super::{error::error_helpers, node::Node, types::NodeId};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::{sync::Arc};
use rayon::prelude::*;
use std::marker::Sync;
use std::sync::atomic::{AtomicUsize, Ordering};
use rpds::{VectorSync};

// 用于生成唯一ID的计数器
static POOL_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

type NodeConditionRef<'a> = Box<dyn Fn(&Node) -> bool + Send + Sync + 'a>;

/// 线程安全的节点池封装
///
/// 使用 [`Arc`] 实现快速克隆，内部使用不可变数据结构保证线程安全
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NodePool {
    // 使用 Arc 包裹内部结构，实现快速克隆
    inner: Arc<Tree>,
    // 节点池的唯一标识符
    key: String,
}

impl NodePool {
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(inner), fields(
        crate_name = "model",
        node_count = inner.nodes.iter().map(|i| i.values().len()).sum::<usize>()
    )))]
    pub fn new(inner: Arc<Tree>) -> Arc<NodePool> {
        let id = POOL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pool = Self { inner, key: format!("pool_{id}") };
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

    pub fn root(&self) -> Option<&Node> {
        self.inner.get_node(&self.inner.root_id)
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
    pub fn from(nodes: NodeTree) -> Arc<NodePool> {
        let id = POOL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pool = Self {
            inner: Arc::new(Tree::from(nodes)),
            key: format!("pool_{id}"),
        };
        let pool: Arc<NodePool> = Arc::new(pool);
        pool
    }

    // -- 核心查询方法 --

    /// 根据ID获取节点(immutable)
    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<&Node> {
        self.inner.get_node(id)
    }
    pub fn get_parent_node(
        &self,
        id: &NodeId,
    ) -> Option<&Node> {
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
    ) -> Option<VectorSync<NodeId>> {
        self.get_node(parent_id).map(|n| n.content.clone())
    }

    /// 递归获取所有子节点（深度优先）
    pub fn descendants(
        &self,
        parent_id: &NodeId,
    ) -> Vec<Node> {
        let mut result: Vec<Node> = Vec::new();
        self._collect_descendants(parent_id, &mut result);
        result
    }

    fn _collect_descendants(
        &self,
        parent_id: &NodeId,
        result: &mut Vec<Node>,
    ) {
        if let Some(children) = self.children(parent_id) {
            for child_id in &children {
                if let Some(child) = self.get_node(child_id) {
                    result.push(child.clone());
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
        F: Fn(&Node),
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
    ) -> Vec<&Node> {
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
                let has = children.iter().any(|a| a.eq(child_id));
                if !has {
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
    ) -> Vec<&Node>
    where
        P: Fn(&Node) -> bool,
    {
        self.get_all_nodes().into_iter().filter(|n| predicate(n)).collect()
    }
    /// 查找第一个匹配节点
    pub fn find_node<P>(
        &self,
        predicate: P,
    ) -> Option<&Node>
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
    ) -> Vec<&Node> {
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
                if let Some(index) =
                    siblings.iter().position(|id| id == node_id)
                {
                    return siblings.iter().take(index).cloned().collect();
                } else {
                    // 节点不在父节点的children列表中，可能是数据不一致
                    eprintln!(
                        "Warning: Node {node_id:?} not found in parent's children list"
                    );
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
                if let Some(index) =
                    siblings.iter().position(|id| id == node_id)
                {
                    return siblings.iter().skip(index + 1).cloned().collect();
                } else {
                    // 节点不在父节点的children列表中，可能是数据不一致
                    eprintln!(
                        "Warning: Node {node_id:?} not found in parent's children list"
                    );
                }
            }
        }
        Vec::new()
    }
    /// 获取左边的所有节点
    pub fn get_left_nodes(
        &self,
        node_id: &NodeId,
    ) -> Vec<&Node> {
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
    ) -> Vec<&Node> {
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
    ) -> Vec<&Node>
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
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    // Add helper method to get all nodes
    fn get_all_nodes(&self) -> Vec<&Node> {
        let mut result = Vec::new();
        for shard in &self.inner.nodes {
            for node in shard.values() {
                result.push(node);
            }
        }
        result
    }
}

/// 查询条件构建器
pub struct QueryEngine<'a> {
    pool: &'a NodePool,
    conditions: Vec<NodeConditionRef<'a>>,
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
        self.conditions
            .push(Box::new(move |node| node.attrs.get(&key) == Some(&value)));
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
            pool.get_node_depth(&node.id) == Some(depth)
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
    pub fn find_all(&self) -> Vec<&Node> {
        self.pool
            .get_all_nodes()
            .into_iter()
            .filter(|node| {
                self.conditions.iter().all(|condition| condition(node))
            })
            .collect()
    }

    /// 执行查询并返回第一个匹配的节点
    pub fn find_first(&self) -> Option<&Node> {
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
    pub fn parallel_find_all(&self) -> Vec<&Node> {
        let conditions = &self.conditions;
        self.pool.parallel_query(|node| {
            conditions.iter().all(|condition| condition(node))
        })
    }

    /// 并行执行查询并返回第一个匹配的节点
    pub fn parallel_find_first(&self) -> Option<&Node> {
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
    pub fn query(&self) -> QueryEngine<'_> {
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

/// 实时构建的懒加载查询引擎
pub struct LazyQueryEngine<'a> {
    pool: &'a NodePool,
}

// 注意：LazyQueryEngine 包含非原子内部缓存，不应跨线程共享可变引用

impl<'a> LazyQueryEngine<'a> {
    pub fn new(pool: &'a NodePool) -> Self {
        Self { pool: pool }
    }

    /// 懒加载类型索引
    pub fn by_type_lazy(
        &'a mut self,
        node_type: &str,
    ) -> Vec<&'a Node> {
        // 更新查询统计
        // 实时构建索引
        let start = Instant::now();
        let nodes = self.build_type_index(node_type);
        let duration = start.elapsed();

        println!(
            "实时构建类型索引 '{}', 耗时: {:?}, 节点数: {}",
            node_type,
            duration,
            nodes.len()
        );
        nodes
    }

    /// 懒加载深度索引
    pub fn by_depth_lazy(
        &mut self,
        depth: usize,
    ) -> Vec<&Node> {
        let start = Instant::now();
        let nodes = self.build_depth_index(depth);
        let duration = start.elapsed();

        println!(
            "实时构建深度索引 {}, 耗时: {:?}, 节点数: {}",
            depth,
            duration,
            nodes.len()
        );
        nodes
    }

    /// 懒加载标记索引
    pub fn by_mark_lazy(
        &'a mut self,
        mark_type: &str,
    ) -> Vec<&'a Node> {
        let start = Instant::now();
        let nodes = self.build_mark_index(mark_type);
        let duration = start.elapsed();
        println!(
            "实时构建标记索引 '{}', 耗时: {:?}, 节点数: {}",
            mark_type,
            duration,
            nodes.len()
        );
        nodes
    }

    /// 组合查询（支持索引优化）
    pub fn parallel_query(
        &'a mut self,
        conditions: &[QueryCondition],
    ) -> Vec<&'a Node> {
        let result = self.pool.parallel_query(|node| {
            conditions.iter().all(|cond| cond.matches(node))
        });

        result
    }

    fn build_type_index(
        &self,
        node_type: &str,
    ) -> Vec<&Node> {
        self.pool.parallel_query(|node| node.r#type == node_type)
    }

    fn build_depth_index(
        &self,
        target_depth: usize,
    ) -> Vec<&Node> {
        self.pool.parallel_query(|node| {
            self.pool
                .get_node_depth(&node.id)
                .map(|depth| depth == target_depth)
                .unwrap_or(false)
        })
    }

    fn build_mark_index(
        &self,
        mark_type: &str,
    ) -> Vec<&Node> {
        self.pool.parallel_query(|node| {
            node.marks.iter().any(|mark| mark.r#type == mark_type)
        })
    }
}

/// 查询条件枚举
#[derive(Debug, Clone)]
pub enum QueryCondition {
    ByType(String),
    ByMark(String),
    ByAttr { key: String, value: serde_json::Value },
    IsLeaf,
    HasChildren,
}

impl QueryCondition {
    pub fn matches(
        &self,
        node: &Node,
    ) -> bool {
        match self {
            QueryCondition::ByType(type_name) => node.r#type == *type_name,
            QueryCondition::ByMark(mark_type) => {
                node.marks.iter().any(|mark| mark.r#type == *mark_type)
            },
            QueryCondition::ByAttr { key, value } => {
                node.attrs.get(key) == Some(value)
            },
            QueryCondition::IsLeaf => node.content.is_empty(),
            QueryCondition::HasChildren => !node.content.is_empty(),
        }
    }

    pub fn cache_key(&self) -> String {
        match self {
            QueryCondition::ByType(t) => format!("type_{t}"),
            QueryCondition::ByMark(m) => format!("mark_{m}"),
            QueryCondition::ByAttr { key, value } => {
                format!(
                    "attr_{}_{}",
                    key,
                    serde_json::to_string(value).unwrap_or_default()
                )
            },
            QueryCondition::IsLeaf => "is_leaf".to_string(),
            QueryCondition::HasChildren => "has_children".to_string(),
        }
    }
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
    pub fn lazy_query(&self) -> LazyQueryEngine<'_> {
        // 保持懒查询引擎构造不失败，如需校验容量可在配置层保障
        LazyQueryEngine::new(self)
    }
}

// ========================================
// DataContainer trait 实现
// ========================================

use crate::traits::DataContainer;

impl DataContainer for NodePool {
    type Item = Node;
    type InnerState = Tree;

    fn get(&self, id: &NodeId) -> Option<&Self::Item> {
        self.get_node(id)
    }

    fn contains(&self, id: &NodeId) -> bool {
        self.contains_node(id)
    }

    fn size(&self) -> usize {
        NodePool::size(self)
    }

    fn key(&self) -> &str {
        NodePool::key(self)
    }

    fn items(&self) -> Vec<&Self::Item> {
        let mut result = Vec::new();
        for shard in &self.inner.nodes {
            for node in shard.values() {
                result.push(node);
            }
        }
        result
    }

    fn inner(&self) -> &Self::InnerState {
        &self.inner
    }

    fn from_inner(inner: Self::InnerState) -> Self {
        let id = POOL_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            inner: Arc::new(inner),
            key: format!("pool_{id}"),
        }
    }
}

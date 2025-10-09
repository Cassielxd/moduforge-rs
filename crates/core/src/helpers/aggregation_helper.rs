//! 节点聚合助手 - 并行的自下而上树形节点聚合计算
//!
//! 提供高性能的树形结构聚合计算，支持自定义聚合逻辑和并行处理。
//!
//! # 核心特性
//!
//! - **并行计算**: 使用 rayon 实现同层节点的并行处理
//! - **层级策略**: 支持自定义层级计算策略，内置缓存优化
//! - **类型安全**: 泛型设计支持任意聚合数据类型
//! - **高性能**: 全局 tokio Runtime，避免重复创建开销
//!
//! # 示例
//!
//! ```rust,ignore
//! use mf_core::helpers::aggregation_helper::NodeAggregator;
//!
//! // 定义聚合逻辑：求和
//! let aggregator = NodeAggregator::new(
//!     |node_id: NodeId, state: Arc<State>, cache: Arc<ConcurrentCache<i64>>| async move {
//!         let node = state.get_node(&node_id)?;
//!         let children: Vec<NodeId> = state.get_children(&node_id);
//!
//!         let sum: i64 = children.iter()
//!             .filter_map(|child_id| cache.get(child_id))
//!             .sum();
//!
//!         Ok(sum + node.get_value())
//!     },
//!     CachedLevelStrategy::new(state.clone()),
//! );
//!
//! // 从叶子节点开始聚合
//! let results = aggregator.aggregate_up(&leaf_node_id, state)?;
//! ```

use dashmap::DashMap;
use mf_model::NodeId;
use mf_state::state::State;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::error::ForgeResult;

// ============================================================================
// 并发安全的缓存结构
// ============================================================================

/// 并发安全的缓存，用于存储节点聚合结果
///
/// 内部使用 DashMap 提供无锁并发访问
#[derive(Clone)]
pub struct ConcurrentCache<T: Clone + Send + Sync> {
    inner: Arc<DashMap<NodeId, T>>,
}

impl<T: Clone + Send + Sync> ConcurrentCache<T> {
    /// 创建新的并发缓存
    pub fn new() -> Self {
        Self { inner: Arc::new(DashMap::new()) }
    }

    /// 插入或更新缓存值
    pub fn insert(
        &self,
        key: NodeId,
        value: T,
    ) {
        self.inner.insert(key, value);
    }

    /// 获取缓存值
    pub fn get(
        &self,
        key: &NodeId,
    ) -> Option<T> {
        self.inner.get(key).map(|v| v.clone())
    }

    /// 批量获取缓存值
    pub fn get_all(&self) -> HashMap<NodeId, T> {
        self.inner
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// 清空缓存
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// 检查是否包含指定 key
    pub fn contains(
        &self,
        key: &NodeId,
    ) -> bool {
        self.inner.contains_key(key)
    }
}

impl<T: Clone + Send + Sync> Default for ConcurrentCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 并发安全的计数器
// ============================================================================

/// 并发安全的原子计数器
#[derive(Clone)]
pub struct ConcurrentCounter {
    count: Arc<AtomicUsize>,
}

impl ConcurrentCounter {
    /// 创建新的计数器
    pub fn new() -> Self {
        Self { count: Arc::new(AtomicUsize::new(0)) }
    }

    /// 增加计数
    pub fn increment(&self) -> usize {
        self.count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// 获取当前计数
    pub fn get(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.count.store(0, Ordering::SeqCst);
    }
}

impl Default for ConcurrentCounter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 层级策略 Trait
// ============================================================================

/// 层级计算策略 Trait
///
/// 定义如何计算节点在树中的层级（深度）
pub trait LevelStrategy: Send + Sync {
    /// 计算指定节点的层级
    ///
    /// # 参数
    /// - `node_id`: 节点 ID
    /// - `state`: 状态引用
    ///
    /// # 返回值
    /// 节点层级，根节点为 0
    fn get_level(
        &self,
        node_id: &NodeId,
        state: &Arc<State>,
    ) -> usize;
}

// ============================================================================
// 默认层级策略（每次都计算）
// ============================================================================

/// 默认层级策略 - 每次都遍历父节点链计算层级
///
/// 时间复杂度: O(depth)
pub struct DefaultLevelStrategy;

impl LevelStrategy for DefaultLevelStrategy {
    fn get_level(
        &self,
        node_id: &NodeId,
        state: &Arc<State>,
    ) -> usize {
        let node_pool = &state.node_pool;
        let mut level = 0;
        let mut current = node_id.clone();

        while let Some(parent_id) = node_pool.parent_id(&current) {
            level += 1;
            current = parent_id.clone();
        }

        level
    }
}

// ============================================================================
// 缓存层级策略（推荐使用）
// ============================================================================

/// 缓存层级策略 - 缓存已计算的层级结果
///
/// 时间复杂度: 首次 O(depth)，后续 O(1)
pub struct CachedLevelStrategy {
    cache: Arc<DashMap<NodeId, usize>>,
}

impl CachedLevelStrategy {
    /// 创建新的缓存层级策略
    pub fn new() -> Self {
        Self { cache: Arc::new(DashMap::new()) }
    }

    /// 清空缓存
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl Default for CachedLevelStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelStrategy for CachedLevelStrategy {
    fn get_level(
        &self,
        node_id: &NodeId,
        state: &Arc<State>,
    ) -> usize {
        // 先检查缓存
        if let Some(level) = self.cache.get(node_id) {
            return *level;
        }

        // 缓存未命中，计算层级
        let node_pool = &state.node_pool;
        let mut level = 0;
        let mut current = node_id.clone();

        while let Some(parent_id) = node_pool.parent_id(&current) {
            level += 1;
            current = parent_id.clone();

            // 如果父节点已缓存，直接使用
            if let Some(parent_level) = self.cache.get(&current) {
                level += *parent_level;
                break;
            }
        }

        // 缓存结果
        self.cache.insert(node_id.clone(), level);
        level
    }
}

// ============================================================================
// 节点聚合器 Trait
// ============================================================================

/// 节点聚合处理器类型定义
///
/// 定义单个节点的聚合计算逻辑
///
/// 使用 Arc 包装以支持在多个异步任务间共享
pub type NodeProcessor<T> = Arc<
    dyn Fn(
            NodeId,
            Arc<State>,
            Arc<ConcurrentCache<T>>,
        ) -> Pin<Box<dyn Future<Output = ForgeResult<T>> + Send>>
        + Send
        + Sync,
>;

/// 节点聚合器 Trait
///
/// 定义树形结构的自下而上聚合计算接口
pub trait NodeAggregatorTrait<T: Clone + Send + Sync>: Send + Sync {
    /// 执行自下而上的聚合计算
    ///
    /// # 参数
    /// - `start_node`: 起始节点 ID（通常是叶子节点）
    /// - `state`: 状态引用
    ///
    /// # 返回值
    /// 所有节点的聚合结果 HashMap
    fn aggregate_up(
        &self,
        start_node: &NodeId,
        state: Arc<State>,
    ) -> impl Future<Output = ForgeResult<HashMap<NodeId, T>>> + Send;
}

// ============================================================================
// 节点聚合器实现
// ============================================================================

/// 并行节点聚合器
///
/// 提供高性能的树形节点聚合计算，支持自定义聚合逻辑
///
/// # 类型参数
/// - `T`: 聚合结果类型
pub struct NodeAggregator<T: Clone + Send + Sync + 'static> {
    /// 聚合结果缓存（修复：使用共享实例）
    cache: Arc<ConcurrentCache<T>>,

    /// 节点处理器
    processor: NodeProcessor<T>,

    /// 层级计算策略
    level_strategy: Arc<dyn LevelStrategy>,
}

impl<T: Clone + Send + Sync + 'static> NodeAggregator<T> {
    /// 创建新的节点聚合器
    ///
    /// # 参数
    /// - `processor`: 节点聚合处理函数
    /// - `level_strategy`: 层级计算策略
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let aggregator = NodeAggregator::new(
    ///     |node_id, state, cache| async move {
    ///         // 自定义聚合逻辑
    ///         Ok(result)
    ///     },
    ///     CachedLevelStrategy::new(),
    /// );
    /// ```
    pub fn new<F, Fut>(
        processor: F,
        level_strategy: impl LevelStrategy + 'static,
    ) -> Self
    where
        F: Fn(NodeId, Arc<State>, Arc<ConcurrentCache<T>>) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ForgeResult<T>> + Send + 'static,
    {
        // 创建共享缓存实例（修复 P0 问题：确保只有一个缓存实例）
        let cache = Arc::new(ConcurrentCache::new());

        // 创建处理器闭包（使用 Arc 包装以支持多任务共享）
        let processor_arc: NodeProcessor<T> =
            Arc::new(move |id, state, cache| {
                Box::pin(processor(id, state, cache))
            });

        Self {
            cache,
            processor: processor_arc,
            level_strategy: Arc::new(level_strategy),
        }
    }

    /// 使用默认层级策略创建聚合器
    pub fn with_default_strategy<F, Fut>(processor: F) -> Self
    where
        F: Fn(NodeId, Arc<State>, Arc<ConcurrentCache<T>>) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ForgeResult<T>> + Send + 'static,
    {
        Self::new(processor, DefaultLevelStrategy)
    }

    /// 使用缓存层级策略创建聚合器（推荐）
    pub fn with_cached_strategy<F, Fut>(processor: F) -> Self
    where
        F: Fn(NodeId, Arc<State>, Arc<ConcurrentCache<T>>) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = ForgeResult<T>> + Send + 'static,
    {
        Self::new(processor, CachedLevelStrategy::new())
    }

    /// 收集从起始节点到根节点的所有祖先
    fn collect_ancestors(
        &self,
        start_node: &NodeId,
        state: &Arc<State>,
    ) -> Vec<NodeId> {
        let node_pool = &state.node_pool;
        let mut ancestors = vec![start_node.clone()];
        let mut current = start_node.clone();

        while let Some(parent_id) = node_pool.parent_id(&current) {
            ancestors.push(parent_id.clone());
            current = parent_id.clone();
        }

        ancestors
    }

    /// 按层级分组节点
    ///
    /// 返回: HashMap<层级, Vec<节点ID>>
    fn group_by_level(
        &self,
        nodes: &[NodeId],
        state: &Arc<State>,
    ) -> HashMap<usize, Vec<NodeId>> {
        let mut groups: HashMap<usize, Vec<NodeId>> = HashMap::new();

        for node_id in nodes {
            let level = self.level_strategy.get_level(node_id, state);
            groups.entry(level).or_default().push(node_id.clone());
        }

        groups
    }

    /// 处理单层节点（并发执行）
    ///
    /// 使用 tokio::spawn 实现真正的异步并发
    async fn process_layer(
        &self,
        layer_nodes: &[NodeId],
        state: Arc<State>,
    ) -> ForgeResult<()> {
        // 为每个节点创建异步任务
        let handles: Vec<_> = layer_nodes
            .iter()
            .map(|node_id| {
                let state = state.clone();
                let cache = self.cache.clone();
                let node_id = node_id.clone();
                let processor = self.processor.clone();

                // 使用 tokio::spawn 并发执行
                tokio::spawn(async move {
                    let result =
                        processor(node_id.clone(), state, cache.clone())
                            .await?;
                    cache.insert(node_id.clone(), result);
                    Ok::<_, crate::error::ForgeError>(())
                })
            })
            .collect();

        // 等待所有任务完成
        for handle in handles {
            handle.await.map_err(|e| {
                crate::error::error_utils::engine_error(format!(
                    "任务执行失败: {}",
                    e
                ))
            })??;
        }

        Ok(())
    }
}

impl<T: Clone + Send + Sync + 'static> NodeAggregatorTrait<T>
    for NodeAggregator<T>
{
    /// 执行自下而上的层级聚合
    ///
    /// # 算法流程
    ///
    /// 1. 收集从起始节点到根节点的所有祖先
    /// 2. 按层级分组（叶子节点层级最大）
    /// 3. 从最深层级开始，逐层向上处理
    /// 4. 每层内部使用 tokio::spawn 并发处理
    /// 5. 确保当前层完全处理完成后再处理上一层
    ///
    /// # 性能特性
    ///
    /// - 同层并发: 使用 tokio::spawn 真正异步并发
    /// - 层间串行: 保证数据依赖正确性
    /// - 零开销: 使用调用方现有的 tokio runtime
    /// - 层级缓存: O(1) 层级查询
    async fn aggregate_up(
        &self,
        start_node: &NodeId,
        state: Arc<State>,
    ) -> ForgeResult<HashMap<NodeId, T>> {
        // 1. 清空缓存（每次聚合重新计算）
        self.cache.clear();

        // 2. 收集所有需要聚合的节点（从叶子到根）
        let all_nodes = self.collect_ancestors(start_node, &state);

        // 3. 按层级分组
        let level_groups = self.group_by_level(&all_nodes, &state);

        // 4. 获取层级列表并排序（从深到浅，即从叶子到根）
        let mut levels: Vec<usize> = level_groups.keys().copied().collect();
        levels.sort_by(|a, b| b.cmp(a)); // 降序排列

        // 5. 逐层处理（层级完成后再处理下一层）
        for level in levels {
            if let Some(layer_nodes) = level_groups.get(&level) {
                self.process_layer(layer_nodes, state.clone()).await?;
            }
        }

        // 6. 返回所有聚合结果
        Ok(self.cache.get_all())
    }
}

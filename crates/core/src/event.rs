use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use async_channel::{Receiver, Sender};
use mf_state::{state::State, Transaction};
// 进程信号处理应由应用层负责，不在库层拦截
use arc_swap::ArcSwap;
use dashmap::DashMap;

use crate::{
    config::EventConfig,
    debug::debug,
    error::{ForgeResult, error_utils},
};

// 事件类型定义
#[derive(Debug, Clone)]
pub enum Event {
    /// 状态创建事件
    Create(Arc<State>),

    /// 事务应用事件 (old_state, new_state, transactions)
    /// 统一使用新旧状态模式，与 Undo/Redo 保持一致
    TrApply {
        old_state: Arc<State>,
        new_state: Arc<State>,
        transactions: Vec<Arc<Transaction>>,
    },

    /// 撤销事件 (old_state, new_state, undone_transactions)
    /// 包含被撤销的事务列表，供其他组件（如搜索索引）使用
    Undo {
        old_state: Arc<State>,
        new_state: Arc<State>,
        transactions: Vec<Arc<Transaction>>,
    },

    /// 重做事件 (old_state, new_state, redone_transactions)
    /// 包含重做的事务列表，供其他组件（如搜索索引）使用
    Redo {
        old_state: Arc<State>,
        new_state: Arc<State>,
        transactions: Vec<Arc<Transaction>>,
    },

    /// 历史跳转事件 (old_state, new_state, transactions, steps)
    /// 当用户跳转到历史中的特定位置时触发
    /// transactions 包含跳转过程中所有被影响的事务
    Jump {
        old_state: Arc<State>,
        new_state: Arc<State>,
        transactions: Vec<Arc<Transaction>>,
        steps: isize,
    },

    /// 事务失败事件
    /// 当事务应用失败时触发，供错误处理和日志记录使用
    TrFailed { state: Arc<State>, transaction: Transaction, error: String },

    /// 历史清空事件
    /// 当历史记录被清空时触发
    HistoryCleared,

    /// 销毁事件
    Destroy,

    /// 停止事件（需要重启）
    Stop,
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Event::Create(_) => "Create",
            Event::TrApply { .. } => "TrApply",
            Event::Undo { .. } => "Undo",
            Event::Redo { .. } => "Redo",
            Event::Jump { .. } => "Jump",
            Event::TrFailed { .. } => "TrFailed",
            Event::HistoryCleared => "HistoryCleared",
            Event::Destroy => "Destroy",
            Event::Stop => "Stop",
        }
    }
}

/// 事件处理器 ID 类型
pub type HandlerId = u64;

/// 高性能事件总线
///
/// 使用以下优化策略：
/// - ArcSwap 实现无锁读取事件处理器列表
/// - DashMap 用于快速查找和管理事件处理器
/// - 原子计数器生成唯一 ID
/// - 批量事件处理优化
pub struct EventBus<T: Send + Sync + Clone + 'static> {
    tx: Sender<T>,
    rt: Receiver<T>,
    /// 使用 ArcSwap 实现无锁读取的事件处理器列表
    event_handlers: Arc<ArcSwap<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>>,
    /// 使用 DashMap 快速查找事件处理器
    handler_registry:
        Arc<DashMap<HandlerId, Arc<dyn EventHandler<T> + Send + Sync>>>,
    /// 原子计数器生成唯一 ID
    next_handler_id: Arc<AtomicU64>,
    shutdown: (Sender<()>, Receiver<()>),
    config: EventConfig,
    /// 事件统计
    stats: EventBusStats,
}

/// 事件总线统计信息
#[derive(Clone, Debug)]
pub struct EventBusStats {
    /// 已处理事件总数
    pub events_processed: Arc<AtomicU64>,
    /// 当前活跃处理器数量
    pub active_handlers: Arc<AtomicU64>,
    /// 事件处理失败次数
    pub processing_failures: Arc<AtomicU64>,
    /// 事件处理超时次数
    pub processing_timeouts: Arc<AtomicU64>,
}

impl Default for EventBusStats {
    fn default() -> Self {
        Self {
            events_processed: Arc::new(AtomicU64::new(0)),
            active_handlers: Arc::new(AtomicU64::new(0)),
            processing_failures: Arc::new(AtomicU64::new(0)),
            processing_timeouts: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl<T: Send + Sync + Clone + 'static> Default for EventBus<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + Sync + Clone + 'static> Clone for EventBus<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rt: self.rt.clone(),
            event_handlers: self.event_handlers.clone(),
            handler_registry: self.handler_registry.clone(),
            next_handler_id: self.next_handler_id.clone(),
            shutdown: (self.shutdown.0.clone(), self.shutdown.1.clone()),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl<T: Send + Sync + Clone + 'static> EventBus<T> {
    /// 添加事件处理器，返回处理器 ID
    pub fn add_event_handler(
        &self,
        event_handler: Arc<dyn EventHandler<T> + Send + Sync>,
    ) -> ForgeResult<HandlerId> {
        let handler_id = self.next_handler_id.fetch_add(1, Ordering::Relaxed);

        // 添加到注册表
        self.handler_registry.insert(handler_id, event_handler.clone());

        // 更新处理器列表（无锁操作）
        self.update_handler_list();

        // 更新统计
        self.stats.active_handlers.fetch_add(1, Ordering::Relaxed);

        Ok(handler_id)
    }

    /// 批量添加事件处理器
    pub fn add_event_handlers(
        &self,
        event_handlers: Vec<Arc<dyn EventHandler<T> + Send + Sync>>,
    ) -> ForgeResult<Vec<HandlerId>> {
        let mut handler_ids = Vec::with_capacity(event_handlers.len());

        for handler in event_handlers {
            let handler_id =
                self.next_handler_id.fetch_add(1, Ordering::Relaxed);
            self.handler_registry.insert(handler_id, handler);
            handler_ids.push(handler_id);
        }

        // 批量更新处理器列表
        self.update_handler_list();

        // 更新统计
        self.stats
            .active_handlers
            .fetch_add(handler_ids.len() as u64, Ordering::Relaxed);

        Ok(handler_ids)
    }

    /// 移除事件处理器
    pub fn remove_event_handler(
        &self,
        handler_id: HandlerId,
    ) -> ForgeResult<bool> {
        let removed = self.handler_registry.remove(&handler_id).is_some();

        if removed {
            self.update_handler_list();
            self.stats.active_handlers.fetch_sub(1, Ordering::Relaxed);
        }

        Ok(removed)
    }

    /// 批量移除事件处理器
    pub fn remove_event_handlers(
        &self,
        handler_ids: &[HandlerId],
    ) -> ForgeResult<usize> {
        let mut removed_count = 0;

        for &handler_id in handler_ids {
            if self.handler_registry.remove(&handler_id).is_some() {
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            self.update_handler_list();
            self.stats
                .active_handlers
                .fetch_sub(removed_count as u64, Ordering::Relaxed);
        }

        Ok(removed_count)
    }

    /// 更新处理器列表（内部方法）
    fn update_handler_list(&self) {
        let handlers: Vec<Arc<dyn EventHandler<T> + Send + Sync>> = self
            .handler_registry
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        self.event_handlers.store(Arc::new(handlers));
    }

    /// 获取当前活跃的处理器数量
    pub fn handler_count(&self) -> usize {
        self.handler_registry.len()
    }

    /// 清空所有事件处理器
    pub fn clear_handlers(&self) -> ForgeResult<()> {
        self.handler_registry.clear();
        self.event_handlers.store(Arc::new(Vec::new()));
        self.stats.active_handlers.store(0, Ordering::Relaxed);
        Ok(())
    }
    /// 异步销毁事件总线
    pub async fn destroy(&self) -> ForgeResult<()> {
        self.shutdown.0.send(()).await.map_err(|e| {
            error_utils::event_error(format!("发送关闭信号失败: {e}"))
        })
    }

    /// 同步销毁事件总线（仅在非异步上下文中使用）
    ///
    /// ⚠️ 警告：此方法可能阻塞，应优先使用 `destroy()` 异步版本
    pub fn destroy_blocking(&self) {
        let _ = self.shutdown.0.send_blocking(());
    }
    /// 启动事件循环
    pub fn start_event_loop(&self) {
        let rx: async_channel::Receiver<T> = self.subscribe();
        let event_handlers = self.event_handlers.clone();
        let shutdown_rt = self.shutdown.1.clone();
        let config = self.config.clone();
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut join_set = tokio::task::JoinSet::new();

            // 定义清理函数，确保所有任务都被正确清理
            let cleanup_timeout = config.handler_timeout;
            async fn cleanup_tasks(
                join_set: &mut tokio::task::JoinSet<()>,
                timeout: std::time::Duration,
            ) {
                debug!("开始清理事件处理任务...");
                // 首先停止接受新任务
                join_set.shutdown().await;
                // 然后等待所有现有任务完成，设置超时防止无限等待
                match tokio::time::timeout(timeout, async {
                    while let Some(result) = join_set.join_next().await {
                        if let Err(e) = result {
                            debug!("事件处理任务错误: {}", e);
                        }
                    }
                })
                .await
                {
                    Ok(_) => debug!("所有事件处理任务已正常清理"),
                    Err(_) => debug!("事件处理任务清理超时"),
                }
            }
            loop {
                tokio::select! {
                    event = rx.recv() => match event {
                        Ok(event) => {
                            // 限制并发任务数量，防止无限制spawning
                            if join_set.len() >= config.max_concurrent_handlers {
                                debug!("事件处理任务数量达到上限，等待部分任务完成...");
                                // 等待至少一个任务完成
                                if let Some(Err(e)) = join_set.join_next().await {
                                    debug!("事件处理任务错误: {}", e);
                                }
                            }

                            // 无锁读取事件处理器列表
                            let handlers = event_handlers.load();
                            let handler_timeout = config.handler_timeout;
                            let event_stats = stats.clone();

                            // 更新事件处理统计
                            event_stats.events_processed.fetch_add(1, Ordering::Relaxed);

                            join_set.spawn(async move {
                                // 为该事件并发执行所有 handler
                                let mut handler_set = tokio::task::JoinSet::new();
                                #[allow(clippy::unnecessary_to_owned)]
                                for handler in handlers.iter().cloned() {
                                    let event_for_task = event.clone();
                                    handler_set.spawn(async move {
                                        // 每个任务持有自己的事件克隆，避免跨任务借用问题
                                        let e = event_for_task;
                                        match tokio::time::timeout(handler_timeout, handler.handle(&e)).await {
                                            Ok(Ok(_)) => (true, false, false),
                                            Ok(Err(e)) => { debug!("事件处理器执行失败: {}", e); (false, true, false) },
                                            Err(_) => { debug!("事件处理器执行超时"); (false, false, true) },
                                        }
                                    });
                                }

                                let mut success_count = 0u64;
                                let mut failure_count = 0u64;
                                let mut timeout_count = 0u64;
                                while let Some(res) = handler_set.join_next().await {
                                    match res {
                                        Ok((ok, fail, timeout)) => {
                                            if ok { success_count += 1; }
                                            if fail { failure_count += 1; }
                                            if timeout { timeout_count += 1; }
                                        }
                                        Err(e) => debug!("事件处理器任务错误: {}", e),
                                    }
                                }

                                if failure_count > 0 {
                                    event_stats.processing_failures.fetch_add(failure_count, Ordering::Relaxed);
                                }
                                if timeout_count > 0 {
                                    event_stats.processing_timeouts.fetch_add(timeout_count, Ordering::Relaxed);
                                }

                                debug!("事件处理完成: 成功={}, 失败={}, 超时={}", success_count, failure_count, timeout_count);
                            });
                        },
                        Err(e) => {
                            debug!("事件接收错误: {}", e);
                            cleanup_tasks(&mut join_set, cleanup_timeout).await;
                            break;
                        },
                    },
                    _ = shutdown_rt.recv() => {
                        // 使用统一清理流程，带超时
                        cleanup_tasks(&mut join_set, cleanup_timeout).await;
                        debug!("事件管理器接收到关闭信号，正在退出...");
                        break;
                    },
                    // 定期清理已完成的任务，防止JoinSet无限增长
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                        // 非阻塞地清理已完成的任务
                        while let Some(result) = join_set.try_join_next() {
                            if let Err(e) = result {
                                debug!("事件处理任务错误: {}", e);
                            }
                        }
                    },
                }
            }
        });
    }

    pub fn new() -> Self {
        Self::with_config(EventConfig::default())
    }

    pub fn with_config(config: EventConfig) -> Self {
        let (tx, rt) = async_channel::bounded(config.max_queue_size);
        let (shutdown_tx, shutdown_rt) = async_channel::bounded(1);
        Self {
            tx,
            rt,
            event_handlers: Arc::new(ArcSwap::new(Arc::new(Vec::new()))),
            handler_registry: Arc::new(DashMap::new()),
            next_handler_id: Arc::new(AtomicU64::new(1)),
            shutdown: (shutdown_tx, shutdown_rt),
            config,
            stats: EventBusStats::default(),
        }
    }

    pub fn subscribe(&self) -> Receiver<T> {
        self.rt.clone()
    }

    pub async fn broadcast(
        &self,
        event: T,
    ) -> ForgeResult<()> {
        self.tx
            .send(event)
            .await
            .map_err(|e| error_utils::event_error(format!("广播事件失败: {e}")))
    }
    /// 同步广播事件（仅在非异步上下文中使用）
    ///
    /// ⚠️ 警告：此方法可能阻塞当前线程，应优先使用 `broadcast()` 异步版本
    ///
    /// # 使用场景
    /// - 在 Drop 实现中
    /// - 在同步的测试代码中
    /// - 在非异步的回调函数中
    ///
    /// # 示例
    /// ```rust,no_run
    /// // 在异步上下文中，优先使用：
    /// // event_bus.broadcast(event).await?;
    ///
    /// // 仅在必要时使用阻塞版本：
    /// event_bus.broadcast_blocking(event)?;
    /// ```
    pub fn broadcast_blocking(
        &self,
        event: T,
    ) -> ForgeResult<()> {
        self.tx
            .send_blocking(event)
            .map_err(|e| error_utils::event_error(format!("广播事件失败: {e}")))
    }

    /// 获取事件配置
    pub fn get_config(&self) -> &EventConfig {
        &self.config
    }

    /// 更新事件配置（注意：某些配置更改需要重启事件循环才能生效）
    pub fn update_config(
        &mut self,
        config: EventConfig,
    ) {
        self.config = config;
    }

    /// 获取事件总线统计信息
    pub fn get_stats(&self) -> EventBusStats {
        self.stats.clone()
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        self.stats.events_processed.store(0, Ordering::Relaxed);
        self.stats.processing_failures.store(0, Ordering::Relaxed);
        self.stats.processing_timeouts.store(0, Ordering::Relaxed);
        // 注意：active_handlers 不重置，因为它反映当前状态
    }

    /// 获取详细的性能报告
    pub fn get_performance_report(&self) -> EventBusPerformanceReport {
        let stats = &self.stats;
        EventBusPerformanceReport {
            total_events_processed: stats
                .events_processed
                .load(Ordering::Relaxed),
            active_handlers_count: stats
                .active_handlers
                .load(Ordering::Relaxed),
            total_processing_failures: stats
                .processing_failures
                .load(Ordering::Relaxed),
            total_processing_timeouts: stats
                .processing_timeouts
                .load(Ordering::Relaxed),
            handler_registry_size: self.handler_registry.len(),
            success_rate: {
                let total = stats.events_processed.load(Ordering::Relaxed);
                let failures =
                    stats.processing_failures.load(Ordering::Relaxed);
                if total > 0 {
                    ((total - failures) as f64 / total as f64) * 100.0
                } else {
                    100.0
                }
            },
        }
    }
}

/// 事件总线性能报告
#[derive(Debug, Clone)]
pub struct EventBusPerformanceReport {
    /// 已处理事件总数
    pub total_events_processed: u64,
    /// 当前活跃处理器数量
    pub active_handlers_count: u64,
    /// 处理失败总数
    pub total_processing_failures: u64,
    /// 处理超时总数
    pub total_processing_timeouts: u64,
    /// 处理器注册表大小
    pub handler_registry_size: usize,
    /// 成功率（百分比）
    pub success_rate: f64,
}

// 事件处理器特征
#[async_trait::async_trait]
pub trait EventHandler<T>: Send + Sync + Debug {
    async fn handle(
        &self,
        event: &T,
    ) -> ForgeResult<()>;
}

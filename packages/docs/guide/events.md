# 事件系统

ModuForge-RS 提供了强大的事件驱动架构，允许响应文档状态变化、事务操作和系统生命周期事件。本指南展示了 Price-RS 项目中的实际事件系统应用。

## 核心概念

### 事件类型定义

ModuForge-RS 定义了一套完整的事件类型，涵盖文档生命周期的所有阶段：

```rust
use std::sync::Arc;
use mf_state::{state::StateGeneric, transaction::TransactionGeneric};
use mf_model::traits::{DataContainer, SchemaDefinition};

/// 事件类型定义（泛型版本）
#[derive(Debug, Clone)]
pub enum EventGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 状态创建事件
    Create(Arc<StateGeneric<C, S>>),

    /// 事务应用事件
    TrApply {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 撤销事件
    Undo {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 重做事件
    Redo {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
    },

    /// 历史跳转事件
    Jump {
        old_state: Arc<StateGeneric<C, S>>,
        new_state: Arc<StateGeneric<C, S>>,
        transactions: Vec<Arc<TransactionGeneric<C, S>>>,
        steps: isize,
    },

    /// 事务失败事件
    TrFailed {
        state: Arc<StateGeneric<C, S>>,
        transaction: TransactionGeneric<C, S>,
        error: String,
    },

    /// 历史清空事件
    HistoryCleared,

    /// 销毁事件
    Destroy,

    /// 停止事件
    Stop,
}
```

### EventHandler 特征

所有事件处理器都必须实现 `EventHandler` 特征：

```rust
use mf_core::{Event, ForgeResult};
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait EventHandler<T>: Send + Sync + Debug {
    async fn handle(&self, event: &T) -> ForgeResult<()>;
}
```

## 高性能事件总线

ModuForge-RS 提供了优化的事件总线实现，支持高并发和低延迟：

```rust
use mf_core::{EventBus, EventHandler, EventConfig};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;
use arc_swap::ArcSwap;

pub struct EventBus<T: Send + Sync + Clone + 'static> {
    tx: Sender<T>,
    rt: Receiver<T>,
    /// 使用 ArcSwap 实现无锁读取的事件处理器列表
    event_handlers: Arc<ArcSwap<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>>,
    /// 使用 DashMap 快速查找事件处理器
    handler_registry: Arc<DashMap<HandlerId, Arc<dyn EventHandler<T> + Send + Sync>>>,
    /// 原子计数器生成唯一 ID
    next_handler_id: Arc<AtomicU64>,
    config: EventConfig,
    stats: EventBusStats,
}

/// 事件总线统计信息
#[derive(Clone, Debug)]
pub struct EventBusStats {
    pub events_processed: Arc<AtomicU64>,
    pub active_handlers: Arc<AtomicU64>,
    pub processing_failures: Arc<AtomicU64>,
    pub processing_timeouts: Arc<AtomicU64>,
}
```

### 事件总线特性

1. **无锁读取**：使用 ArcSwap 实现处理器列表的无锁读取
2. **快速查找**：DashMap 提供 O(1) 的处理器查找
3. **并发执行**：事件处理器并发执行，提高吞吐量
4. **超时保护**：内置超时机制防止处理器阻塞
5. **性能统计**：实时监控事件处理性能

## Price-RS 事件处理器实现

### 1. 搜索索引处理器

Price-RS 实现了搜索索引处理器，自动更新搜索索引：

```rust
use mf_core::{Event, EventHandler, ForgeResult};
use mf_search::{IndexEvent, IndexService, RebuildScope, SqliteBackend};
use mf_state::Transaction;
use std::sync::Arc;

pub struct SearchIndexHandler {
    service: Arc<IndexService>,
}

#[async_trait::async_trait]
impl EventHandler<Event> for SearchIndexHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        match event {
            Event::Create(state_arc) => {
                // 创建事件：重建完整索引
                let service_ref = self.service.clone();
                let node_pool_ref = state_arc.node_pool.clone();

                tokio::spawn(async move {
                    let _ = service_ref.handle(IndexEvent::Rebuild {
                        pool: node_pool_ref,
                        scope: RebuildScope::Full,
                    }).await;
                });
            }

            Event::TrApply { old_state, new_state, transactions } => {
                // 事务应用：增量更新索引
                let steps = collect_steps(transactions, StepDirection::Forward);
                let pool_before = old_state.doc();
                let pool_after = new_state.doc();

                self.spawn_index_event(Some(pool_before), pool_after, steps);
            }

            Event::Undo { old_state, new_state, transactions } => {
                // 撤销操作：反向更新索引
                let steps = collect_steps(transactions, StepDirection::Backward);
                let pool_before = old_state.doc();
                let pool_after = new_state.doc();

                self.spawn_index_event(Some(pool_before), pool_after, steps);
            }

            Event::Redo { old_state, new_state, transactions } => {
                // 重做操作：正向更新索引
                let steps = collect_steps(transactions, StepDirection::Forward);
                let pool_before = old_state.doc();
                let pool_after = new_state.doc();

                self.spawn_index_event(Some(pool_before), pool_after, steps);
            }

            Event::Jump { old_state, new_state, transactions, steps } => {
                // 历史跳转：批量更新
                if *steps == 0 {
                    return Ok(());
                }

                let mode = if *steps < 0 {
                    StepDirection::Backward
                } else {
                    StepDirection::Forward
                };

                let collected = collect_steps(transactions, mode);
                let pool_before = old_state.doc();
                let pool_after = new_state.doc();

                self.spawn_index_event(Some(pool_before), pool_after, collected);
            }

            _ => {} // 忽略其他事件
        }
        Ok(())
    }
}

impl SearchIndexHandler {
    fn spawn_index_event(
        &self,
        pool_before: Option<Arc<NodePool>>,
        pool_after: Arc<NodePool>,
        steps: Vec<Arc<dyn StepGeneric<NodePool, Schema>>>,
    ) {
        let svc = self.service.clone();

        if steps.is_empty() {
            // 无步骤，重建完整索引
            tokio::spawn(async move {
                let _ = svc.handle(IndexEvent::Rebuild {
                    pool: pool_after,
                    scope: RebuildScope::Full,
                }).await;
            });
        } else {
            // 有步骤，增量更新
            tokio::spawn(async move {
                let _ = svc.handle(IndexEvent::TransactionCommitted {
                    pool_before,
                    pool_after,
                    steps,
                }).await;
            });
        }
    }
}

/// 创建搜索索引处理器
pub async fn create_search_index_handler(
    temp_root: &Path,
) -> ForgeResult<Arc<SearchIndexHandler>> {
    ensure_default_step_indexers();
    tokio::fs::create_dir_all(temp_root).await?;

    let backend = Arc::new(SqliteBackend::new_in_temp_root(temp_root).await?);
    let service = Arc::new(IndexService::new(backend));

    Ok(Arc::new(SearchIndexHandler { service }))
}
```

### 2. 快照持久化处理器

Price-RS 使用快照处理器实现自动持久化：

```rust
use mf_persistence::{
    api::{CommitMode, PersistOptions},
    sqlite::SqliteEventStore,
    subscriber::SnapshotSubscriber,
};
use std::path::Path;

/// 创建快照持久化处理器
pub async fn create_snapshot_handler(
    project_path: &Path,
) -> ForgeResult<Arc<SnapshotSubscriber>> {
    // 1. 初始化事件存储
    let store = SqliteEventStore::open(
        project_path,
        CommitMode::AsyncDurable { group_window_ms: 8 },
    ).await?;

    // 2. 配置持久化选项
    let persist_opts = PersistOptions {
        commit_mode: CommitMode::AsyncDurable { group_window_ms: 8 },
        snapshot_every_n_events: 1000,        // 每1000个事件创建快照
        snapshot_every_bytes: 8 * 1024 * 1024, // 每8MB创建快照
        snapshot_every_ms: 5 * 60 * 1000,     // 每5分钟创建快照
        compression: true,                     // 启用压缩
    };

    // 3. 创建快照订阅者
    let subscriber = Arc::new(SnapshotSubscriber::new(
        store,
        persist_opts,
        "default_doc",
    ));

    Ok(subscriber)
}
```

### 3. 审计日志处理器

记录所有重要操作的审计日志：

```rust
use chrono::Utc;
use serde_json::json;

pub struct AuditLogHandler {
    log_path: PathBuf,
}

#[async_trait::async_trait]
impl EventHandler<Event> for AuditLogHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        let audit_entry = match event {
            Event::TrApply { new_state, transactions, .. } => {
                json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "event_type": "transaction_applied",
                    "state_version": new_state.version,
                    "transaction_count": transactions.len(),
                    "transactions": transactions.iter().map(|t| {
                        json!({
                            "id": t.id,
                            "steps": t.steps.len(),
                        })
                    }).collect::<Vec<_>>(),
                })
            }

            Event::Undo { transactions, .. } => {
                json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "event_type": "undo",
                    "undone_transactions": transactions.len(),
                })
            }

            Event::TrFailed { transaction, error, .. } => {
                json!({
                    "timestamp": Utc::now().to_rfc3339(),
                    "event_type": "transaction_failed",
                    "transaction_id": transaction.id,
                    "error": error,
                })
            }

            _ => return Ok(()),
        };

        // 异步写入审计日志
        let log_path = self.log_path.clone();
        tokio::spawn(async move {
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .await
                .unwrap();

            let log_line = format!("{}\n", audit_entry.to_string());
            file.write_all(log_line.as_bytes()).await.unwrap();
        });

        Ok(())
    }
}
```

### 4. 价格汇总处理器

Price-RS 特有的业务事件处理器：

```rust
pub struct PriceAggregationHandler {
    name: String,
}

#[async_trait::async_trait]
impl EventHandler<Event> for PriceAggregationHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        match event {
            Event::TrApply { new_state, transactions, .. } => {
                // 检查是否有价格相关的变更
                let has_price_change = transactions.iter().any(|t| {
                    t.steps.iter().any(|s| {
                        matches!(s.action.as_str(), "update_price" | "insert" | "delete")
                    })
                });

                if has_price_change {
                    self.aggregate_prices(new_state.clone()).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl PriceAggregationHandler {
    async fn aggregate_prices(&self, state: Arc<State>) -> ForgeResult<()> {
        info!("💰 触发价格汇总计算");

        // 获取项目根节点
        if let Some(root) = state.doc.root() {
            let total = self.calculate_total(&state.doc, &root.id).await?;

            // 更新汇总价格
            let update_event = Event::TrApply {
                old_state: state.clone(),
                new_state: state.clone(), // 简化示例
                transactions: vec![],
            };

            info!("💰 价格汇总完成: {:.2}", total);
        }

        Ok(())
    }

    async fn calculate_total(&self, doc: &NodePool, node_id: &str) -> ForgeResult<f64> {
        let node = doc.get_node(node_id)
            .ok_or_else(|| anyhow::anyhow!("节点不存在"))?;

        let mut total = 0.0;

        // 递归计算子节点价格
        for child_id in node.children() {
            let child = doc.get_node(child_id).unwrap();

            match child.r#type.as_str() {
                "FbNode" | "QdNode" => {
                    // 分部或清单节点，递归计算
                    total += self.calculate_total(doc, child_id).await?;
                }
                "UnitProjectNode" => {
                    // 单位工程节点，直接获取价格
                    if let Some(price) = child.attrs.get("total_price")
                        .and_then(|v| v.as_f64()) {
                        total += price;
                    }
                }
                _ => {}
            }
        }

        Ok(total)
    }
}
```

## 事件处理器提供器模式

Price-RS 使用提供器模式配置事件处理器：

```rust
use price_common::bootstrap::{
    BootstrapResult, EventHandlerProvider, ProjectProfile, ProjectPhase,
};
use mf_core::{Event, EventHandler};

/// 默认事件处理器提供器
pub struct DefaultEventHandlerProvider;

#[async_trait]
impl EventHandlerProvider for DefaultEventHandlerProvider {
    async fn provide(
        &self,
        profile: &ProjectProfile,
    ) -> BootstrapResult<Vec<Arc<dyn EventHandler<Event> + Send + Sync>>> {
        let mut handlers: Vec<Arc<dyn EventHandler<Event> + Send + Sync>> = vec![];

        match profile.phase {
            ProjectPhase::Budget => {
                let path = Path::new(&profile.project_id);

                // 1. 添加快照持久化处理器
                let snapshot_handler = create_snapshot_handler(path).await?;
                handlers.push(snapshot_handler);

                // 2. 添加搜索索引处理器
                let search_handler = create_search_index_handler(path).await?;
                handlers.push(search_handler);

                // 3. 添加价格汇总处理器
                handlers.push(Arc::new(PriceAggregationHandler {
                    name: "PriceAggregation".to_string(),
                }));

                // 4. 添加审计日志处理器
                handlers.push(Arc::new(AuditLogHandler {
                    log_path: path.join("audit.log"),
                }));
            }

            ProjectPhase::Settlement => {
                // 结算阶段的事件处理器
                handlers.push(Arc::new(SettlementValidationHandler::new()));
                handlers.push(Arc::new(ContractTrackingHandler::new()));
            }

            _ => {
                // 其他阶段使用基础处理器
            }
        }

        Ok(handlers)
    }

    fn name(&self) -> &'static str {
        "default_event_handler_provider"
    }
}
```

## 事件总线配置

### 配置选项

```rust
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct EventConfig {
    /// 最大队列大小
    pub max_queue_size: usize,
    /// 最大并发处理器数量
    pub max_concurrent_handlers: usize,
    /// 处理器超时时间
    pub handler_timeout: Duration,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            max_concurrent_handlers: 100,
            handler_timeout: Duration::from_secs(30),
        }
    }
}
```

### 初始化事件总线

```rust
use mf_core::{EventBus, EventConfig};

pub fn initialize_event_bus() -> EventBus<Event> {
    let config = EventConfig {
        max_queue_size: 50000,
        max_concurrent_handlers: 200,
        handler_timeout: Duration::from_secs(60),
    };

    let event_bus = EventBus::with_config(config);

    // 启动事件循环
    event_bus.start_event_loop();

    event_bus
}
```

## 事件总线操作

### 添加和移除处理器

```rust
// 添加单个处理器
let handler_id = event_bus.add_event_handler(
    Arc::new(LoggingHandler::new())
)?;

// 批量添加处理器
let handler_ids = event_bus.add_event_handlers(vec![
    Arc::new(SearchIndexHandler::new()),
    Arc::new(SnapshotHandler::new()),
    Arc::new(AuditLogHandler::new()),
])?;

// 移除处理器
event_bus.remove_event_handler(handler_id)?;

// 批量移除处理器
let removed_count = event_bus.remove_event_handlers(&handler_ids)?;

// 清空所有处理器
event_bus.clear_handlers()?;
```

### 广播事件

```rust
// 异步广播（推荐）
event_bus.broadcast(Event::Create(state)).await?;

// 同步广播（仅在必要时使用，如 Drop 实现中）
event_bus.broadcast_blocking(Event::Destroy)?;
```

### 监控和统计

```rust
// 获取统计信息
let stats = event_bus.get_stats();
println!("已处理事件: {}", stats.events_processed.load(Ordering::Relaxed));
println!("活跃处理器: {}", stats.active_handlers.load(Ordering::Relaxed));
println!("处理失败: {}", stats.processing_failures.load(Ordering::Relaxed));
println!("处理超时: {}", stats.processing_timeouts.load(Ordering::Relaxed));

// 获取性能报告
let report = event_bus.get_performance_report();
println!("成功率: {:.2}%", report.success_rate);
println!("注册表大小: {}", report.handler_registry_size);

// 重置统计
event_bus.reset_stats();
```

## 最佳实践

### 1. 处理器设计原则

```rust
pub struct WellDesignedHandler {
    name: String,
    // 使用 Arc 共享资源
    shared_resource: Arc<SharedResource>,
}

#[async_trait::async_trait]
impl EventHandler<Event> for WellDesignedHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        // 1. 快速过滤不相关的事件
        if !self.should_handle(event) {
            return Ok(());
        }

        // 2. 避免阻塞操作
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            self.process_event(event),
        ).await;

        // 3. 优雅处理错误
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                warn!("事件处理失败: {}", e);
                // 记录但不中断
                Ok(())
            }
            Err(_) => {
                error!("事件处理超时");
                Ok(())
            }
        }
    }
}
```

### 2. 异步处理模式

```rust
impl SomeHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        match event {
            Event::TrApply { .. } => {
                // 对于耗时操作，使用异步任务
                let resource = self.resource.clone();
                tokio::spawn(async move {
                    // 异步处理，不阻塞事件循环
                    if let Err(e) = resource.process().await {
                        error!("后台处理失败: {}", e);
                    }
                });
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 3. 错误恢复策略

```rust
pub struct ResilientHandler {
    retry_count: usize,
    retry_delay: Duration,
}

impl ResilientHandler {
    async fn process_with_retry(&self, event: &Event) -> ForgeResult<()> {
        let mut attempts = 0;

        loop {
            match self.try_process(event).await {
                Ok(()) => return Ok(()),
                Err(e) if attempts < self.retry_count => {
                    attempts += 1;
                    warn!("处理失败，重试 {}/{}: {}", attempts, self.retry_count, e);
                    tokio::time::sleep(self.retry_delay).await;
                }
                Err(e) => {
                    error!("处理最终失败: {}", e);
                    return Err(e);
                }
            }
        }
    }
}
```

### 4. 测试事件处理器

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mf_test::create_test_state;

    #[tokio::test]
    async fn test_search_index_handler() {
        // 创建测试处理器
        let handler = SearchIndexHandler::new_for_test();

        // 创建测试事件
        let state = create_test_state();
        let event = Event::Create(Arc::new(state));

        // 测试处理
        let result = handler.handle(&event).await;
        assert!(result.is_ok());

        // 验证索引更新
        let index_count = handler.get_index_count().await;
        assert!(index_count > 0);
    }

    #[tokio::test]
    async fn test_event_bus_performance() {
        let event_bus = EventBus::new();
        event_bus.start_event_loop();

        // 添加测试处理器
        let handler = Arc::new(TestHandler::new());
        event_bus.add_event_handler(handler)?;

        // 批量发送事件
        for i in 0..1000 {
            event_bus.broadcast(Event::test(i)).await?;
        }

        // 等待处理完成
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 验证统计
        let stats = event_bus.get_stats();
        assert_eq!(
            stats.events_processed.load(Ordering::Relaxed),
            1000
        );
    }
}
```

## 集成示例

### 完整的事件系统设置

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeOptions};
use price_budget::bootstrap::DefaultEventHandlerProvider;

async fn setup_event_system() -> ForgeResult<()> {
    // 1. 创建事件总线
    let event_bus = EventBus::with_config(EventConfig {
        max_queue_size: 100000,
        max_concurrent_handlers: 500,
        handler_timeout: Duration::from_secs(120),
    });

    // 2. 启动事件循环
    event_bus.start_event_loop();

    // 3. 获取项目配置
    let profile = ProjectProfile {
        phase: ProjectPhase::Budget,
        project_id: "project_001".to_string(),
        // 其他配置...
    };

    // 4. 使用提供器获取处理器
    let handler_provider = DefaultEventHandlerProvider;
    let handlers = handler_provider.provide(&profile).await?;

    // 5. 注册所有处理器
    event_bus.add_event_handlers(handlers)?;

    // 6. 构建运行时
    let runtime = ForgeRuntimeBuilder::new()
        .with_event_bus(event_bus)
        .build()
        .await?;

    Ok(())
}
```

## 性能优化

### 1. 批量事件处理

```rust
pub struct BatchingHandler {
    batch_size: usize,
    batch_timeout: Duration,
    pending: Arc<Mutex<Vec<Event>>>,
}

impl BatchingHandler {
    async fn process_batch(&self, events: Vec<Event>) {
        // 批量处理多个事件，减少开销
        info!("批量处理 {} 个事件", events.len());
        // 实际批量处理逻辑...
    }

    async fn start_batch_processor(&self) {
        let pending = self.pending.clone();
        let batch_size = self.batch_size;
        let batch_timeout = self.batch_timeout;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(batch_timeout).await;

                let mut events = pending.lock().unwrap();
                if events.len() >= batch_size || !events.is_empty() {
                    let batch = events.drain(..).collect::<Vec<_>>();
                    drop(events);

                    self.process_batch(batch).await;
                }
            }
        });
    }
}
```

### 2. 事件过滤优化

```rust
pub struct FilteredHandler {
    event_filter: Box<dyn Fn(&Event) -> bool + Send + Sync>,
}

impl FilteredHandler {
    pub fn new<F>(filter: F) -> Self
    where
        F: Fn(&Event) -> bool + Send + Sync + 'static,
    {
        Self {
            event_filter: Box::new(filter),
        }
    }
}

#[async_trait::async_trait]
impl EventHandler<Event> for FilteredHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        // 快速过滤，减少不必要的处理
        if !(self.event_filter)(event) {
            return Ok(());
        }

        // 实际处理逻辑...
        Ok(())
    }
}
```

## 总结

ModuForge-RS 的事件系统提供了：

- **完整的事件类型**：涵盖文档生命周期所有阶段
- **高性能事件总线**：无锁读取、并发处理、性能监控
- **灵活的处理器模式**：易于扩展和定制
- **业务集成示例**：Price-RS 展示了搜索索引、持久化、审计等实际应用
- **性能优化策略**：批处理、过滤、异步处理等优化技术

通过事件系统，您可以构建响应式、解耦的应用架构，轻松实现日志记录、持久化、搜索索引、业务逻辑触发等功能。
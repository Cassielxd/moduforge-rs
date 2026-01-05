# moduforge-persistence 文档

`moduforge-persistence` 提供 ModuForge-RS 的持久化抽象层，包括事件溯源、快照管理、SQLite 存储后端和自动恢复机制。

## 概述

Persistence 层实现了基于事件溯源（Event Sourcing）的持久化架构，支持可调一致性选项、增量快照、压缩存储和故障恢复。通过抽象的 `EventStore` trait，可以轻松切换不同的存储后端。

## 核心功能

- **事件溯源架构**：所有状态变更以事件形式持久化
- **快照管理**：自动快照策略，加速恢复
- **可调一致性**：内存模式、异步持久、同步持久三种模式
- **SQLite 后端**：默认使用 SQLite WAL 模式，高性能且可靠
- **自动恢复**：启动时自动恢复到最新状态
- **幂等性保证**：基于幂等键去重，支持安全重试
- **压缩存储**：使用 Zstandard 压缩事件和快照
- **订阅者模式**：通过事件总线自动持久化

## 架构设计

### 持久化流程

```
事务执行 → 事件序列化 → 压缩 → 写入 WAL → 快照检查 → 返回
    ↓                                              ↓
订阅者监听                                    定期快照
    ↓                                              ↓
事件总线                                      状态序列化
```

### 恢复流程

```
加载快照 → 重放增量事件 → 恢复完整状态
    ↓            ↓              ↓
最新快照    快照后的事件    最终状态
```

## 一致性模式

### CommitMode 枚举

```rust
use mf_persistence::api::CommitMode;

/// 写入持久性与延迟之间的权衡
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CommitMode {
    /// 仅内存队列，最低延迟、无持久性（开发模式）
    MemoryOnly,

    /// 异步持久化，写入 WAL 后返回，fsync 由后台批处理
    /// 平衡延迟和持久性（默认推荐）
    AsyncDurable {
        group_window_ms: u32  // fsync 分组窗口（毫秒）
    },

    /// 同步持久化，强制 fsync 后返回，最高持久性
    SyncDurable,
}
```

### 使用示例

```rust
use mf_persistence::api::{CommitMode, PersistOptions};

// 开发环境：纯内存模式
let dev_options = PersistOptions {
    commit_mode: CommitMode::MemoryOnly,
    snapshot_every_n_events: 100,
    snapshot_every_bytes: 10_000_000,
    snapshot_every_ms: 60_000,
    compression: false,
};

// 生产环境：异步持久化
let prod_options = PersistOptions {
    commit_mode: CommitMode::AsyncDurable {
        group_window_ms: 10  // 10ms 分组
    },
    snapshot_every_n_events: 1000,
    snapshot_every_bytes: 100_000_000,  // 100MB
    snapshot_every_ms: 300_000,  // 5 分钟
    compression: true,
};

// 关键业务：同步持久化
let critical_options = PersistOptions {
    commit_mode: CommitMode::SyncDurable,
    snapshot_every_n_events: 500,
    snapshot_every_bytes: 50_000_000,
    snapshot_every_ms: 180_000,
    compression: true,
};
```

## EventStore API

### 核心 Trait

```rust
use async_trait::async_trait;
use mf_persistence::api::{EventStore, PersistedEvent, Snapshot};

#[async_trait]
pub trait EventStore: Send + Sync {
    /// 追加单条事件，返回分配的日志序号 (LSN)
    async fn append(&self, ev: PersistedEvent) -> anyhow::Result<i64>;

    /// 批量追加事件（原子事务）
    async fn append_batch(&self, evs: Vec<PersistedEvent>) -> anyhow::Result<i64>;

    /// 读取指定文档从某 LSN 后的增量事件
    async fn load_since(
        &self,
        doc_id: &str,
        from_lsn: i64,
        limit: u32,
    ) -> anyhow::Result<Vec<PersistedEvent>>;

    /// 获取最新快照
    async fn latest_snapshot(&self, doc_id: &str) -> anyhow::Result<Option<Snapshot>>;

    /// 写入快照（原子替换）
    async fn write_snapshot(&self, snap: Snapshot) -> anyhow::Result<()>;

    /// 压缩历史事件（删除被快照覆盖的事件）
    async fn compact(&self, doc_id: &str) -> anyhow::Result<()>;
}
```

### PersistedEvent 结构

```rust
/// 事件存储的仅追加记录
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistedEvent {
    pub lsn: i64,                    // 日志序号（单调递增）
    pub tr_id: u64,                  // 事务 ID
    pub doc_id: String,              // 文档 ID
    pub ts: i64,                     // 时间戳
    pub actor: Option<String>,       // 执行者
    pub idempotency_key: String,    // 幂等键（全局唯一）
    pub payload: Vec<u8>,            // 压缩的事件数据
    pub meta: serde_json::Value,     // 元数据
    pub checksum: u32,               // CRC32 校验和
}
```

### Snapshot 结构

```rust
/// 状态快照
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub doc_id: String,         // 文档 ID
    pub upto_lsn: i64,         // 快照截止的 LSN
    pub created_at: i64,       // 创建时间戳
    pub state_blob: Vec<u8>,   // 压缩的状态数据
    pub version: i32,          // 快照版本
}
```

## SQLite 后端

### 初始化

```rust
use mf_persistence::sqlite::SqliteEventStore;
use mf_persistence::api::CommitMode;

// 创建 SQLite 存储
let store = SqliteEventStore::open(
    "./data",  // 数据目录
    CommitMode::AsyncDurable { group_window_ms: 10 }
).await?;

// SQLite 配置（自动应用）：
// - WAL 模式：并发读写
// - NORMAL 同步：平衡性能和安全
// - 64MB 缓存：提高查询性能
// - 内存临时存储：减少磁盘 IO
```

### 表结构

```sql
-- 事件表
CREATE TABLE events (
    lsn INTEGER PRIMARY KEY AUTOINCREMENT,
    tr_id INTEGER NOT NULL,
    doc_id TEXT NOT NULL,
    ts INTEGER NOT NULL,
    actor TEXT,
    idempotency_key TEXT NOT NULL UNIQUE,
    meta TEXT NOT NULL,
    payload BLOB NOT NULL,
    checksum INTEGER NOT NULL
);
CREATE INDEX ix_events_doc_lsn ON events(doc_id, lsn);

-- 快照表
CREATE TABLE snapshots (
    doc_id TEXT NOT NULL,
    upto_lsn INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    state_blob BLOB NOT NULL,
    version INTEGER NOT NULL,
    PRIMARY KEY (doc_id, upto_lsn)
);
CREATE INDEX ix_snapshots_doc_created ON snapshots(doc_id, created_at DESC);
```

## 事件订阅者

### SnapshotSubscriber

```rust
use mf_persistence::subscriber::SnapshotSubscriber;
use mf_core::event::{EventBus, EventHandler};
use std::sync::Arc;

// 创建快照订阅者
let subscriber = SnapshotSubscriber::new(
    Arc::new(store),
    persist_options,
    "default_doc"  // 默认文档 ID
);

// 注册到事件总线
let event_bus = EventBus::new();
event_bus.subscribe(Box::new(subscriber));

// 订阅者自动处理：
// - TrApply 事件：持久化事务
// - TrUndo 事件：持久化撤销
// - TrRedo 事件：持久化重做
// - 自动触发快照
```

### 快照策略

```rust
impl SnapshotSubscriber {
    /// 根据策略检查是否需要快照
    fn should_snapshot(&self, counters: &SnapshotCounters) -> bool {
        let now = chrono::Utc::now().timestamp_millis();

        // 事件数量阈值
        if counters.events_since >= self.options.snapshot_every_n_events {
            return true;
        }

        // 数据量阈值
        if counters.bytes_since >= self.options.snapshot_every_bytes {
            return true;
        }

        // 时间间隔阈值
        if now - counters.last_snapshot_ms >= self.options.snapshot_every_ms as i64 {
            return true;
        }

        false
    }
}
```

## 状态恢复

### 恢复函数

```rust
use mf_persistence::recovery::recover_state;
use mf_persistence::step_factory::StepFactoryRegistry;

/// 从存储恢复状态
pub async fn recover_state<E: EventStore>(
    store: &E,
    doc_id: &str,
    configuration: &mf_state::Configuration,
    step_factory: &StepFactoryRegistry,
    batch: u32,  // 批量加载大小
) -> anyhow::Result<Arc<mf_state::State>> {
    // 1. 加载最新快照
    let mut state = if let Some(snap) = store.latest_snapshot(doc_id).await? {
        // 解压快照
        let bytes = zstd::decode_all(snap.state_blob)?;
        let snap_data: SnapshotData = serde_json::from_slice(&bytes)?;

        // 反序列化状态
        let ser = StateSerialize {
            node_pool: snap_data.node_pool,
            state_fields: snap_data.state_fields,
        };
        Arc::new(State::deserialize(&ser, configuration).await?)
    } else {
        // 无快照，创建空状态
        Arc::new(State::new(Arc::new(configuration.clone()))?)
    };

    // 2. 重放增量事件
    let mut from_lsn = snap.map(|s| s.upto_lsn).unwrap_or(0);
    loop {
        let events = store.load_since(doc_id, from_lsn, batch).await?;
        if events.is_empty() {
            break;
        }

        for event in events {
            // 解压事件
            let payload = zstd::decode_all(event.payload)?;
            let frames: Vec<TypeWrapper> = serde_json::from_slice(&payload)?;

            // 重建事务
            let mut tr = Transaction::new(&state);
            for frame in frames {
                tr.step(step_factory.create(&frame.type_id, &frame.data)?);
            }

            // 应用事务
            state = state.apply(tr).await?.state;
            from_lsn = event.lsn;
        }
    }

    Ok(state)
}
```

## StepFactory 注册

### StepFactoryRegistry

```rust
use mf_persistence::step_factory::{StepFactoryRegistry, StepFactory};
use mf_transform::Step;

/// 步骤工厂注册表
pub struct StepFactoryRegistry {
    factories: HashMap<String, Box<dyn StepFactory>>,
}

impl StepFactoryRegistry {
    /// 注册步骤工厂
    pub fn register<F: StepFactory + 'static>(&mut self, type_id: &str, factory: F) {
        self.factories.insert(type_id.to_string(), Box::new(factory));
    }

    /// 创建步骤实例
    pub fn create(&self, type_id: &str, data: &[u8]) -> Result<Box<dyn Step>> {
        let factory = self.factories.get(type_id)
            .ok_or_else(|| anyhow!("Unknown step type: {}", type_id))?;
        factory.create(data)
    }
}

// 实现具体的步骤工厂
struct InsertNodeFactory;

impl StepFactory for InsertNodeFactory {
    fn create(&self, data: &[u8]) -> Result<Box<dyn Step>> {
        let params: InsertParams = serde_json::from_slice(data)?;
        Ok(Box::new(InsertNodeStep::new(params)))
    }
}

// 注册所有步骤类型
let mut registry = StepFactoryRegistry::new();
registry.register("insert_node", InsertNodeFactory);
registry.register("delete_node", DeleteNodeFactory);
registry.register("update_attrs", UpdateAttrsFactory);
```

## Price-RS 集成示例

### 持久化配置

```rust
use mf_persistence::{
    api::{CommitMode, PersistOptions},
    sqlite::SqliteEventStore,
    subscriber::SnapshotSubscriber,
};
use price_rs::PriceRuntime;

/// 配置 Price-RS 持久化
pub async fn setup_persistence(runtime: &mut PriceRuntime) -> Result<()> {
    // 1. 创建存储后端
    let store = SqliteEventStore::open(
        "./price_data",
        CommitMode::AsyncDurable { group_window_ms: 10 }
    ).await?;

    // 2. 配置持久化选项
    let options = PersistOptions {
        commit_mode: CommitMode::AsyncDurable { group_window_ms: 10 },
        snapshot_every_n_events: 500,      // 每 500 个事件快照
        snapshot_every_bytes: 50_000_000,  // 每 50MB 快照
        snapshot_every_ms: 300_000,        // 每 5 分钟快照
        compression: true,
    };

    // 3. 创建订阅者
    let subscriber = SnapshotSubscriber::new(
        store.clone(),
        options,
        "price_project"
    );

    // 4. 注册到事件总线
    runtime.event_bus.subscribe(Box::new(subscriber));

    // 5. 恢复状态（如果存在）
    if let Ok(state) = recover_state(
        &*store,
        "price_project",
        &runtime.configuration,
        &runtime.step_factory,
        100
    ).await {
        runtime.state = state;
        println!("已恢复项目状态");
    }

    Ok(())
}
```

### 事务持久化

```rust
/// Price-RS 事务处理与持久化
pub async fn execute_price_command(
    runtime: &PriceRuntime,
    command: PriceCommand
) -> Result<()> {
    // 创建事务
    let mut tr = runtime.state.tr();

    // 执行命令
    match command {
        PriceCommand::InsertNode { id, price, unit } => {
            tr.insert_node(id, PriceNode::new(price, unit));
        }
        PriceCommand::UpdatePrice { id, new_price } => {
            tr.update_attrs(id, hashmap!{
                "price" => new_price.into()
            });
        }
        PriceCommand::DeleteNode { id } => {
            tr.delete_node(id);
        }
    }

    // 应用事务（自动触发持久化）
    let result = runtime.state.apply(tr).await?;
    runtime.state = result.state;

    // 事件总线会自动：
    // 1. 序列化事务步骤
    // 2. 压缩数据
    // 3. 写入 SQLite
    // 4. 检查快照策略
    // 5. 必要时创建快照

    Ok(())
}
```

## 性能优化

### 1. WAL 模式优化

```rust
// SQLite WAL 模式配置
PRAGMA journal_mode = WAL;        // 并发读写
PRAGMA synchronous = NORMAL;      // 平衡性能
PRAGMA cache_size = -64000;       // 64MB 缓存
PRAGMA temp_store = MEMORY;       // 内存临时表
PRAGMA wal_autocheckpoint = 1000; // 自动检查点
```

### 2. 批量操作

```rust
// 批量追加事件（单事务）
let events = vec![event1, event2, event3];
store.append_batch(events).await?;  // 原子操作

// 批量加载（分页）
let batch_size = 1000;
let events = store.load_since(doc_id, from_lsn, batch_size).await?;
```

### 3. 压缩策略

```rust
// Zstandard 压缩配置
fn compress_if_needed(data: &[u8], compress: bool) -> Vec<u8> {
    if compress && data.len() > 1024 {  // 大于 1KB 才压缩
        zstd::encode_all(data, 3).unwrap_or_else(|_| data.to_vec())
    } else {
        data.to_vec()
    }
}
```

### 4. 快照优化

```rust
// 智能快照策略
impl SnapshotStrategy {
    fn optimize(&mut self) {
        // 根据负载动态调整
        if self.event_rate > 100 {  // 高频事件
            self.snapshot_every_n_events = 200;
        } else {  // 低频事件
            self.snapshot_every_n_events = 1000;
        }

        // 根据数据量调整
        if self.avg_event_size > 10_000 {  // 大事件
            self.snapshot_every_bytes = 10_000_000;
        } else {  // 小事件
            self.snapshot_every_bytes = 100_000_000;
        }
    }
}
```

## 错误处理

```rust
use mf_persistence::error::{PersistenceError, Result};

// 处理持久化错误
match store.append(event).await {
    Ok(lsn) => {
        println!("事件已持久化，LSN: {}", lsn);
    }
    Err(e) => {
        match e.downcast_ref::<PersistenceError>() {
            Some(PersistenceError::DuplicateKey(key)) => {
                // 幂等键重复，说明已处理
                println!("事件已存在: {}", key);
            }
            Some(PersistenceError::StorageFull) => {
                // 存储空间不足
                trigger_alert("磁盘空间不足");
            }
            Some(PersistenceError::Corrupted(msg)) => {
                // 数据损坏，需要恢复
                println!("数据损坏: {}", msg);
                recover_from_backup().await?;
            }
            _ => {
                // 其他错误
                return Err(e);
            }
        }
    }
}
```

## 监控与诊断

### 性能指标

```rust
/// 持久化性能监控
pub struct PersistenceMetrics {
    pub total_events: u64,           // 总事件数
    pub total_snapshots: u64,        // 总快照数
    pub avg_event_size: usize,       // 平均事件大小
    pub avg_snapshot_size: usize,    // 平均快照大小
    pub compression_ratio: f32,      // 压缩率
    pub write_latency_ms: f64,       // 写入延迟
    pub recovery_time_ms: f64,       // 恢复时间
}

impl PersistenceMetrics {
    pub fn report(&self) {
        println!("=== 持久化指标 ===");
        println!("事件总数: {}", self.total_events);
        println!("快照总数: {}", self.total_snapshots);
        println!("压缩率: {:.2}%", self.compression_ratio * 100.0);
        println!("平均写入延迟: {:.2}ms", self.write_latency_ms);
        println!("恢复时间: {:.2}ms", self.recovery_time_ms);
    }
}
```

### 健康检查

```rust
/// 存储健康检查
pub async fn check_storage_health(store: &dyn EventStore) -> HealthStatus {
    // 检查连接
    if let Err(e) = store.load_since("_health", 0, 1).await {
        return HealthStatus::Unhealthy(format!("连接失败: {}", e));
    }

    // 检查磁盘空间
    let free_space = get_free_disk_space();
    if free_space < 100_000_000 {  // 小于 100MB
        return HealthStatus::Warning("磁盘空间不足");
    }

    // 检查数据完整性
    if let Some(corruption) = check_data_integrity().await {
        return HealthStatus::Critical(format!("数据损坏: {}", corruption));
    }

    HealthStatus::Healthy
}
```

## 最佳实践

### 1. 选择合适的一致性模式

```rust
// 开发环境：快速迭代
CommitMode::MemoryOnly

// 普通业务：平衡性能
CommitMode::AsyncDurable { group_window_ms: 10 }

// 金融业务：强一致性
CommitMode::SyncDurable
```

### 2. 合理配置快照策略

```rust
// 高频小事件：频繁快照
snapshot_every_n_events: 100
snapshot_every_bytes: 10_000_000

// 低频大事件：减少快照
snapshot_every_n_events: 1000
snapshot_every_bytes: 100_000_000
```

### 3. 幂等性设计

```rust
// 使用唯一的幂等键
let idempotency_key = format!("{}-{}-{}",
    user_id,
    operation_id,
    timestamp
);

// 重试安全
for attempt in 0..3 {
    match persist_with_key(idempotency_key.clone()).await {
        Ok(_) => break,
        Err(DuplicateKey) => break,  // 已成功
        Err(e) if attempt < 2 => continue,  // 重试
        Err(e) => return Err(e),  // 失败
    }
}
```

## 下一步

- 查看 [moduforge-state](./state.md) 了解状态管理
- 查看 [moduforge-file](./file.md) 了解文件存储格式
- 查看 [moduforge-collaboration](./collaboration.md) 了解协作功能
- 浏览 [Price-RS 项目](https://github.com/LiRenTech/price-rs) 查看实际应用
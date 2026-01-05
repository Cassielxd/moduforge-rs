# 性能优化指南

ModuForge-RS 的设计充分考虑了性能，通过不可变数据结构、结构共享和异步处理等技术实现高性能文档编辑。本指南将帮助你优化 ModuForge-RS 应用的性能。

## 性能特征

### 核心性能指标

| 操作 | 时间复杂度 | 空间复杂度 | 说明 |
|-----|-----------|-----------|------|
| 节点访问 | O(1) | O(1) | 通过 ID 直接访问 |
| 节点插入 | O(log n) | O(log n) | 不可变树操作 |
| 节点删除 | O(log n) | O(log n) | 结构共享 |
| 属性更新 | O(log m) | O(1) | m 为属性数量 |
| 文档遍历 | O(n) | O(h) | n 为节点数，h 为树高 |
| 事务应用 | O(s) | O(s) | s 为步骤数量 |
| 撤销/重做 | O(s) | O(1) | s 为步骤数量 |

### 基准测试结果

```rust
// 10,000 节点文档的操作性能
插入节点: 0.12ms
更新属性: 0.08ms
删除节点: 0.15ms
遍历文档: 2.3ms
应用事务 (10 步骤): 1.1ms
序列化到 JSON: 8.5ms
```

## 内存优化

### 1. 结构共享

ModuForge-RS 使用持久化数据结构实现结构共享：

```rust
use im::{HashMap, Vector};
use std::sync::Arc;

/// 不可变节点池，实现结构共享
pub struct NodePool {
    nodes: HashMap<NodeId, Arc<Node>>,
    roots: Vector<NodeId>,
}

impl NodePool {
    /// 添加节点（创建新版本）
    pub fn add_node(&self, node: Node) -> Self {
        // 只复制修改的部分，未修改的部分共享
        let mut new_nodes = self.nodes.clone();
        new_nodes.insert(node.id.clone(), Arc::new(node));

        NodePool {
            nodes: new_nodes,
            roots: self.roots.clone(),
        }
    }

    /// 内存效率：仅存储差异
    pub fn memory_usage(&self) -> usize {
        // 实际内存使用远小于完整复制
        std::mem::size_of_val(&self.nodes) +
        self.nodes.len() * std::mem::size_of::<Arc<Node>>()
    }
}
```

### 2. 懒加载策略

对于大文档，实现按需加载：

```rust
/// 懒加载文档
pub struct LazyDocument {
    /// 已加载的块
    loaded_chunks: DashMap<ChunkId, Arc<NodeChunk>>,
    /// 块元数据
    chunk_metadata: Vec<ChunkMeta>,
    /// 加载策略
    loader: Box<dyn ChunkLoader>,
}

impl LazyDocument {
    /// 获取节点（按需加载）
    pub async fn get_node(&self, node_id: &NodeId) -> ForgeResult<Arc<Node>> {
        let chunk_id = self.find_chunk_id(node_id);

        // 检查是否已加载
        if let Some(chunk) = self.loaded_chunks.get(&chunk_id) {
            return chunk.get_node(node_id);
        }

        // 异步加载块
        let chunk = self.loader.load_chunk(chunk_id).await?;
        self.loaded_chunks.insert(chunk_id, chunk.clone());

        chunk.get_node(node_id)
    }

    /// 预加载策略
    pub async fn preload_visible(&self, viewport: Range<usize>) {
        let chunks_to_load = self.find_chunks_in_range(viewport);

        // 并发加载多个块
        let futures = chunks_to_load
            .into_iter()
            .filter(|id| !self.loaded_chunks.contains_key(id))
            .map(|id| self.loader.load_chunk(id));

        futures::future::join_all(futures).await;
    }

    /// 内存回收
    pub fn evict_unused_chunks(&self, max_memory: usize) {
        let current_memory = self.estimate_memory_usage();

        if current_memory > max_memory {
            // LRU 淘汰策略
            self.evict_least_recently_used(current_memory - max_memory);
        }
    }
}
```

### 3. 对象池

重用频繁创建的对象：

```rust
use crossbeam::queue::ArrayQueue;

/// 对象池，减少内存分配
pub struct ObjectPool<T> {
    pool: Arc<ArrayQueue<T>>,
    factory: Box<dyn Fn() -> T>,
}

impl<T> ObjectPool<T> {
    pub fn new(capacity: usize, factory: impl Fn() -> T + 'static) -> Self {
        Self {
            pool: Arc::new(ArrayQueue::new(capacity)),
            factory: Box::new(factory),
        }
    }

    /// 借用对象
    pub fn acquire(&self) -> PooledObject<T> {
        let obj = self.pool.pop().unwrap_or_else(|| (self.factory)());
        PooledObject {
            inner: Some(obj),
            pool: self.pool.clone(),
        }
    }
}

/// 自动归还的池化对象
pub struct PooledObject<T> {
    inner: Option<T>,
    pool: Arc<ArrayQueue<T>>,
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.inner.take() {
            // 归还到池中
            let _ = self.pool.push(obj);
        }
    }
}

// 使用示例
lazy_static! {
    static ref TRANSACTION_POOL: ObjectPool<Transaction> =
        ObjectPool::new(100, || Transaction::new());
}
```

## 计算优化

### 1. 批量操作

合并多个操作减少开销：

```rust
/// 批量操作构建器
pub struct BatchOperationBuilder {
    operations: Vec<Operation>,
}

impl BatchOperationBuilder {
    /// 添加操作
    pub fn add(&mut self, op: Operation) -> &mut Self {
        // 尝试合并相邻操作
        if let Some(last) = self.operations.last_mut() {
            if let Some(merged) = last.try_merge(&op) {
                *last = merged;
                return self;
            }
        }
        self.operations.push(op);
        self
    }

    /// 执行批量操作
    pub async fn execute(self, state: State) -> ForgeResult<State> {
        // 单个事务执行所有操作
        let mut tr = state.tr();

        for op in self.operations {
            op.apply(&mut tr)?;
        }

        state.apply(tr)
    }
}

// 使用示例
let batch = BatchOperationBuilder::new()
    .add(InsertNode { ... })
    .add(UpdateAttrs { ... })
    .add(MoveNode { ... })
    .execute(state)
    .await?;
```

### 2. 并行处理

利用多核 CPU 加速处理：

```rust
use rayon::prelude::*;
use tokio::task;

/// 并行文档处理
pub struct ParallelProcessor;

impl ParallelProcessor {
    /// 并行遍历文档树
    pub fn parallel_traverse<F, R>(&self, doc: &NodePool, f: F) -> Vec<R>
    where
        F: Fn(&Node) -> R + Send + Sync,
        R: Send,
    {
        doc.nodes
            .par_iter()
            .map(|(_, node)| f(node))
            .collect()
    }

    /// 并行验证
    pub async fn parallel_validate(
        &self,
        nodes: Vec<NodeId>,
        schema: Arc<Schema>
    ) -> Vec<ValidationResult> {
        let chunks: Vec<Vec<NodeId>> = nodes
            .chunks(100)
            .map(|chunk| chunk.to_vec())
            .collect();

        // 创建并发任务
        let futures = chunks.into_iter().map(|chunk| {
            let schema = schema.clone();
            task::spawn(async move {
                chunk.iter()
                    .map(|id| schema.validate(id))
                    .collect::<Vec<_>>()
            })
        });

        // 等待所有任务完成
        let results = futures::future::join_all(futures).await;
        results.into_iter().flatten().flatten().collect()
    }
}
```

### 3. 缓存策略

实现多级缓存：

```rust
use lru::LruCache;
use std::num::NonZeroUsize;

/// 多级缓存系统
pub struct CacheSystem {
    /// L1 缓存：热点数据
    l1_cache: Arc<RwLock<LruCache<CacheKey, CacheValue>>>,
    /// L2 缓存：次热数据
    l2_cache: Arc<RwLock<LruCache<CacheKey, CacheValue>>>,
    /// 缓存统计
    stats: Arc<CacheStats>,
}

impl CacheSystem {
    pub fn new(l1_size: usize, l2_size: usize) -> Self {
        Self {
            l1_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(l1_size).unwrap())
            )),
            l2_cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(l2_size).unwrap())
            )),
            stats: Arc::new(CacheStats::default()),
        }
    }

    /// 获取缓存值
    pub fn get(&self, key: &CacheKey) -> Option<CacheValue> {
        // 尝试 L1 缓存
        if let Some(value) = self.l1_cache.write().get(key) {
            self.stats.l1_hits.fetch_add(1, Ordering::Relaxed);
            return Some(value.clone());
        }

        // 尝试 L2 缓存
        if let Some(value) = self.l2_cache.write().get(key) {
            self.stats.l2_hits.fetch_add(1, Ordering::Relaxed);
            // 提升到 L1
            self.l1_cache.write().put(key.clone(), value.clone());
            return Some(value.clone());
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// 缓存预热
    pub async fn warm_up(&self, keys: Vec<CacheKey>) {
        let futures = keys.into_iter().map(|key| {
            async move {
                if let Some(value) = self.compute_value(&key).await {
                    self.put(key, value);
                }
            }
        });

        futures::future::join_all(futures).await;
    }
}
```

## I/O 优化

### 1. 异步 I/O

使用异步 I/O 避免阻塞：

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;

/// 异步文档持久化
pub struct AsyncPersistence;

impl AsyncPersistence {
    /// 异步保存文档
    pub async fn save_document(
        &self,
        doc: &NodePool,
        path: &Path
    ) -> ForgeResult<()> {
        // 使用缓冲写入
        let mut file = File::create(path).await?;
        let mut buffer = Vec::with_capacity(1024 * 1024); // 1MB 缓冲

        // 流式序列化
        let mut encoder = JsonEncoder::new(&mut buffer);
        doc.stream_encode(&mut encoder).await?;

        // 异步写入
        file.write_all(&buffer).await?;
        file.flush().await?;

        Ok(())
    }

    /// 异步加载文档（流式）
    pub async fn load_document_stream(
        &self,
        path: &Path
    ) -> ForgeResult<impl Stream<Item = Result<Node>>> {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);

        // 返回节点流
        Ok(JsonStreamDecoder::new(reader)
            .map(|result| result.map_err(Into::into)))
    }
}
```

### 2. 内存映射文件

对于大文件使用内存映射：

```rust
use memmap2::{Mmap, MmapMut};

/// 内存映射文档
pub struct MmapDocument {
    mmap: Mmap,
    index: BTreeMap<NodeId, Range<usize>>,
}

impl MmapDocument {
    /// 创建内存映射文档
    pub fn open(path: &Path) -> ForgeResult<Self> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // 构建索引
        let index = Self::build_index(&mmap)?;

        Ok(Self { mmap, index })
    }

    /// 零拷贝读取节点
    pub fn get_node(&self, node_id: &NodeId) -> ForgeResult<Node> {
        let range = self.index.get(node_id)
            .ok_or(ForgeError::NodeNotFound)?;

        // 直接从内存映射解析
        let data = &self.mmap[range.clone()];
        bincode::deserialize(data).map_err(Into::into)
    }

    /// 预取数据到内存
    pub fn prefetch(&self, node_ids: &[NodeId]) {
        for node_id in node_ids {
            if let Some(range) = self.index.get(node_id) {
                // 触发页面错误，加载到内存
                let _ = self.mmap[range.clone()].len();
            }
        }
    }
}
```

## 渲染优化

### 1. 虚拟滚动

只渲染可见部分：

```typescript
// React 虚拟滚动实现
import { VariableSizeList } from 'react-window';

function DocumentRenderer({ document }) {
  const getItemSize = (index) => {
    // 动态计算每行高度
    return document.lines[index].height;
  };

  const Row = ({ index, style }) => {
    const line = document.lines[index];
    return (
      <div style={style}>
        <LineRenderer line={line} />
      </div>
    );
  };

  return (
    <VariableSizeList
      height={800}
      itemCount={document.lines.length}
      itemSize={getItemSize}
      width="100%"
      overscanCount={5} // 预渲染行数
    >
      {Row}
    </VariableSizeList>
  );
}
```

### 2. 增量更新

只更新变化的部分：

```rust
/// 增量更新追踪
pub struct IncrementalUpdater {
    last_state: Arc<State>,
    dirty_nodes: DashSet<NodeId>,
}

impl IncrementalUpdater {
    /// 计算增量更新
    pub fn compute_delta(
        &mut self,
        new_state: Arc<State>
    ) -> Vec<UpdatePatch> {
        let mut patches = Vec::new();

        // 比较状态差异
        for (node_id, new_node) in new_state.doc.nodes.iter() {
            if let Some(old_node) = self.last_state.doc.nodes.get(node_id) {
                if !Arc::ptr_eq(old_node, new_node) {
                    // 节点已修改
                    patches.push(UpdatePatch::NodeUpdate {
                        id: node_id.clone(),
                        node: new_node.clone(),
                    });
                    self.dirty_nodes.insert(node_id.clone());
                }
            } else {
                // 新增节点
                patches.push(UpdatePatch::NodeInsert {
                    id: node_id.clone(),
                    node: new_node.clone(),
                });
            }
        }

        // 检查删除的节点
        for (node_id, _) in self.last_state.doc.nodes.iter() {
            if !new_state.doc.nodes.contains_key(node_id) {
                patches.push(UpdatePatch::NodeDelete {
                    id: node_id.clone(),
                });
            }
        }

        self.last_state = new_state;
        patches
    }

    /// 应用增量更新到 UI
    pub fn apply_patches(&self, patches: Vec<UpdatePatch>) {
        for patch in patches {
            match patch {
                UpdatePatch::NodeUpdate { id, node } => {
                    // 只更新特定节点的 UI
                    ui::update_node(id, node);
                }
                UpdatePatch::NodeInsert { id, node } => {
                    ui::insert_node(id, node);
                }
                UpdatePatch::NodeDelete { id } => {
                    ui::remove_node(id);
                }
            }
        }
    }
}
```

### 3. Web Worker 处理

将计算密集型任务移到 Worker：

```typescript
// worker.ts
self.addEventListener('message', async (event) => {
  const { type, payload } = event.data;

  switch (type) {
    case 'PARSE_MARKDOWN':
      const parsed = await parseMarkdown(payload);
      self.postMessage({ type: 'PARSED', result: parsed });
      break;

    case 'VALIDATE_DOCUMENT':
      const validation = await validateDocument(payload);
      self.postMessage({ type: 'VALIDATED', result: validation });
      break;

    case 'COMPUTE_DIFF':
      const diff = await computeDiff(payload.old, payload.new);
      self.postMessage({ type: 'DIFF_COMPUTED', result: diff });
      break;
  }
});

// main.ts
class WorkerPool {
  private workers: Worker[] = [];
  private taskQueue: Task[] = [];

  constructor(size: number) {
    for (let i = 0; i < size; i++) {
      this.workers.push(new Worker('/worker.js'));
    }
  }

  async execute(task: Task): Promise<any> {
    const worker = this.getAvailableWorker();
    return new Promise((resolve) => {
      worker.postMessage(task);
      worker.onmessage = (event) => {
        resolve(event.data.result);
        this.releaseWorker(worker);
      };
    });
  }
}
```

## 网络优化

### 1. 请求合并

减少网络往返：

```rust
/// 请求批处理器
pub struct RequestBatcher {
    pending: Arc<RwLock<Vec<Request>>>,
    interval: Duration,
}

impl RequestBatcher {
    /// 添加请求到批次
    pub async fn add_request(&self, request: Request) -> ResponseFuture {
        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending.write().await;
            pending.push(RequestWithCallback {
                request,
                callback: tx,
            });
        }

        // 返回 Future
        ResponseFuture { receiver: rx }
    }

    /// 定期发送批量请求
    pub async fn start_batching(&self) {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;

            let requests = {
                let mut pending = self.pending.write().await;
                std::mem::take(&mut *pending)
            };

            if !requests.is_empty() {
                // 批量发送
                let responses = self.send_batch(requests).await;

                // 分发响应
                for (req, resp) in responses {
                    req.callback.send(resp).ok();
                }
            }
        }
    }
}
```

### 2. 压缩传输

使用压缩减少数据传输：

```rust
use flate2::Compression;
use flate2::write::{GzEncoder, GzDecoder};

/// 压缩中间件
pub struct CompressionMiddleware;

impl CompressionMiddleware {
    /// 压缩数据
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    /// 解压数据
    pub fn decompress(&self, compressed: &[u8]) -> Vec<u8> {
        let mut decoder = GzDecoder::new(Vec::new());
        decoder.write_all(compressed).unwrap();
        decoder.finish().unwrap()
    }

    /// 智能压缩（根据数据大小决定）
    pub fn smart_compress(&self, data: &[u8]) -> CompressedData {
        if data.len() < 1024 {
            // 小数据不压缩
            CompressedData::Raw(data.to_vec())
        } else {
            let compressed = self.compress(data);
            if compressed.len() < data.len() * 0.9 {
                // 压缩有效
                CompressedData::Compressed(compressed)
            } else {
                // 压缩无效
                CompressedData::Raw(data.to_vec())
            }
        }
    }
}
```

## 监控和分析

### 1. 性能监控

```rust
use prometheus::{Histogram, IntCounter, register_histogram, register_int_counter};

lazy_static! {
    static ref OPERATION_DURATION: Histogram = register_histogram!(
        "moduforge_operation_duration_seconds",
        "Operation duration in seconds"
    ).unwrap();

    static ref NODE_COUNT: IntCounter = register_int_counter!(
        "moduforge_node_count",
        "Total number of nodes"
    ).unwrap();
}

/// 性能监控
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// 记录操作时间
    pub fn record_operation<F, R>(name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let timer = OPERATION_DURATION.start_timer();
        let result = f();
        timer.observe_duration();
        result
    }

    /// 追踪内存使用
    pub fn track_memory() {
        let memory = get_memory_usage();
        metrics::gauge!("memory_usage_bytes", memory as f64);
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> PerformanceReport {
        PerformanceReport {
            avg_operation_time: OPERATION_DURATION.get_sample_sum() /
                               OPERATION_DURATION.get_sample_count(),
            total_nodes: NODE_COUNT.get() as usize,
            memory_usage: get_memory_usage(),
            cache_hit_rate: calculate_cache_hit_rate(),
        }
    }
}
```

### 2. 火焰图分析

```bash
# 使用 cargo-flamegraph 生成火焰图
cargo install flamegraph

# 运行性能分析
cargo flamegraph --bin moduforge-benchmark

# 使用 perf 进行更详细的分析
perf record -F 99 -g ./target/release/moduforge-benchmark
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

## 最佳实践

### 1. 文档大小优化

- **分片加载**：将大文档分成多个块
- **懒加载**：只加载可见部分
- **压缩存储**：使用高效的序列化格式

### 2. 操作优化

- **批量操作**：合并多个小操作
- **异步处理**：使用异步 I/O
- **并行计算**：利用多核 CPU

### 3. 内存管理

- **结构共享**：利用不可变数据结构
- **对象池**：重用频繁创建的对象
- **及时清理**：定期清理未使用的缓存

### 4. 网络优化

- **请求合并**：批量发送请求
- **压缩传输**：使用 gzip 压缩
- **缓存策略**：实现多级缓存

## 性能调优检查清单

- [ ] 启用 release 模式编译
- [ ] 使用 PGO (Profile-Guided Optimization)
- [ ] 配置合适的内存限制
- [ ] 实现缓存预热
- [ ] 启用压缩传输
- [ ] 配置连接池
- [ ] 使用虚拟滚动
- [ ] 实现增量更新
- [ ] 启用 Web Workers
- [ ] 配置监控和告警

## 相关资源

- [Rust 性能手册](https://nnethercote.github.io/perf-book/)
- [基准测试结果](../benchmarks/README.md)
- [架构设计](./architecture.md)
- [协作优化](./collaborative-editing.md#性能优化)

## 下一步

- [部署指南](./deployment.md) - 生产环境部署
- [监控配置](./monitoring.md) - 设置性能监控
- [故障排除](./troubleshooting.md) - 性能问题诊断
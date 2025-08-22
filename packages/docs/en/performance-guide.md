# ModuForge-RS 性能优化指南

本指南提供 ModuForge-RS 框架的性能优化策略、最佳实践和调优技巧，帮助您构建高性能的应用。

## 🎯 性能优化概览

### 性能特征

ModuForge-RS 在设计上就考虑了高性能：

- **不可变数据结构**: 基于 `im-rs` 的结构共享
- **零拷贝操作**: 大量使用 `Arc` 和引用
- **异步优先**: 基于 Tokio 的高并发处理
- **内存高效**: 写时复制和惰性计算
- **并发安全**: 无锁数据结构设计

### 性能瓶颈识别

常见的性能瓶颈包括：

1. **过度的状态克隆**
2. **频繁的小事务提交**
3. **插件执行开销**
4. **内存泄漏和过度分配**
5. **I/O 阻塞操作**

## 🔧 核心性能优化

### 1. 状态管理优化

#### ✅ 正确使用不可变数据

```rust
use std::sync::Arc;
use mf_model::{Node, Tree};

// ✅ 推荐：使用 Arc 共享数据
fn share_node_efficiently(node: Arc<Node>) -> Arc<Node> {
    // 直接返回 Arc，无需克隆整个数据
    node
}

// ❌ 避免：不必要的克隆
fn avoid_unnecessary_clone(node: &Node) -> Node {
    // 这会创建完整的数据副本，开销大
    node.clone()
}
```

#### ✅ 批量操作优化

```rust
use mf_transform::{batch_step::BatchStep, node_step::AddNodeStep};

// ✅ 推荐：批量操作
async fn batch_add_nodes(runtime: &ForgeAsyncRuntime, nodes: Vec<Node>) -> Result<()> {
    let mut batch = BatchStep::new(vec![]);
    
    for node in nodes {
        batch.add_step(Box::new(AddNodeStep::new_single(node, None)));
    }
    
    let mut transaction = runtime.get_state().tr();
    transaction.add_step(Box::new(batch));
    
    // 单次事务提交，减少开销
    runtime.dispatch_flow(transaction).await
}

// ❌ 避免：多次小事务
async fn avoid_multiple_transactions(runtime: &ForgeAsyncRuntime, nodes: Vec<Node>) -> Result<()> {
    for node in nodes {
        let mut transaction = runtime.get_state().tr();
        transaction.add_step(Box::new(AddNodeStep::new_single(node, None)));
        
        // 每个节点都提交一次事务，开销大
        runtime.dispatch_flow(transaction).await?;
    }
    Ok(())
}
```

### 2. 内存管理优化

#### ✅ 对象池模式

```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// 节点对象池
pub struct NodePool {
    pool: Arc<Mutex<VecDeque<Node>>>,
    max_size: usize,
}

impl NodePool {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
        }
    }
    
    pub fn acquire(&self) -> Option<Node> {
        self.pool.lock().unwrap().pop_front()
    }
    
    pub fn release(&self, mut node: Node) {
        // 重置节点状态
        node.reset();
        
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push_back(node);
        }
    }
}

// 使用对象池
lazy_static! {
    static ref NODE_POOL: NodePool = NodePool::new(1000);
}

fn create_optimized_node() -> Node {
    NODE_POOL.acquire().unwrap_or_else(|| Node::default())
}
```

#### ✅ 智能缓存策略

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

// LRU 缓存实现
pub struct LRUCache<K, V> {
    cache: HashMap<K, (V, Instant)>,
    max_size: usize,
    ttl: Duration,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LRUCache<K, V> {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            ttl,
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some((value, timestamp)) = self.cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            } else {
                self.cache.remove(key);
            }
        }
        None
    }
    
    pub fn put(&mut self, key: K, value: V) {
        // 清理过期项
        self.cleanup_expired();
        
        // 如果达到最大容量，移除最旧的项
        if self.cache.len() >= self.max_size {
            if let Some((oldest_key, _)) = self.cache.iter()
                .min_by_key(|(_, (_, timestamp))| timestamp)
                .map(|(k, _)| k.clone()) {
                self.cache.remove(&oldest_key);
            }
        }
        
        self.cache.insert(key, (value, Instant::now()));
    }
    
    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }
}
```

### 3. 插件性能优化

#### ✅ 高效的插件实现

```rust
use mf_state::plugin::{PluginTrait, StateField, PluginMetadata, PluginConfig};
use std::sync::Arc;
use dashmap::DashMap;

// 使用无锁数据结构
#[derive(Debug)]
pub struct HighPerformancePlugin {
    cache: Arc<DashMap<String, CachedData>>,
}

impl HighPerformancePlugin {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }
    
    // 快速缓存查找
    fn get_cached_result(&self, key: &str) -> Option<CachedData> {
        self.cache.get(key).map(|entry| entry.value().clone())
    }
    
    // 异步批量处理
    async fn process_batch(&self, items: Vec<String>) -> Vec<ProcessedItem> {
        use futures::stream::{self, StreamExt};
        
        stream::iter(items)
            .map(|item| async move { self.process_single_item(item).await })
            .buffer_unordered(10) // 控制并发数
            .collect()
            .await
    }
    
    async fn process_single_item(&self, item: String) -> ProcessedItem {
        // 具体处理逻辑
        ProcessedItem { data: item }
    }
}

#[async_trait]
impl PluginTrait for HighPerformancePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "high_performance_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "高性能插件示例".to_string(),
            author: "性能团队".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec!["performance".to_string(), "optimization".to_string()],
        }
    }
    
    fn config(&self) -> PluginConfig {
        PluginConfig {
            enabled: true,
            priority: 5,
            settings: std::collections::HashMap::new(),
        }
    }
    
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 只处理相关的事务
        if !self.should_process(transactions) {
            return Ok(None);
        }
        
        // 批量处理以提高效率
        let items_to_process: Vec<String> = transactions.iter()
            .filter_map(|tr| tr.get_meta("items_to_process"))
            .flatten()
            .collect();
        
        if !items_to_process.is_empty() {
            let results = self.process_batch(items_to_process).await;
            
            // 创建结果事务
            let mut result_transaction = new_state.tr();
            result_transaction.set_meta("processing_results", results);
            
            return Ok(Some(result_transaction));
        }
        
        Ok(None)
    }
    
    fn should_process(&self, transactions: &[Transaction]) -> bool {
        transactions.iter().any(|tr| tr.has_meta("items_to_process"))
    }
}
```

### 4. 并发性能优化

#### ✅ 异步任务优化

```rust
use tokio::sync::{Semaphore, RwLock};
use std::sync::Arc;

// 并发控制器
pub struct ConcurrencyController {
    semaphore: Arc<Semaphore>,
    active_tasks: Arc<RwLock<usize>>,
}

impl ConcurrencyController {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            active_tasks: Arc::new(RwLock::new(0)),
        }
    }
    
    pub async fn execute<F, R>(&self, task: F) -> Result<R>
    where
        F: Future<Output = Result<R>>,
    {
        let permit = self.semaphore.acquire().await.unwrap();
        
        {
            let mut active = self.active_tasks.write().await;
            *active += 1;
        }
        
        let result = task.await;
        
        {
            let mut active = self.active_tasks.write().await;
            *active -= 1;
        }
        
        drop(permit);
        result
    }
    
    pub async fn get_active_count(&self) -> usize {
        *self.active_tasks.read().await
    }
}

// 使用示例
async fn process_documents_concurrently(docs: Vec<Document>) -> Result<Vec<ProcessedDoc>> {
    let controller = ConcurrencyController::new(10);
    let mut handles = Vec::new();
    
    for doc in docs {
        let controller_clone = controller.clone();
        let handle = tokio::spawn(async move {
            controller_clone.execute(process_single_document(doc)).await
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let results: Result<Vec<_>, _> = futures::future::try_join_all(handles).await;
    Ok(results?.into_iter().collect::<Result<Vec<_>, _>>()?)
}
```

## 📊 性能监控和诊断

### 1. 内置性能指标

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// 性能指标收集器
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub transaction_times: Vec<Duration>,
    pub plugin_execution_times: HashMap<String, Vec<Duration>>,
    pub memory_usage_samples: Vec<usize>,
    pub error_counts: HashMap<String, usize>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            transaction_times: Vec::new(),
            plugin_execution_times: HashMap::new(),
            memory_usage_samples: Vec::new(),
            error_counts: HashMap::new(),
        }
    }
    
    pub fn record_transaction_time(&mut self, duration: Duration) {
        self.transaction_times.push(duration);
        
        // 保持最近1000个样本
        if self.transaction_times.len() > 1000 {
            self.transaction_times.drain(0..500);
        }
    }
    
    pub fn record_plugin_time(&mut self, plugin_name: &str, duration: Duration) {
        self.plugin_execution_times
            .entry(plugin_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    pub fn get_average_transaction_time(&self) -> Option<Duration> {
        if self.transaction_times.is_empty() {
            return None;
        }
        
        let total: Duration = self.transaction_times.iter().sum();
        Some(total / self.transaction_times.len() as u32)
    }
    
    pub fn get_slowest_plugins(&self, top_n: usize) -> Vec<(String, Duration)> {
        let mut plugin_averages: Vec<_> = self.plugin_execution_times
            .iter()
            .map(|(name, times)| {
                let avg = times.iter().sum::<Duration>() / times.len() as u32;
                (name.clone(), avg)
            })
            .collect();
        
        plugin_averages.sort_by_key(|(_, duration)| *duration);
        plugin_averages.reverse();
        plugin_averages.truncate(top_n);
        plugin_averages
    }
}

// 性能监控中间件
#[derive(Debug)]
pub struct PerformanceMonitoringMiddleware {
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl PerformanceMonitoringMiddleware {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::new())),
        }
    }
    
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.lock().unwrap().clone()
    }
}

#[async_trait]
impl Middleware for PerformanceMonitoringMiddleware {
    fn name(&self) -> String {
        "performance_monitoring".to_string()
    }
    
    async fn before_dispatch(
        &self,
        _state: Option<Arc<State>>,
        _transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        // 记录开始时间到事务元数据
        Ok(None)
    }
    
    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        // 计算执行时间并记录
        for tr in transactions {
            if let Some(start_time) = tr.get_meta::<Instant>("start_time") {
                let duration = start_time.elapsed();
                self.metrics.lock().unwrap().record_transaction_time(duration);
                
                if duration > Duration::from_millis(100) {
                    warn!("慢事务检测: {:?}", duration);
                }
            }
        }
        
        Ok(None)
    }
}
```

### 2. 内存使用监控

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// 自定义内存分配器用于监控
pub struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
    }
}

// 内存使用情况报告
pub fn get_memory_usage() -> (usize, usize, usize) {
    let allocated = ALLOCATED.load(Ordering::Relaxed);
    let deallocated = DEALLOCATED.load(Ordering::Relaxed);
    let current = allocated.saturating_sub(deallocated);
    (allocated, deallocated, current)
}

// 在应用中使用
#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;
```

## 🔍 性能分析工具

### 1. 火焰图生成

```bash
# 安装 flamegraph 工具
cargo install flamegraph

# 生成火焰图
sudo cargo flamegraph --bin your-app

# 分析结果
open flamegraph.svg
```

### 2. 内存分析

```bash
# 使用 heaptrack 分析内存
heaptrack cargo run --release
heaptrack_gui heaptrack.your-app.*.gz

# 使用 valgrind 检测内存泄漏
valgrind --tool=memcheck --leak-check=full cargo run
```

### 3. 基准测试

```rust
// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use your_app::*;

fn bench_transaction_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let runtime = rt.block_on(create_test_runtime());
    
    c.bench_function("process 1000 transactions", |b| {
        b.to_async(&rt).iter(|| async {
            let transactions = create_test_transactions(1000);
            for tx in transactions {
                runtime.dispatch_flow(black_box(tx)).await.unwrap();
            }
        });
    });
}

fn bench_plugin_execution(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let plugin = create_test_plugin();
    let state = create_test_state();
    
    c.bench_function("plugin execution", |b| {
        b.to_async(&rt).iter(|| async {
            plugin.process_data(black_box(&state)).await.unwrap();
        });
    });
}

criterion_group!(benches, bench_transaction_processing, bench_plugin_execution);
criterion_main!(benches);
```

## 🎯 性能优化检查清单

### ✅ 代码层面优化

- [ ] **避免不必要的克隆**: 使用 `Arc` 和引用
- [ ] **批量操作**: 合并小的事务为大的批量操作
- [ ] **惰性计算**: 延迟计算直到真正需要结果
- [ ] **缓存策略**: 缓存频繁访问的计算结果
- [ ] **对象复用**: 使用对象池避免频繁分配

### ✅ 架构层面优化

- [ ] **插件优先级**: 合理设置插件执行顺序
- [ ] **异步处理**: 利用异步 I/O 提高并发能力
- [ ] **资源管理**: 及时释放不用的资源
- [ ] **内存监控**: 监控内存使用和泄漏
- [ ] **错误处理**: 避免错误处理路径的性能开销

### ✅ 配置层面优化

- [ ] **历史限制**: 设置合理的历史记录限制
- [ ] **缓存配置**: 调优缓存大小和过期时间
- [ ] **并发设置**: 根据硬件调整并发参数
- [ ] **日志级别**: 生产环境使用合适的日志级别
- [ ] **资源池**: 配置连接池和线程池大小

## 📈 性能基准参考

### 典型性能指标

| 操作类型 | 预期性能 | 优秀性能 |
|---------|---------|---------|
| 简单事务处理 | < 1ms | < 0.1ms |
| 插件执行 | < 10ms | < 1ms |
| 状态序列化 | < 100ms | < 10ms |
| 大文档加载 | < 1s | < 100ms |
| 搜索查询 | < 50ms | < 10ms |

### 内存使用指标

| 应用规模 | 基础内存 | 峰值内存 |
|---------|---------|---------|
| 小型应用 | 50MB | 200MB |
| 中型应用 | 200MB | 800MB |
| 大型应用 | 500MB | 2GB |

## 🚀 极致性能优化

### 1. SIMD 优化

```rust
// 使用 SIMD 指令加速数组操作
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[target_feature(enable = "avx2")]
unsafe fn simd_sum(values: &[f32]) -> f32 {
    let mut sum = _mm256_setzero_ps();
    let chunks = values.chunks_exact(8);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let vec = _mm256_loadu_ps(chunk.as_ptr());
        sum = _mm256_add_ps(sum, vec);
    }
    
    // 处理剩余元素
    let mut result = 0.0;
    let sum_array: [f32; 8] = std::mem::transmute(sum);
    for &val in sum_array.iter() {
        result += val;
    }
    for &val in remainder {
        result += val;
    }
    
    result
}
```

### 2. 内存映射文件

```rust
use memmap2::MmapOptions;
use std::fs::File;

// 使用内存映射处理大文件
pub fn process_large_file(file_path: &str) -> Result<()> {
    let file = File::open(file_path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    
    // 直接在内存映射上操作，避免拷贝
    let data = &mmap[..];
    process_data_in_place(data)?;
    
    Ok(())
}
```

### 3. 零拷贝序列化

```rust
use zerocopy::{AsBytes, FromBytes};

#[derive(AsBytes, FromBytes)]
#[repr(C)]
struct OptimizedNode {
    id: [u8; 16],
    type_id: u32,
    data_offset: u64,
    data_length: u32,
}

// 零拷贝序列化
fn serialize_zero_copy(nodes: &[OptimizedNode]) -> &[u8] {
    nodes.as_bytes()
}

// 零拷贝反序列化
fn deserialize_zero_copy(data: &[u8]) -> Option<&[OptimizedNode]> {
    OptimizedNode::slice_from(data)
}
```

---

通过遵循这些性能优化指南，您可以构建出高性能、可扩展的 ModuForge-RS 应用。记住，性能优化是一个持续的过程，需要根据实际使用情况不断调整和改进。
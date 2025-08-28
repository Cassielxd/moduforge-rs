# ModuForge-RS 基准测试实现指南

## 概览

本指南提供了 ModuForge-RS 框架各核心库的详细基准测试实现方案，包含具体的代码示例、配置文件和最佳实践。

## 1. 基准测试基础设施

### 1.1 共享基准测试工具库

```rust
// benches/shared/mod.rs
use criterion::Criterion;
use std::time::Duration;
use tokio::runtime::Runtime;

pub struct BenchmarkHarness {
    runtime: Runtime,
    warmup_time: Duration,
    measurement_time: Duration,
}

impl BenchmarkHarness {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            warmup_time: Duration::from_secs(3),
            measurement_time: Duration::from_secs(10),
        }
    }

    pub fn configure_criterion(&self, c: &mut Criterion) {
        c.warm_up_time(self.warmup_time)
         .measurement_time(self.measurement_time)
         .sample_size(100);
    }

    pub fn bench_async<F, Fut, R>(&self, name: &str, f: F, c: &mut Criterion)
    where
        F: Fn() -> Fut + Clone + 'static,
        Fut: std::future::Future<Output = R> + 'static,
        R: 'static,
    {
        c.bench_function(name, |b| {
            b.to_async(&self.runtime).iter(|| async {
                criterion::black_box(f().await)
            })
        });
    }
}

// 测试数据生成器
pub mod test_data {
    use mf_model::{Node, Document};
    use serde_json::json;
    
    pub fn create_test_document(node_count: usize) -> Document {
        let mut doc = Document::new();
        for i in 0..node_count {
            let node = Node::new(format!("node_{}", i))
                .with_attribute("id", json!(i))
                .with_attribute("text", json!(format!("Test content {}", i)));
            doc.add_node(node);
        }
        doc
    }
    
    pub fn create_large_json_data(size_mb: usize) -> serde_json::Value {
        let items_per_mb = 1000;
        let total_items = size_mb * items_per_mb;
        
        json!({
            "data": (0..total_items).map(|i| json!({
                "id": i,
                "title": format!("Item {}", i),
                "description": "A".repeat(100),
                "metadata": {
                    "created_at": "2024-01-01T00:00:00Z",
                    "tags": ["tag1", "tag2", "tag3"],
                    "scores": [0.1, 0.2, 0.3, 0.4, 0.5]
                }
            })).collect::<Vec<_>>()
        })
    }
}
```

### 1.2 性能监控工具

```rust
// benches/shared/profiling.rs
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, ProcessExt};

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub execution_time: Duration,
    pub peak_memory_mb: f64,
    pub avg_cpu_percent: f64,
    pub allocations: u64,
    pub deallocations: u64,
}

pub struct PerformanceProfiler {
    system: System,
    start_time: Option<Instant>,
    initial_memory: u64,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            start_time: None,
            initial_memory: 0,
        }
    }
    
    pub fn start_profiling(&mut self) {
        self.start_time = Some(Instant::now());
        self.system.refresh_memory();
        self.initial_memory = self.system.used_memory();
    }
    
    pub fn end_profiling(&mut self) -> PerformanceMetrics {
        let execution_time = self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_default();
            
        self.system.refresh_all();
        let current_memory = self.system.used_memory();
        let peak_memory_mb = ((current_memory - self.initial_memory) as f64) / 1024.0 / 1024.0;
        
        PerformanceMetrics {
            execution_time,
            peak_memory_mb,
            avg_cpu_percent: self.system.global_cpu_info().cpu_usage() as f64,
            allocations: 0, // 需要集成内存分配器统计
            deallocations: 0,
        }
    }
}
```

## 2. 核心库基准测试实现

### 2.1 mf-model 基准测试

```rust
// crates/model/benches/node_operations.rs
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_model::{Node, Attribute, Mark};
use serde_json::json;

fn bench_node_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("节点创建");
    
    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("简单节点", size), size, |b, &size| {
            b.iter(|| {
                let nodes: Vec<_> = (0..size).map(|i| {
                    Node::new(format!("node_{}", i))
                }).collect();
                criterion::black_box(nodes)
            })
        });
        
        group.bench_with_input(BenchmarkId::new("带属性节点", size), size, |b, &size| {
            b.iter(|| {
                let nodes: Vec<_> = (0..size).map(|i| {
                    Node::new(format!("node_{}", i))
                        .with_attribute("id", json!(i))
                        .with_attribute("text", json!(format!("内容 {}", i)))
                }).collect();
                criterion::black_box(nodes)
            })
        });
    }
    
    group.finish();
}

fn bench_attribute_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("属性操作");
    
    let mut node = Node::new("test_node");
    
    group.bench_function("设置属性", |b| {
        b.iter(|| {
            let mut n = node.clone();
            n.set_attribute("test_attr", json!({"value": 42, "type": "number"}));
            criterion::black_box(n)
        })
    });
    
    group.bench_function("获取属性", |b| {
        let n = node.clone().with_attribute("test_attr", json!(42));
        b.iter(|| {
            let value = n.get_attribute("test_attr");
            criterion::black_box(value)
        })
    });
    
    group.bench_function("删除属性", |b| {
        b.iter(|| {
            let mut n = node.clone().with_attribute("test_attr", json!(42));
            n.remove_attribute("test_attr");
            criterion::black_box(n)
        })
    });
    
    group.finish();
}

fn bench_mark_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("标记操作");
    
    group.bench_function("添加标记", |b| {
        let mut node = Node::new("test_node").with_text("这是一段测试文本内容");
        b.iter(|| {
            let mut n = node.clone();
            n.add_mark(0, 5, Mark::strong());
            n.add_mark(6, 10, Mark::emphasis());
            criterion::black_box(n)
        })
    });
    
    group.bench_function("查找标记", |b| {
        let mut node = Node::new("test_node").with_text("这是一段测试文本内容");
        node.add_mark(0, 5, Mark::strong());
        node.add_mark(6, 10, Mark::emphasis());
        
        b.iter(|| {
            let marks = node.get_marks_at_range(0, 15);
            criterion::black_box(marks)
        })
    });
    
    group.finish();
}

criterion_group!(
    model_benches,
    bench_node_creation,
    bench_attribute_operations,
    bench_mark_operations
);
criterion_main!(model_benches);
```

### 2.2 mf-state 基准测试

```rust
// crates/state/benches/transaction_processing.rs
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_state::{State, Transaction};
use mf_transform::node_step::AddNodeStep;
use mf_model::Node;
use serde_json::json;
use tokio::runtime::Runtime;

fn bench_transaction_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("事务创建");
    
    for step_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量节点添加事务", step_count),
            step_count,
            |b, &step_count| {
                b.iter(|| {
                    let mut transaction = Transaction::new();
                    for i in 0..step_count {
                        let node = Node::new(format!("node_{}", i))
                            .with_attribute("id", json!(i));
                        transaction.add_step(AddNodeStep::new(node, None));
                    }
                    criterion::black_box(transaction)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_transaction_application(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("事务应用");
    
    for step_count in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("应用事务", step_count),
            step_count,
            |b, &step_count| {
                b.to_async(&rt).iter(|| async {
                    let mut state = State::new();
                    let mut transaction = Transaction::new();
                    
                    for i in 0..step_count {
                        let node = Node::new(format!("node_{}", i));
                        transaction.add_step(AddNodeStep::new(node, None));
                    }
                    
                    let result = state.apply_transaction(transaction).await;
                    criterion::black_box(result)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_state_querying(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("状态查询");
    
    // 预创建状态
    let state = rt.block_on(async {
        let mut state = State::new();
        let mut transaction = Transaction::new();
        
        for i in 0..1000 {
            let node = Node::new(format!("node_{}", i))
                .with_attribute("type", json!("test"))
                .with_attribute("index", json!(i));
            transaction.add_step(AddNodeStep::new(node, None));
        }
        
        state.apply_transaction(transaction).await.unwrap();
        state
    });
    
    group.bench_function("按ID查找节点", |b| {
        b.to_async(&rt).iter(|| async {
            let node = state.get_node_by_id("node_500").await;
            criterion::black_box(node)
        })
    });
    
    group.bench_function("按类型筛选节点", |b| {
        b.to_async(&rt).iter(|| async {
            let nodes = state.query_nodes_by_attribute("type", &json!("test")).await;
            criterion::black_box(nodes)
        })
    });
    
    group.finish();
}

criterion_group!(
    state_benches,
    bench_transaction_creation,
    bench_transaction_application,
    bench_state_querying
);
criterion_main!(state_benches);
```

### 2.3 mf-collaboration 基准测试

```rust
// crates/collaboration/benches/sync_performance.rs
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_collaboration::{CollaborationManager, DocumentState};
use mf_model::Node;
use serde_json::json;
use tokio::runtime::Runtime;
use std::sync::Arc;

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("并发操作");
    
    for user_count in [2, 5, 10, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("并发文本编辑", user_count),
            user_count,
            |b, &user_count| {
                b.to_async(&rt).iter(|| async {
                    let collab_manager = Arc::new(CollaborationManager::new());
                    let doc_state = Arc::new(DocumentState::new("测试文档"));
                    
                    // 模拟多用户并发编辑
                    let tasks: Vec<_> = (0..user_count).map(|user_id| {
                        let manager = collab_manager.clone();
                        let state = doc_state.clone();
                        
                        tokio::spawn(async move {
                            for i in 0..10 {
                                let operation = format!("用户{}操作{}", user_id, i);
                                manager.apply_operation(&state, operation).await.unwrap();
                            }
                        })
                    }).collect();
                    
                    futures::future::join_all(tasks).await;
                    criterion::black_box(doc_state)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_sync_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("同步性能");
    
    group.bench_function("CRDT状态合并", |b| {
        b.to_async(&rt).iter(|| async {
            let state1 = DocumentState::new("文档1");
            let state2 = DocumentState::new("文档2");
            
            // 在两个状态中进行不同的修改
            state1.insert_text(0, "Hello ").await.unwrap();
            state2.insert_text(0, "World!").await.unwrap();
            
            // 合并状态
            let merged = state1.merge(&state2).await;
            criterion::black_box(merged)
        })
    });
    
    group.bench_function("操作历史同步", |b| {
        b.to_async(&rt).iter(|| async {
            let manager = CollaborationManager::new();
            let state = DocumentState::new("测试文档");
            
            // 创建操作历史
            for i in 0..100 {
                let op = format!("操作{}", i);
                manager.apply_operation(&state, op).await.unwrap();
            }
            
            // 同步到新客户端
            let synced_state = manager.sync_with_client("client_1", &state).await;
            criterion::black_box(synced_state)
        })
    });
    
    group.finish();
}

criterion_group!(
    collaboration_benches,
    bench_concurrent_operations,
    bench_sync_performance
);
criterion_main!(collaboration_benches);
```

### 2.4 mf-search 基准测试

```rust
// crates/search/benches/indexing_and_search.rs
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_search::{SearchEngine, Document as SearchDoc, Query};
use serde_json::json;
use tokio::runtime::Runtime;

fn bench_document_indexing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("文档索引");
    
    for doc_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量索引", doc_count),
            doc_count,
            |b, &doc_count| {
                b.to_async(&rt).iter(|| async {
                    let mut search_engine = SearchEngine::new();
                    
                    for i in 0..doc_count {
                        let doc = SearchDoc::new(format!("doc_{}", i))
                            .with_title(&format!("文档标题 {}", i))
                            .with_content(&format!(
                                "这是第{}个测试文档的内容，包含各种关键词和短语。\
                                 文档内容应该足够长以测试索引性能。", i
                            ))
                            .with_metadata(json!({
                                "category": "测试",
                                "tags": ["标签1", "标签2"],
                                "created_at": "2024-01-01T00:00:00Z"
                            }));
                        
                        search_engine.index_document(doc).await.unwrap();
                    }
                    
                    criterion::black_box(search_engine)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_search_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // 预建立索引
    let search_engine = rt.block_on(async {
        let mut engine = SearchEngine::new();
        
        for i in 0..10000 {
            let doc = SearchDoc::new(format!("doc_{}", i))
                .with_title(&format!("技术文档 {}", i))
                .with_content(&format!(
                    "Rust编程语言 性能优化 内存安全 并发编程 系统编程 \
                     Web开发 数据结构 算法设计 软件工程 {}",
                    i
                ));
            engine.index_document(doc).await.unwrap();
        }
        
        engine
    });
    
    let mut group = c.benchmark_group("搜索查询");
    
    group.bench_function("简单关键词搜索", |b| {
        b.to_async(&rt).iter(|| async {
            let query = Query::new("Rust编程");
            let results = search_engine.search(&query).await.unwrap();
            criterion::black_box(results)
        })
    });
    
    group.bench_function("复合条件搜索", |b| {
        b.to_async(&rt).iter(|| async {
            let query = Query::new("性能优化")
                .with_filter("category", "技术")
                .with_limit(50);
            let results = search_engine.search(&query).await.unwrap();
            criterion::black_box(results)
        })
    });
    
    group.bench_function("模糊搜索", |b| {
        b.to_async(&rt).iter(|| async {
            let query = Query::new("编程").with_fuzzy(true);
            let results = search_engine.search(&query).await.unwrap();
            criterion::black_box(results)
        })
    });
    
    group.finish();
}

criterion_group!(
    search_benches,
    bench_document_indexing,
    bench_search_queries
);
criterion_main!(search_benches);
```

### 2.5 mf-file 基准测试

```rust
// crates/file/benches/serialization.rs
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mf_file::{FileFormat, DocumentSerializer};
use mf_model::Document;
use serde_json::json;
use std::io::Cursor;

fn create_test_document(size_category: &str) -> Document {
    let (node_count, content_size) = match size_category {
        "small" => (10, 100),
        "medium" => (100, 1000),
        "large" => (1000, 10000),
        "xlarge" => (10000, 100000),
        _ => (100, 1000),
    };
    
    let mut doc = Document::new();
    for i in 0..node_count {
        let content = "测试内容 ".repeat(content_size / 10);
        let node = mf_model::Node::new(format!("node_{}", i))
            .with_text(&content)
            .with_attribute("id", json!(i))
            .with_attribute("type", json!("测试节点"))
            .with_attribute("metadata", json!({
                "created_at": "2024-01-01T00:00:00Z",
                "tags": ["标签1", "标签2", "标签3"],
                "score": i as f64 * 0.1
            }));
        doc.add_node(node);
    }
    doc
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("序列化性能");
    
    for size in ["small", "medium", "large"].iter() {
        let doc = create_test_document(size);
        let serialized_size = serde_json::to_string(&doc).unwrap().len();
        
        group.throughput(Throughput::Bytes(serialized_size as u64));
        
        // JSON 序列化
        group.bench_with_input(
            BenchmarkId::new("JSON序列化", size),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let serializer = DocumentSerializer::new(FileFormat::Json);
                    let mut buffer = Vec::new();
                    serializer.serialize(doc, &mut buffer).unwrap();
                    criterion::black_box(buffer)
                })
            }
        );
        
        // CBOR 序列化
        group.bench_with_input(
            BenchmarkId::new("CBOR序列化", size),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let serializer = DocumentSerializer::new(FileFormat::Cbor);
                    let mut buffer = Vec::new();
                    serializer.serialize(doc, &mut buffer).unwrap();
                    criterion::black_box(buffer)
                })
            }
        );
        
        // MessagePack 序列化
        group.bench_with_input(
            BenchmarkId::new("MessagePack序列化", size),
            &doc,
            |b, doc| {
                b.iter(|| {
                    let serializer = DocumentSerializer::new(FileFormat::MessagePack);
                    let mut buffer = Vec::new();
                    serializer.serialize(doc, &mut buffer).unwrap();
                    criterion::black_box(buffer)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("反序列化性能");
    
    for size in ["small", "medium", "large"].iter() {
        let doc = create_test_document(size);
        
        // 预序列化数据
        let json_data = {
            let serializer = DocumentSerializer::new(FileFormat::Json);
            let mut buffer = Vec::new();
            serializer.serialize(&doc, &mut buffer).unwrap();
            buffer
        };
        
        let cbor_data = {
            let serializer = DocumentSerializer::new(FileFormat::Cbor);
            let mut buffer = Vec::new();
            serializer.serialize(&doc, &mut buffer).unwrap();
            buffer
        };
        
        let msgpack_data = {
            let serializer = DocumentSerializer::new(FileFormat::MessagePack);
            let mut buffer = Vec::new();
            serializer.serialize(&doc, &mut buffer).unwrap();
            buffer
        };
        
        group.throughput(Throughput::Bytes(json_data.len() as u64));
        
        // JSON 反序列化
        group.bench_with_input(
            BenchmarkId::new("JSON反序列化", size),
            &json_data,
            |b, data| {
                b.iter(|| {
                    let serializer = DocumentSerializer::new(FileFormat::Json);
                    let cursor = Cursor::new(data);
                    let doc: Document = serializer.deserialize(cursor).unwrap();
                    criterion::black_box(doc)
                })
            }
        );
        
        // CBOR 反序列化
        group.bench_with_input(
            BenchmarkId::new("CBOR反序列化", size),
            &cbor_data,
            |b, data| {
                b.iter(|| {
                    let serializer = DocumentSerializer::new(FileFormat::Cbor);
                    let cursor = Cursor::new(data);
                    let doc: Document = serializer.deserialize(cursor).unwrap();
                    criterion::black_box(doc)
                })
            }
        );
        
        // MessagePack 反序列化
        group.bench_with_input(
            BenchmarkId::new("MessagePack反序列化", size),
            &msgpack_data,
            |b, data| {
                b.iter(|| {
                    let serializer = DocumentSerializer::new(FileFormat::MessagePack);
                    let cursor = Cursor::new(data);
                    let doc: Document = serializer.deserialize(cursor).unwrap();
                    criterion::black_box(doc)
                })
            }
        );
    }
    
    group.finish();
}

fn bench_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("压缩性能");
    
    for size in ["medium", "large", "xlarge"].iter() {
        let doc = create_test_document(size);
        let json_data = serde_json::to_string(&doc).unwrap();
        
        group.throughput(Throughput::Bytes(json_data.len() as u64));
        
        group.bench_with_input(
            BenchmarkId::new("Gzip压缩", size),
            &json_data,
            |b, data| {
                b.iter(|| {
                    let compressed = mf_file::compress_gzip(data.as_bytes()).unwrap();
                    criterion::black_box(compressed)
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("LZ4压缩", size),
            &json_data,
            |b, data| {
                b.iter(|| {
                    let compressed = mf_file::compress_lz4(data.as_bytes()).unwrap();
                    criterion::black_box(compressed)
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    file_benches,
    bench_serialization,
    bench_deserialization,
    bench_compression
);
criterion_main!(file_benches);
```

## 3. 集成基准测试

### 3.1 端到端工作流基准测试

```rust
// benches/integration/end_to_end.rs
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_core::Runtime;
use mf_model::{Document, Node};
use mf_state::Transaction;
use mf_transform::node_step::AddNodeStep;
use mf_search::SearchEngine;
use mf_file::{FileFormat, DocumentSerializer};
use mf_collaboration::CollaborationManager;
use serde_json::json;
use tokio::runtime::Runtime as TokioRuntime;
use std::sync::Arc;

fn bench_complete_workflow(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    let mut group = c.benchmark_group("完整工作流");
    
    group.bench_function("文档创建到搜索完整流程", |b| {
        b.to_async(&rt).iter(|| async {
            // 1. 创建运行时和服务
            let runtime = Runtime::new().await.unwrap();
            let search_engine = Arc::new(SearchEngine::new());
            let collab_manager = Arc::new(CollaborationManager::new());
            
            // 2. 创建文档
            let mut doc = Document::new();
            for i in 0..100 {
                let node = Node::new(format!("节点_{}", i))
                    .with_text(&format!("这是节点{}的测试内容", i))
                    .with_attribute("类型", json!("测试"));
                doc.add_node(node);
            }
            
            // 3. 应用状态变更
            let mut transaction = Transaction::new();
            for node in doc.nodes() {
                transaction.add_step(AddNodeStep::new(node.clone(), None));
            }
            let state = runtime.apply_transaction(transaction).await.unwrap();
            
            // 4. 建立搜索索引
            for node in doc.nodes() {
                let search_doc = mf_search::Document::new(node.id())
                    .with_content(node.text().unwrap_or(""))
                    .with_metadata(json!(node.attributes()));
                search_engine.index_document(search_doc).await.unwrap();
            }
            
            // 5. 搜索测试
            let query = mf_search::Query::new("测试内容");
            let search_results = search_engine.search(&query).await.unwrap();
            
            // 6. 序列化保存
            let serializer = DocumentSerializer::new(FileFormat::Json);
            let mut buffer = Vec::new();
            serializer.serialize(&doc, &mut buffer).unwrap();
            
            criterion::black_box((state, search_results, buffer))
        })
    });
    
    group.finish();
}

fn bench_collaborative_editing_workflow(c: &mut Criterion) {
    let rt = TokioRuntime::new().unwrap();
    let mut group = c.benchmark_group("协作编辑工作流");
    
    for user_count in [2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("多用户协作编辑", user_count),
            user_count,
            |b, &user_count| {
                b.to_async(&rt).iter(|| async {
                    let runtime = Runtime::new().await.unwrap();
                    let collab_manager = Arc::new(CollaborationManager::new());
                    
                    // 创建初始文档
                    let mut doc = Document::new();
                    let root_node = Node::new("根节点")
                        .with_text("这是一个协作编辑测试文档");
                    doc.add_node(root_node);
                    
                    // 模拟多用户并发编辑
                    let tasks: Vec<_> = (0..user_count).map(|user_id| {
                        let manager = collab_manager.clone();
                        let document = doc.clone();
                        
                        tokio::spawn(async move {
                            // 每个用户进行一系列编辑操作
                            for op_id in 0..20 {
                                let mut transaction = Transaction::new();
                                let node = Node::new(format!("用户{}节点{}", user_id, op_id))
                                    .with_text(&format!("用户{}的第{}个编辑", user_id, op_id));
                                transaction.add_step(AddNodeStep::new(node, None));
                                
                                // 应用协作操作
                                manager.apply_collaborative_transaction(
                                    &format!("用户{}", user_id),
                                    transaction
                                ).await.unwrap();
                            }
                        })
                    }).collect();
                    
                    // 等待所有编辑完成
                    futures::future::join_all(tasks).await;
                    
                    // 获取最终同步后的文档状态
                    let final_doc = collab_manager.get_synchronized_document().await.unwrap();
                    
                    criterion::black_box(final_doc)
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    integration_benches,
    bench_complete_workflow,
    bench_collaborative_editing_workflow
);
criterion_main!(integration_benches);
```

## 4. 基准测试配置

### 4.1 Cargo.toml 配置

```toml
# 工作空间根目录 Cargo.toml
[workspace.metadata.benchmarks]
# 基准测试全局配置
runner = "moduforge-bench"
timeout = "30m"
isolation = "process"
resource_limits = { memory = "4GB", cpu_cores = 4 }

# 各crate基准测试配置示例
# crates/model/Cargo.toml
[[bench]]
name = "node_operations"
harness = false
required-features = ["benchmark"]

[[bench]]
name = "document_operations"
harness = false
required-features = ["benchmark"]

[features]
default = []
benchmark = ["criterion"]

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### 4.2 基准测试阈值配置

```toml
# benchmarks/config/thresholds.toml
[regression_thresholds]
# 性能回归阈值设置（百分比）
default = 10.0
critical_path = 5.0
memory_usage = 15.0

[performance_targets]
# 各组件性能目标

[performance_targets.mf_model]
node_creation = { time = "1us", memory = "1KB" }
attribute_access = { time = "100ns", memory = "0KB" }
mark_operations = { time = "10us", memory = "5KB" }

[performance_targets.mf_state]
transaction_apply = { time = "10ms", memory = "10MB", tps = 1000 }
state_query = { time = "1ms", memory = "1MB" }

[performance_targets.mf_collaboration]
sync_operation = { time = "50ms", memory = "5MB" }
concurrent_users = 1000

[performance_targets.mf_search]
index_document = { time = "10ms", memory = "100KB", throughput = "1000 docs/s" }
search_query = { time = "100ms", memory = "50MB" }

[performance_targets.mf_file]
json_serialize = { throughput = "100MB/s", memory = "10MB" }
json_deserialize = { throughput = "80MB/s", memory = "15MB" }
compression = { ratio = 0.3, throughput = "50MB/s" }

[alert_settings]
email_notifications = true
slack_webhook = "https://hooks.slack.com/services/..."
github_status_checks = true
```

### 4.3 CI 基准测试配置

```yaml
# .github/workflows/benchmarks.yml
name: 性能基准测试

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # 每日凌晨2点执行

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  基准测试:
    name: 运行基准测试
    runs-on: ubuntu-latest
    timeout-minutes: 60
    
    strategy:
      matrix:
        tier:
          - foundation  # 基础层
          - core        # 核心层
          - service     # 服务层
          - integration # 集成层
    
    steps:
    - name: 检出代码
      uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: 设置Rust工具链
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    
    - name: 配置缓存
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: benchmark-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-${{ matrix.tier }}
        restore-keys: |
          benchmark-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-
          benchmark-${{ runner.os }}-
    
    - name: 系统环境检查
      run: |
        echo "=== 系统信息 ==="
        echo "CPU核心数: $(nproc)"
        echo "内存信息: $(free -h)"
        echo "磁盘空间: $(df -h)"
        echo "Rust版本: $(rustc --version)"
        echo "Cargo版本: $(cargo --version)"
    
    - name: 运行基础层基准测试
      if: matrix.tier == 'foundation'
      run: |
        echo "运行基础层基准测试..."
        cargo bench --package mf-model --package mf-derive --package mf-macro
    
    - name: 运行核心层基准测试
      if: matrix.tier == 'core'
      run: |
        echo "运行核心层基准测试..."
        cargo bench --package mf-transform --package mf-expression --package mf-template
    
    - name: 运行服务层基准测试
      if: matrix.tier == 'service'
      run: |
        echo "运行服务层基准测试..."
        cargo bench --package mf-state --package mf-engine --package mf-file
        cargo bench --package mf-search --package mf-persistence
    
    - name: 运行集成层基准测试
      if: matrix.tier == 'integration'
      run: |
        echo "运行集成层基准测试..."
        cargo bench --package mf-core --package mf-collaboration --package mf-collaboration-client
    
    - name: 上传基准测试结果
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-results-${{ matrix.tier }}-${{ github.sha }}
        path: |
          target/criterion/
          benchmarks/results/
        retention-days: 30
    
    - name: 性能回归检测
      run: |
        if [ "${{ github.event_name }}" = "pull_request" ]; then
          echo "检查性能回归..."
          cargo run --bin regression-detector -- \
            --base-ref origin/${{ github.base_ref }} \
            --threshold 10% \
            --tier ${{ matrix.tier }}
        fi

  汇总报告:
    name: 生成性能报告
    needs: [基准测试]
    runs-on: ubuntu-latest
    if: always()
    
    steps:
    - name: 检出代码
      uses: actions/checkout@v4
      
    - name: 下载所有基准测试结果
      uses: actions/download-artifact@v3
      with:
        path: benchmark-artifacts/
      
    - name: 生成综合性能报告
      run: |
        cargo run --bin bench-reporter -- \
          --input benchmark-artifacts/ \
          --output reports/ \
          --format html \
          --include-history
      
    - name: 部署性能报告到GitHub Pages
      if: github.ref == 'refs/heads/main'
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./reports
        destination_dir: benchmarks
      
    - name: 发送性能报告通知
      if: github.event_name == 'schedule'
      run: |
        cargo run --bin notification-sender -- \
          --report-path reports/summary.html \
          --webhook ${{ secrets.SLACK_WEBHOOK }}
```

## 5. 性能监控仪表板

### 5.1 实时监控脚本

```python
# scripts/performance_dashboard.py
#!/usr/bin/env python3
"""ModuForge-RS 性能监控仪表板"""

import json
import sqlite3
import matplotlib.pyplot as plt
import pandas as pd
from datetime import datetime, timedelta
import argparse
from pathlib import Path
import numpy as np

class PerformanceDashboard:
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.conn = sqlite3.connect(db_path)
        self.setup_database()
    
    def setup_database(self):
        """初始化数据库表结构"""
        self.conn.execute('''
            CREATE TABLE IF NOT EXISTS benchmark_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                crate_name TEXT NOT NULL,
                benchmark_name TEXT NOT NULL,
                scenario TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                git_commit TEXT,
                execution_time_ns INTEGER,
                memory_usage_bytes INTEGER,
                throughput_ops_per_sec REAL,
                cpu_utilization_percent REAL,
                metadata_json TEXT
            )
        ''')
        
        self.conn.execute('''
            CREATE INDEX IF NOT EXISTS idx_crate_time 
            ON benchmark_results(crate_name, timestamp)
        ''')
        
        self.conn.commit()
    
    def import_criterion_results(self, results_dir: Path):
        """导入Criterion基准测试结果"""
        for crate_dir in results_dir.glob('*/'):
            crate_name = crate_dir.name
            
            for benchmark_dir in crate_dir.glob('*/'):
                benchmark_name = benchmark_dir.name
                
                # 读取Criterion输出的estimates.json
                estimates_file = benchmark_dir / 'base' / 'estimates.json'
                if estimates_file.exists():
                    with open(estimates_file) as f:
                        estimates = json.load(f)
                    
                    # 提取性能指标
                    mean_time = estimates['mean']['point_estimate']
                    
                    # 插入数据库
                    self.conn.execute('''
                        INSERT INTO benchmark_results 
                        (crate_name, benchmark_name, scenario, timestamp, 
                         execution_time_ns, metadata_json)
                        VALUES (?, ?, ?, ?, ?, ?)
                    ''', (
                        crate_name, benchmark_name, 'default', 
                        datetime.now(), mean_time, json.dumps(estimates)
                    ))
        
        self.conn.commit()
    
    def generate_trend_chart(self, crate_name: str, days: int = 30):
        """生成性能趋势图表"""
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)
        
        df = pd.read_sql_query('''
            SELECT benchmark_name, timestamp, execution_time_ns
            FROM benchmark_results
            WHERE crate_name = ? AND timestamp BETWEEN ? AND ?
            ORDER BY timestamp
        ''', self.conn, params=(crate_name, start_date, end_date))
        
        if df.empty:
            print(f"没有找到 {crate_name} 的基准测试数据")
            return
        
        plt.figure(figsize=(12, 8))
        
        for benchmark in df['benchmark_name'].unique():
            benchmark_data = df[df['benchmark_name'] == benchmark]
            plt.plot(
                pd.to_datetime(benchmark_data['timestamp']),
                benchmark_data['execution_time_ns'] / 1_000_000,  # 转换为毫秒
                marker='o',
                label=benchmark
            )
        
        plt.title(f'{crate_name} 性能趋势 (最近{days}天)')
        plt.xlabel('日期')
        plt.ylabel('执行时间 (毫秒)')
        plt.legend()
        plt.xticks(rotation=45)
        plt.tight_layout()
        
        output_path = f'reports/{crate_name}_trend_{days}days.png'
        plt.savefig(output_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"性能趋势图已保存到: {output_path}")
    
    def detect_regressions(self, threshold_percent: float = 10.0):
        """检测性能回归"""
        regressions = []
        
        # 获取最近两次运行的结果
        query = '''
            WITH latest_runs AS (
                SELECT crate_name, benchmark_name, timestamp,
                       ROW_NUMBER() OVER (
                           PARTITION BY crate_name, benchmark_name 
                           ORDER BY timestamp DESC
                       ) as rn,
                       execution_time_ns
                FROM benchmark_results
                WHERE timestamp > datetime('now', '-7 days')
            )
            SELECT 
                crate_name, benchmark_name,
                MAX(CASE WHEN rn = 1 THEN execution_time_ns END) as latest_time,
                MAX(CASE WHEN rn = 2 THEN execution_time_ns END) as previous_time
            FROM latest_runs
            WHERE rn <= 2
            GROUP BY crate_name, benchmark_name
            HAVING COUNT(*) = 2
        '''
        
        results = self.conn.execute(query).fetchall()
        
        for crate_name, benchmark_name, latest_time, previous_time in results:
            if previous_time and latest_time:
                change_percent = ((latest_time - previous_time) / previous_time) * 100
                
                if change_percent > threshold_percent:
                    regressions.append({
                        'crate': crate_name,
                        'benchmark': benchmark_name,
                        'regression_percent': change_percent,
                        'previous_time_ms': previous_time / 1_000_000,
                        'latest_time_ms': latest_time / 1_000_000
                    })
        
        return regressions
    
    def generate_summary_report(self):
        """生成性能摘要报告"""
        # 获取各库的最新性能数据
        query = '''
            WITH latest_results AS (
                SELECT crate_name, benchmark_name, execution_time_ns,
                       ROW_NUMBER() OVER (
                           PARTITION BY crate_name, benchmark_name 
                           ORDER BY timestamp DESC
                       ) as rn
                FROM benchmark_results
            )
            SELECT crate_name, 
                   COUNT(*) as benchmark_count,
                   AVG(execution_time_ns) as avg_time_ns,
                   MIN(execution_time_ns) as min_time_ns,
                   MAX(execution_time_ns) as max_time_ns
            FROM latest_results
            WHERE rn = 1
            GROUP BY crate_name
            ORDER BY crate_name
        '''
        
        df = pd.read_sql_query(query, self.conn)
        
        print("=== ModuForge-RS 性能摘要报告 ===")
        print(f"报告生成时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        print()
        
        for _, row in df.iterrows():
            print(f"📦 {row['crate_name']}")
            print(f"   基准测试数量: {row['benchmark_count']}")
            print(f"   平均执行时间: {row['avg_time_ns']/1_000_000:.2f}ms")
            print(f"   执行时间范围: {row['min_time_ns']/1_000_000:.2f}ms - {row['max_time_ns']/1_000_000:.2f}ms")
            print()
        
        # 检测性能回归
        regressions = self.detect_regressions()
        if regressions:
            print("🚨 检测到性能回归:")
            for reg in regressions:
                print(f"   - {reg['crate']}/{reg['benchmark']}: "
                      f"{reg['regression_percent']:+.1f}% "
                      f"({reg['previous_time_ms']:.2f}ms → {reg['latest_time_ms']:.2f}ms)")
        else:
            print("✅ 未检测到显著性能回归")

def main():
    parser = argparse.ArgumentParser(description='ModuForge-RS 性能监控仪表板')
    parser.add_argument('--db', default='benchmarks/results/performance.db',
                       help='性能数据库路径')
    parser.add_argument('--import-dir', type=Path,
                       help='导入Criterion结果目录')
    parser.add_argument('--generate-trends', action='store_true',
                       help='生成趋势图表')
    parser.add_argument('--crate', help='指定库名称')
    parser.add_argument('--days', type=int, default=30,
                       help='趋势分析天数')
    parser.add_argument('--summary', action='store_true',
                       help='生成摘要报告')
    
    args = parser.parse_args()
    
    dashboard = PerformanceDashboard(args.db)
    
    if args.import_dir:
        print(f"导入基准测试结果从: {args.import_dir}")
        dashboard.import_criterion_results(args.import_dir)
    
    if args.generate_trends:
        if args.crate:
            dashboard.generate_trend_chart(args.crate, args.days)
        else:
            # 为所有库生成趋势图
            crates = ['mf-model', 'mf-state', 'mf-collaboration', 'mf-search', 'mf-file']
            for crate in crates:
                dashboard.generate_trend_chart(crate, args.days)
    
    if args.summary:
        dashboard.generate_summary_report()

if __name__ == '__main__':
    main()
```

## 6. 使用说明

### 6.1 运行基准测试

```bash
# 运行所有基准测试
cargo bench --workspace

# 运行特定库的基准测试
cargo bench --package mf-model

# 运行特定基准测试
cargo bench --package mf-state transaction_apply

# 生成HTML报告
cargo bench --package mf-model -- --output-format html

# 与基线比较
cargo bench --package mf-model -- --save-baseline main
git checkout feature-branch
cargo bench --package mf-model -- --baseline main
```

### 6.2 生成性能报告

```bash
# 导入基准测试结果并生成报告
python scripts/performance_dashboard.py \
    --import-dir target/criterion \
    --generate-trends \
    --summary

# 检测性能回归
cargo run --bin regression-detector -- \
    --threshold 10% \
    --output reports/regression.json

# 生成性能对比报告
cargo run --bin bench-reporter -- \
    --compare-with baseline \
    --format html \
    --output reports/comparison.html
```

### 6.3 CI/CD 集成

```bash
# 在CI环境中运行基准测试
BENCHMARK_MODE=ci cargo bench --workspace

# 上传结果到性能追踪系统
curl -X POST \
    -H "Content-Type: application/json" \
    -d @benchmark-results.json \
    https://performance.moduforge.dev/api/results
```

本实现指南提供了完整的基准测试代码示例和配置文件，确保 ModuForge-RS 框架能够建立全面、准确的性能监控体系。通过这些基准测试，开发团队可以：

1. **持续监控性能**：每次代码变更都会自动运行基准测试
2. **及时发现回归**：自动检测并报告性能下降
3. **优化决策支持**：基于准确数据进行性能优化
4. **质量保证**：确保发布版本的性能符合标准

所有代码都遵循 Rust 最佳实践，使用标准化的工具链，确保结果的准确性和可重现性。
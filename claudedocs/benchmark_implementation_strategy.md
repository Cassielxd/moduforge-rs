# ModuForge-RS Benchmark Implementation Strategy

## Detailed Implementation Plan

### 1. Shared Benchmark Infrastructure

#### 1.1 Common Utilities
```rust
// benches/common/mod.rs
use criterion::{Criterion, BenchmarkId};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

pub struct BenchmarkContext {
    pub rt: Runtime,
    pub warmup_duration: Duration,
    pub measurement_duration: Duration,
}

impl Default for BenchmarkContext {
    fn default() -> Self {
        Self {
            rt: Runtime::new().unwrap(),
            warmup_duration: Duration::from_secs(3),
            measurement_duration: Duration::from_secs(10),
        }
    }
}

pub trait AsyncBenchmark {
    type Setup;
    type Input;
    type Output;
    
    fn setup(&self) -> Self::Setup;
    async fn execute(&self, setup: &Self::Setup, input: Self::Input) -> Self::Output;
}

pub fn bench_async<B, F, Fut>(c: &mut Criterion, name: &str, mut f: F)
where
    B: AsyncBenchmark,
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let rt = Runtime::new().unwrap();
    c.bench_function(name, |b| {
        b.to_async(&rt).iter(|| f())
    });
}

pub struct MemoryProfiler {
    initial_usage: usize,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            initial_usage: get_memory_usage(),
        }
    }
    
    pub fn measure<F, R>(&self, f: F) -> (R, usize)
    where F: FnOnce() -> R {
        let start = get_memory_usage();
        let result = f();
        let end = get_memory_usage();
        (result, end.saturating_sub(start))
    }
}

fn get_memory_usage() -> usize {
    // Platform-specific memory usage collection
    #[cfg(target_os = "linux")]
    {
        // Parse /proc/self/status
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with("VmRSS:"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|size| size.parse::<usize>().ok())
                    .map(|kb| kb * 1024)
            })
            .unwrap_or(0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        0 // Fallback for other platforms
    }
}
```

#### 1.2 Performance Regression Detection
```rust
// benches/common/regression.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration_ns: u64,
    pub throughput: Option<f64>,
    pub memory_usage: Option<usize>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct BenchmarkBaseline {
    pub results: HashMap<String, BenchmarkResult>,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct RegressionDetector {
    baseline: BenchmarkBaseline,
    threshold: f64, // 10% = 0.1
}

impl RegressionDetector {
    pub fn load_baseline<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let baseline: BenchmarkBaseline = serde_json::from_str(&content)?;
        
        Ok(Self {
            baseline,
            threshold: 0.1, // 10% regression threshold
        })
    }
    
    pub fn check_regression(&self, current: &BenchmarkResult) -> Option<RegressionAlert> {
        let baseline_result = self.baseline.results.get(&current.name)?;
        
        let regression_ratio = (current.duration_ns as f64) / (baseline_result.duration_ns as f64);
        
        if regression_ratio > (1.0 + self.threshold) {
            Some(RegressionAlert {
                benchmark_name: current.name.clone(),
                baseline_duration_ns: baseline_result.duration_ns,
                current_duration_ns: current.duration_ns,
                regression_percentage: (regression_ratio - 1.0) * 100.0,
                severity: if regression_ratio > 1.5 { 
                    RegressionSeverity::Critical 
                } else if regression_ratio > 1.2 { 
                    RegressionSeverity::Major 
                } else { 
                    RegressionSeverity::Minor 
                },
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct RegressionAlert {
    pub benchmark_name: String,
    pub baseline_duration_ns: u64,
    pub current_duration_ns: u64,
    pub regression_percentage: f64,
    pub severity: RegressionSeverity,
}

#[derive(Debug)]
pub enum RegressionSeverity {
    Minor,   // 10-20% regression
    Major,   // 20-50% regression  
    Critical, // >50% regression
}
```

### 2. Core Crate Benchmark Implementation

#### 2.1 mf-core Benchmarks
```rust
// crates/core/benches/runtime.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use mf_core::runtime::Runtime;
use mf_core::event::{Event, EventBus};
use tokio::runtime::Runtime as TokioRuntime;

fn bench_runtime_creation(c: &mut Criterion) {
    c.bench_function("runtime/creation", |b| {
        b.iter(|| {
            let rt = Runtime::new();
            criterion::black_box(rt);
        })
    });
}

fn bench_event_dispatch(c: &mut Criterion) {
    let tokio_rt = TokioRuntime::new().unwrap();
    let event_bus = EventBus::new();
    
    c.bench_function("event/dispatch_single", |b| {
        b.to_async(&tokio_rt).iter(|| async {
            let event = Event::new("test", serde_json::Value::Null);
            event_bus.dispatch(event).await.unwrap();
        })
    });
    
    let mut group = c.benchmark_group("event/dispatch_batch");
    for batch_size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("batch_size", batch_size), batch_size, |b, &size| {
            b.to_async(&tokio_rt).iter(|| async {
                let events: Vec<_> = (0..size)
                    .map(|i| Event::new(&format!("test_{}", i), serde_json::Value::Null))
                    .collect();
                
                for event in events {
                    event_bus.dispatch(event).await.unwrap();
                }
            })
        });
    }
    group.finish();
}

fn bench_middleware_pipeline(c: &mut Criterion) {
    // Benchmark middleware chain processing
    let tokio_rt = TokioRuntime::new().unwrap();
    
    let mut group = c.benchmark_group("middleware/pipeline");
    for middleware_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("middleware_count", middleware_count), 
            middleware_count, 
            |b, &count| {
                b.to_async(&tokio_rt).iter(|| async {
                    // Setup middleware chain with `count` middleware
                    // Process request through pipeline
                    criterion::black_box(process_middleware_chain(count).await);
                })
            }
        );
    }
    group.finish();
}

async fn process_middleware_chain(count: usize) -> u64 {
    // Mock middleware processing
    let mut result = 0;
    for i in 0..count {
        result += i as u64;
        tokio::task::yield_now().await; // Simulate async work
    }
    result
}

criterion_group!(
    runtime_benches, 
    bench_runtime_creation,
    bench_event_dispatch,
    bench_middleware_pipeline
);
criterion_main!(runtime_benches);
```

#### 2.2 mf-state Benchmarks
```rust
// crates/state/benches/transaction.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use mf_state::{State, Transaction};
use mf_transform::node_step::AddNodeStep;
use mf_model::node::Node;
use tokio::runtime::Runtime;

fn bench_transaction_creation(c: &mut Criterion) {
    c.bench_function("transaction/creation", |b| {
        b.iter(|| {
            let mut tx = Transaction::new();
            tx.add_step(AddNodeStep::new(
                Node::new("test", "paragraph"),
                None
            ));
            criterion::black_box(tx);
        })
    });
}

fn bench_transaction_application(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let state = State::new();
    
    let mut group = c.benchmark_group("transaction/application");
    for step_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("step_count", step_count), 
            step_count, 
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let mut tx = Transaction::new();
                    
                    // Add multiple steps to transaction
                    for i in 0..count {
                        tx.add_step(AddNodeStep::new(
                            Node::new(&format!("node_{}", i), "paragraph"),
                            None
                        ));
                    }
                    
                    let result = state.apply_transaction(tx).await;
                    criterion::black_box(result);
                })
            }
        );
    }
    group.finish();
}

fn bench_concurrent_transactions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let state = State::new();
    
    c.bench_function("transaction/concurrent", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks: Vec<_> = (0..10).map(|i| {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    let mut tx = Transaction::new();
                    tx.add_step(AddNodeStep::new(
                        Node::new(&format!("concurrent_node_{}", i), "paragraph"),
                        None
                    ));
                    state_clone.apply_transaction(tx).await
                })
            }).collect();
            
            let results = futures::future::try_join_all(tasks).await.unwrap();
            criterion::black_box(results);
        })
    });
}

criterion_group!(
    state_benches,
    bench_transaction_creation,
    bench_transaction_application,
    bench_concurrent_transactions
);
criterion_main!(state_benches);
```

#### 2.3 mf-collaboration Benchmarks
```rust
// crates/collaboration/benches/yrs_operations.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use mf_collaboration::{YrsManager, SyncService};
use tokio::runtime::Runtime;

fn bench_room_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let yrs_manager = YrsManager::new();
    
    c.bench_function("collaboration/room_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let room_id = format!("room_{}", fastrand::u64(..));
            let awareness = yrs_manager.get_or_create_awareness(&room_id);
            criterion::black_box(awareness);
        })
    });
}

fn bench_concurrent_users(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let sync_service = SyncService::new();
    
    let mut group = c.benchmark_group("collaboration/concurrent_users");
    for user_count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("user_count", user_count),
            user_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let room_id = format!("room_{}", fastrand::u64(..));
                    
                    // Simulate concurrent user connections
                    let tasks: Vec<_> = (0..count).map(|i| {
                        let service = sync_service.clone();
                        let room = room_id.clone();
                        tokio::spawn(async move {
                            let user_id = format!("user_{}", i);
                            service.join_room(&room, &user_id).await
                        })
                    }).collect();
                    
                    let results = futures::future::try_join_all(tasks).await.unwrap();
                    criterion::black_box(results);
                })
            }
        );
    }
    group.finish();
}

fn bench_crdt_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let yrs_manager = YrsManager::new();
    let room_id = "test_room";
    let awareness = yrs_manager.get_or_create_awareness(room_id);
    
    c.bench_function("collaboration/crdt_text_insert", |b| {
        b.to_async(&rt).iter(|| async {
            let awareness_guard = awareness.write().await;
            let doc = awareness_guard.doc();
            let text = doc.get_or_insert_text("content");
            
            let content = format!("Hello World {}", fastrand::u64(..));
            text.insert(&mut doc.transact_mut(), 0, &content);
            
            criterion::black_box(text.len(&doc.transact()));
        })
    });
}

criterion_group!(
    collaboration_benches,
    bench_room_creation,
    bench_concurrent_users,
    bench_crdt_operations
);
criterion_main!(collaboration_benches);
```

### 3. File I/O and Serialization Benchmarks

#### 3.1 mf-file Benchmarks
```rust
// crates/file/benches/serialization.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use mf_file::{DocumentExporter, ExportFormat};
use mf_model::document::Document;
use tokio::runtime::Runtime;

fn create_test_document(node_count: usize) -> Document {
    let mut doc = Document::new();
    
    for i in 0..node_count {
        // Add nodes with various content types
        let node = mf_model::node::Node::new(
            &format!("node_{}", i),
            if i % 3 == 0 { "paragraph" } else if i % 3 == 1 { "heading" } else { "list_item" }
        );
        doc.add_node(node, None).unwrap();
    }
    
    doc
}

fn bench_serialization_formats(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let formats = vec![
        ExportFormat::Json,
        ExportFormat::Cbor,
        ExportFormat::MessagePack,
    ];
    
    for format in formats {
        let mut group = c.benchmark_group(format!("serialization/{:?}", format));
        
        for node_count in [100, 1000, 10000].iter() {
            let doc = create_test_document(*node_count);
            let doc_size = estimate_document_size(&doc);
            
            group.throughput(Throughput::Bytes(doc_size as u64));
            group.bench_with_input(
                BenchmarkId::new("node_count", node_count),
                node_count,
                |b, _| {
                    b.to_async(&rt).iter(|| async {
                        let exporter = DocumentExporter::new(format);
                        let result = exporter.export(&doc).await.unwrap();
                        criterion::black_box(result);
                    })
                }
            );
        }
        group.finish();
    }
}

fn bench_compression_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let doc = create_test_document(10000);
    
    let compression_levels = vec![1, 6, 9]; // Fast, balanced, best compression
    
    let mut group = c.benchmark_group("compression/efficiency");
    for level in compression_levels {
        group.bench_with_input(
            BenchmarkId::new("level", level),
            &level,
            |b, &lvl| {
                b.to_async(&rt).iter(|| async {
                    let exporter = DocumentExporter::new(ExportFormat::JsonZip { 
                        compression_level: lvl 
                    });
                    let result = exporter.export(&doc).await.unwrap();
                    criterion::black_box(result);
                })
            }
        );
    }
    group.finish();
}

fn estimate_document_size(doc: &Document) -> usize {
    // Rough estimation based on node count and content
    doc.node_count() * 200 // Average bytes per node
}

criterion_group!(
    file_benches,
    bench_serialization_formats,
    bench_compression_efficiency
);
criterion_main!(file_benches);
```

### 4. Search Performance Benchmarks

#### 4.1 mf-search Benchmarks
```rust
// crates/search/benches/indexing.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use mf_search::{SearchIndex, IndexBuilder, QueryProcessor};
use mf_model::document::Document;
use tokio::runtime::Runtime;

fn create_test_corpus(doc_count: usize, nodes_per_doc: usize) -> Vec<Document> {
    (0..doc_count).map(|i| {
        let mut doc = Document::new();
        
        for j in 0..nodes_per_doc {
            let content = generate_text_content(100); // 100 words
            let node = mf_model::node::Node::with_text(
                &format!("doc_{}_node_{}", i, j),
                "paragraph",
                &content
            );
            doc.add_node(node, None).unwrap();
        }
        
        doc
    }).collect()
}

fn generate_text_content(word_count: usize) -> String {
    let words = vec![
        "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
        "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
        "performance", "benchmark", "search", "index", "query", "processing", "optimization"
    ];
    
    (0..word_count)
        .map(|_| words[fastrand::usize(..words.len())])
        .collect::<Vec<_>>()
        .join(" ")
}

fn bench_index_building(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("search/index_building");
    
    for doc_count in [100, 500, 1000, 5000].iter() {
        let corpus = create_test_corpus(*doc_count, 10); // 10 nodes per doc
        let total_content_size: usize = corpus.iter()
            .map(|doc| estimate_document_text_size(doc))
            .sum();
        
        group.throughput(Throughput::Bytes(total_content_size as u64));
        group.bench_with_input(
            BenchmarkId::new("doc_count", doc_count),
            &corpus,
            |b, docs| {
                b.to_async(&rt).iter(|| async {
                    let mut index_builder = IndexBuilder::new();
                    
                    for doc in docs {
                        index_builder.add_document(doc).await.unwrap();
                    }
                    
                    let index = index_builder.build().await.unwrap();
                    criterion::black_box(index);
                })
            }
        );
    }
    group.finish();
}

fn bench_query_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Pre-build index for querying
    let corpus = create_test_corpus(1000, 10);
    let index = rt.block_on(async {
        let mut builder = IndexBuilder::new();
        for doc in &corpus {
            builder.add_document(doc).await.unwrap();
        }
        builder.build().await.unwrap()
    });
    
    let queries = vec![
        "quick brown fox",
        "lorem ipsum",
        "performance optimization",
        "search query processing",
        "benchmark test",
    ];
    
    let mut group = c.benchmark_group("search/query_processing");
    
    for query in queries {
        group.bench_with_input(
            BenchmarkId::new("query", query),
            &query,
            |b, &q| {
                b.to_async(&rt).iter(|| async {
                    let processor = QueryProcessor::new(&index);
                    let results = processor.search(q).await.unwrap();
                    criterion::black_box(results);
                })
            }
        );
    }
    group.finish();
}

fn bench_concurrent_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Pre-build index
    let corpus = create_test_corpus(1000, 10);
    let index = rt.block_on(async {
        let mut builder = IndexBuilder::new();
        for doc in &corpus {
            builder.add_document(doc).await.unwrap();
        }
        builder.build().await.unwrap()
    });
    
    c.bench_function("search/concurrent_queries", |b| {
        b.to_async(&rt).iter(|| async {
            let queries = vec![
                "quick brown fox",
                "lorem ipsum dolor",
                "performance benchmark",
                "search optimization",
                "concurrent processing",
            ];
            
            let tasks: Vec<_> = queries.into_iter().map(|query| {
                let index_ref = &index;
                tokio::spawn(async move {
                    let processor = QueryProcessor::new(index_ref);
                    processor.search(query).await
                })
            }).collect();
            
            let results = futures::future::try_join_all(tasks).await.unwrap();
            criterion::black_box(results);
        })
    });
}

fn estimate_document_text_size(doc: &Document) -> usize {
    doc.node_count() * 500 // Rough estimate of text content per node
}

criterion_group!(
    search_benches,
    bench_index_building,
    bench_query_processing,
    bench_concurrent_queries
);
criterion_main!(search_benches);
```

### 5. CI/CD Integration Script

```bash
#!/bin/bash
# scripts/run_benchmarks.sh

set -e

echo "🚀 Running ModuForge-RS Performance Benchmarks"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BASELINE_DIR="benchmark_baselines"
RESULTS_DIR="benchmark_results"
REGRESSION_THRESHOLD="10" # 10% regression threshold

# Create directories if they don't exist
mkdir -p "$BASELINE_DIR" "$RESULTS_DIR"

# Function to run benchmarks for a specific crate
run_crate_benchmarks() {
    local crate_name=$1
    echo -e "${YELLOW}📊 Benchmarking $crate_name${NC}"
    
    # Run benchmarks and save results
    cargo bench -p $crate_name -- --output-format json \
        > "$RESULTS_DIR/${crate_name}_$(date +%Y%m%d_%H%M%S).json" 2>&1
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $crate_name benchmarks completed${NC}"
    else
        echo -e "${RED}❌ $crate_name benchmarks failed${NC}"
        return 1
    fi
}

# Function to detect regressions
detect_regressions() {
    local crate_name=$1
    local current_results=$2
    local baseline_file="$BASELINE_DIR/${crate_name}_baseline.json"
    
    if [ -f "$baseline_file" ]; then
        echo -e "${YELLOW}🔍 Checking regressions for $crate_name${NC}"
        
        # Use custom regression detection script
        if python3 scripts/detect_regression.py \
            --baseline "$baseline_file" \
            --current "$current_results" \
            --threshold "$REGRESSION_THRESHOLD"; then
            echo -e "${GREEN}✅ No performance regressions detected${NC}"
        else
            echo -e "${RED}⚠️  Performance regression detected in $crate_name${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}📝 Creating baseline for $crate_name${NC}"
        cp "$current_results" "$baseline_file"
    fi
}

# List of crates to benchmark
BENCHMARK_CRATES=(
    "moduforge-core"
    "moduforge-state" 
    "moduforge-model"
    "moduforge-transform"
    "moduforge-collaboration"
    "moduforge-file"
    "moduforge-search"
    "moduforge-persistence"
    "moduforge-rules-engine"
    "moduforge-rules-expression"
    "moduforge-rules-template"
)

# Run benchmarks for all crates
echo -e "${GREEN}🏃 Starting benchmark run for ${#BENCHMARK_CRATES[@]} crates${NC}"

FAILED_CRATES=()
REGRESSION_CRATES=()

for crate in "${BENCHMARK_CRATES[@]}"; do
    if run_crate_benchmarks "$crate"; then
        # Find the most recent results file
        LATEST_RESULTS=$(ls -t "$RESULTS_DIR/${crate}_"*.json | head -n1)
        
        if detect_regressions "$crate" "$LATEST_RESULTS"; then
            echo -e "${GREEN}✅ $crate passed all checks${NC}"
        else
            REGRESSION_CRATES+=("$crate")
        fi
    else
        FAILED_CRATES+=("$crate")
    fi
done

# Summary
echo -e "\n${GREEN}📈 Benchmark Summary${NC}"
echo "Total crates: ${#BENCHMARK_CRATES[@]}"
echo "Successful: $((${#BENCHMARK_CRATES[@]} - ${#FAILED_CRATES[@]} - ${#REGRESSION_CRATES[@]}))"
echo "Failed: ${#FAILED_CRATES[@]}"
echo "Regressions: ${#REGRESSION_CRATES[@]}"

if [ ${#FAILED_CRATES[@]} -gt 0 ]; then
    echo -e "${RED}❌ Failed crates: ${FAILED_CRATES[*]}${NC}"
fi

if [ ${#REGRESSION_CRATES[@]} -gt 0 ]; then
    echo -e "${RED}⚠️  Regression crates: ${REGRESSION_CRATES[*]}${NC}"
fi

# Exit with error if there were failures or regressions
if [ ${#FAILED_CRATES[@]} -gt 0 ] || [ ${#REGRESSION_CRATES[@]} -gt 0 ]; then
    exit 1
fi

echo -e "${GREEN}🎉 All benchmarks completed successfully!${NC}"
```

### 6. GitHub Actions Workflow

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  schedule:
    # Run benchmarks daily at 2 AM UTC
    - cron: '0 2 * * *'

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  benchmarks:
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      
    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
        
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-bench-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-bench-
          ${{ runner.os }}-cargo-
          
    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y python3 python3-pip
        pip3 install matplotlib pandas numpy
        
    - name: Download baseline benchmarks
      uses: actions/download-artifact@v3
      with:
        name: benchmark-baselines
        path: benchmark_baselines/
      continue-on-error: true
      
    - name: Run benchmarks
      run: |
        chmod +x scripts/run_benchmarks.sh
        ./scripts/run_benchmarks.sh
        
    - name: Generate performance report
      run: |
        python3 scripts/generate_report.py \
          --results-dir benchmark_results \
          --output-dir reports
          
    - name: Upload benchmark results
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-results-${{ github.sha }}
        path: benchmark_results/
        
    - name: Upload performance report
      uses: actions/upload-artifact@v3
      with:
        name: performance-report-${{ github.sha }}
        path: reports/
        
    - name: Update baselines (main branch only)
      if: github.ref == 'refs/heads/main'
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-baselines
        path: benchmark_baselines/
        
    - name: Comment PR with results
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const path = require('path');
          
          // Read performance report
          const reportPath = path.join('reports', 'performance_summary.md');
          if (fs.existsSync(reportPath)) {
            const report = fs.readFileSync(reportPath, 'utf8');
            
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## 📊 Performance Benchmark Results\n\n${report}`
            });
          }
```

This comprehensive implementation strategy provides:

1. **Shared Infrastructure**: Common utilities for async benchmarking, memory profiling, and regression detection
2. **Crate-Specific Benchmarks**: Detailed implementation for core performance-critical components
3. **Automation**: Complete CI/CD integration with automated regression detection
4. **Scalability**: Framework that can grow with additional crates and benchmarks
5. **Developer Experience**: Clear scripts and tooling for local development

The benchmarks cover the most critical performance paths while providing actionable insights for optimization and regression prevention.
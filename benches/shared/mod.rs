// 共享基准测试工具库
use criterion::Criterion;
use std::time::Duration;
use tokio::runtime::Runtime;
use std::collections::HashMap;
use sysinfo::{System, SystemExt, ProcessExt, CpuExt};

/// 基准测试运行环境
pub struct BenchmarkHarness {
    runtime: Runtime,
    warmup_time: Duration,
    measurement_time: Duration,
    system: System,
}

impl BenchmarkHarness {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            runtime: Runtime::new().unwrap(),
            warmup_time: Duration::from_secs(3),
            measurement_time: Duration::from_secs(10),
            system,
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

impl Default for BenchmarkHarness {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能监控和分析器
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
    start_time: Option<std::time::Instant>,
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
        self.start_time = Some(std::time::Instant::now());
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

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// 测试数据生成器
pub mod test_data {
    use moduforge_model::{Node, Document};
    use serde_json::json;
    
    /// 创建指定节点数量的测试文档
    pub fn create_test_document(node_count: usize) -> Document {
        let mut doc = Document::new();
        for i in 0..node_count {
            let node = Node::new(format!("node_{}", i))
                .with_attribute("id", json!(i))
                .with_attribute("text", json!(format!("测试内容 {}", i)));
            doc.add_node(node);
        }
        doc
    }
    
    /// 创建指定大小的JSON测试数据
    pub fn create_large_json_data(size_mb: usize) -> serde_json::Value {
        let items_per_mb = 1000;
        let total_items = size_mb * items_per_mb;
        
        json!({
            "data": (0..total_items).map(|i| json!({
                "id": i,
                "title": format!("项目 {}", i),
                "description": "测试".repeat(100),
                "metadata": {
                    "created_at": "2024-01-01T00:00:00Z",
                    "tags": ["标签1", "标签2", "标签3"],
                    "scores": [0.1, 0.2, 0.3, 0.4, 0.5]
                }
            })).collect::<Vec<_>>()
        })
    }
}

/// 依赖解析和执行顺序管理
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub path: String,
    pub dependencies: Vec<String>,
    pub tier: ExecutionTier,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionTier {
    Foundation,   // 基础层: model, derive, macro
    CoreLogic,    // 核心逻辑层: transform, expression, template
    Service,      // 服务层: state, engine, file, search, persistence
    Integration,  // 集成层: core, collaboration, collaboration_client
}

pub struct BenchmarkOrchestrator {
    crates: Vec<CrateInfo>,
}

impl BenchmarkOrchestrator {
    pub fn new() -> Self {
        let crates = vec![
            // 基础层 (无依赖)
            CrateInfo {
                name: "moduforge-model".to_string(),
                path: "crates/model".to_string(),
                dependencies: vec![],
                tier: ExecutionTier::Foundation,
            },
            CrateInfo {
                name: "moduforge-macros-derive".to_string(),
                path: "crates/derive".to_string(),
                dependencies: vec![],
                tier: ExecutionTier::Foundation,
            },
            CrateInfo {
                name: "moduforge-macros".to_string(),
                path: "crates/macro".to_string(),
                dependencies: vec![],
                tier: ExecutionTier::Foundation,
            },
            
            // 核心逻辑层 (1-2个依赖)
            CrateInfo {
                name: "moduforge-transform".to_string(),
                path: "crates/transform".to_string(),
                dependencies: vec!["moduforge-model".to_string()],
                tier: ExecutionTier::CoreLogic,
            },
            CrateInfo {
                name: "moduforge-rules-expression".to_string(),
                path: "crates/expression".to_string(),
                dependencies: vec![],
                tier: ExecutionTier::CoreLogic,
            },
            CrateInfo {
                name: "moduforge-rules-template".to_string(),
                path: "crates/template".to_string(),
                dependencies: vec!["moduforge-rules-expression".to_string()],
                tier: ExecutionTier::CoreLogic,
            },
            
            // 服务层 (2-4个依赖)
            CrateInfo {
                name: "moduforge-state".to_string(),
                path: "crates/state".to_string(),
                dependencies: vec!["moduforge-model".to_string(), "moduforge-transform".to_string()],
                tier: ExecutionTier::Service,
            },
            CrateInfo {
                name: "moduforge-rules-engine".to_string(),
                path: "crates/engine".to_string(),
                dependencies: vec!["moduforge-rules-expression".to_string(), "moduforge-rules-template".to_string()],
                tier: ExecutionTier::Service,
            },
            CrateInfo {
                name: "moduforge-file".to_string(),
                path: "crates/file".to_string(),
                dependencies: vec!["moduforge-model".to_string()],
                tier: ExecutionTier::Service,
            },
            CrateInfo {
                name: "moduforge-search".to_string(),
                path: "crates/search".to_string(),
                dependencies: vec!["moduforge-model".to_string()],
                tier: ExecutionTier::Service,
            },
            CrateInfo {
                name: "moduforge-persistence".to_string(),
                path: "crates/persistence".to_string(),
                dependencies: vec!["moduforge-model".to_string(), "moduforge-state".to_string()],
                tier: ExecutionTier::Service,
            },
            
            // 集成层 (4+个依赖)
            CrateInfo {
                name: "moduforge-core".to_string(),
                path: "crates/core".to_string(),
                dependencies: vec![
                    "moduforge-model".to_string(),
                    "moduforge-state".to_string(),
                    "moduforge-transform".to_string(),
                    "moduforge-rules-engine".to_string(),
                ],
                tier: ExecutionTier::Integration,
            },
            CrateInfo {
                name: "moduforge-collaboration".to_string(),
                path: "crates/collaboration".to_string(),
                dependencies: vec![
                    "moduforge-model".to_string(),
                    "moduforge-state".to_string(),
                    "moduforge-transform".to_string(),
                ],
                tier: ExecutionTier::Integration,
            },
            CrateInfo {
                name: "moduforge-collaboration-client".to_string(),
                path: "crates/collaboration_client".to_string(),
                dependencies: vec!["moduforge-collaboration".to_string()],
                tier: ExecutionTier::Integration,
            },
        ];
        
        Self { crates }
    }
    
    /// 获取按执行顺序排序的crate列表
    pub fn get_execution_order(&self) -> Vec<Vec<&CrateInfo>> {
        let mut batches: HashMap<ExecutionTier, Vec<&CrateInfo>> = HashMap::new();
        
        for crate_info in &self.crates {
            batches.entry(crate_info.tier.clone())
                .or_insert_with(Vec::new)
                .push(crate_info);
        }
        
        vec![
            batches.get(&ExecutionTier::Foundation).unwrap_or(&vec![]).clone(),
            batches.get(&ExecutionTier::CoreLogic).unwrap_or(&vec![]).clone(),
            batches.get(&ExecutionTier::Service).unwrap_or(&vec![]).clone(),
            batches.get(&ExecutionTier::Integration).unwrap_or(&vec![]).clone(),
        ]
    }
    
    /// 检查是否所有依赖都已完成基准测试
    pub fn can_execute(&self, crate_name: &str, completed: &[String]) -> bool {
        if let Some(crate_info) = self.crates.iter().find(|c| c.name == crate_name) {
            crate_info.dependencies.iter().all(|dep| completed.contains(dep))
        } else {
            false
        }
    }
}

impl Default for BenchmarkOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
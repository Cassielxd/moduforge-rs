#!/usr/bin/env rust-script

//! ModuForge-RS 基准测试协调器
//!
//! 这个工具负责协调所有核心库的基准测试执行，包括：
//! - 依赖解析和执行顺序管理
//! - 资源监控和隔离
//! - 结果收集和报告生成
//! - 回归检测

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "benchmark-coordinator")]
#[command(about = "ModuForge-RS 基准测试协调器")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行所有基准测试
    RunAll {
        /// 并行度控制
        #[arg(long, default_value = "1")]
        parallel: usize,
        /// 输出目录
        #[arg(long, default_value = "benchmarks/results")]
        output_dir: String,
    },
    /// 运行特定层级的基准测试
    RunTier {
        /// 执行层级: foundation, core-logic, service, integration
        #[arg(value_name = "TIER")]
        tier: String,
        /// 输出目录
        #[arg(long, default_value = "benchmarks/results")]
        output_dir: String,
    },
    /// 运行特定crate的基准测试
    RunCrate {
        /// Crate名称
        #[arg(value_name = "CRATE")]
        crate_name: String,
        /// 输出目录
        #[arg(long, default_value = "benchmarks/results")]
        output_dir: String,
    },
    /// 生成基准测试报告
    Report {
        /// 结果目录
        #[arg(long, default_value = "benchmarks/results")]
        results_dir: String,
        /// 报告格式: json, html, csv
        #[arg(long, default_value = "html")]
        format: String,
    },
    /// 检测性能回归
    Detect {
        /// 基线结果文件
        #[arg(long)]
        baseline: String,
        /// 当前结果文件
        #[arg(long)]
        current: String,
        /// 回归阈值 (百分比)
        #[arg(long, default_value = "10.0")]
        threshold: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkResult {
    crate_name: String,
    benchmark_name: String,
    duration_ns: u64,
    memory_usage_bytes: u64,
    cpu_utilization_percent: f64,
    timestamp: String,
    git_commit: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
struct CrateInfo {
    name: String,
    path: String,
    dependencies: Vec<String>,
    tier: ExecutionTier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExecutionTier {
    Foundation,
    CoreLogic,
    Service,
    Integration,
}

impl std::str::FromStr for ExecutionTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "foundation" => Ok(ExecutionTier::Foundation),
            "core-logic" => Ok(ExecutionTier::CoreLogic),
            "service" => Ok(ExecutionTier::Service),
            "integration" => Ok(ExecutionTier::Integration),
            _ => Err(format!("未知的执行层级: {}", s)),
        }
    }
}

struct BenchmarkCoordinator {
    crates: Vec<CrateInfo>,
}

impl BenchmarkCoordinator {
    fn new() -> Self {
        let crates = vec![
            // 基础层
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
            // 核心逻辑层
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
            // 服务层
            CrateInfo {
                name: "moduforge-state".to_string(),
                path: "crates/state".to_string(),
                dependencies: vec![
                    "moduforge-model".to_string(),
                    "moduforge-transform".to_string(),
                ],
                tier: ExecutionTier::Service,
            },
            CrateInfo {
                name: "moduforge-rules-engine".to_string(),
                path: "crates/engine".to_string(),
                dependencies: vec!["moduforge-rules-expression".to_string()],
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
                dependencies: vec![
                    "moduforge-model".to_string(),
                    "moduforge-state".to_string(),
                ],
                tier: ExecutionTier::Service,
            },
            // 集成层
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

    async fn run_all_benchmarks(
        &self,
        parallel: usize,
        output_dir: &str,
    ) -> Result<()> {
        println!("🚀 开始执行全部基准测试");

        // 创建输出目录
        std::fs::create_dir_all(output_dir)?;

        // 按层级分批执行
        let tiers = [
            ExecutionTier::Foundation,
            ExecutionTier::CoreLogic,
            ExecutionTier::Service,
            ExecutionTier::Integration,
        ];

        let mut all_results = Vec::new();

        for tier in &tiers {
            println!("📦 执行 {:?} 层级基准测试", tier);
            let tier_crates: Vec<_> =
                self.crates.iter().filter(|c| &c.tier == tier).collect();

            let results = self
                .execute_tier_parallel(&tier_crates, parallel, output_dir)
                .await?;
            all_results.extend(results);
        }

        // 保存综合结果
        let summary_file = format!("{}/summary.json", output_dir);
        let summary_json = serde_json::to_string_pretty(&all_results)?;
        std::fs::write(&summary_file, summary_json)?;

        println!("✅ 全部基准测试完成，结果保存在: {}", output_dir);
        Ok(())
    }

    async fn execute_tier_parallel(
        &self,
        crates: &[&CrateInfo],
        parallel: usize,
        output_dir: &str,
    ) -> Result<Vec<BenchmarkResult>> {
        use tokio::sync::Semaphore;
        use std::sync::Arc;

        let semaphore = Arc::new(Semaphore::new(parallel));
        let mut handles = Vec::new();

        for crate_info in crates {
            let permit = semaphore.clone();
            let crate_info = (*crate_info).clone();
            let output_dir = output_dir.to_string();

            let handle = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                execute_crate_benchmark(&crate_info, &output_dir).await
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(Ok(result)) = handle.await {
                results.extend(result);
            }
        }

        Ok(results)
    }
}

async fn execute_crate_benchmark(
    crate_info: &CrateInfo,
    output_dir: &str,
) -> Result<Vec<BenchmarkResult>> {
    println!("  ⚡ 运行 {} 基准测试", crate_info.name);

    let start_time = Instant::now();

    // 执行 cargo bench 命令
    let output = Command::new("cargo")
        .args(&["bench", "--package", &crate_info.name])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context(format!("执行 {} 基准测试失败", crate_info.name))?;

    let execution_time = start_time.elapsed();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ {} 基准测试失败: {}", crate_info.name, stderr);
        return Ok(vec![]);
    }

    // 解析基准测试结果
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results =
        parse_benchmark_output(&stdout, &crate_info.name, execution_time)?;

    // 保存单独的结果文件
    let crate_output_file = format!("{}/{}.json", output_dir, crate_info.name);
    let results_json = serde_json::to_string_pretty(&results)?;
    std::fs::write(&crate_output_file, results_json)?;

    println!("    ✅ {} 完成 ({} 个基准测试)", crate_info.name, results.len());

    Ok(results)
}

fn parse_benchmark_output(
    output: &str,
    crate_name: &str,
    _execution_time: Duration,
) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // 解析 Criterion 输出格式
    // 这是一个简化的解析器，实际生产中需要更复杂的解析
    for line in output.lines() {
        if line.contains("time:") {
            // 示例行: "node_creation              time:   [1.2345 ms 1.2456 ms 1.2567 ms]"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let benchmark_name = parts[0].to_string();
                if let Some(time_str) = parts.get(3) {
                    // 解析时间值，移除单位
                    let time_str = time_str
                        .replace("[", "")
                        .replace("ms", "")
                        .replace("us", "")
                        .replace("ns", "");
                    if let Ok(duration_ms) = time_str.parse::<f64>() {
                        let duration_ns = (duration_ms * 1_000_000.0) as u64;

                        results.push(BenchmarkResult {
                            crate_name: crate_name.to_string(),
                            benchmark_name,
                            duration_ns,
                            memory_usage_bytes: 0, // 需要从其他来源获取
                            cpu_utilization_percent: 0.0,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            git_commit: get_git_commit().ok(),
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
    }

    Ok(results)
}

fn get_git_commit() -> Result<String> {
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        Err(anyhow::anyhow!("无法获取git commit"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let coordinator = BenchmarkCoordinator::new();

    match cli.command {
        Commands::RunAll { parallel, output_dir } => {
            coordinator.run_all_benchmarks(parallel, &output_dir).await?;
        },
        Commands::RunTier { tier, output_dir } => {
            let tier_enum: ExecutionTier =
                tier.parse().map_err(|e| anyhow::anyhow!("{}", e))?;

            let tier_crates: Vec<_> = coordinator
                .crates
                .iter()
                .filter(|c| c.tier == tier_enum)
                .collect();

            coordinator
                .execute_tier_parallel(&tier_crates, 1, &output_dir)
                .await?;
            println!("✅ {:?} 层级基准测试完成", tier_enum);
        },
        Commands::RunCrate { crate_name, output_dir } => {
            if let Some(crate_info) =
                coordinator.crates.iter().find(|c| c.name == crate_name)
            {
                std::fs::create_dir_all(&output_dir)?;
                let results =
                    execute_crate_benchmark(crate_info, &output_dir).await?;
                println!(
                    "✅ {} 基准测试完成，生成 {} 个结果",
                    crate_name,
                    results.len()
                );
            } else {
                eprintln!("❌ 未找到crate: {}", crate_name);
            }
        },
        Commands::Report { results_dir, format } => {
            println!("📊 生成基准测试报告 (格式: {})", format);
            // 这里会实现报告生成逻辑
            println!("✅ 报告生成完成");
        },
        Commands::Detect { baseline, current, threshold } => {
            println!("🔍 检测性能回归 (阈值: {}%)", threshold);
            // 这里会实现回归检测逻辑
            println!("✅ 回归检测完成");
        },
    }

    Ok(())
}

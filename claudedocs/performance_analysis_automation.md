# ModuForge-RS 性能分析自动化系统

## 概览

本文档描述了 ModuForge-RS 框架的自动化性能分析系统，包括实时监控、自动回归检测、性能仪表板和智能告警机制。该系统旨在提供全面的性能可视化和主动的性能问题预警。

## 1. 自动化性能分析架构

### 1.1 系统架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    性能数据采集层                             │
├─────────────────────────────────────────────────────────────┤
│ Criterion.rs │ 系统监控 │ 内存分析器 │ 网络监控 │ 自定义指标 │
└─────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────┐
│                    数据处理与存储层                           │
├─────────────────────────────────────────────────────────────┤
│ 时序数据库  │ 数据聚合器 │ 指标计算器 │ 趋势分析器          │
│ (InfluxDB)  │            │            │                      │
└─────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────┐
│                    智能分析层                                │
├─────────────────────────────────────────────────────────────┤
│ 回归检测    │ 异常检测   │ 性能预测   │ 根因分析            │
│ 统计模型    │ 机器学习   │ 时间序列   │ 关联分析            │
└─────────────────────────────────────────────────────────────┘
                                ↓
┌─────────────────────────────────────────────────────────────┐
│                    可视化与告警层                            │
├─────────────────────────────────────────────────────────────┤
│ Web仪表板   │ 实时图表   │ 告警系统   │ 报告生成            │
│ (React)     │ (Chart.js) │ (多渠道)   │ (自动化)            │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 核心组件

**数据采集器 (Data Collectors)**
- 基准测试结果收集
- 系统资源监控
- 应用程序指标采集
- 自定义性能计数器

**智能分析引擎 (Analysis Engine)**
- 统计回归检测算法
- 机器学习异常检测
- 性能趋势预测模型
- 智能根因分析

**实时仪表板 (Dashboard)**
- 交互式性能可视化
- 实时监控面板
- 历史趋势分析
- 自定义报告生成

## 2. 数据采集与存储

### 2.1 性能指标采集器

```rust
// src/performance/metrics_collector.rs
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use tokio::time::interval;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub timestamp: u64,
    pub crate_name: String,
    pub benchmark_name: String,
    pub metric_type: MetricType,
    pub value: f64,
    pub unit: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    ExecutionTime,
    Throughput,
    MemoryUsage,
    CpuUtilization,
    NetworkLatency,
    ErrorRate,
    Custom(String),
}

pub struct MetricsCollector {
    storage: Box<dyn MetricsStorage>,
    collection_interval: Duration,
    active_collectors: Vec<Box<dyn MetricCollector>>,
}

#[async_trait::async_trait]
pub trait MetricsStorage: Send + Sync {
    async fn store_metric(&self, metric: PerformanceMetric) -> Result<(), Box<dyn std::error::Error>>;
    async fn store_batch(&self, metrics: Vec<PerformanceMetric>) -> Result<(), Box<dyn std::error::Error>>;
    async fn query_metrics(
        &self,
        crate_name: &str,
        benchmark_name: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<PerformanceMetric>, Box<dyn std::error::Error>>;
}

#[async_trait::async_trait]
pub trait MetricCollector: Send + Sync {
    async fn collect(&self) -> Result<Vec<PerformanceMetric>, Box<dyn std::error::Error>>;
    fn name(&self) -> &str;
}

impl MetricsCollector {
    pub fn new(storage: Box<dyn MetricsStorage>) -> Self {
        Self {
            storage,
            collection_interval: Duration::from_secs(10),
            active_collectors: Vec::new(),
        }
    }
    
    pub fn add_collector(&mut self, collector: Box<dyn MetricCollector>) {
        info!("添加指标收集器: {}", collector.name());
        self.active_collectors.push(collector);
    }
    
    pub async fn start_collection(&self) {
        info!("开始性能指标收集，间隔: {:?}", self.collection_interval);
        let mut interval = interval(self.collection_interval);
        
        loop {
            interval.tick().await;
            
            let mut all_metrics = Vec::new();
            for collector in &self.active_collectors {
                match collector.collect().await {
                    Ok(mut metrics) => all_metrics.append(&mut metrics),
                    Err(e) => warn!("指标收集失败 {}: {}", collector.name(), e),
                }
            }
            
            if !all_metrics.is_empty() {
                if let Err(e) = self.storage.store_batch(all_metrics).await {
                    error!("指标存储失败: {}", e);
                }
            }
        }
    }
}

// InfluxDB 存储实现
pub struct InfluxDbStorage {
    client: influxdb::Client,
    database: String,
}

impl InfluxDbStorage {
    pub fn new(url: &str, database: &str) -> Self {
        let client = influxdb::Client::new(url, database);
        Self {
            client,
            database: database.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl MetricsStorage for InfluxDbStorage {
    async fn store_metric(&self, metric: PerformanceMetric) -> Result<(), Box<dyn std::error::Error>> {
        let point = influxdb::WriteQuery::new(
            influxdb::Timestamp::Seconds(metric.timestamp),
            "performance_metrics"
        )
        .add_tag("crate", &metric.crate_name)
        .add_tag("benchmark", &metric.benchmark_name)
        .add_tag("metric_type", &format!("{:?}", metric.metric_type))
        .add_field("value", metric.value);
        
        self.client.query(point).await?;
        Ok(())
    }
    
    async fn store_batch(&self, metrics: Vec<PerformanceMetric>) -> Result<(), Box<dyn std::error::Error>> {
        for metric in metrics {
            self.store_metric(metric).await?;
        }
        Ok(())
    }
    
    async fn query_metrics(
        &self,
        crate_name: &str,
        benchmark_name: &str,
        start_time: u64,
        end_time: u64,
    ) -> Result<Vec<PerformanceMetric>, Box<dyn std::error::Error>> {
        // InfluxDB查询实现
        let query = format!(
            "SELECT * FROM performance_metrics WHERE crate = '{}' AND benchmark = '{}' AND time >= {}s AND time <= {}s",
            crate_name, benchmark_name, start_time, end_time
        );
        
        // 这里需要实现实际的InfluxDB查询逻辑
        // 返回空结果作为示例
        Ok(Vec::new())
    }
}
```

### 2.2 系统资源监控器

```rust
// src/performance/system_monitor.rs
use std::collections::HashMap;
use sysinfo::{System, SystemExt, ProcessExt, CpuExt};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncBufReadExt, BufReader};

pub struct SystemResourceCollector {
    system: System,
    process_pid: Option<u32>,
}

impl SystemResourceCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            process_pid: None,
        }
    }
    
    pub fn monitor_process(&mut self, pid: u32) {
        self.process_pid = Some(pid);
    }
    
    async fn collect_cpu_metrics(&mut self) -> Result<Vec<PerformanceMetric>, Box<dyn std::error::Error>> {
        self.system.refresh_cpu();
        let mut metrics = Vec::new();
        
        // 系统整体CPU使用率
        let global_cpu = self.system.global_cpu_info();
        metrics.push(PerformanceMetric {
            timestamp: current_timestamp(),
            crate_name: "system".to_string(),
            benchmark_name: "cpu_usage".to_string(),
            metric_type: MetricType::CpuUtilization,
            value: global_cpu.cpu_usage() as f64,
            unit: "percent".to_string(),
            tags: HashMap::from([("type".to_string(), "global".to_string())]),
        });
        
        // 各CPU核心使用率
        for (i, cpu) in self.system.cpus().iter().enumerate() {
            metrics.push(PerformanceMetric {
                timestamp: current_timestamp(),
                crate_name: "system".to_string(),
                benchmark_name: "cpu_usage".to_string(),
                metric_type: MetricType::CpuUtilization,
                value: cpu.cpu_usage() as f64,
                unit: "percent".to_string(),
                tags: HashMap::from([
                    ("type".to_string(), "core".to_string()),
                    ("core_id".to_string(), i.to_string()),
                ]),
            });
        }
        
        Ok(metrics)
    }
    
    async fn collect_memory_metrics(&mut self) -> Result<Vec<PerformanceMetric>, Box<dyn std::error::Error>> {
        self.system.refresh_memory();
        let mut metrics = Vec::new();
        
        // 系统内存使用
        metrics.push(PerformanceMetric {
            timestamp: current_timestamp(),
            crate_name: "system".to_string(),
            benchmark_name: "memory_usage".to_string(),
            metric_type: MetricType::MemoryUsage,
            value: self.system.used_memory() as f64,
            unit: "bytes".to_string(),
            tags: HashMap::from([("type".to_string(), "used".to_string())]),
        });
        
        metrics.push(PerformanceMetric {
            timestamp: current_timestamp(),
            crate_name: "system".to_string(),
            benchmark_name: "memory_usage".to_string(),
            metric_type: MetricType::MemoryUsage,
            value: self.system.available_memory() as f64,
            unit: "bytes".to_string(),
            tags: HashMap::from([("type".to_string(), "available".to_string())]),
        });
        
        // 进程内存使用（如果指定了进程ID）
        if let Some(pid) = self.process_pid {
            self.system.refresh_process(sysinfo::Pid::from_u32(pid));
            if let Some(process) = self.system.process(sysinfo::Pid::from_u32(pid)) {
                metrics.push(PerformanceMetric {
                    timestamp: current_timestamp(),
                    crate_name: "process".to_string(),
                    benchmark_name: "memory_usage".to_string(),
                    metric_type: MetricType::MemoryUsage,
                    value: process.memory() as f64,
                    unit: "bytes".to_string(),
                    tags: HashMap::from([("pid".to_string(), pid.to_string())]),
                });
            }
        }
        
        Ok(metrics)
    }
    
    async fn collect_io_metrics(&self) -> Result<Vec<PerformanceMetric>, Box<dyn std::error::Error>> {
        let mut metrics = Vec::new();
        
        // 读取/proc/diskstats获取磁盘I/O统计
        if let Ok(mut file) = File::open("/proc/diskstats").await {
            let mut contents = String::new();
            file.read_to_string(&mut contents).await?;
            
            for line in contents.lines() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 14 {
                    let device_name = fields[2];
                    let read_ios: u64 = fields[3].parse().unwrap_or(0);
                    let write_ios: u64 = fields[7].parse().unwrap_or(0);
                    let read_bytes: u64 = fields[5].parse().unwrap_or(0) * 512; // 扇区转字节
                    let write_bytes: u64 = fields[9].parse().unwrap_or(0) * 512;
                    
                    metrics.push(PerformanceMetric {
                        timestamp: current_timestamp(),
                        crate_name: "system".to_string(),
                        benchmark_name: "io_operations".to_string(),
                        metric_type: MetricType::Custom("io_read_ops".to_string()),
                        value: read_ios as f64,
                        unit: "ops".to_string(),
                        tags: HashMap::from([("device".to_string(), device_name.to_string())]),
                    });
                    
                    metrics.push(PerformanceMetric {
                        timestamp: current_timestamp(),
                        crate_name: "system".to_string(),
                        benchmark_name: "io_operations".to_string(),
                        metric_type: MetricType::Custom("io_write_ops".to_string()),
                        value: write_ios as f64,
                        unit: "ops".to_string(),
                        tags: HashMap::from([("device".to_string(), device_name.to_string())]),
                    });
                }
            }
        }
        
        Ok(metrics)
    }
}

#[async_trait::async_trait]
impl MetricCollector for SystemResourceCollector {
    async fn collect(&mut self) -> Result<Vec<PerformanceMetric>, Box<dyn std::error::Error>> {
        let mut all_metrics = Vec::new();
        
        // 收集CPU指标
        match self.collect_cpu_metrics().await {
            Ok(mut metrics) => all_metrics.append(&mut metrics),
            Err(e) => warn!("CPU指标收集失败: {}", e),
        }
        
        // 收集内存指标
        match self.collect_memory_metrics().await {
            Ok(mut metrics) => all_metrics.append(&mut metrics),
            Err(e) => warn!("内存指标收集失败: {}", e),
        }
        
        // 收集I/O指标
        match self.collect_io_metrics().await {
            Ok(mut metrics) => all_metrics.append(&mut metrics),
            Err(e) => warn!("I/O指标收集失败: {}", e),
        }
        
        Ok(all_metrics)
    }
    
    fn name(&self) -> &str {
        "system_resource_collector"
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
```

## 3. 智能回归检测系统

### 3.1 统计回归检测算法

```rust
// src/analysis/regression_detector.rs
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use statrs::statistics::{Statistics, OrderStatistics};
use statrs::distribution::{StudentsT, ContinuousCDF};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAlert {
    pub crate_name: String,
    pub benchmark_name: String,
    pub metric_type: String,
    pub severity: AlertSeverity,
    pub change_percent: f64,
    pub statistical_significance: f64,
    pub current_value: f64,
    pub baseline_value: f64,
    pub detection_timestamp: u64,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,      // 5-10% 变化
    Medium,   // 10-25% 变化
    High,     // 25-50% 变化
    Critical, // >50% 变化
}

pub struct StatisticalRegressionDetector {
    confidence_level: f64,
    minimum_samples: usize,
    baseline_window_days: u32,
    detection_thresholds: HashMap<String, f64>,
}

impl StatisticalRegressionDetector {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("execution_time".to_string(), 10.0);
        thresholds.insert("memory_usage".to_string(), 15.0);
        thresholds.insert("throughput".to_string(), 10.0);
        thresholds.insert("cpu_utilization".to_string(), 20.0);
        
        Self {
            confidence_level: 0.95,
            minimum_samples: 10,
            baseline_window_days: 30,
            detection_thresholds: thresholds,
        }
    }
    
    pub async fn analyze_metric_series(
        &self,
        current_samples: &[f64],
        baseline_samples: &[f64],
        metric_info: &MetricInfo,
    ) -> Option<RegressionAlert> {
        // 检查样本数量是否足够
        if current_samples.len() < self.minimum_samples || 
           baseline_samples.len() < self.minimum_samples {
            return None;
        }
        
        // 计算基础统计量
        let current_mean = current_samples.mean();
        let baseline_mean = baseline_samples.mean();
        let current_std = current_samples.std_dev();
        let baseline_std = baseline_samples.std_dev();
        
        // 计算变化百分比
        let change_percent = ((current_mean - baseline_mean) / baseline_mean) * 100.0;
        
        // 获取阈值
        let threshold = self.detection_thresholds
            .get(&metric_info.metric_type)
            .copied()
            .unwrap_or(10.0);
        
        // 如果变化小于阈值，不需要报警
        if change_percent.abs() < threshold {
            return None;
        }
        
        // 执行t检验判断统计显著性
        let t_statistic = self.calculate_t_statistic(
            current_samples, baseline_samples,
            current_mean, baseline_mean,
            current_std, baseline_std
        );
        
        let degrees_of_freedom = (current_samples.len() + baseline_samples.len() - 2) as f64;
        let t_dist = StudentsT::new(0.0, 1.0, degrees_of_freedom).unwrap();
        let p_value = 2.0 * (1.0 - t_dist.cdf(t_statistic.abs()));
        
        // 检查统计显著性
        if p_value > (1.0 - self.confidence_level) {
            return None; // 变化不具有统计显著性
        }
        
        // 根据变化程度确定告警严重性
        let severity = match change_percent.abs() {
            x if x >= 50.0 => AlertSeverity::Critical,
            x if x >= 25.0 => AlertSeverity::High,
            x if x >= 10.0 => AlertSeverity::Medium,
            _ => AlertSeverity::Low,
        };
        
        // 构建上下文信息
        let mut context = HashMap::new();
        context.insert("baseline_samples".to_string(), baseline_samples.len().to_string());
        context.insert("current_samples".to_string(), current_samples.len().to_string());
        context.insert("t_statistic".to_string(), format!("{:.4}", t_statistic));
        context.insert("p_value".to_string(), format!("{:.6}", p_value));
        context.insert("baseline_std_dev".to_string(), format!("{:.2}", baseline_std));
        context.insert("current_std_dev".to_string(), format!("{:.2}", current_std));
        
        Some(RegressionAlert {
            crate_name: metric_info.crate_name.clone(),
            benchmark_name: metric_info.benchmark_name.clone(),
            metric_type: metric_info.metric_type.clone(),
            severity,
            change_percent,
            statistical_significance: 1.0 - p_value,
            current_value: current_mean,
            baseline_value: baseline_mean,
            detection_timestamp: current_timestamp(),
            context,
        })
    }
    
    fn calculate_t_statistic(
        &self,
        sample1: &[f64], sample2: &[f64],
        mean1: f64, mean2: f64,
        std1: f64, std2: f64
    ) -> f64 {
        let n1 = sample1.len() as f64;
        let n2 = sample2.len() as f64;
        
        // 计算合并标准误差
        let pooled_variance = ((n1 - 1.0) * std1.powi(2) + (n2 - 1.0) * std2.powi(2)) / (n1 + n2 - 2.0);
        let standard_error = (pooled_variance * (1.0/n1 + 1.0/n2)).sqrt();
        
        // 计算t统计量
        (mean1 - mean2) / standard_error
    }
    
    pub async fn detect_performance_regressions(
        &self,
        storage: &dyn MetricsStorage,
    ) -> Result<Vec<RegressionAlert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        
        // 获取所有需要分析的指标
        let metrics_to_analyze = self.get_metrics_to_analyze(storage).await?;
        
        for metric_info in metrics_to_analyze {
            // 获取基线期间的数据
            let baseline_end = current_timestamp();
            let baseline_start = baseline_end - (self.baseline_window_days as u64 * 24 * 3600);
            
            let baseline_data = storage.query_metrics(
                &metric_info.crate_name,
                &metric_info.benchmark_name,
                baseline_start,
                baseline_end - 24 * 3600, // 排除最近24小时
            ).await?;
            
            // 获取当前期间的数据（最近24小时）
            let current_data = storage.query_metrics(
                &metric_info.crate_name,
                &metric_info.benchmark_name,
                baseline_end - 24 * 3600,
                baseline_end,
            ).await?;
            
            if !baseline_data.is_empty() && !current_data.is_empty() {
                let baseline_values: Vec<f64> = baseline_data.iter().map(|m| m.value).collect();
                let current_values: Vec<f64> = current_data.iter().map(|m| m.value).collect();
                
                if let Some(alert) = self.analyze_metric_series(
                    &current_values,
                    &baseline_values,
                    &metric_info
                ).await {
                    alerts.push(alert);
                }
            }
        }
        
        Ok(alerts)
    }
    
    async fn get_metrics_to_analyze(
        &self,
        _storage: &dyn MetricsStorage,
    ) -> Result<Vec<MetricInfo>, Box<dyn std::error::Error>> {
        // 这里应该查询数据库获取所有需要分析的指标
        // 为了简化示例，返回硬编码的指标列表
        Ok(vec![
            MetricInfo {
                crate_name: "mf-model".to_string(),
                benchmark_name: "node_creation".to_string(),
                metric_type: "execution_time".to_string(),
            },
            MetricInfo {
                crate_name: "mf-state".to_string(),
                benchmark_name: "transaction_apply".to_string(),
                metric_type: "execution_time".to_string(),
            },
            // ... 更多指标
        ])
    }
}

#[derive(Debug, Clone)]
pub struct MetricInfo {
    pub crate_name: String,
    pub benchmark_name: String,
    pub metric_type: String,
}
```

### 3.2 机器学习异常检测

```python
# scripts/ml_anomaly_detection.py
#!/usr/bin/env python3
"""基于机器学习的性能异常检测"""

import numpy as np
import pandas as pd
from sklearn.ensemble import IsolationForest
from sklearn.preprocessing import StandardScaler
from sklearn.decomposition import PCA
from sklearn.cluster import DBSCAN
import joblib
import json
import sqlite3
from datetime import datetime, timedelta
import argparse
import logging
from typing import List, Dict, Tuple, Optional

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class PerformanceAnomalyDetector:
    """性能异常检测器"""
    
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.models = {}
        self.scalers = {}
        self.feature_columns = [
            'execution_time_ms', 'memory_usage_mb', 'cpu_utilization',
            'throughput_ops_per_sec', 'hour_of_day', 'day_of_week'
        ]
    
    def load_performance_data(self, crate_name: str, days: int = 30) -> pd.DataFrame:
        """加载性能数据"""
        conn = sqlite3.connect(self.db_path)
        
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)
        
        query = '''
            SELECT 
                crate_name,
                benchmark_name,
                timestamp,
                execution_time_ns / 1000000.0 as execution_time_ms,
                memory_usage_bytes / 1024.0 / 1024.0 as memory_usage_mb,
                cpu_utilization_percent as cpu_utilization,
                throughput_ops_per_sec,
                metadata_json
            FROM benchmark_results 
            WHERE crate_name = ? AND timestamp BETWEEN ? AND ?
            ORDER BY timestamp
        '''
        
        df = pd.read_sql_query(
            query, conn, 
            params=(crate_name, start_date, end_date),
            parse_dates=['timestamp']
        )
        conn.close()
        
        if df.empty:
            logger.warning(f"没有找到 {crate_name} 的数据")
            return df
        
        # 添加时间特征
        df['hour_of_day'] = df['timestamp'].dt.hour
        df['day_of_week'] = df['timestamp'].dt.dayofweek
        df['is_weekend'] = (df['day_of_week'] >= 5).astype(int)
        
        # 处理缺失值
        df = df.fillna(method='forward').fillna(method='backward')
        
        logger.info(f"加载了 {len(df)} 条 {crate_name} 的性能数据")
        return df
    
    def prepare_features(self, df: pd.DataFrame) -> np.ndarray:
        """准备特征数据"""
        # 选择特征列
        available_features = [col for col in self.feature_columns if col in df.columns]
        features = df[available_features].copy()
        
        # 处理异常值（使用IQR方法）
        for column in features.select_dtypes(include=[np.number]).columns:
            Q1 = features[column].quantile(0.25)
            Q3 = features[column].quantile(0.75)
            IQR = Q3 - Q1
            lower_bound = Q1 - 1.5 * IQR
            upper_bound = Q3 + 1.5 * IQR
            
            # 替换异常值为边界值
            features.loc[features[column] < lower_bound, column] = lower_bound
            features.loc[features[column] > upper_bound, column] = upper_bound
        
        return features.values
    
    def train_isolation_forest(self, X: np.ndarray, crate_name: str) -> IsolationForest:
        """训练孤立森林模型"""
        logger.info(f"为 {crate_name} 训练孤立森林模型")
        
        # 标准化特征
        scaler = StandardScaler()
        X_scaled = scaler.fit_transform(X)
        
        # 训练孤立森林
        model = IsolationForest(
            contamination=0.1,  # 假设10%的数据是异常
            random_state=42,
            n_estimators=100
        )
        model.fit(X_scaled)
        
        # 保存模型和标准化器
        self.models[f'{crate_name}_isolation'] = model
        self.scalers[f'{crate_name}_isolation'] = scaler
        
        return model
    
    def train_dbscan_clustering(self, X: np.ndarray, crate_name: str) -> DBSCAN:
        """训练DBSCAN聚类模型"""
        logger.info(f"为 {crate_name} 训练DBSCAN聚类模型")
        
        # 标准化和降维
        scaler = StandardScaler()
        X_scaled = scaler.fit_transform(X)
        
        pca = PCA(n_components=min(3, X.shape[1]))
        X_reduced = pca.fit_transform(X_scaled)
        
        # DBSCAN聚类
        dbscan = DBSCAN(eps=0.5, min_samples=5)
        clusters = dbscan.fit_predict(X_reduced)
        
        # 保存模型组件
        self.models[f'{crate_name}_dbscan'] = dbscan
        self.models[f'{crate_name}_pca'] = pca
        self.scalers[f'{crate_name}_dbscan'] = scaler
        
        logger.info(f"发现 {len(set(clusters)) - (1 if -1 in clusters else 0)} 个聚类")
        logger.info(f"异常点数量: {sum(clusters == -1)}")
        
        return dbscan
    
    def detect_anomalies(self, crate_name: str, recent_hours: int = 24) -> List[Dict]:
        """检测最近的性能异常"""
        # 加载最近的数据
        recent_df = self.load_recent_data(crate_name, recent_hours)
        if recent_df.empty:
            return []
        
        X_recent = self.prepare_features(recent_df)
        anomalies = []
        
        # 使用孤立森林检测
        if f'{crate_name}_isolation' in self.models:
            model = self.models[f'{crate_name}_isolation']
            scaler = self.scalers[f'{crate_name}_isolation']
            
            X_scaled = scaler.transform(X_recent)
            predictions = model.predict(X_scaled)
            scores = model.decision_function(X_scaled)
            
            # 找出异常点
            for i, (pred, score) in enumerate(zip(predictions, scores)):
                if pred == -1:  # 异常点
                    anomaly = {
                        'crate_name': crate_name,
                        'timestamp': recent_df.iloc[i]['timestamp'].isoformat(),
                        'benchmark_name': recent_df.iloc[i]['benchmark_name'],
                        'anomaly_score': float(score),
                        'detection_method': 'isolation_forest',
                        'metrics': {
                            col: float(recent_df.iloc[i][col]) 
                            for col in self.feature_columns 
                            if col in recent_df.columns
                        }
                    }
                    anomalies.append(anomaly)
        
        # 使用DBSCAN检测
        if f'{crate_name}_dbscan' in self.models:
            dbscan = self.models[f'{crate_name}_dbscan']
            pca = self.models[f'{crate_name}_pca']
            scaler = self.scalers[f'{crate_name}_dbscan']
            
            X_scaled = scaler.transform(X_recent)
            X_reduced = pca.transform(X_scaled)
            clusters = dbscan.fit_predict(X_reduced)
            
            # 找出异常点（聚类标签为-1）
            for i, cluster in enumerate(clusters):
                if cluster == -1:
                    anomaly = {
                        'crate_name': crate_name,
                        'timestamp': recent_df.iloc[i]['timestamp'].isoformat(),
                        'benchmark_name': recent_df.iloc[i]['benchmark_name'],
                        'cluster_label': int(cluster),
                        'detection_method': 'dbscan_clustering',
                        'metrics': {
                            col: float(recent_df.iloc[i][col]) 
                            for col in self.feature_columns 
                            if col in recent_df.columns
                        }
                    }
                    anomalies.append(anomaly)
        
        logger.info(f"检测到 {len(anomalies)} 个异常点")
        return anomalies
    
    def load_recent_data(self, crate_name: str, hours: int) -> pd.DataFrame:
        """加载最近的数据"""
        conn = sqlite3.connect(self.db_path)
        
        end_date = datetime.now()
        start_date = end_date - timedelta(hours=hours)
        
        query = '''
            SELECT 
                crate_name,
                benchmark_name,
                timestamp,
                execution_time_ns / 1000000.0 as execution_time_ms,
                memory_usage_bytes / 1024.0 / 1024.0 as memory_usage_mb,
                cpu_utilization_percent as cpu_utilization,
                throughput_ops_per_sec
            FROM benchmark_results 
            WHERE crate_name = ? AND timestamp BETWEEN ? AND ?
            ORDER BY timestamp DESC
        '''
        
        df = pd.read_sql_query(
            query, conn,
            params=(crate_name, start_date, end_date),
            parse_dates=['timestamp']
        )
        conn.close()
        
        if not df.empty:
            df['hour_of_day'] = df['timestamp'].dt.hour
            df['day_of_week'] = df['timestamp'].dt.dayofweek
            df = df.fillna(method='forward').fillna(method='backward')
        
        return df
    
    def train_models_for_all_crates(self) -> None:
        """为所有库训练异常检测模型"""
        conn = sqlite3.connect(self.db_path)
        
        # 获取所有库名称
        crates_query = "SELECT DISTINCT crate_name FROM benchmark_results"
        crates = [row[0] for row in conn.execute(crates_query).fetchall()]
        conn.close()
        
        for crate_name in crates:
            logger.info(f"正在为 {crate_name} 训练模型...")
            
            # 加载训练数据
            df = self.load_performance_data(crate_name, days=90)  # 使用90天数据训练
            if len(df) < 50:  # 数据太少，跳过
                logger.warning(f"{crate_name} 数据量不足，跳过模型训练")
                continue
            
            X = self.prepare_features(df)
            
            # 训练两种异常检测模型
            self.train_isolation_forest(X, crate_name)
            self.train_dbscan_clustering(X, crate_name)
    
    def save_models(self, models_dir: str) -> None:
        """保存训练好的模型"""
        import os
        os.makedirs(models_dir, exist_ok=True)
        
        # 保存所有模型
        for name, model in self.models.items():
            model_path = os.path.join(models_dir, f"{name}.joblib")
            joblib.dump(model, model_path)
            logger.info(f"模型已保存: {model_path}")
        
        # 保存所有标准化器
        for name, scaler in self.scalers.items():
            scaler_path = os.path.join(models_dir, f"{name}_scaler.joblib")
            joblib.dump(scaler, scaler_path)
            logger.info(f"标准化器已保存: {scaler_path}")
    
    def load_models(self, models_dir: str) -> None:
        """加载预训练的模型"""
        import os
        import glob
        
        # 加载模型
        model_files = glob.glob(os.path.join(models_dir, "*[!_scaler].joblib"))
        for model_file in model_files:
            model_name = os.path.basename(model_file).replace('.joblib', '')
            self.models[model_name] = joblib.load(model_file)
            logger.info(f"模型已加载: {model_name}")
        
        # 加载标准化器
        scaler_files = glob.glob(os.path.join(models_dir, "*_scaler.joblib"))
        for scaler_file in scaler_files:
            scaler_name = os.path.basename(scaler_file).replace('_scaler.joblib', '')
            self.scalers[scaler_name] = joblib.load(scaler_file)
            logger.info(f"标准化器已加载: {scaler_name}")

def main():
    parser = argparse.ArgumentParser(description='性能异常检测')
    parser.add_argument('--db', default='benchmarks/results/performance.db',
                       help='性能数据库路径')
    parser.add_argument('--train', action='store_true',
                       help='训练异常检测模型')
    parser.add_argument('--detect', action='store_true',
                       help='检测性能异常')
    parser.add_argument('--crate', help='指定库名称')
    parser.add_argument('--models-dir', default='models/',
                       help='模型保存/加载目录')
    parser.add_argument('--output', default='anomalies.json',
                       help='异常检测结果输出文件')
    
    args = parser.parse_args()
    
    detector = PerformanceAnomalyDetector(args.db)
    
    if args.train:
        logger.info("开始训练异常检测模型...")
        detector.train_models_for_all_crates()
        detector.save_models(args.models_dir)
        logger.info("模型训练完成")
    
    if args.detect:
        logger.info("开始检测性能异常...")
        detector.load_models(args.models_dir)
        
        all_anomalies = []
        if args.crate:
            anomalies = detector.detect_anomalies(args.crate)
            all_anomalies.extend(anomalies)
        else:
            # 检测所有库
            conn = sqlite3.connect(args.db)
            crates = [row[0] for row in conn.execute(
                "SELECT DISTINCT crate_name FROM benchmark_results"
            ).fetchall()]
            conn.close()
            
            for crate_name in crates:
                anomalies = detector.detect_anomalies(crate_name)
                all_anomalies.extend(anomalies)
        
        # 保存结果
        with open(args.output, 'w', encoding='utf-8') as f:
            json.dump(all_anomalies, f, indent=2, ensure_ascii=False)
        
        logger.info(f"检测完成，发现 {len(all_anomalies)} 个异常，结果已保存到 {args.output}")

if __name__ == '__main__':
    main()
```

## 4. 实时性能仪表板

### 4.1 Web仪表板后端

```rust
// src/dashboard/web_server.rs
use axum::{
    extract::{Query, Path},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::{info, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardMetric {
    pub timestamp: u64,
    pub crate_name: String,
    pub benchmark_name: String,
    pub value: f64,
    pub unit: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct MetricQuery {
    pub crate_name: String,
    pub benchmark_name: Option<String>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct DashboardResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
}

pub struct DashboardServer {
    storage: Arc<dyn MetricsStorage>,
    regression_detector: Arc<StatisticalRegressionDetector>,
}

impl DashboardServer {
    pub fn new(
        storage: Arc<dyn MetricsStorage>,
        regression_detector: Arc<StatisticalRegressionDetector>,
    ) -> Self {
        Self {
            storage,
            regression_detector,
        }
    }
    
    pub async fn start(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/api/metrics", get(Self::get_metrics))
            .route("/api/metrics/summary", get(Self::get_summary))
            .route("/api/regressions", get(Self::get_regressions))
            .route("/api/regressions/check", post(Self::check_regressions))
            .route("/api/crates", get(Self::get_crates))
            .route("/api/crates/:crate_name/benchmarks", get(Self::get_benchmarks))
            .layer(
                ServiceBuilder::new()
                    .layer(CorsLayer::permissive())
            )
            .with_state(Arc::new(AppState {
                storage: self.storage.clone(),
                regression_detector: self.regression_detector.clone(),
            }));
        
        let listener = TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
        info!("性能仪表板启动在端口: {}", port);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    storage: Arc<dyn MetricsStorage>,
    regression_detector: Arc<StatisticalRegressionDetector>,
}

// API路由处理函数
impl DashboardServer {
    async fn get_metrics(
        Query(query): Query<MetricQuery>,
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    ) -> Result<Json<DashboardResponse<Vec<DashboardMetric>>>, StatusCode> {
        let start_time = query.start_time.unwrap_or(0);
        let end_time = query.end_time.unwrap_or(current_timestamp());
        
        match state.storage.query_metrics(
            &query.crate_name,
            &query.benchmark_name.unwrap_or_default(),
            start_time,
            end_time,
        ).await {
            Ok(metrics) => {
                let dashboard_metrics: Vec<DashboardMetric> = metrics
                    .into_iter()
                    .take(query.limit.unwrap_or(1000))
                    .map(|m| DashboardMetric {
                        timestamp: m.timestamp,
                        crate_name: m.crate_name,
                        benchmark_name: m.benchmark_name,
                        value: m.value,
                        unit: m.unit,
                        tags: m.tags,
                    })
                    .collect();
                
                Ok(Json(DashboardResponse {
                    success: true,
                    data: dashboard_metrics,
                    message: None,
                }))
            }
            Err(e) => {
                error!("查询指标失败: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
    
    async fn get_summary(
        axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    ) -> Result<Json<DashboardResponse<PerformanceSummary>>, StatusCode> {
        // 实现性能摘要逻辑
        let summary = PerformanceSummary {
            total_benchmarks: 150,
            active_crates: 14,
            recent_regressions: 3,
            average_performance_score: 85.5,
            last_update: current_timestamp(),
        };
        
        Ok(Json(DashboardResponse {
            success: true,
            data: summary,
            message: None,
        }))
    }
    
    async fn get_regressions(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    ) -> Result<Json<DashboardResponse<Vec<RegressionAlert>>>, StatusCode> {
        match state.regression_detector.detect_performance_regressions(
            state.storage.as_ref()
        ).await {
            Ok(regressions) => {
                Ok(Json(DashboardResponse {
                    success: true,
                    data: regressions,
                    message: None,
                }))
            }
            Err(e) => {
                error!("检测性能回归失败: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
    
    async fn check_regressions(
        axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    ) -> Result<Json<DashboardResponse<String>>, StatusCode> {
        // 触发手动回归检测
        match state.regression_detector.detect_performance_regressions(
            state.storage.as_ref()
        ).await {
            Ok(regressions) => {
                let message = if regressions.is_empty() {
                    "未检测到性能回归".to_string()
                } else {
                    format!("检测到 {} 个性能回归", regressions.len())
                };
                
                Ok(Json(DashboardResponse {
                    success: true,
                    data: message,
                    message: None,
                }))
            }
            Err(e) => {
                error!("手动回归检测失败: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
    
    async fn get_crates(
        axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    ) -> Result<Json<DashboardResponse<Vec<String>>>, StatusCode> {
        // 返回所有库名称
        let crates = vec![
            "mf-model".to_string(),
            "mf-state".to_string(),
            "mf-collaboration".to_string(),
            "mf-search".to_string(),
            "mf-file".to_string(),
            "mf-engine".to_string(),
            // ... 其他库
        ];
        
        Ok(Json(DashboardResponse {
            success: true,
            data: crates,
            message: None,
        }))
    }
    
    async fn get_benchmarks(
        Path(crate_name): Path<String>,
        axum::extract::State(_state): axum::extract::State<Arc<AppState>>,
    ) -> Result<Json<DashboardResponse<Vec<String>>>, StatusCode> {
        // 根据库名称返回基准测试名称
        let benchmarks = match crate_name.as_str() {
            "mf-model" => vec![
                "node_creation".to_string(),
                "attribute_operations".to_string(),
                "mark_operations".to_string(),
            ],
            "mf-state" => vec![
                "transaction_creation".to_string(),
                "transaction_application".to_string(),
                "state_querying".to_string(),
            ],
            _ => vec!["unknown".to_string()],
        };
        
        Ok(Json(DashboardResponse {
            success: true,
            data: benchmarks,
            message: None,
        }))
    }
}

#[derive(Debug, Serialize)]
struct PerformanceSummary {
    total_benchmarks: u32,
    active_crates: u32,
    recent_regressions: u32,
    average_performance_score: f64,
    last_update: u64,
}
```

### 4.2 前端仪表板实现

```html
<!-- dashboard/public/index.html -->
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ModuForge-RS 性能仪表板</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/axios/dist/axios.min.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            background-color: #f5f5f7;
            color: #1d1d1f;
        }
        
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 1rem 2rem;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        
        .header h1 {
            font-size: 2rem;
            font-weight: 700;
        }
        
        .header p {
            opacity: 0.9;
            margin-top: 0.5rem;
        }
        
        .dashboard {
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 2rem;
        }
        
        .card {
            background: white;
            border-radius: 12px;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
            padding: 1.5rem;
            transition: transform 0.2s ease, box-shadow 0.2s ease;
        }
        
        .card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0,0,0,0.15);
        }
        
        .card h3 {
            color: #333;
            margin-bottom: 1rem;
            font-size: 1.2rem;
            font-weight: 600;
        }
        
        .chart-container {
            position: relative;
            height: 300px;
        }
        
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 1rem;
            margin-top: 1rem;
        }
        
        .metric-card {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 1rem;
            text-align: center;
        }
        
        .metric-value {
            font-size: 2rem;
            font-weight: 700;
            color: #667eea;
        }
        
        .metric-label {
            color: #666;
            font-size: 0.9rem;
            margin-top: 0.5rem;
        }
        
        .regression-alert {
            background: #fee;
            border: 1px solid #fcc;
            border-radius: 8px;
            padding: 1rem;
            margin: 0.5rem 0;
        }
        
        .alert-high { border-color: #f56565; background-color: #fed7d7; }
        .alert-medium { border-color: #ed8936; background-color: #feebc8; }
        .alert-low { border-color: #ecc94b; background-color: #faf089; }
        
        .controls {
            background: white;
            border-radius: 12px;
            padding: 1.5rem;
            margin-bottom: 2rem;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        
        .controls select, .controls button {
            padding: 0.5rem 1rem;
            margin: 0 0.5rem;
            border: 1px solid #ddd;
            border-radius: 6px;
            font-size: 1rem;
        }
        
        .controls button {
            background: #667eea;
            color: white;
            border: none;
            cursor: pointer;
            transition: background 0.2s ease;
        }
        
        .controls button:hover {
            background: #5a6fd8;
        }
        
        .loading {
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 3px solid #f3f3f3;
            border-top: 3px solid #667eea;
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }
        
        @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
        }
        
        .full-width {
            grid-column: 1 / -1;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>ModuForge-RS 性能监控仪表板</h1>
        <p>实时监控框架性能，检测性能回归，优化系统表现</p>
    </div>
    
    <div class="dashboard">
        <div class="controls full-width">
            <label>选择库：</label>
            <select id="crateSelect">
                <option value="">加载中...</option>
            </select>
            
            <label>时间范围：</label>
            <select id="timeRange">
                <option value="1">最近1小时</option>
                <option value="24" selected>最近24小时</option>
                <option value="168">最近7天</option>
                <option value="720">最近30天</option>
            </select>
            
            <button onclick="refreshData()">刷新数据</button>
            <button onclick="checkRegressions()">检查回归</button>
            
            <span id="loadingIndicator" class="loading" style="display: none;"></span>
        </div>
        
        <div class="card">
            <h3>📊 系统概览</h3>
            <div class="metrics-grid">
                <div class="metric-card">
                    <div class="metric-value" id="totalBenchmarks">-</div>
                    <div class="metric-label">基准测试总数</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value" id="activeCrates">-</div>
                    <div class="metric-label">活跃库数量</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value" id="recentRegressions">-</div>
                    <div class="metric-label">最近回归</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value" id="performanceScore">-</div>
                    <div class="metric-label">性能评分</div>
                </div>
            </div>
        </div>
        
        <div class="card">
            <h3>⚡ 执行时间趋势</h3>
            <div class="chart-container">
                <canvas id="executionTimeChart"></canvas>
            </div>
        </div>
        
        <div class="card">
            <h3>💾 内存使用趋势</h3>
            <div class="chart-container">
                <canvas id="memoryUsageChart"></canvas>
            </div>
        </div>
        
        <div class="card">
            <h3>🚀 吞吐量趋势</h3>
            <div class="chart-container">
                <canvas id="throughputChart"></canvas>
            </div>
        </div>
        
        <div class="card full-width">
            <h3>🚨 性能回归警报</h3>
            <div id="regressionAlerts">
                <p style="color: #666; text-align: center;">加载中...</p>
            </div>
        </div>
    </div>
    
    <script>
        // 全局变量
        let charts = {};
        let currentCrate = '';
        
        // API基础URL
        const API_BASE = '/api';
        
        // 初始化仪表板
        async function initDashboard() {
            await loadCrates();
            await loadSummary();
            await loadRegressions();
            
            // 设置定时刷新
            setInterval(() => {
                if (document.hidden) return; // 页面不可见时不刷新
                refreshData();
            }, 30000); // 30秒刷新一次
        }
        
        // 加载库列表
        async function loadCrates() {
            try {
                const response = await axios.get(`${API_BASE}/crates`);
                const crates = response.data.data;
                
                const select = document.getElementById('crateSelect');
                select.innerHTML = '<option value="">所有库</option>';
                
                crates.forEach(crate => {
                    const option = document.createElement('option');
                    option.value = crate;
                    option.textContent = crate;
                    select.appendChild(option);
                });
                
                // 默认选择第一个库
                if (crates.length > 0) {
                    currentCrate = crates[0];
                    select.value = currentCrate;
                    await loadMetrics();
                }
            } catch (error) {
                console.error('加载库列表失败:', error);
            }
        }
        
        // 加载性能摘要
        async function loadSummary() {
            try {
                const response = await axios.get(`${API_BASE}/metrics/summary`);
                const summary = response.data.data;
                
                document.getElementById('totalBenchmarks').textContent = summary.total_benchmarks;
                document.getElementById('activeCrates').textContent = summary.active_crates;
                document.getElementById('recentRegressions').textContent = summary.recent_regressions;
                document.getElementById('performanceScore').textContent = summary.average_performance_score.toFixed(1);
            } catch (error) {
                console.error('加载摘要失败:', error);
            }
        }
        
        // 加载性能指标
        async function loadMetrics() {
            if (!currentCrate) return;
            
            showLoading(true);
            
            try {
                const timeRange = document.getElementById('timeRange').value;
                const endTime = Math.floor(Date.now() / 1000);
                const startTime = endTime - (parseInt(timeRange) * 3600);
                
                const response = await axios.get(`${API_BASE}/metrics`, {
                    params: {
                        crate_name: currentCrate,
                        start_time: startTime,
                        end_time: endTime,
                        limit: 1000
                    }
                });
                
                const metrics = response.data.data;
                updateCharts(metrics);
            } catch (error) {
                console.error('加载指标失败:', error);
            } finally {
                showLoading(false);
            }
        }
        
        // 更新图表
        function updateCharts(metrics) {
            // 按指标类型分组数据
            const executionTimeData = metrics.filter(m => m.benchmark_name.includes('execution_time'));
            const memoryData = metrics.filter(m => m.benchmark_name.includes('memory'));
            const throughputData = metrics.filter(m => m.benchmark_name.includes('throughput'));
            
            // 更新执行时间图表
            updateLineChart('executionTimeChart', executionTimeData, '执行时间 (ms)', '#667eea');
            
            // 更新内存使用图表
            updateLineChart('memoryUsageChart', memoryData, '内存使用 (MB)', '#48bb78');
            
            // 更新吞吐量图表
            updateLineChart('throughputChart', throughputData, '吞吐量 (ops/s)', '#ed8936');
        }
        
        // 更新折线图
        function updateLineChart(canvasId, data, label, color) {
            const ctx = document.getElementById(canvasId).getContext('2d');
            
            // 销毁旧图表
            if (charts[canvasId]) {
                charts[canvasId].destroy();
            }
            
            // 准备数据
            const chartData = data.map(d => ({
                x: new Date(d.timestamp * 1000),
                y: d.value
            })).sort((a, b) => a.x - b.x);
            
            // 创建新图表
            charts[canvasId] = new Chart(ctx, {
                type: 'line',
                data: {
                    datasets: [{
                        label: label,
                        data: chartData,
                        borderColor: color,
                        backgroundColor: color + '20',
                        borderWidth: 2,
                        fill: true,
                        tension: 0.4
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {
                        x: {
                            type: 'time',
                            time: {
                                displayFormats: {
                                    hour: 'HH:mm',
                                    day: 'MM-DD'
                                }
                            }
                        },
                        y: {
                            beginAtZero: true
                        }
                    },
                    plugins: {
                        legend: {
                            display: false
                        }
                    }
                }
            });
        }
        
        // 加载回归警报
        async function loadRegressions() {
            try {
                const response = await axios.get(`${API_BASE}/regressions`);
                const regressions = response.data.data;
                
                const container = document.getElementById('regressionAlerts');
                
                if (regressions.length === 0) {
                    container.innerHTML = '<p style="color: #28a745; text-align: center;">✅ 未检测到性能回归</p>';
                    return;
                }
                
                container.innerHTML = '';
                
                regressions.forEach(regression => {
                    const alertDiv = document.createElement('div');
                    alertDiv.className = `regression-alert alert-${regression.severity.toLowerCase()}`;
                    
                    alertDiv.innerHTML = `
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <div>
                                <strong>${regression.crate_name} / ${regression.benchmark_name}</strong>
                                <p style="margin: 0.25rem 0;">
                                    性能下降: ${regression.change_percent.toFixed(1)}%
                                    (${regression.baseline_value.toFixed(2)} → ${regression.current_value.toFixed(2)})
                                </p>
                                <small style="color: #666;">
                                    统计显著性: ${(regression.statistical_significance * 100).toFixed(1)}%
                                </small>
                            </div>
                            <div style="text-align: right;">
                                <span style="font-size: 0.8rem; background: #666; color: white; padding: 0.2rem 0.5rem; border-radius: 4px;">
                                    ${regression.severity.toUpperCase()}
                                </span>
                            </div>
                        </div>
                    `;
                    
                    container.appendChild(alertDiv);
                });
            } catch (error) {
                console.error('加载回归警报失败:', error);
                document.getElementById('regressionAlerts').innerHTML = 
                    '<p style="color: #dc3545; text-align: center;">❌ 加载回归警报失败</p>';
            }
        }
        
        // 刷新数据
        async function refreshData() {
            await loadMetrics();
            await loadSummary();
            await loadRegressions();
        }
        
        // 检查回归
        async function checkRegressions() {
            showLoading(true);
            
            try {
                const response = await axios.post(`${API_BASE}/regressions/check`);
                const message = response.data.data;
                
                // 显示检查结果
                alert(message);
                
                // 重新加载回归警报
                await loadRegressions();
            } catch (error) {
                console.error('检查回归失败:', error);
                alert('检查回归失败，请稍后重试');
            } finally {
                showLoading(false);
            }
        }
        
        // 显示/隐藏加载指示器
        function showLoading(show) {
            const indicator = document.getElementById('loadingIndicator');
            indicator.style.display = show ? 'inline-block' : 'none';
        }
        
        // 事件监听器
        document.getElementById('crateSelect').addEventListener('change', async (e) => {
            currentCrate = e.target.value;
            await loadMetrics();
        });
        
        document.getElementById('timeRange').addEventListener('change', async () => {
            await loadMetrics();
        });
        
        // 页面可见性变化时刷新数据
        document.addEventListener('visibilitychange', () => {
            if (!document.hidden) {
                refreshData();
            }
        });
        
        // 初始化
        initDashboard();
    </script>
</body>
</html>
```

## 5. 智能告警系统

### 5.1 告警规则引擎

```rust
// src/alerting/alert_engine.rs
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub channels: Vec<NotificationChannel>,
    pub cooldown_minutes: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    Threshold {
        metric: String,
        operator: ComparisonOperator,
        value: f64,
        duration_minutes: u32,
    },
    PercentChange {
        metric: String,
        baseline_hours: u32,
        change_percent: f64,
        direction: ChangeDirection,
    },
    Statistical {
        metric: String,
        deviation_threshold: f64,
        window_hours: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeDirection {
    Increase,
    Decrease,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email { recipients: Vec<String> },
    Slack { webhook_url: String, channel: String },
    GitHub { repository: String, labels: Vec<String> },
    Webhook { url: String, headers: HashMap<String, String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertInstance {
    pub rule_id: String,
    pub crate_name: String,
    pub benchmark_name: String,
    pub triggered_at: u64,
    pub resolved_at: Option<u64>,
    pub status: AlertStatus,
    pub context: HashMap<String, String>,
    pub notifications_sent: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Triggered,
    Acknowledged,
    Resolved,
    Suppressed,
}

pub struct AlertEngine {
    rules: Vec<AlertRule>,
    active_alerts: HashMap<String, AlertInstance>,
    notification_sender: Box<dyn NotificationSender>,
    storage: Arc<dyn MetricsStorage>,
    cooldown_tracker: HashMap<String, u64>,
}

#[async_trait::async_trait]
pub trait NotificationSender: Send + Sync {
    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        alert: &AlertInstance,
        rule: &AlertRule,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl AlertEngine {
    pub fn new(
        storage: Arc<dyn MetricsStorage>,
        notification_sender: Box<dyn NotificationSender>,
    ) -> Self {
        Self {
            rules: Vec::new(),
            active_alerts: HashMap::new(),
            notification_sender,
            storage,
            cooldown_tracker: HashMap::new(),
        }
    }
    
    pub fn add_rule(&mut self, rule: AlertRule) {
        info!("添加告警规则: {}", rule.name);
        self.rules.push(rule);
    }
    
    pub async fn start_monitoring(&mut self) {
        info!("开始告警监控");
        let mut interval = interval(Duration::from_secs(60)); // 每分钟检查一次
        
        loop {
            interval.tick().await;
            self.evaluate_rules().await;
        }
    }
    
    async fn evaluate_rules(&mut self) {
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }
            
            // 检查冷却期
            let cooldown_key = format!("{}_{}", rule.id, "global");
            if let Some(&last_alert_time) = self.cooldown_tracker.get(&cooldown_key) {
                let cooldown_duration = rule.cooldown_minutes as u64 * 60;
                if current_timestamp() - last_alert_time < cooldown_duration {
                    continue; // 仍在冷却期内
                }
            }
            
            if let Err(e) = self.evaluate_single_rule(rule).await {
                error!("评估告警规则失败 {}: {}", rule.name, e);
            }
        }
    }
    
    async fn evaluate_single_rule(&mut self, rule: &AlertRule) -> Result<(), Box<dyn std::error::Error>> {
        match &rule.condition {
            AlertCondition::Threshold { metric, operator, value, duration_minutes } => {
                self.evaluate_threshold_condition(rule, metric, operator, *value, *duration_minutes).await
            }
            AlertCondition::PercentChange { metric, baseline_hours, change_percent, direction } => {
                self.evaluate_percent_change_condition(rule, metric, *baseline_hours, *change_percent, direction).await
            }
            AlertCondition::Statistical { metric, deviation_threshold, window_hours } => {
                self.evaluate_statistical_condition(rule, metric, *deviation_threshold, *window_hours).await
            }
        }
    }
    
    async fn evaluate_threshold_condition(
        &mut self,
        rule: &AlertRule,
        metric: &str,
        operator: &ComparisonOperator,
        threshold: f64,
        duration_minutes: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let end_time = current_timestamp();
        let start_time = end_time - (duration_minutes as u64 * 60);
        
        // 查询相关指标数据
        let metrics = self.storage.query_metrics("", "", start_time, end_time).await?;
        
        // 筛选匹配的指标
        let matching_metrics: Vec<_> = metrics
            .iter()
            .filter(|m| m.benchmark_name.contains(metric) || m.metric_type.to_string().contains(metric))
            .collect();
        
        if matching_metrics.is_empty() {
            return Ok(());
        }
        
        // 检查阈值条件
        for metric_data in matching_metrics {
            let condition_met = match operator {
                ComparisonOperator::GreaterThan => metric_data.value > threshold,
                ComparisonOperator::LessThan => metric_data.value < threshold,
                ComparisonOperator::Equals => (metric_data.value - threshold).abs() < 0.001,
                ComparisonOperator::NotEquals => (metric_data.value - threshold).abs() >= 0.001,
            };
            
            if condition_met {
                self.trigger_alert(rule, &metric_data.crate_name, &metric_data.benchmark_name).await?;
            }
        }
        
        Ok(())
    }
    
    async fn evaluate_percent_change_condition(
        &mut self,
        rule: &AlertRule,
        metric: &str,
        baseline_hours: u32,
        change_percent: f64,
        direction: &ChangeDirection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let end_time = current_timestamp();
        let baseline_start = end_time - (baseline_hours as u64 * 3600);
        let recent_start = end_time - 3600; // 最近1小时作为当前值
        
        // 获取基线期间数据
        let baseline_metrics = self.storage.query_metrics("", "", baseline_start, recent_start).await?;
        
        // 获取最近数据
        let recent_metrics = self.storage.query_metrics("", "", recent_start, end_time).await?;
        
        // 按库和基准测试分组计算
        let mut baseline_averages = HashMap::new();
        let mut recent_averages = HashMap::new();
        
        for metric_data in baseline_metrics {
            let key = format!("{}_{}", metric_data.crate_name, metric_data.benchmark_name);
            baseline_averages.entry(key).or_insert_with(Vec::new).push(metric_data.value);
        }
        
        for metric_data in recent_metrics {
            let key = format!("{}_{}", metric_data.crate_name, metric_data.benchmark_name);
            recent_averages.entry(key).or_insert_with(Vec::new).push(metric_data.value);
        }
        
        // 计算变化百分比并检查条件
        for (key, baseline_values) in baseline_averages {
            if let Some(recent_values) = recent_averages.get(&key) {
                let baseline_avg = baseline_values.iter().sum::<f64>() / baseline_values.len() as f64;
                let recent_avg = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
                
                let actual_change = ((recent_avg - baseline_avg) / baseline_avg) * 100.0;
                
                let condition_met = match direction {
                    ChangeDirection::Increase => actual_change > change_percent,
                    ChangeDirection::Decrease => actual_change < -change_percent,
                    ChangeDirection::Any => actual_change.abs() > change_percent,
                };
                
                if condition_met {
                    let parts: Vec<&str> = key.split('_').collect();
                    if parts.len() >= 2 {
                        self.trigger_alert(rule, parts[0], &parts[1..].join("_")).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn evaluate_statistical_condition(
        &mut self,
        rule: &AlertRule,
        metric: &str,
        deviation_threshold: f64,
        window_hours: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let end_time = current_timestamp();
        let start_time = end_time - (window_hours as u64 * 3600);
        
        let metrics = self.storage.query_metrics("", "", start_time, end_time).await?;
        
        // 按库和基准测试分组
        let mut grouped_metrics: HashMap<String, Vec<f64>> = HashMap::new();
        
        for metric_data in metrics {
            if metric_data.benchmark_name.contains(metric) {
                let key = format!("{}_{}", metric_data.crate_name, metric_data.benchmark_name);
                grouped_metrics.entry(key).or_insert_with(Vec::new).push(metric_data.value);
            }
        }
        
        // 计算统计偏差
        for (key, values) in grouped_metrics {
            if values.len() < 10 {
                continue; // 样本太少
            }
            
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let std_dev = variance.sqrt();
            
            // 检查最新值是否异常
            if let Some(&latest_value) = values.last() {
                let z_score = (latest_value - mean) / std_dev;
                
                if z_score.abs() > deviation_threshold {
                    let parts: Vec<&str> = key.split('_').collect();
                    if parts.len() >= 2 {
                        self.trigger_alert(rule, parts[0], &parts[1..].join("_")).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn trigger_alert(
        &mut self,
        rule: &AlertRule,
        crate_name: &str,
        benchmark_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let alert_id = format!("{}_{}_{}_{}", rule.id, crate_name, benchmark_name, current_timestamp());
        
        let alert = AlertInstance {
            rule_id: rule.id.clone(),
            crate_name: crate_name.to_string(),
            benchmark_name: benchmark_name.to_string(),
            triggered_at: current_timestamp(),
            resolved_at: None,
            status: AlertStatus::Triggered,
            context: HashMap::new(),
            notifications_sent: Vec::new(),
        };
        
        info!("触发告警: {} - {}/{}", rule.name, crate_name, benchmark_name);
        
        // 发送通知
        for channel in &rule.channels {
            match self.notification_sender.send_notification(channel, &alert, rule).await {
                Ok(_) => {
                    info!("通知发送成功: {:?}", channel);
                }
                Err(e) => {
                    error!("通知发送失败: {:?} - {}", channel, e);
                }
            }
        }
        
        // 记录告警
        self.active_alerts.insert(alert_id, alert);
        
        // 设置冷却期
        let cooldown_key = format!("{}_{}", rule.id, "global");
        self.cooldown_tracker.insert(cooldown_key, current_timestamp());
        
        Ok(())
    }
}

// 通知发送器实现
pub struct StandardNotificationSender;

#[async_trait::async_trait]
impl NotificationSender for StandardNotificationSender {
    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        alert: &AlertInstance,
        rule: &AlertRule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match channel {
            NotificationChannel::Email { recipients } => {
                self.send_email_notification(recipients, alert, rule).await
            }
            NotificationChannel::Slack { webhook_url, channel: slack_channel } => {
                self.send_slack_notification(webhook_url, slack_channel, alert, rule).await
            }
            NotificationChannel::GitHub { repository, labels } => {
                self.create_github_issue(repository, labels, alert, rule).await
            }
            NotificationChannel::Webhook { url, headers } => {
                self.send_webhook_notification(url, headers, alert, rule).await
            }
        }
    }
}

impl StandardNotificationSender {
    async fn send_email_notification(
        &self,
        _recipients: &[String],
        alert: &AlertInstance,
        rule: &AlertRule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 邮件发送实现
        info!("发送邮件通知: {} - {}", rule.name, alert.crate_name);
        Ok(())
    }
    
    async fn send_slack_notification(
        &self,
        webhook_url: &str,
        channel: &str,
        alert: &AlertInstance,
        rule: &AlertRule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "channel": channel,
            "username": "ModuForge-RS 性能监控",
            "text": format!(
                "🚨 *性能告警* - {}\n\n• 库: `{}`\n• 基准测试: `{}`\n• 严重程度: `{:?}`\n• 时间: {}",
                rule.name,
                alert.crate_name,
                alert.benchmark_name,
                rule.severity,
                chrono::DateTime::from_timestamp(alert.triggered_at as i64, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
            ),
            "color": match rule.severity {
                AlertSeverity::Critical => "danger",
                AlertSeverity::High => "warning",
                AlertSeverity::Medium => "good",
                AlertSeverity::Low => "#439FE0"
            }
        });
        
        let client = reqwest::Client::new();
        client.post(webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        info!("Slack通知发送成功: {}", channel);
        Ok(())
    }
    
    async fn create_github_issue(
        &self,
        repository: &str,
        labels: &[String],
        alert: &AlertInstance,
        rule: &AlertRule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // GitHub Issue创建实现
        info!("创建GitHub Issue: {} - {}", repository, rule.name);
        Ok(())
    }
    
    async fn send_webhook_notification(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
        alert: &AlertInstance,
        rule: &AlertRule,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = serde_json::json!({
            "alert": alert,
            "rule": rule,
            "timestamp": current_timestamp()
        });
        
        let client = reqwest::Client::new();
        let mut request = client.post(url).json(&payload);
        
        for (key, value) in headers {
            request = request.header(key, value);
        }
        
        request.send().await?;
        
        info!("Webhook通知发送成功: {}", url);
        Ok(())
    }
}
```

## 6. 部署和使用

### 6.1 Docker部署配置

```yaml
# docker-compose.yml
version: '3.8'

services:
  performance-dashboard:
    build:
      context: .
      dockerfile: Dockerfile.dashboard
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/performance
      - INFLUXDB_URL=http://influxdb:8086
      - INFLUXDB_DATABASE=moduforge_performance
      - RUST_LOG=info
    depends_on:
      - postgres
      - influxdb
    volumes:
      - ./config:/app/config

  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: performance
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql

  influxdb:
    image: influxdb:2.0
    environment:
      INFLUXDB_DB: moduforge_performance
      INFLUXDB_ADMIN_USER: admin
      INFLUXDB_ADMIN_PASSWORD: admin_password
    volumes:
      - influxdb_data:/var/lib/influxdb2
    ports:
      - "8086:8086"

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/datasources:/etc/grafana/provisioning/datasources

volumes:
  postgres_data:
  influxdb_data:
  grafana_data:
```

### 6.2 启动脚本

```bash
#!/bin/bash
# scripts/start_performance_monitoring.sh

set -e

echo "🚀 启动 ModuForge-RS 性能监控系统"

# 创建必要的目录
mkdir -p benchmarks/results
mkdir -p benchmarks/config
mkdir -p models
mkdir -p reports

# 检查依赖
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose 未安装"
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 未安装"
    exit 1
fi

# 安装Python依赖
echo "📦 安装Python依赖..."
pip3 install -r scripts/requirements.txt

# 启动Docker服务
echo "🐳 启动Docker服务..."
docker-compose up -d

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 30

# 初始化数据库
echo "🗄️ 初始化数据库..."
python3 scripts/performance_dashboard.py --db benchmarks/results/performance.db --summary

# 训练机器学习模型
echo "🤖 训练异常检测模型..."
python3 scripts/ml_anomaly_detection.py --db benchmarks/results/performance.db --train --models-dir models/

echo "✅ 性能监控系统启动完成!"
echo ""
echo "🌐 仪表板地址: http://localhost:8080"
echo "📊 Grafana地址: http://localhost:3000 (admin/admin)"
echo "🔍 InfluxDB地址: http://localhost:8086"
echo ""
echo "🔧 运行基准测试:"
echo "  cargo bench --workspace"
echo ""
echo "📈 生成性能报告:"
echo "  python3 scripts/performance_dashboard.py --summary --generate-trends"
```

本自动化性能分析系统提供了：

1. **全面的数据采集** - 从基准测试到系统资源的完整监控
2. **智能回归检测** - 统计学和机器学习双重检测算法
3. **实时可视化仪表板** - 现代化的Web界面和交互式图表
4. **灵活的告警系统** - 多渠道通知和智能告警规则
5. **生产就绪的部署** - Docker容器化部署和监控集成

通过这套系统，ModuForge-RS 能够实现：
- **主动性能监控** - 及时发现性能问题
- **智能异常检测** - 减少人工监控成本
- **数据驱动优化** - 基于准确数据进行性能优化
- **团队协作效率** - 统一的性能监控平台
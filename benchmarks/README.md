# ModuForge-RS 性能基准测试系统

这是 ModuForge-RS 框架的全面性能基准测试系统，提供了从基础设施搭建到智能分析的完整工作流。

## 🎯 系统特性

### 系统化基准测试
- **完整覆盖**: 14个核心库的全面基准测试
- **分层架构**: 基础层 → 核心层 → 服务层 → 集成层
- **依赖感知**: 智能执行顺序和并行优化
- **质量保证**: 统计严密的性能测量

### 智能性能监控
- **自动化采集**: 基准测试结果、系统资源、自定义指标
- **实时分析**: 统计学和机器学习双重检测算法
- **可视化仪表板**: 现代化Web界面和交互式图表
- **主动告警**: 多渠道通知和智能告警规则

### 生产就绪部署
- **容器化部署**: Docker Compose配置
- **CI/CD集成**: GitHub Actions工作流
- **数据持久化**: SQLite/InfluxDB时序数据库
- **监控集成**: 支持Grafana仪表板

## 🚀 快速开始

### 1. 基础环境设置

```bash
# 设置环境
./scripts/start_benchmarks.sh setup

# 或者Windows
scripts\start_benchmarks.bat setup
```

### 2. 运行基准测试

```bash
# 运行所有基准测试
./scripts/start_benchmarks.sh all

# 运行特定层级的基准测试
./scripts/start_benchmarks.sh foundation
./scripts/start_benchmarks.sh core-logic
./scripts/start_benchmarks.sh service
./scripts/start_benchmarks.sh integration

# 运行特定crate的基准测试
./scripts/start_benchmarks.sh crate moduforge-model
```

### 3. 生成和查看报告

```bash
# 生成性能报告
./scripts/start_benchmarks.sh report

# 检测性能回归
./scripts/start_benchmarks.sh detect current.json --baseline baseline.json
```

## 📊 核心库架构

### 分层结构

**基础层 (Foundation Tier)**
- `moduforge-model`: 核心数据结构
- `moduforge-macros-derive`: 过程宏  
- `moduforge-macros`: 代码生成

**核心逻辑层 (Core Logic Tier)**
- `moduforge-transform`: 数据转换
- `moduforge-rules-expression`: 表达式引擎
- `moduforge-rules-template`: 模板系统

**服务层 (Service Layer Tier)**
- `moduforge-state`: 状态管理
- `moduforge-rules-engine`: 规则引擎
- `moduforge-file`: 文件处理
- `moduforge-search`: 搜索功能
- `moduforge-persistence`: 数据持久化

**集成层 (Integration Tier)**
- `moduforge-core`: 框架核心
- `moduforge-collaboration`: 协作编辑
- `moduforge-collaboration-client`: 客户端协作

## 📈 性能目标

| 组件 | 指标 | 目标值 |
|------|------|--------|
| **事件分发** | 延迟 | <1ms p95 |
| **事务处理** | 吞吐量 | >1k TPS |
| **事务处理** | 延迟 | <10ms p95 |
| **协作同步** | 并发用户 | >1000 |
| **协作同步** | 同步延迟 | <50ms |
| **搜索查询** | 响应时间 | <100ms |
| **搜索索引** | 索引速度 | >1k docs/s |
| **文件操作** | 吞吐量 | >100 MB/s |

## 🛠️ 工具和脚本

### 主要脚本

1. **`scripts/start_benchmarks.sh`** (Linux/macOS) / **`scripts/start_benchmarks.bat`** (Windows)
   - 主要的基准测试启动脚本
   - 支持所有常用操作：运行、报告、回归检测

2. **`scripts/performance_metrics.py`**
   - 性能指标管理工具
   - 数据库操作、基线设置、回归检测

3. **`scripts/generate_comprehensive_report.py`**
   - 生成综合性能报告
   - HTML格式，包含图表和可视化

4. **`scripts/regression_detector.py`**
   - 简化的回归检测工具
   - 支持阈值配置和多种输出格式

### 工具二进制

- **`benchmark-coordinator`**
  - Rust编写的基准测试协调器
  - 依赖解析、执行调度、结果收集

## 🔧 高级用法

### 自定义基准测试阈值

```bash
# 设置10%的回归阈值
./scripts/start_benchmarks.sh detect current.json --baseline base.json --threshold 10.0
```

### 并行执行优化

```bash
# 使用4个并行进程
./scripts/start_benchmarks.sh all --parallel 4
```

### 自定义输出目录

```bash
# 指定输出目录
./scripts/start_benchmarks.sh all --output custom_results/
```

### 生成特定格式报告

```python
# 使用Python脚本直接控制
python3 scripts/performance_metrics.py report \
    --crate moduforge-model \
    --days 30 \
    --format chart \
    --output model_performance.png
```

## 📁 目录结构

```
benchmarks/
├── README.md              # 本文档
├── results/               # 基准测试结果
│   ├── *.json            # JSON格式结果文件
│   └── *.txt             # 原始Criterion输出
├── reports/              # 生成的报告
│   ├── *.html            # HTML格式报告
│   ├── *.png             # 图表文件
│   └── regression_*.txt  # 回归分析报告
├── baseline/             # 性能基线数据
│   ├── performance.db    # SQLite数据库
│   └── *.json           # 基线结果文件
└── config/               # 配置文件
    ├── thresholds.toml   # 回归检测阈值
    └── scenarios.toml    # 测试场景配置
```

## 🔍 性能分析功能

### 回归检测算法

**统计回归检测**
- t检验判断统计显著性
- 可配置的置信区间
- 多样本验证减少误报

**严重性分级**
- LOW: 10-15% 性能变化
- MEDIUM: 15-25% 性能变化
- HIGH: 25-50% 性能变化
- CRITICAL: >50% 性能变化

### 智能告警

**多渠道通知**
- 邮件通知
- Slack集成
- GitHub Issues
- 自定义Webhook

**告警条件**
- 阈值监控
- 趋势分析
- 异常检测
- 基线偏差

## 🚦 CI/CD 集成

### GitHub Actions 工作流

基准测试会在以下情况自动运行：

1. **Push到主分支**: 更新性能基线
2. **Pull Request**: 进行回归检测
3. **定时执行**: 每日性能监控

### 状态检查

- ✅ 通过：无性能回归
- ⚠️ 警告：轻微性能下降
- ❌ 失败：严重性能回归

## 🐛 故障排查

### 常见问题

**1. Cargo基准测试失败**
```bash
# 确保所有依赖都已安装
cargo build --workspace

# 检查特定crate的基准测试
cargo bench --package moduforge-model --verbose
```

**2. Python脚本执行错误**
```bash
# 安装必要依赖
pip3 install pandas matplotlib scipy numpy

# 检查Python版本
python3 --version  # 需要Python 3.7+
```

**3. 没有找到基准测试结果**
```bash
# 检查输出目录
ls -la benchmarks/results/

# 重新运行基准测试
./scripts/start_benchmarks.sh foundation
```

### 调试模式

```bash
# 启用详细输出
RUST_LOG=debug cargo bench --package moduforge-model

# 检查基准测试工具状态
cargo run --bin benchmark-coordinator -- --help
```

## 📚 文档和资源

- **完整工作流文档**: `claudedocs/performance_benchmarking_workflow.md`
- **实现指南**: `claudedocs/benchmark_implementation_guide.md`  
- **自动化分析**: `claudedocs/performance_analysis_automation.md`
- **系统概览**: `claudedocs/performance_benchmarking_overview.md`

## 🤝 贡献指南

### 添加新的基准测试

1. 在相应crate的`benches/`目录创建基准测试文件
2. 使用Criterion.rs框架编写测试
3. 更新`benchmark-coordinator`中的依赖关系
4. 运行测试确保正常工作

### 扩展分析功能

1. 在`scripts/`目录添加新的分析脚本
2. 确保与现有工具集成
3. 添加相应的文档和使用示例
4. 提交PR进行代码审查

---

**立即开始**: 运行 `./scripts/start_benchmarks.sh setup` 启动您的性能监控之旅！
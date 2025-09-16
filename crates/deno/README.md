# ModuForge Deno Integration

ModuForge 的 Deno 运行时集成库，允许使用 JavaScript/TypeScript 编写高性能插件，通过零序列化 Op 映射架构实现与 Rust 核心的高效数据交互。

## ✨ 特性

- 🚀 **零序列化架构**: 通过 Deno Op 系统直接访问 Rust 数据结构
- ⚡ **高性能运行时池**: 预分配的 Deno 运行时实例，支持并发执行
- 🔧 **完整的插件 API**: 提供状态、事务、节点操作的完整 JavaScript API
- 🛡️ **类型安全**: TypeScript 支持，完整的类型定义
- 🧪 **全面测试**: 单元测试和集成测试覆盖
- 📦 **易于使用**: 简洁的 API 设计和丰富的示例

## 🏗️ 架构概览

### 核心组件

```
┌─────────────────────────────────────────┐
│           JavaScript Plugin             │
│  ┌─────────────────────────────────────┐ │
│  │        ModuForge API               │ │
│  │  - State: 状态访问                 │ │
│  │  - Transaction: 事务操作           │ │
│  │  - Node: 节点操作                  │ │
│  └─────────────────────────────────────┘ │
└─────────────────┬───────────────────────┘
                  │ Deno Op 调用
┌─────────────────▼───────────────────────┐
│              Deno Core                  │
│  ┌─────────────────────────────────────┐ │
│  │          Op Functions              │ │
│  │  - op_state_*: 状态操作            │ │
│  │  - op_transaction_*: 事务操作      │ │
│  │  - op_node_*: 节点操作             │ │
│  └─────────────────────────────────────┘ │
└─────────────────┬───────────────────────┘
                  │ 直接内存访问
┌─────────────────▼───────────────────────┐
│         ModuForge Context               │
│  ┌─────────────────────────────────────┐ │
│  │          State & Data              │ │
│  │  - current_state: Arc<State>       │ │
│  │  - transactions: DashMap           │ │
│  │  - context_version: u64            │ │
│  └─────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### 零序列化数据流

```
JavaScript Call  →  Deno Op  →  Rust Function  →  Direct Memory Access
     |                |              |                    |
   API 调用        Op 路由       业务逻辑            数据读取/修改
     ↓                ↓              ↓                    ↓
   返回结果        返回值        操作结果            状态更新
```

## 🚀 快速开始

### 1. 添加依赖

```toml
[dependencies]
mf-deno = { path = "path/to/moduforge-deno" }
moduforge-state = { workspace = true }
tokio = { version = "1.0", features = ["full"] }
```

### 2. 基本使用

```rust
use std::sync::Arc;
use mf_deno::*;
use moduforge_state::{State, StateConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建 ModuForge 状态
    let state = Arc::new(State::create(StateConfig::default()).await?);

    // 创建 Deno 集成实例
    let deno = ModuForgeDeno::new(state, Some(4));
    deno.initialize().await?;

    // 加载插件
    let plugin = deno.load_plugin_from_file("my-plugin", "plugin.js").await?;

    // 执行插件方法
    let result = deno.manager()
        .execute_plugin_method("my-plugin", "processData", serde_json::json!({
            "input": "Hello ModuForge"
        }))
        .await?;

    println!("Result: {}", result);

    // 清理
    deno.shutdown().await;
    Ok(())
}
```

### 3. 编写插件

创建 `plugin.js`：

```javascript
// 实现插件核心方法
function appendTransaction(args) {
    console.log('Plugin appendTransaction:', args);

    if (args.transactionCount > 1) {
        const transactionId = ModuForge.Transaction.new();
        ModuForge.Transaction.setMeta(transactionId, 'batchSize', args.transactionCount);
        return { transactionId };
    }

    return null;
}

function filterTransaction(args) {
    console.log('Plugin filterTransaction:', args);
    return true; // 允许所有事务
}

// 自定义方法
function processData(args) {
    const { input } = args;

    // 访问 ModuForge 状态
    const stateVersion = ModuForge.State.getVersion();
    const docId = ModuForge.State.getDoc();

    return {
        output: input.toUpperCase(),
        stateVersion,
        docId,
        timestamp: Date.now()
    };
}

console.log('Plugin loaded successfully');
```

## 📚 API 参考

### JavaScript API

#### ModuForge.State
- `getVersion()`: 获取状态版本
- `hasField(name)`: 检查字段是否存在
- `getField(name)`: 获取字段数据
- `getDoc()`: 获取文档根节点 ID
- `getSchema()`: 获取 Schema 信息

#### ModuForge.Transaction
- `new()`: 创建新事务
- `setNodeAttribute(trId, nodeId, attrs)`: 设置节点属性
- `addNode(trId, parentId, nodes)`: 添加子节点
- `removeNode(trId, parentId, nodeIds)`: 删除节点
- `setMeta(trId, key, value)`: 设置事务元数据
- `getMeta(trId, key)`: 获取事务元数据

#### ModuForge.Node
- `getAttribute(nodeId, attrName)`: 获取节点属性
- `getChildren(nodeId)`: 获取子节点列表
- `getParent(nodeId)`: 获取父节点 ID
- `findById(nodeId)`: 检查节点是否存在
- `getInfo(nodeId)`: 获取节点详细信息

### Rust API

#### ModuForgeDeno
```rust
impl ModuForgeDeno {
    pub fn new(state: Arc<State>, pool_size: Option<usize>) -> Self;
    pub async fn initialize(&self) -> DenoResult<()>;
    pub async fn load_plugin_from_file(&self, id: &str, path: &Path) -> DenoResult<Arc<Plugin>>;
    pub async fn create_plugin_from_code(&self, id: &str, code: &str) -> DenoResult<Arc<Plugin>>;
    pub async fn build_plugin(&self, builder: DenoPluginBuilder) -> DenoResult<Arc<Plugin>>;
    pub async fn unload_plugin(&self, id: &str) -> DenoResult<()>;
    pub async fn list_plugins(&self) -> Vec<String>;
    pub async fn shutdown(self);
}
```

#### DenoPluginBuilder
```rust
impl DenoPluginBuilder {
    pub fn new(id: impl Into<String>) -> Self;
    pub fn code(self, code: impl Into<String>) -> Self;
    pub async fn code_from_file(self, path: impl AsRef<Path>) -> DenoResult<Self>;
    pub fn priority(self, priority: i32) -> Self;
    pub fn enabled(self, enabled: bool) -> Self;
    pub fn build(self) -> DenoResult<DenoPlugin>;
}
```

## 🧪 测试

运行测试套件：

```bash
# 单元测试
cargo test

# 集成测试
cargo test --test integration_tests

# 性能测试
cargo test --release test_runtime_pool_performance
```

运行示例：

```bash
# 基本使用示例
cargo run --example usage_example

# 高级特性示例
cargo run --example advanced_features
```

## 📈 性能特征

### 基准测试结果

在标准测试环境下（Intel i7-12700K, 32GB RAM）的性能表现：

- **运行时池初始化**: ~50ms (4个实例)
- **插件加载**: ~10-20ms per plugin
- **方法调用延迟**: ~0.1-0.5ms
- **并发性能**: 1000+ 并发调用/秒
- **内存使用**: ~8MB per runtime instance

### 优化建议

1. **运行时池大小**: 根据并发需求调整，推荐 CPU 核心数的 1-2 倍
2. **插件缓存**: 避免频繁加载/卸载插件
3. **批量操作**: 对于大量数据处理，使用批量 API
4. **状态更新**: 合理控制状态更新频率

## 🔧 高级特性

### 1. 自定义 Op 函数

```rust
use deno_core::op2;

#[op2]
pub fn op_custom_operation(
    state: &mut OpState,
    #[string] input: String,
) -> Result<String, String> {
    // 自定义操作逻辑
    Ok(format!("Processed: {}", input))
}
```

### 2. 插件状态管理

```javascript
class PluginState {
    constructor() {
        this.data = new Map();
        this.history = [];
    }

    set(key, value) {
        this.data.set(key, value);
        this.history.push({ key, value, timestamp: Date.now() });
    }

    get(key) {
        return this.data.get(key);
    }
}

const state = new PluginState();
```

### 3. 错误处理

```javascript
function safeOperation(args) {
    try {
        const result = riskyOperation(args);
        return { success: true, data: result };
    } catch (error) {
        return {
            success: false,
            error: error.message,
            stack: error.stack
        };
    }
}
```

## 🐛 故障排除

### 常见问题

1. **运行时池耗尽**: 增加池大小或优化插件执行时间
2. **内存泄漏**: 确保正确清理插件资源
3. **性能问题**: 检查插件代码效率和状态更新频率
4. **类型错误**: 验证 JavaScript 参数类型和返回值格式

### 调试技巧

```rust
// 启用详细日志
RUST_LOG=debug cargo test

// 性能分析
cargo test --release --features=profiling
```

```javascript
// 插件内调试
console.log('Debug info:', {
    args,
    stateVersion: ModuForge.State.getVersion(),
    timestamp: Date.now()
});
```

## 🤝 贡献指南

欢迎贡献！请查看 [CONTRIBUTING.md](../../CONTRIBUTING.md) 了解详细信息。

### 开发环境设置

```bash
# 克隆项目
git clone https://github.com/your-org/moduforge-rs.git
cd moduforge-rs/crates/deno-integration

# 安装依赖
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example usage_example
```

## 📄 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../../LICENSE) 文件。

## 🔗 相关资源

- [ModuForge 核心文档](../README.md)
- [Deno 官方文档](https://deno.land/manual)
- [Deno Core 文档](https://github.com/denoland/deno_core)
- [示例项目](./examples/)

---

**注意**: 这是一个实验性功能，API 可能在未来版本中发生变化。生产环境使用请谨慎评估。
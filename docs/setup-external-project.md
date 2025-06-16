# ModuForge-RS 外部项目快速设置指南

## 快速开始

### 1. 选择合适的规则文件

根据项目复杂度选择合适的规则文件：

- **完整版本** (`.cursorrules-for-external-projects`): 适用于大型项目，包含详细的架构指南和最佳实践
- **简化版本** (`.cursorrules-external-simple`): 适用于小型项目，包含核心集成要点

### 2. 复制规则文件

```bash
# 方式1：复制完整版本
cp path/to/moduforge-rs/.cursorrules-for-external-projects ./.cursorrules

# 方式2：复制简化版本  
cp path/to/moduforge-rs/.cursorrules-external-simple ./.cursorrules
```

### 3. 配置项目依赖

在你的 `Cargo.toml` 中添加：

```toml
[dependencies]
# 核心依赖
moduforge-core = "0.3.8"
moduforge-model = "0.3.8" 
moduforge-state = "0.3.8"
moduforge-transform = "0.3.8"

# 必需的支持库
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
im = { version = "15.1", features = ["serde"] }
anyhow = "1"
async-trait = "0.1"
```

### 4. 基础代码模板

创建基本的集成代码：

```rust
// main.rs 或 lib.rs
use moduforge_core::{RuntimeResult, runtime::Runtime, types::RuntimeOptionsBuilder};
use moduforge_state::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    init_logging("info", None)?;
    
    // 创建编辑器
    let options = RuntimeOptionsBuilder::new()
        .history_limit(100)
        .build();
    let runtime = Runtime::create(options).await?;
    
    println!("ModuForge-RS 集成成功!");
    Ok(())
}
```

## 常见使用场景

### 场景1：文本编辑器项目

```rust
use moduforge_core::{RuntimeResult, runtime::Runtime, types::RuntimeOptionsBuilder};
use moduforge_transform::attr_step::AttrStep;
use serde_json::json;
use std::sync::Arc;

pub struct TextRuntime {
    runtime: Runtime,
}

impl TextRuntime {
    pub async fn new() -> RuntimeResult<Self> {
        let options = RuntimeOptionsBuilder::new().build();
        let runtime = Runtime::create(options).await?;
        Ok(Self { runtime })
    }
    
    pub async fn insert_text(&mut self, node_id: &str, text: &str) -> RuntimeResult<()> {
        let mut transaction = self.runtime.get_tr();
        
        let mut attrs = im::HashMap::new();
        attrs.insert("content".to_string(), json!(text));
        
        transaction.step(Arc::new(AttrStep::new(
            node_id.to_string(),
            attrs
        )))?;
        
        self.runtime.dispatch(transaction).await
    }
}
```

### 场景2：文档管理系统

```rust
use moduforge_state::{State, StateConfig, plugin::PluginState};
use std::collections::HashMap;

pub struct DocumentManager {
    state: State,
}

impl DocumentManager {
    pub async fn new() -> anyhow::Result<Self> {
        let mut config = StateConfig::default();
        
        // 注册文档插件
        config.register_plugin("document", document_plugin_init);
        
        let state = State::new(config).await?;
        Ok(Self { state })
    }
}

async fn document_plugin_init(
    _config: &StateConfig,
    _state: Option<&State>
) -> PluginState {
    Arc::new(HashMap::new())
}
```

### 场景3：协作编辑平台

```rust
use moduforge_state::{State, StateConfig};
use moduforge_core::event::{Event, EventSystem};

pub struct CollaborativeRuntime {
    runtime: Runtime,
    event_system: EventSystem,
}

impl CollaborativeRuntime {
    pub async fn new() -> RuntimeResult<Self> {
        let config = StateConfig::default();
        let runtime = Runtime::new(config).await?;
        let event_system = EventSystem::new();
        
        Ok(Self { runtime, event_system })
    }
    
    pub async fn handle_remote_change(&mut self, change: RemoteChange) -> RuntimeResult<()> {
        // 处理远程协作变更
        let transaction = change.to_transaction();
        self.runtime.apply_transaction(transaction).await
    }
}
```

## 验证设置

创建一个简单的测试来验证集成：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use moduforge_state::{State, StateConfig};

    #[tokio::test]
    async fn test_integration() {
        let config = StateConfig::default();
        let state = State::new(config).await.unwrap();
        
        // 验证状态创建成功
        assert!(state.version() > 0);
        println!("✅ ModuForge-RS 集成测试通过");
    }
}
```

运行测试：
```bash
cargo test test_integration
```

## 常见问题解决

### 问题1：依赖版本冲突

```bash
# 清理依赖
cargo clean

# 更新依赖
cargo update

# 检查依赖树
cargo tree
```

### 问题2：异步运行时问题

确保在 `Cargo.toml` 中启用了完整的 tokio 功能：

```toml
tokio = { version = "1.0", features = ["full"] }
```

### 问题3：不可变数据结构使用

```rust
// ❌ 避免
use std::collections::HashMap;

// ✅ 推荐
use im::HashMap as ImHashMap;
```

## 进阶配置

### 自定义插件模板

```rust
use moduforge_state::{StateConfig, plugin::PluginState};
use std::sync::Arc;
use std::collections::HashMap;

pub async fn my_custom_plugin(
    config: &StateConfig,
    state: Option<&State>
) -> PluginState {
    let mut plugin_data = HashMap::new();
    
    // 插件初始化逻辑
    plugin_data.insert("initialized".to_string(), Box::new(true));
    
    Arc::new(plugin_data)
}

// 注册插件
let mut config = StateConfig::default();
config.register_plugin("my_plugin", my_custom_plugin);
```

### 中间件配置

```rust
use moduforge_core::middleware::Middleware;

pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    async fn process(&self, context: &mut Context) -> RuntimeResult<()> {
        println!("处理请求: {:?}", context);
        Ok(())
    }
}
```

## 项目结构建议

```
your-project/
├── .cursorrules          # 从 ModuForge-RS 复制的规则
├── Cargo.toml           # 项目配置
├── src/
│   ├── main.rs          # 应用入口
│   ├── lib.rs           # 库入口
│   ├── plugins/         # 自定义插件
│   ├── handlers/        # 业务处理器
│   └── config/          # 配置管理
├── tests/              # 集成测试
└── examples/           # 使用示例
```

## 下一步

1. 阅读 ModuForge-RS 文档了解更多 API
2. 查看示例项目学习最佳实践
3. 根据业务需求开发自定义插件
4. 集成到现有项目中

## 获取帮助

- 查看 ModuForge-RS 项目文档
- 参考 `example-integration-project.md` 示例
- 在项目中使用 Cursor AI 助手，它会根据规则文件提供准确的建议

通过以上步骤，你的项目将能够充分利用 ModuForge-RS 的强大功能，同时 Cursor AI 助手也能为你提供准确的代码建议和架构指导。 
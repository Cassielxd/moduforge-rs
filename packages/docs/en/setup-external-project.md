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
moduforge-core = "0.4.11"
moduforge-model = "0.4.11" 
moduforge-state = "0.4.11"
moduforge-transform = "0.4.11"
moduforge-rules-engine = "0.4.11"
moduforge-rules-expression = "0.4.11"
moduforge-collaboration = "0.4.11"
moduforge-template = "0.4.11"

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
use mf_core::{ForgeResult, async_runtime::AsyncRuntime, types::RuntimeOptions};
use mf_state::{init_logging, StateConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    init_logging("info", None)?;
    
    // 创建运行时
    let options = RuntimeOptions::default();
    let state_config = StateConfig::default();
    let runtime = AsyncRuntime::new(options, state_config).await?;
    
    println!("ModuForge-RS 集成成功!");
    Ok(())
}
```

## 常见使用场景

### 场景1：文本编辑器项目

```rust
use mf_core::{ForgeResult, async_runtime::AsyncRuntime, types::RuntimeOptions};
use mf_state::{StateConfig, transaction::Transaction};
use mf_transform::attr_step::AttrStep;
use serde_json::json;
use std::sync::Arc;

pub struct TextRuntime {
    runtime: AsyncRuntime,
}

impl TextRuntime {
    pub async fn new() -> ForgeResult<Self> {
        let options = RuntimeOptions::default();
        let state_config = StateConfig::default();
        let runtime = AsyncRuntime::new(options, state_config).await?;
        Ok(Self { runtime })
    }
    
    pub async fn insert_text(&mut self, node_id: &str, text: &str) -> ForgeResult<()> {
        let mut transaction = Transaction::new();
        
        let mut attrs = im::HashMap::new();
        attrs.insert("content".to_string(), json!(text));
        
        let step = AttrStep::new(node_id.to_string(), attrs);
        transaction.add_step(Box::new(step));
        
        self.runtime.apply_transaction(transaction).await
    }
}
```

### 场景2：文档管理系统

```rust
use mf_state::{State, StateConfig, resource::Resource};
use std::sync::Arc;

pub struct DocumentManager {
    state: State,
}

impl DocumentManager {
    pub async fn new() -> anyhow::Result<Self> {
        let config = StateConfig::default();
        let state = State::new(config).await?;
        Ok(Self { state })
    }
    
    pub async fn create_document(&mut self, title: &str) -> anyhow::Result<()> {
        let mut transaction = Transaction::new();
        transaction.set_meta("action", "create_document");
        transaction.set_meta("title", title);
        
        // 应用事务到状态
        self.state.apply(transaction).await?;
        Ok(())
    }
}
```

### 场景3：协作编辑平台

```rust
use mf_core::{async_runtime::AsyncRuntime, types::RuntimeOptions};
use mf_state::{StateConfig, transaction::Transaction};
use mf_collaboration::{sync_service::SyncService, ws_server::WebSocketServer};

pub struct CollaborativeRuntime {
    runtime: AsyncRuntime,
    sync_service: SyncService,
}

impl CollaborativeRuntime {
    pub async fn new() -> ForgeResult<Self> {
        let options = RuntimeOptions::default();
        let state_config = StateConfig::default();
        let runtime = AsyncRuntime::new(options, state_config).await?;
        let sync_service = SyncService::new();
        
        Ok(Self { runtime, sync_service })
    }
    
    pub async fn handle_remote_change(&mut self, change_data: Vec<u8>) -> ForgeResult<()> {
        // 处理远程协作变更
        let transaction = self.sync_service.decode_change(change_data)?;
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
    use mf_state::{State, StateConfig};

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

### 问题2：编译错误

确保所有必需的特性已启用：

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive", "rc"] }
im = { version = "15.1", features = ["serde"] }
```

### 问题3：运行时错误

检查日志配置和异步运行时设置：

```rust
// 确保正确初始化日志
init_logging("debug", Some("logs/debug.log"))?;

// 检查运行时配置
let mut options = RuntimeOptions::default();
options.set_debug_mode(true);
```

## 性能优化建议

### 1. 使用不可变数据结构
```rust
use im::{HashMap, Vector};

// 优先使用 im 集合
let mut data: HashMap<String, String> = HashMap::new();
data.insert("key".to_string(), "value".to_string());
```

### 2. 批量操作
```rust
// 批量处理事务
let mut transaction = Transaction::new();
for change in changes {
    transaction.add_step(Box::new(change));
}
runtime.apply_transaction(transaction).await?;
```

### 3. 异步并发
```rust
// 并发处理多个任务
let tasks: Vec<_> = documents.into_iter()
    .map(|doc| tokio::spawn(process_document(doc)))
    .collect();

for task in tasks {
    task.await??;
}
```

## 调试技巧

### 1. 启用详细日志
```rust
init_logging("trace", Some("logs/trace.log"))?;
```

### 2. 使用调试宏
```rust
#[cfg(debug_assertions)]
{
    println!("调试信息: {:?}", state);
}
```

### 3. 事务追踪
```rust
transaction.set_meta("debug_info", "user_action_123");
transaction.set_meta("timestamp", chrono::Utc::now().to_rfc3339());
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
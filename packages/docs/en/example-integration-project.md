# ModuForge-RS 集成示例项目

## 项目结构示例

```
my-editor-project/
├── .cursorrules                    # 从 ModuForge-RS 复制的规则文件
├── Cargo.toml                      # 项目配置
├── src/
│   ├── main.rs                     # 应用入口
│   ├── lib.rs                      # 库入口
│   ├── config/
│   │   ├── mod.rs
│   │   └── app_config.rs           # 应用配置
│   ├── plugins/
│   │   ├── mod.rs
│   │   ├── markdown_plugin.rs      # Markdown 支持插件
│   │   └── syntax_plugin.rs        # 语法高亮插件
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── document_handler.rs     # 文档处理
│   │   └── file_handler.rs         # 文件操作
│   └── extensions/
│       ├── mod.rs
│       └── custom_commands.rs      # 自定义命令
├── tests/
│   ├── integration_tests.rs
│   └── plugin_tests.rs
└── README.md
```

## Cargo.toml 示例

```toml
[package]
name = "my-editor-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# ModuForge-RS 核心组件
moduforge-core = "0.4.11"
moduforge-model = "0.4.11"
moduforge-state = "0.4.11"
moduforge-transform = "0.4.11"
moduforge-rules-engine = "0.4.11"
moduforge-rules-expression = "0.4.11"
moduforge-collaboration = "0.4.11"
moduforge-template = "0.4.11"

# 必需依赖
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"
im = { version = "15.1", features = ["serde"] }
anyhow = "1"
thiserror = "2.0.12"
async-trait = "0.1"
tracing = "0.1"
uuid = { version = "1.0", features = ["v4"] }

# 应用特定依赖
clap = { version = "4.0", features = ["derive"] }
```

## .cursorrules 文件

将以下内容复制到外部项目的 `.cursorrules` 文件中：

```markdown
# 复制 .cursorrules-for-external-projects 的内容
# 或者使用简化版本 .cursorrules-external-simple
```

## 主要代码示例

### src/main.rs
```rust
use anyhow::Result;
use my_editor_project::{AppConfig, EditorApp};
use mf_state::init_logging;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging("info", Some("logs/editor.log"))?;
    
    // 加载配置
    let config = AppConfig::load()?;
    
    // 启动编辑器应用
    let mut app = EditorApp::new(config).await?;
    app.run().await?;
    
    Ok(())
}
```

### src/lib.rs
```rust
//! My Editor Project - 基于 ModuForge-RS 的文本编辑器

pub mod config;
pub mod handlers;
pub mod plugins;
pub mod extensions;

pub use config::AppConfig;

use mf_core::{
    ForgeResult, 
    async_runtime::AsyncRuntime, 
    types::{RuntimeOptions, Extensions}
};
use mf_state::StateConfig;
use anyhow::Result;

/// 编辑器应用主结构
pub struct EditorApp {
    runtime: AsyncRuntime,
    config: AppConfig,
}

impl EditorApp {
    /// 创建新的编辑器应用实例
    pub async fn new(config: AppConfig) -> Result<Self> {
        // 创建扩展
        let markdown_ext = Extensions::E(extensions::create_markdown_extension());
        let syntax_ext = Extensions::E(extensions::create_syntax_extension());
        
        // 配置运行时选项
        let mut options = RuntimeOptions::default();
        options.add_extension(markdown_ext);
        options.add_extension(syntax_ext);
        
        let state_config = StateConfig::default();
        let runtime = AsyncRuntime::new(options, state_config).await?;
        
        Ok(Self { runtime, config })
    }
    
    /// 运行编辑器应用
    pub async fn run(&mut self) -> Result<()> {
        // 应用主循环逻辑
        println!("编辑器启动成功");
        Ok(())
    }
    
    /// 获取当前运行时
    pub fn runtime(&self) -> &AsyncRuntime {
        &self.runtime
    }
    
    /// 获取可变运行时引用
    pub fn runtime_mut(&mut self) -> &mut AsyncRuntime {
        &mut self.runtime
    }
}
```

### src/config/app_config.rs
```rust
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 编辑器主题
    pub theme: String,
    /// 是否启用语法高亮
    pub syntax_highlighting: bool,
    /// 自动保存间隔（秒）
    pub auto_save_interval: u64,
    /// 插件配置
    pub plugins: PluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// 启用的插件列表
    pub enabled: Vec<String>,
    /// 插件特定配置
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            syntax_highlighting: true,
            auto_save_interval: 30,
            plugins: PluginConfig {
                enabled: vec!["markdown".to_string(), "syntax".to_string()],
                settings: std::collections::HashMap::new(),
            },
        }
    }
}

impl AppConfig {
    /// 从文件加载配置
    pub fn load() -> Result<Self> {
        // 实际实现中可以从文件加载
        // 这里返回默认配置作为示例
        Ok(Self::default())
    }
    
    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        // 实际实现中保存到文件
        println!("配置已保存");
        Ok(())
    }
}
```

### src/plugins/markdown_plugin.rs
```rust
use mf_core::extension::Extension;
use mf_state::{
    plugin::{Plugin, PluginSpec, PluginTrait, StateField},
    resource::Resource,
    state::{State, StateConfig},
    transaction::Transaction,
    error::StateResult,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Markdown 插件状态
#[derive(Debug, Clone)]
pub struct MarkdownState {
    pub enabled: bool,
    pub syntax_rules: im::HashMap<String, String>,
}

impl Resource for MarkdownState {}

impl MarkdownState {
    pub fn new() -> Self {
        let mut syntax_rules = im::HashMap::new();
        syntax_rules.insert("header".to_string(), "^#{1,6}\\s".to_string());
        syntax_rules.insert("bold".to_string(), "\\*\\*.*\\*\\*".to_string());
        syntax_rules.insert("italic".to_string(), "\\*.*\\*".to_string());
        
        Self {
            enabled: true,
            syntax_rules,
        }
    }
}

/// Markdown 状态字段管理器
#[derive(Debug)]
pub struct MarkdownStateField;

#[async_trait]
impl StateField for MarkdownStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        Arc::new(MarkdownState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Ok(state) = value.clone().downcast::<MarkdownState>() {
            let mut new_state = (*state).clone();
            
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "toggle_markdown" => {
                        new_state.enabled = !new_state.enabled;
                    }
                    "add_syntax_rule" => {
                        if let Some(rule_name) = tr.get_meta::<String>("rule_name") {
                            if let Some(pattern) = tr.get_meta::<String>("pattern") {
                                new_state.syntax_rules.insert(
                                    rule_name.as_str().to_string(),
                                    pattern.as_str().to_string()
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            Arc::new(new_state)
        } else {
            value
        }
    }
}

/// Markdown 插件行为
#[derive(Debug)]
pub struct MarkdownPlugin;

#[async_trait]
impl PluginTrait for MarkdownPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(content_type) = tr.get_meta::<String>("content_type") {
                if content_type.as_str() == "markdown" {
                    // 自动启用 Markdown 处理
                    let mut markdown_tr = Transaction::new();
                    markdown_tr.set_meta("action", "process_markdown");
                    markdown_tr.set_meta("generated_by", "markdown_plugin");
                    return Ok(Some(markdown_tr));
                }
            }
        }
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        _transaction: &Transaction,
        _state: &State,
    ) -> bool {
        // 允许所有事务
        true
    }
}

/// 创建 Markdown 扩展
pub fn create_markdown_extension() -> Extension {
    let mut extension = Extension::new();
    
    let plugin = Plugin::new(PluginSpec {
        key: ("markdown_plugin".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(MarkdownStateField)),
        tr: Some(Arc::new(MarkdownPlugin)),
        priority: 20,
    });
    
    extension.add_plugin(Arc::new(plugin));
    extension
}
```

### src/handlers/document_handler.rs
```rust
use mf_core::{ForgeResult, async_runtime::AsyncRuntime};
use mf_state::transaction::Transaction;
use mf_transform::node_step::AddNodeStep;
use mf_model::{node::Node, attrs::Attrs, types::NodeId};
use serde_json::Value;

/// 文档处理器
pub struct DocumentHandler {
    runtime: AsyncRuntime,
}

impl DocumentHandler {
    pub fn new(runtime: AsyncRuntime) -> Self {
        Self { runtime }
    }
    
    /// 创建新文档
    pub async fn create_document(&mut self, title: &str) -> ForgeResult<String> {
        let mut transaction = Transaction::new();
        
        // 创建文档根节点
        let doc_id = format!("doc_{}", uuid::Uuid::new_v4());
        let mut attrs = Attrs::default();
        attrs.insert("title".to_string(), Value::String(title.to_string()));
        
        let doc_node = Node::new(
            &doc_id,
            "document".to_string(),
            attrs,
            vec![],
            vec![]
        );
        
        let add_step = AddNodeStep::new(doc_node, None);
        transaction.add_step(Box::new(add_step));
        
        // 设置事务元数据
        transaction.set_meta("action", "create_document");
        transaction.set_meta("document_id", &doc_id);
        transaction.set_meta("title", title);
        
        // 应用事务
        self.runtime.apply_transaction(transaction).await?;
        
        Ok(doc_id)
    }
    
    /// 添加段落到文档
    pub async fn add_paragraph(&mut self, doc_id: &str, content: &str) -> ForgeResult<String> {
        let mut transaction = Transaction::new();
        
        let para_id = format!("para_{}", uuid::Uuid::new_v4());
        let mut attrs = Attrs::default();
        attrs.insert("content".to_string(), Value::String(content.to_string()));
        
        let para_node = Node::new(
            &para_id,
            "paragraph".to_string(),
            attrs,
            vec![],
            vec![]
        );
        
        let add_step = AddNodeStep::new(para_node, Some(NodeId::from(doc_id)));
        transaction.add_step(Box::new(add_step));
        
        transaction.set_meta("action", "add_paragraph");
        transaction.set_meta("document_id", doc_id);
        transaction.set_meta("paragraph_id", &para_id);
        
        self.runtime.apply_transaction(transaction).await?;
        
        Ok(para_id)
    }
}
```

### src/extensions/custom_commands.rs
```rust
use mf_core::extension::Extension;
use mf_state::{
    plugin::{Plugin, PluginSpec, PluginTrait},
    transaction::Transaction,
    state::State,
    error::StateResult,
};
use async_trait::async_trait;
use std::sync::Arc;

/// 自定义命令插件
#[derive(Debug)]
pub struct CustomCommandsPlugin;

#[async_trait]
impl PluginTrait for CustomCommandsPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(command) = tr.get_meta::<String>("command") {
                match command.as_str() {
                    "format_document" => {
                        let mut format_tr = Transaction::new();
                        format_tr.set_meta("action", "apply_formatting");
                        format_tr.set_meta("generated_by", "custom_commands");
                        return Ok(Some(format_tr));
                    }
                    "auto_save" => {
                        let mut save_tr = Transaction::new();
                        save_tr.set_meta("action", "save_document");
                        save_tr.set_meta("generated_by", "custom_commands");
                        save_tr.set_meta("auto_save", "true");
                        return Ok(Some(save_tr));
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        transaction: &Transaction,
        _state: &State,
    ) -> bool {
        // 检查是否有禁止的命令
        if let Some(command) = transaction.get_meta::<String>("command") {
            return command.as_str() != "dangerous_command";
        }
        true
    }
}

/// 创建自定义命令扩展
pub fn create_custom_commands_extension() -> Extension {
    let mut extension = Extension::new();
    
    let plugin = Plugin::new(PluginSpec {
        key: ("custom_commands".to_string(), "v1".to_string()),
        state_field: None,
        tr: Some(Arc::new(CustomCommandsPlugin)),
        priority: 15,
    });
    
    extension.add_plugin(Arc::new(plugin));
    extension
}
```

## 测试示例

### tests/integration_tests.rs
```rust
use my_editor_project::{AppConfig, EditorApp};
use mf_state::init_logging;

#[tokio::test]
async fn test_editor_initialization() {
    init_logging("debug", None).unwrap();
    
    let config = AppConfig::default();
    let app = EditorApp::new(config).await.unwrap();
    
    // 验证运行时已正确初始化
    assert!(app.runtime().is_healthy());
}

#[tokio::test]
async fn test_document_creation() {
    let config = AppConfig::default();
    let mut app = EditorApp::new(config).await.unwrap();
    
    // 创建文档处理器
    let runtime = app.runtime().clone();
    let mut doc_handler = my_editor_project::handlers::DocumentHandler::new(runtime);
    
    // 创建文档
    let doc_id = doc_handler.create_document("测试文档").await.unwrap();
    assert!(!doc_id.is_empty());
    
    // 添加段落
    let para_id = doc_handler.add_paragraph(&doc_id, "这是一个测试段落").await.unwrap();
    assert!(!para_id.is_empty());
}
```

### tests/plugin_tests.rs
```rust
use my_editor_project::plugins::create_markdown_extension;
use mf_core::extension::Extension;

#[test]
fn test_markdown_plugin_creation() {
    let extension = create_markdown_extension();
    
    // 验证扩展已正确创建
    assert_eq!(extension.plugins().len(), 1);
    
    let plugin = &extension.plugins()[0];
    assert_eq!(plugin.spec().key.0, "markdown_plugin");
}

#[tokio::test]
async fn test_plugin_state_management() {
    use mf_state::{State, StateConfig};
    use my_editor_project::plugins::MarkdownStateField;
    use mf_state::plugin::StateField;
    
    let state_field = MarkdownStateField;
    let config = StateConfig::default();
    
    let resource = state_field.init(&config, None).await;
    
    // 验证插件状态已正确初始化
    assert!(resource.downcast_ref::<my_editor_project::plugins::MarkdownState>().is_some());
}
```

## 运行项目

```bash
# 构建项目
cargo build

# 运行项目
cargo run

# 运行测试
cargo test

# 运行特定测试
cargo test test_editor_initialization

# 启用详细日志运行
RUST_LOG=debug cargo run
```

## 部署配置

### Docker 配置
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-editor-project /usr/local/bin/

CMD ["my-editor-project"]
```

### 配置文件示例
```yaml
# config.yaml
theme: "dark"
syntax_highlighting: true
auto_save_interval: 30

plugins:
  enabled:
    - "markdown"
    - "syntax"
    - "custom_commands"
  settings:
    markdown:
      enable_tables: true
      enable_strikethrough: true
    syntax:
      theme: "monokai"
```
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
moduforge-core = { version = "0.3.8", path = "../moduforge-rs/core" }
moduforge-model = { version = "0.3.8", path = "../moduforge-rs/model" }
moduforge-state = { version = "0.3.8", path = "../moduforge-rs/state" }
moduforge-transform = { version = "0.3.8", path = "../moduforge-rs/transform" }

# 必需依赖
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"
im = { version = "15.1", features = ["serde"] }
anyhow = "1"
thiserror = "2.0.12"
async-trait = "0.1"
tracing = "0.1"
uuid = "1"

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
use moduforge_state::init_logging;

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

use moduforge_core::{
    EditorResult, 
    runtime::Editor, 
    types::{EditorOptionsBuilder, Extensions}
};
use anyhow::Result;

/// 编辑器应用主结构
pub struct EditorApp {
    editor: Editor,
    config: AppConfig,
}

impl EditorApp {
    /// 创建新的编辑器应用实例
    pub async fn new(config: AppConfig) -> Result<Self> {
        // 创建扩展
        let markdown_ext = Extensions::E(extensions::create_markdown_extension());
        let syntax_ext = Extensions::E(extensions::create_syntax_extension());
        
        // 配置编辑器选项
        let options = EditorOptionsBuilder::new()
            .add_extension(markdown_ext)
            .add_extension(syntax_ext)
            .history_limit(100)
            .build();
        
        let editor = Editor::create(options).await?;
        
        Ok(Self { editor, config })
    }
    
    /// 运行编辑器应用
    pub async fn run(&mut self) -> Result<()> {
        // 应用主循环逻辑
        println!("编辑器启动成功");
        Ok(())
    }
    
    /// 获取当前编辑器
    pub fn editor(&self) -> &Editor {
        &self.editor
    }
    
    /// 获取可变编辑器引用
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
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
        Ok(Self::default())
    }
    
    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        // 实际实现中保存到文件
        Ok(())
    }
}
```

### src/plugins/markdown_plugin.rs
```rust
use moduforge_state::{State, StateConfig, plugin::PluginState};
use moduforge_core::EditorResult;
use std::sync::Arc;
use std::collections::HashMap;
use im::HashMap as ImHashMap;

/// Markdown 插件状态
#[derive(Debug)]
pub struct MarkdownPluginState {
    /// 解析器配置  
    pub parser_config: ImHashMap<String, String>,
    /// 渲染选项
    pub render_options: ImHashMap<String, bool>,
}

impl Default for MarkdownPluginState {
    fn default() -> Self {
        let mut parser_config = ImHashMap::new();
        parser_config.insert("mode".to_string(), "gfm".to_string());
        
        let mut render_options = ImHashMap::new();
        render_options.insert("tables".to_string(), true);
        render_options.insert("strikethrough".to_string(), true);
        
        Self {
            parser_config,
            render_options,
        }
    }
}

/// Markdown 插件初始化函数
pub async fn markdown_plugin_init(
    _config: &StateConfig,
    _state: Option<&State>
) -> PluginState {
    let plugin_state = MarkdownPluginState::default();
    let mut state_map = HashMap::new();
    state_map.insert("markdown_state".to_string(), Box::new(plugin_state) as Box<dyn std::any::Any + Send + Sync>);
    Arc::new(state_map)
}

/// Markdown 相关操作
pub struct MarkdownOperations;

impl MarkdownOperations {
    /// 解析 Markdown 文本
    pub fn parse_markdown(text: &str) -> EditorResult<String> {
        // 实际的 Markdown 解析逻辑
        Ok(format!("Parsed: {}", text))
    }
    
    /// 渲染为 HTML
    pub fn render_to_html(markdown: &str) -> EditorResult<String> {
        // 实际的 HTML 渲染逻辑
        Ok(format!("<p>{}</p>", markdown))
    }
}
```

### src/handlers/document_handler.rs
```rust
use moduforge_core::{EditorResult, runtime::Editor};
use moduforge_transform::{
    node_step::AddNodeStep,
    attr_step::AttrStep
};
use moduforge_model::{
    node::Node, 
    node_type::{NodeType, NodeEnum}, 
    attrs::Attrs
};
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

/// 文档处理器
pub struct DocumentHandler;

impl DocumentHandler {
    /// 创建新文档
    pub async fn create_document(editor: &mut Editor, title: &str) -> EditorResult<()> {
        let mut transaction = editor.get_tr();
        
        // 创建文档根节点
        let doc_attrs = Attrs::new().set("title", json!(title));
        let doc_node = Node::new(
            "doc_root",
            "document",
            doc_attrs,
            vec![],
            vec![]
        );
        let node_enum = NodeEnum(doc_node, vec![]);
        
        // 添加节点步骤
        transaction.step(Arc::new(AddNodeStep::new(
            "root".to_string(),
            vec![node_enum]
        )))?;
        
        // 应用事务
        editor.dispatch(transaction).await?;
        
        Ok(())
    }
    
    /// 插入文本内容
    pub async fn insert_text(
        editor: &mut Editor, 
        node_id: &str,
        text: &str
    ) -> EditorResult<()> {
        let mut transaction = editor.get_tr();
        
        // 设置文本内容属性
        let mut attrs = im::HashMap::new();
        attrs.insert("content".to_string(), json!(text));
        
        transaction.step(Arc::new(AttrStep::new(
            node_id.to_string(),
            attrs
        )))?;
        
        editor.dispatch(transaction).await?;
        Ok(())
    }
    
    /// 获取文档内容
    pub fn get_document_content(editor: &Editor) -> Result<String> {
        // 从编辑器状态中提取文档内容
        let doc = editor.doc();
        let root_node = doc.get_root();
        
        // 递归提取文本内容
        fn extract_text(node: &Node) -> String {
            node.attrs.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        
        Ok(extract_text(&root_node))
    }
}
```

## 测试示例

### tests/integration_tests.rs
```rust
use my_editor_project::{AppConfig, EditorApp};
use moduforge_state::StateConfig;
use tokio_test;

#[tokio::test]
async fn test_editor_initialization() {
    let config = AppConfig::default();
    let app = EditorApp::new(config).await.unwrap();
    
    // 验证状态初始化
    let state = app.state();
    assert!(state.version() > 0);
}

#[tokio::test]
async fn test_plugin_loading() {
    let config = AppConfig::default();
    let app = EditorApp::new(config).await.unwrap();
    
    // 验证插件加载
    let state = app.state();
    // 检查插件是否正确加载
}

#[tokio::test]
async fn test_document_operations() {
    let config = AppConfig::default();
    let mut app = EditorApp::new(config).await.unwrap();
    
    // 测试文档操作
    let state = app.state();
    // 执行文档操作测试
}
```

## 使用指南

1. **复制规则文件**: 将 `.cursorrules-for-external-projects` 重命名为 `.cursorrules` 放在项目根目录

2. **配置依赖**: 根据项目需求调整 `Cargo.toml` 中的依赖版本和路径

3. **自定义插件**: 在 `src/plugins/` 目录下创建自己的插件实现

4. **扩展功能**: 在 `src/extensions/` 目录下添加自定义扩展

5. **配置管理**: 根据应用需求调整 `AppConfig` 结构

6. **测试覆盖**: 为所有关键功能编写集成测试

这样配置后，Cursor AI 助手将能够：
- 理解 ModuForge-RS 的架构模式
- 提供符合框架约定的代码建议  
- 正确使用不可变数据结构和事件驱动模式
- 遵循插件和中间件架构
- 提供适当的错误处理和异步编程建议 
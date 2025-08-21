# ModuForge-RS 快速入门指南

本指南将帮助您快速上手 ModuForge-RS 框架，从安装配置到构建第一个应用。

## 🚀 安装和配置

### 系统要求

- **Rust**: 1.70+ (推荐使用最新稳定版)
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 20.04+)
- **内存**: 至少 4GB RAM
- **磁盘空间**: 至少 2GB 可用空间

### 创建新项目

```bash
# 创建新的 Rust 项目
cargo new my-moduforge-app
cd my-moduforge-app

# 编辑 Cargo.toml 添加依赖
```

### Cargo.toml 配置

```toml
[package]
name = "my-moduforge-app"
version = "0.1.0"
edition = "2021"

[dependencies]
# 核心框架
mf-core = "0.4.12"
mf-model = "0.4.12"
mf-state = "0.4.12"
mf-transform = "0.4.12"

# 可选组件
mf-engine = "0.4.12"           # 规则引擎
mf-expression = "0.4.12"       # 表达式语言
mf-collaboration = "0.4.12"    # 协作功能
mf-file = "0.4.12"             # 文件处理
mf-search = "0.4.12"           # 搜索功能
mf-persistence = "0.4.12"      # 持久化

# 必需的异步运行时和工具
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## 📝 第一个 ModuForge 应用

### 1. 基础文档编辑器

创建一个简单的文档编辑器：

```rust
// src/main.rs
use std::sync::Arc;
use anyhow::Result;
use mf_core::{
    runtime::async_runtime::ForgeAsyncRuntime,
    types::{RuntimeOptions, EditorOptionsBuilder, Content, NodePoolFnTrait}
};
use mf_model::{node_pool::NodePool, Node, NodeType, Attrs};
use mf_transform::node_step::AddNodeStep;

// 定义节点池创建函数
struct DefaultNodePoolFn;

impl NodePoolFnTrait for DefaultNodePoolFn {
    fn call(&self) -> NodePool {
        NodePool::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建编辑器配置
    let create_callback: Arc<dyn NodePoolFnTrait> = Arc::new(DefaultNodePoolFn);
    let mut builder = EditorOptionsBuilder::new();
    let options = builder
        .content(Content::NodePoolFn(create_callback))
        .history_limit(20)
        .build();
    
    // 创建运行时
    let runtime = ForgeAsyncRuntime::create(options).await?;
    
    println!("✅ ModuForge 运行时启动成功！");
    
    // 创建文档节点
    let doc_node = Node::new(
        "doc".to_string(),
        NodeType::block("document"),
        Attrs::new(),
        None,
    );
    
    // 创建段落节点
    let paragraph = Node::new(
        "para_1".to_string(),
        NodeType::block("paragraph"),
        Attrs::new(),
        Some("欢迎使用 ModuForge-RS 框架！".to_string()),
    );
    
    // 创建事务添加节点
    let mut transaction = runtime.get_state().tr();
    transaction.add_step(Box::new(AddNodeStep::new_single(doc_node, None)));
    transaction.add_step(Box::new(AddNodeStep::new_single(paragraph, Some("doc".to_string()))));
    
    // 执行事务
    runtime.dispatch_flow(transaction).await?;
    
    // 获取当前状态
    let state = runtime.get_state();
    println!("📄 文档创建完成，节点数量: {}", state.doc().size());
    
    Ok(())
}
```

### 2. 运行项目

```bash
cargo run
```

期望输出：
```
✅ ModuForge 运行时启动成功！
📄 文档创建完成，节点数量: 2
```

## 🔧 添加自定义节点类型

### 定义业务节点

```rust
// src/nodes.rs
use lazy_static::lazy_static;
use mf_model::{Node, NodeType, Attrs, AttrValue};

lazy_static! {
    // 文章节点
    pub static ref ARTICLE: Node = Node::new(
        "article".to_string(),
        NodeType::block("article"),
        Attrs::from([
            ("title".to_string(), AttrValue::string("".to_string())),
            ("author".to_string(), AttrValue::string("".to_string())),
            ("created_at".to_string(), AttrValue::string("".to_string())),
        ]),
        None,
    );
    
    // 标题节点
    pub static ref HEADING: Node = Node::new(
        "heading".to_string(),
        NodeType::block("heading"),
        Attrs::from([
            ("level".to_string(), AttrValue::number(1.0)),
        ]),
        None,
    );
    
    // 代码块节点
    pub static ref CODE_BLOCK: Node = Node::new(
        "code_block".to_string(),
        NodeType::block("code_block"),
        Attrs::from([
            ("language".to_string(), AttrValue::string("rust".to_string())),
            ("line_numbers".to_string(), AttrValue::boolean(true)),
        ]),
        None,
    );
}

// 创建节点的便捷函数
pub fn create_article(title: &str, author: &str) -> Node {
    let mut node = ARTICLE.clone();
    node.attrs.set("title".to_string(), AttrValue::string(title.to_string()));
    node.attrs.set("author".to_string(), AttrValue::string(author.to_string()));
    node.attrs.set("created_at".to_string(), AttrValue::string(
        chrono::Utc::now().to_rfc3339()
    ));
    node.id = format!("article_{}", uuid::Uuid::new_v4());
    node
}

pub fn create_heading(level: u8, text: &str) -> Node {
    let mut node = HEADING.clone();
    node.attrs.set("level".to_string(), AttrValue::number(level as f64));
    node.content = Some(text.to_string());
    node.id = format!("heading_{}", uuid::Uuid::new_v4());
    node
}

pub fn create_code_block(language: &str, code: &str) -> Node {
    let mut node = CODE_BLOCK.clone();
    node.attrs.set("language".to_string(), AttrValue::string(language.to_string()));
    node.content = Some(code.to_string());
    node.id = format!("code_{}", uuid::Uuid::new_v4());
    node
}
```

### 使用自定义节点

```rust
// src/main.rs (更新后的版本)
mod nodes;

use nodes::*;

#[tokio::main]
async fn main() -> Result<()> {
    // ... 前面的初始化代码 ...
    
    // 创建文章结构
    let article = create_article("ModuForge-RS 入门指南", "技术团队");
    let title = create_heading(1, "ModuForge-RS 快速入门");
    let intro = Node::new(
        "intro".to_string(),
        NodeType::block("paragraph"),
        Attrs::new(),
        Some("这是一个强大的 Rust 框架...".to_string()),
    );
    let code_example = create_code_block("rust", r#"
fn main() {
    println!("Hello, ModuForge!");
}
"#);
    
    // 构建文档结构
    let mut transaction = runtime.get_state().tr();
    transaction.add_step(Box::new(AddNodeStep::new_single(article.clone(), None)));
    transaction.add_step(Box::new(AddNodeStep::new_single(title, Some(article.id.clone()))));
    transaction.add_step(Box::new(AddNodeStep::new_single(intro, Some(article.id.clone()))));
    transaction.add_step(Box::new(AddNodeStep::new_single(code_example, Some(article.id.clone()))));
    
    runtime.dispatch_flow(transaction).await?;
    
    println!("📚 复杂文档结构创建完成！");
    
    Ok(())
}
```

## 🔌 插件系统入门

### 创建第一个插件

```rust
// src/plugins/word_count.rs
use std::sync::Arc;
use async_trait::async_trait;
use mf_state::{
    plugin::{PluginTrait, StateField, PluginMetadata, PluginConfig, PluginSpec, Plugin},
    resource::Resource,
    State, StateConfig, Transaction,
    error::StateResult,
};
use serde::{Serialize, Deserialize};

// 插件状态数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordCountData {
    pub total_words: usize,
    pub total_characters: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Resource for WordCountData {}

// 状态字段管理器
#[derive(Debug)]
pub struct WordCountStateField;

#[async_trait]
impl StateField for WordCountStateField {
    async fn init(&self, _config: &StateConfig, _instance: &State) -> Arc<dyn Resource> {
        Arc::new(WordCountData {
            total_words: 0,
            total_characters: 0,
            last_updated: chrono::Utc::now(),
        })
    }
    
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut data = value.downcast_ref::<WordCountData>()
            .expect("状态类型错误")
            .clone();
        
        // 计算文档的字数和字符数
        let (words, chars) = self.count_document_stats(new_state);
        data.total_words = words;
        data.total_characters = chars;
        data.last_updated = chrono::Utc::now();
        
        Arc::new(data)
    }
}

impl WordCountStateField {
    fn count_document_stats(&self, state: &State) -> (usize, usize) {
        let mut word_count = 0;
        let mut char_count = 0;
        
        // 遍历节点池中的所有节点计算统计
        let all_nodes = state.doc().filter_nodes(|_| true);
        for node in all_nodes {
            if let Some(content) = &node.content {
                word_count += content.split_whitespace().count();
                char_count += content.len();
            }
        }
        
        (word_count, char_count)
    }
}

// 插件业务逻辑
#[derive(Debug)]
pub struct WordCountPlugin;

#[async_trait]
impl PluginTrait for WordCountPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "word_count".to_string(),
            version: "1.0.0".to_string(),
            description: "文档字数统计插件".to_string(),
            author: "ModuForge Team".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec!["word_count_data".to_string()],
            tags: vec!["statistics".to_string(), "utility".to_string()],
        }
    }
    
    fn config(&self) -> PluginConfig {
        PluginConfig {
            enabled: true,
            priority: 10,
            settings: std::collections::HashMap::new(),
        }
    }
    
    async fn append_transaction(
        &self,
        _transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 获取统计数据
        if let Some(word_data) = new_state.get_field("word_count_data")
            .and_then(|state| state.downcast_ref::<WordCountData>()) {
            
            println!("📊 文档统计: {} 词, {} 字符", 
                word_data.total_words, 
                word_data.total_characters
            );
        }
        
        Ok(None)
    }
}

// 创建插件实例
pub fn create_word_count_plugin() -> Arc<Plugin> {
    let spec = PluginSpec {
        state_field: Some(Arc::new(WordCountStateField)),
        tr: Arc::new(WordCountPlugin),
    };
    Arc::new(Plugin::new(spec))
}
```

### 注册和使用插件

```rust
// src/main.rs (集成插件)
mod plugins;

use mf_core::extension::Extension;
use mf_core::types::Extensions;
use plugins::word_count::create_word_count_plugin;

async fn setup_extensions() -> Vec<Extensions> {
    let mut extensions = Vec::new();
    
    // 创建扩展容器
    let mut extension = Extension::new();
    
    // 添加自定义节点
    extension.add_node(nodes::ARTICLE.clone());
    extension.add_node(nodes::HEADING.clone());
    extension.add_node(nodes::CODE_BLOCK.clone());
    
    // 添加插件
    extension.add_plugin(create_word_count_plugin());
    
    extensions.push(Extensions::E(extension));
    extensions
}

#[tokio::main]
async fn main() -> Result<()> {
    // ... 初始化代码 ...
    
    // 创建带插件的配置
    let options = builder
        .content(Content::NodePoolFn(create_callback))
        .extensions(setup_extensions().await)
        .history_limit(20)
        .build();
    
    let runtime = ForgeAsyncRuntime::create(options).await?;
    
    // ... 创建文档的代码 ...
    
    Ok(())
}
```

## 🎯 表达式和规则引擎

### 使用表达式语言

```rust
// src/expressions.rs
use mf_expression::{Expression, Variable};
use serde_json::json;
use anyhow::Result;

pub async fn expression_examples() -> Result<()> {
    // 1. 基础数学计算
    let expr = Expression::compile("price * quantity * (1 + tax_rate)")?;
    let data = Variable::from(json!({
        "price": 100.0,
        "quantity": 5,
        "tax_rate": 0.1
    }));
    let result = expr.execute(&data)?;
    println!("💰 总价: {}", result.to_f64().unwrap());
    
    // 2. 条件判断
    let expr = Expression::compile(r#"
        if age >= 18 then 
            "成年人" 
        else 
            "未成年人"
    "#)?;
    let data = Variable::from(json!({"age": 25}));
    let result = expr.execute(&data)?;
    println!("👤 年龄判断: {}", result.to_string());
    
    // 3. 数组操作
    let expr = Expression::compile("scores.sum() / scores.length()")?;
    let data = Variable::from(json!({
        "scores": [85, 92, 78, 96, 88]
    }));
    let result = expr.execute(&data)?;
    println!("📈 平均分: {:.2}", result.to_f64().unwrap());
    
    Ok(())
}
```

### 规则引擎示例

```rust
// src/rules.rs
use mf_engine::{Engine, loader::MemoryLoader};
use mf_expression::Variable;
use serde_json::json;
use anyhow::Result;

pub async fn rules_examples() -> Result<()> {
    // 创建内存加载器
    let mut loader = MemoryLoader::new();
    
    // 添加业务规则
    loader.add_rule("discount_rule".to_string(), json!({
        "kind": "DecisionTable",
        "hitPolicy": "F",
        "inputs": [
            {
                "id": "customer_level",
                "name": "客户等级",
                "type": "string"
            },
            {
                "id": "order_amount", 
                "name": "订单金额",
                "type": "number"
            }
        ],
        "outputs": [
            {
                "id": "discount_rate",
                "name": "折扣率",
                "type": "number"
            }
        ],
        "rules": [
            {
                "id": "rule1",
                "inputs": ["VIP", ">= 1000"],
                "outputs": [0.2]
            },
            {
                "id": "rule2", 
                "inputs": ["VIP", "< 1000"],
                "outputs": [0.1]
            },
            {
                "id": "rule3",
                "inputs": ["Gold", ">= 500"],
                "outputs": [0.15]
            },
            {
                "id": "rule4",
                "inputs": ["Gold", "< 500"], 
                "outputs": [0.05]
            },
            {
                "id": "rule5",
                "inputs": ["*", "*"],
                "outputs": [0.0]
            }
        ]
    }).to_string());
    
    // 创建规则引擎
    let engine = Engine::new(loader);
    
    // 测试规则
    let test_cases = vec![
        ("VIP客户大额订单", json!({"customer_level": "VIP", "order_amount": 1500})),
        ("VIP客户小额订单", json!({"customer_level": "VIP", "order_amount": 800})),
        ("金牌客户中额订单", json!({"customer_level": "Gold", "order_amount": 600})),
        ("普通客户", json!({"customer_level": "Normal", "order_amount": 300})),
    ];
    
    for (desc, input) in test_cases {
        let input_var = Variable::from(input);
        let result = engine.evaluate("discount_rule", &input_var).await?;
        let discount = result.get("discount_rate").unwrap().to_f64().unwrap();
        println!("🎯 {}: 折扣率 {:.1}%", desc, discount * 100.0);
    }
    
    Ok(())
}
```

## 🤝 协作功能

### 基础协作设置

```rust
// src/collaboration.rs
use mf_collaboration::{SyncService, types::RoomConfig};
use mf_collaboration_client::CollaborationClient;
use anyhow::Result;

pub async fn setup_collaboration() -> Result<()> {
    // 1. 创建协作服务
    let mut sync_service = SyncService::new();
    
    // 2. 创建协作房间
    let room_config = RoomConfig {
        room_id: "doc_editor_room".to_string(),
        max_clients: 10,
    };
    sync_service.create_room(room_config).await?;
    
    println!("🤝 协作房间创建成功");
    
    // 3. 创建协作客户端
    let client = CollaborationClient::new("ws://localhost:8080").await?;
    client.join_room("doc_editor_room").await?;
    
    println!("👥 客户端加入房间成功");
    
    Ok(())
}
```

## 💾 数据持久化

### 设置持久化

```rust
// src/persistence.rs
use mf_persistence::{SqlitePersistence, RecoveryManager};
use anyhow::Result;

pub async fn setup_persistence() -> Result<()> {
    // 1. 创建 SQLite 持久化
    let persistence = SqlitePersistence::new("./data/app.db").await?;
    
    // 2. 创建恢复管理器
    let recovery_manager = RecoveryManager::new(Box::new(persistence));
    
    // 3. 保存状态快照
    // recovery_manager.save_snapshot(&state).await?;
    
    println!("💾 持久化设置完成");
    
    Ok(())
}
```

## 🔍 搜索功能

### 添加全文搜索

```rust
// src/search.rs
use mf_search::{SearchService, model::{IndexRequest, SearchRequest}};
use anyhow::Result;

pub async fn setup_search() -> Result<()> {
    // 1. 创建搜索服务
    let mut search_service = SearchService::new();
    
    // 2. 索引文档内容
    let index_req = IndexRequest {
        document_id: "doc_1".to_string(),
        content: "ModuForge-RS 是一个强大的 Rust 框架".to_string(),
        metadata: Default::default(),
    };
    search_service.index_document(index_req).await?;
    
    // 3. 执行搜索
    let search_req = SearchRequest {
        query: "Rust 框架".to_string(),
        limit: 10,
        offset: 0,
    };
    let results = search_service.search(search_req).await?;
    
    println!("🔍 搜索结果: {} 条", results.len());
    
    Ok(())
}
```

## 📂 完整项目结构

完成上述步骤后，您的项目结构应该如下：

```
my-moduforge-app/
├── Cargo.toml
├── src/
│   ├── main.rs              # 主入口
│   ├── nodes.rs             # 自定义节点定义
│   ├── plugins/             # 插件模块
│   │   ├── mod.rs
│   │   └── word_count.rs    # 字数统计插件
│   ├── expressions.rs       # 表达式示例
│   ├── rules.rs             # 规则引擎示例
│   ├── collaboration.rs     # 协作功能
│   ├── persistence.rs       # 持久化功能
│   └── search.rs            # 搜索功能
├── data/                    # 数据存储目录
└── README.md
```

## 🎯 下一步

恭喜！您已经成功创建了第一个 ModuForge-RS 应用。接下来可以：

1. **深入学习**: 阅读 [API 参考文档](./api-reference.md) 了解更多功能
2. **架构理解**: 查看 [架构概览](./architecture-overview.md) 理解框架设计
3. **插件开发**: 参考 [插件开发指南](./plugin-development-guide.md) 开发更复杂的插件
4. **实际项目**: 查看 [集成示例](./example-integration-project.md) 了解实际应用场景
5. **性能优化**: 学习框架的性能优化技巧和最佳实践

## 💡 常见问题

### Q: 如何调试 ModuForge 应用？
A: 使用 `tracing` 日志系统，设置合适的日志级别：
```rust
tracing_subscriber::fmt()
    .with_env_filter("debug")
    .init();
```

### Q: 如何处理错误？
A: 使用 `anyhow` 进行错误处理，并为操作添加上下文：
```rust
operation().context("操作失败的具体描述")?;
```

### Q: 插件间如何通信？
A: 通过事务的 meta 字段进行轻量级通信，或使用资源管理器共享数据。

### Q: 如何优化性能？
A: 
- 使用批量操作减少事务开销
- 合理设置历史记录限制
- 使用插件缓存减少重复计算

---

祝您使用 ModuForge-RS 开发愉快！🎉
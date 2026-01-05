# moduforge-search 文档

`moduforge-search` 提供 ModuForge-RS 的全文搜索和索引服务，基于 SQLite FTS5 实现高性能搜索功能。

## 概述

Search 层与事务系统深度集成，自动追踪文档变更并更新索引。使用 SQLite 作为存储后端，通过 FTS5（Full-Text Search 5）提供全文搜索能力，支持复杂查询、中文分词、增量索引和实时更新。

## 核心功能

- **全文搜索**：基于 SQLite FTS5 的高性能全文检索
- **增量索引**：与事务系统集成，自动索引文档变更
- **复杂查询**：支持多字段组合查询、范围查询、树形查询
- **实时更新**：异步处理索引更新，不阻塞事务执行
- **灵活存储**：支持持久化索引和临时索引
- **Mark 查询**：支持按标记类型和属性精确查询
- **属性过滤**：支持嵌套 JSON 属性的复杂查询
- **排序分页**：支持多字段排序和高效分页

## 架构设计

### 索引流程

```
事务执行 → Step 变更 → IndexEvent → IndexService → SQLiteBackend → FTS5 索引
    ↓                                     ↓                              ↓
State Plugin                         异步处理                      SQLite 存储
```

### 查询流程

```
SearchQuery → SearchService → SQLiteBackend → SQL 生成 → FTS5 查询 → 结果返回
                                    ↓                         ↓
                                属性过滤                  全文匹配
```

## SQLite 后端

### 数据库架构

```sql
-- 启用 WAL 模式和性能优化
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA cache_size=-64000;  -- 64MB 缓存
PRAGMA temp_store=MEMORY;

-- 节点表（存储文档结构）
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,           -- 节点 ID
    node_type TEXT NOT NULL,       -- 节点类型
    parent_id TEXT,                -- 父节点 ID
    path TEXT NOT NULL,            -- 路径（/分隔）
    marks TEXT,                    -- 标记类型列表
    marks_json TEXT,               -- 完整标记 JSON
    attrs TEXT,                    -- 扁平化属性
    attrs_json TEXT,               -- 完整属性 JSON
    text TEXT,                     -- 文本内容
    order_i64 INTEGER,             -- 排序字段
    created_at_i64 INTEGER,        -- 创建时间
    updated_at_i64 INTEGER         -- 更新时间
);

-- 索引优化查询性能
CREATE INDEX idx_node_type ON nodes(node_type);
CREATE INDEX idx_parent_id ON nodes(parent_id);
CREATE INDEX idx_path ON nodes(path);
CREATE INDEX idx_created_at ON nodes(created_at_i64);
CREATE INDEX idx_updated_at ON nodes(updated_at_i64);
CREATE INDEX idx_order ON nodes(order_i64);

-- FTS5 虚拟表（全文索引）
CREATE VIRTUAL TABLE nodes_fts USING fts5(
    id UNINDEXED,      -- 不索引 ID
    text,              -- 索引文本内容
    content='nodes',   -- 关联 nodes 表
    content_rowid='rowid'
);
```

### 自动触发器

```sql
-- 插入时自动更新 FTS
CREATE TRIGGER nodes_ai AFTER INSERT ON nodes BEGIN
    INSERT INTO nodes_fts(rowid, id, text)
    VALUES (new.rowid, new.id, new.text);
END;

-- 删除时自动更新 FTS
CREATE TRIGGER nodes_ad AFTER DELETE ON nodes BEGIN
    INSERT INTO nodes_fts(nodes_fts, rowid, id, text)
    VALUES('delete', old.rowid, old.id, old.text);
END;

-- 更新时自动更新 FTS
CREATE TRIGGER nodes_au AFTER UPDATE ON nodes BEGIN
    INSERT INTO nodes_fts(nodes_fts, rowid, id, text)
    VALUES('delete', old.rowid, old.id, old.text);
    INSERT INTO nodes_fts(rowid, id, text)
    VALUES (new.rowid, new.id, new.text);
END;
```

## 核心 API

### SqliteBackend - 存储后端

```rust
use mf_search::{SqliteBackend, IndexMutation, SearchQuery};
use std::sync::Arc;

// 创建持久化索引
let backend = Arc::new(SqliteBackend::new_in_dir("./index").await?);

// 创建临时索引（用于测试）
let temp_backend = Arc::new(SqliteBackend::new_in_system_temp().await?);

// 在指定临时目录创建索引
let custom_temp = Arc::new(
    SqliteBackend::new_in_temp_root("/tmp/custom").await?
);
```

### IndexMutation - 索引变更

```rust
use mf_search::{IndexMutation, model::IndexDoc};

pub enum IndexMutation {
    /// 添加文档
    Add(IndexDoc),

    /// 更新或插入文档
    Upsert(IndexDoc),

    /// 删除单个文档
    DeleteById(String),

    /// 批量删除文档
    DeleteManyById(Vec<String>),
}

// 应用索引变更
let mutations = vec![
    IndexMutation::Upsert(doc1),
    IndexMutation::DeleteById("old_node".to_string()),
];
backend.apply(mutations).await?;
```

### SearchQuery - 查询条件

```rust
use mf_search::SearchQuery;

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// 全文搜索（使用 FTS5）
    pub text: Option<String>,

    /// 节点类型精确匹配
    pub node_type: Option<String>,

    /// 父节点精确匹配
    pub parent_id: Option<String>,

    /// 路径前缀匹配
    pub path_prefix: Option<String>,

    /// 标记类型过滤
    pub marks: Vec<String>,

    /// 标记属性精确查询 (type, key, value)
    pub mark_attrs: Vec<(String, String, String)>,

    /// 属性匹配 (key, value)
    pub attrs: Vec<(String, String)>,

    /// 返回条数限制
    pub limit: usize,

    /// 偏移量（分页）
    pub offset: usize,

    /// 排序字段
    pub sort_by: Option<String>,

    /// 排序方向（true=升序）
    pub sort_asc: bool,

    /// 包含子树所有节点
    pub include_descendants: bool,

    /// 范围查询字段
    pub range_field: Option<String>,
    pub range_min: Option<i64>,
    pub range_max: Option<i64>,
}
```

## 服务层

### IndexService - 索引服务

```rust
use mf_search::{IndexService, IndexEvent, RebuildScope};
use std::sync::Arc;

// 创建索引服务
let index_service = IndexService::new(backend.clone());

// 处理索引事件
index_service.handle(IndexEvent::TransactionCommitted {
    pool_before: Some(old_pool),
    pool_after: new_pool,
    steps: transaction_steps,
}).await?;

// 全量重建索引
index_service.handle(IndexEvent::Rebuild {
    pool: node_pool,
    scope: RebuildScope::Full,
}).await?;
```

### SearchService - 搜索服务

```rust
use mf_search::{SearchService, SearchQuery};

// 创建搜索服务
let search_service = SearchService::new(backend.clone());

// 全文搜索（返回 ID 列表）
let ids = search_service.search_text("关键词", 10).await?;

// 全文搜索（返回完整文档）
let docs = search_service.search_text_docs("关键词", 10).await?;

// 复杂查询
let results = search_service.search(SearchQuery {
    text: Some("Rust".to_string()),
    node_type: Some("paragraph".to_string()),
    attrs: vec![("lang".to_string(), "zh".to_string())],
    marks: vec!["bold".to_string()],
    limit: 20,
    sort_by: Some("created_at_i64".to_string()),
    sort_asc: false,
    ..Default::default()
}).await?;

// 查询子树
let descendants = search_service.query_descendants("parent_id", 100).await?;
```

## State 插件集成

### 创建搜索插件

```rust
use mf_search::{create_search_index_plugin, create_temp_search_index_plugin};
use mf_state::{State, StateConfig};

// 创建持久化搜索插件
let plugin = create_search_index_plugin("./index").await?;

// 创建临时搜索插件（测试用）
let temp_plugin = create_temp_search_index_plugin().await?;

// 注册到状态
let state_config = StateConfig {
    plugins: Some(vec![plugin]),
    ..Default::default()
};

let state = State::create(state_config).await?;
```

### 插件工作原理

```rust
/// 搜索索引插件实现
struct SearchIndexStateField {
    service: Arc<IndexService>,
}

#[async_trait]
impl StateFieldGeneric<NodePool, Schema> for SearchIndexStateField {
    /// 初始化：全量重建索引
    async fn init(&self, _config: &StateConfig, instance: &State) -> Arc<Resource> {
        // 异步重建索引（不阻塞初始化）
        tokio::spawn(async move {
            self.service.handle(IndexEvent::Rebuild {
                pool: instance.node_pool.clone(),
                scope: RebuildScope::Full,
            }).await;
        });
        // ...
    }

    /// 事务应用：增量更新索引
    async fn apply(&self, tr: &Transaction, value: Arc<Resource>,
                   old_state: &State, new_state: &State) -> Arc<Resource> {
        // 异步处理索引更新（不阻塞事务）
        tokio::spawn(async move {
            self.service.handle(IndexEvent::TransactionCommitted {
                pool_before: Some(old_state.doc()),
                pool_after: new_state.doc(),
                steps: tr.steps.clone(),
            }).await;
        });
        value
    }
}
```

## IndexDoc 数据模型

### 文档结构

```rust
use mf_search::model::IndexDoc;

pub struct IndexDoc {
    /// 节点 ID
    pub node_id: String,

    /// 节点类型
    pub node_type: String,

    /// 父节点 ID
    pub parent_id: Option<String>,

    /// 标记类型列表（简单查询用）
    pub marks: Vec<String>,

    /// 完整标记 JSON（精确查询用）
    pub marks_json: String,

    /// 扁平化属性（简单查询用）
    pub attrs_flat: Vec<(String, String)>,

    /// 完整属性 JSON（复杂查询用）
    pub attrs_json: String,

    /// 文本内容
    pub text: Option<String>,

    /// 节点路径
    pub path: Vec<String>,

    /// 快速字段（用于排序和范围查询）
    pub order_i64: Option<i64>,
    pub created_at_i64: Option<i64>,
    pub updated_at_i64: Option<i64>,
}

// 从节点创建索引文档
let doc = IndexDoc::from_node(&node_pool, &node);

// 转换回节点（不包含子节点）
let node = doc.to_node()?;
```

## 使用示例

### 基础示例

```rust
use mf_search::{SqliteBackend, IndexService, SearchService, SearchQuery};
use mf_search::model::IndexDoc;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 创建临时索引
    let backend = Arc::new(SqliteBackend::new_in_system_temp().await?);
    let index_svc = IndexService::new(backend.clone());
    let search_svc = SearchService::new(backend.clone());

    // 2. 准备文档
    let docs = vec![
        IndexDoc {
            node_id: "n1".into(),
            node_type: "paragraph".into(),
            parent_id: Some("root".into()),
            marks: vec!["bold".into()],
            marks_json: r#"[{"type":"bold","attrs":{}}]"#.into(),
            attrs_flat: vec![("lang".into(), "zh".into())],
            attrs_json: r#"{"lang":"zh"}"#.into(),
            text: Some("Rust 搜索引擎示例".into()),
            path: vec!["root".into(), "n1".into()],
            order_i64: Some(1),
            created_at_i64: Some(1000),
            updated_at_i64: Some(1500),
        },
        IndexDoc {
            node_id: "n2".into(),
            node_type: "paragraph".into(),
            parent_id: Some("root".into()),
            marks: vec![],
            marks_json: "[]".into(),
            attrs_flat: vec![("lang".into(), "en".into())],
            attrs_json: r#"{"lang":"en"}"#.into(),
            text: Some("SQLite backend quick demo".into()),
            path: vec!["root".into(), "n2".into()],
            order_i64: Some(2),
            created_at_i64: Some(2000),
            updated_at_i64: Some(2500),
        },
    ];

    // 3. 全量重建索引
    backend.rebuild_all(docs).await?;

    // 4. 执行查询
    let ids = search_svc.search(SearchQuery {
        text: Some("示例".into()),
        node_type: Some("paragraph".into()),
        limit: 10,
        ..Default::default()
    }).await?;
    println!("搜索结果: {:?}", ids);

    // 5. 按属性过滤
    let english_docs = backend.search_ids(SearchQuery {
        attrs: vec![("lang".into(), "en".into())],
        limit: 10,
        ..Default::default()
    }).await?;
    println!("英文文档: {:?}", english_docs);

    // 6. 排序和分页
    let page1 = backend.search_ids(SearchQuery {
        sort_by: Some("created_at_i64".into()),
        sort_asc: false,
        limit: 1,
        offset: 0,
        ..Default::default()
    }).await?;

    let page2 = backend.search_ids(SearchQuery {
        sort_by: Some("created_at_i64".into()),
        sort_asc: false,
        limit: 1,
        offset: 1,
        ..Default::default()
    }).await?;

    Ok(())
}
```

### 高级查询示例

```rust
use mf_search::{SearchQuery, SqliteBackend};

// 1. 复合条件查询
let results = backend.search_ids(SearchQuery {
    // 全文包含 "Rust"
    text: Some("Rust".into()),
    // 类型为 heading 或 paragraph
    node_type: Some("heading".into()),
    // 包含 bold 标记
    marks: vec!["bold".into()],
    // 语言属性为中文
    attrs: vec![("lang".into(), "zh".into())],
    // 创建时间在指定范围
    range_field: Some("created_at_i64".into()),
    range_min: Some(1000),
    range_max: Some(5000),
    // 按更新时间降序
    sort_by: Some("updated_at_i64".into()),
    sort_asc: false,
    // 返回前 20 条
    limit: 20,
    ..Default::default()
}).await?;

// 2. 树形查询（包含所有子节点）
let tree_nodes = backend.search_ids(SearchQuery {
    parent_id: Some("chapter1".into()),
    include_descendants: true,  // 递归查询子树
    limit: 1000,
    ..Default::default()
}).await?;

// 3. 路径前缀查询
let section_nodes = backend.search_ids(SearchQuery {
    path_prefix: Some("/root/book/chapter1/".into()),
    limit: 100,
    ..Default::default()
}).await?;

// 4. 精确 Mark 属性查询
let highlighted = backend.search_ids(SearchQuery {
    mark_attrs: vec![
        ("highlight".into(), "color".into(), "yellow".into()),
        ("bold".into(), "weight".into(), "700".into()),
    ],
    limit: 50,
    ..Default::default()
}).await?;
```

### Price-RS 集成示例

```rust
use mf_search::{create_search_index_plugin, SearchService};
use price_rs::{PriceNode, PriceRuntime};

/// 配置 Price-RS 搜索功能
pub async fn setup_search(runtime: &mut PriceRuntime) -> Result<()> {
    // 1. 创建搜索插件
    let search_plugin = create_search_index_plugin("./price_index").await?;

    // 2. 注册到运行时
    runtime.register_plugin(search_plugin)?;

    // 搜索插件会自动：
    // - 在初始化时重建索引
    // - 在事务提交时更新索引
    // - 异步处理，不影响主流程性能

    Ok(())
}

/// 搜索价格节点
pub async fn search_prices(
    runtime: &PriceRuntime,
    keyword: &str,
    min_price: Option<f64>,
    max_price: Option<f64>
) -> Result<Vec<PriceNode>> {
    // 获取搜索服务
    let search_resource = runtime.state
        .get_field("search_index")
        .expect("Search plugin not registered");

    let search_service = &search_resource.service;

    // 构建查询
    let mut query = SearchQuery {
        text: Some(keyword.to_string()),
        node_type: Some("price_node".to_string()),
        limit: 100,
        ..Default::default()
    };

    // 价格范围过滤
    if let (Some(min), Some(max)) = (min_price, max_price) {
        query.range_field = Some("price_value".to_string());
        query.range_min = Some((min * 100.0) as i64);  // 转换为分
        query.range_max = Some((max * 100.0) as i64);
    }

    // 执行搜索
    let node_ids = search_service.search(query).await?;

    // 从节点池获取完整节点
    let mut nodes = Vec::new();
    for id in node_ids {
        if let Some(node) = runtime.state.doc().get_node(&id) {
            nodes.push(PriceNode::from_node(node)?);
        }
    }

    Ok(nodes)
}
```

## 性能优化

### 1. SQLite 优化配置

```sql
-- WAL 模式：提高并发性能
PRAGMA journal_mode = WAL;

-- 异步同步：平衡性能和安全
PRAGMA synchronous = NORMAL;

-- 大缓存：减少磁盘 IO
PRAGMA cache_size = -64000;  -- 64MB

-- 内存临时表：加速临时操作
PRAGMA temp_store = MEMORY;

-- 自动检查点：控制 WAL 大小
PRAGMA wal_autocheckpoint = 1000;
```

### 2. 索引策略

```rust
// 批量操作减少事务开销
let mutations = vec![/* 多个变更 */];
backend.apply(mutations).await?;  // 单事务处理

// 异步索引更新不阻塞主流程
tokio::spawn(async move {
    index_service.handle(event).await;
});
```

### 3. 查询优化

```rust
// 使用合适的索引字段
query.sort_by = Some("created_at_i64".into());  // 有索引

// 限制返回数量
query.limit = 100;  // 避免返回过多数据

// 使用分页而非一次获取全部
query.offset = page * page_size;
```

### 4. FTS5 优化

```sql
-- 使用 FTS5 的高级功能
SELECT * FROM nodes_fts WHERE nodes_fts MATCH 'rust AND search';  -- 布尔查询
SELECT * FROM nodes_fts WHERE nodes_fts MATCH '"exact phrase"';   -- 短语查询
SELECT * FROM nodes_fts WHERE nodes_fts MATCH 'NEAR(word1 word2)'; -- 邻近查询
```

## 监控与调试

### 查询分析

```rust
// 启用 tracing 功能
#[cfg(feature = "dev-tracing")]
tracing::info!("执行查询: {:?}", query);

// 查看 SQL 执行计划
let explain = backend.pool.exec(
    "EXPLAIN QUERY PLAN SELECT * FROM nodes WHERE ...",
    vec![]
).await?;
```

### 索引统计

```rust
/// 获取索引统计信息
pub async fn get_index_stats(backend: &SqliteBackend) -> Result<IndexStats> {
    let total_docs = backend.pool.exec_decode::<i64>(
        "SELECT COUNT(*) FROM nodes",
        vec![]
    ).await?;

    let index_size = std::fs::metadata(backend.index_dir().join("index.db"))?.len();

    Ok(IndexStats {
        total_documents: total_docs,
        index_size_bytes: index_size,
        // ...
    })
}
```

## 错误处理

```rust
use mf_search::error::{SearchError, Result};

match backend.search_ids(query).await {
    Ok(ids) => {
        println!("找到 {} 个结果", ids.len());
    }
    Err(e) => {
        match e.downcast_ref::<SearchError>() {
            Some(SearchError::InvalidQuery(msg)) => {
                println!("查询语法错误: {}", msg);
            }
            Some(SearchError::IndexCorrupted) => {
                // 重建索引
                backend.rebuild_all(docs).await?;
            }
            _ => return Err(e),
        }
    }
}
```

## 最佳实践

### 1. 选择合适的后端

```rust
// 生产环境：持久化索引
let backend = SqliteBackend::new_in_dir("./index").await?;

// 测试环境：临时索引
let backend = SqliteBackend::new_in_system_temp().await?;

// 自定义临时目录
let backend = SqliteBackend::new_in_temp_root("/tmp/app").await?;
```

### 2. 索引字段设计

```rust
// 为常用查询字段创建快速字段
pub order_i64: Option<i64>,       // 排序
pub created_at_i64: Option<i64>,  // 时间过滤
pub updated_at_i64: Option<i64>,  // 更新跟踪
pub status_i64: Option<i64>,      // 状态过滤
```

### 3. 查询设计

```rust
// ✅ 好：使用索引字段
query.node_type = Some("paragraph".into());

// ✅ 好：限制返回数量
query.limit = 100;

// ❌ 差：返回所有数据
query.limit = usize::MAX;
```

## 下一步

- 查看 [moduforge-state](./state.md) 了解状态管理和插件系统
- 查看 [moduforge-model](./model.md) 了解文档模型
- 查看 [moduforge-persistence](./persistence.md) 了解持久化
- 浏览 [Price-RS 项目](https://github.com/LiRenTech/price-rs) 查看实际应用
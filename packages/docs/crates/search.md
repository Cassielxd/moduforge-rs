# moduforge-search 文档

`moduforge-search` 基于 Tantivy 提供全文搜索和增量索引能力。

## 概述

Search 层与事务系统集成,自动更新索引,支持中文分词。

## 核心功能

- **全文搜索**：基于 Tantivy 的高性能搜索
- **中文分词**：jieba 分词支持
- **增量索引**：与事务系统集成
- **多字段搜索**：复杂查询支持

## 使用示例

```rust
use mf_search::SearchEngine;

// 创建搜索引擎
let engine = SearchEngine::new("./index")?;

// 索引文档
engine.index_document(doc)?;

// 搜索
let results = engine.search("关键词", 10)?;

for result in results {
    println!("找到: {}", result.title);
}
```

## 与运行时集成

```rust
// 注册搜索插件
runtime.register_plugin(SearchPlugin::new("./index"))?;

// 自动索引所有事务变更
// 无需手动调用
```

## 下一步

- 查看 [moduforge-state](./state.md)

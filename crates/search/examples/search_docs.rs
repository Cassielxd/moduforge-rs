//! 结构化查询示例：返回完整文档

use std::sync::Arc;
use mf_search::{SearchQuery, SqliteBackend, SearchService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建临时索引
    let backend = Arc::new(SqliteBackend::new_in_system_temp()?);
    let search_svc = SearchService::new(backend.clone());

    // 准备测试数据
    let docs = vec![
        mf_search::model::IndexDoc {
            node_id: "article1".into(),
            node_type: "article".into(),
            parent_id: Some("root".into()),
            marks: vec!["featured".into()],
            marks_json: r#"[{"type":"featured","attrs":{}}]"#.into(),
            attrs_flat: vec![
                ("title".into(), "Rust 异步编程指南".into()),
                ("author".into(), "张三".into()),
                ("status".into(), "published".into()),
            ],
            attrs_json: r#"{"title":"Rust 异步编程指南","author":"张三","status":"published","views":1500}"#.into(),
            text: Some("详细介绍 Rust 异步编程的各种概念和最佳实践".into()),
            path: vec!["root".into(), "article1".into()],
            order_i64: Some(1),
            created_at_i64: Some(1704067200000), // 2024-01-01
            updated_at_i64: Some(1704153600000),
        },
        mf_search::model::IndexDoc {
            node_id: "article2".into(),
            node_type: "article".into(),
            parent_id: Some("root".into()),
            marks: vec![],
            marks_json: "[]".into(),
            attrs_flat: vec![
                ("title".into(), "深入理解所有权".into()),
                ("author".into(), "李四".into()),
                ("status".into(), "draft".into()),
            ],
            attrs_json: r#"{"title":"深入理解所有权","author":"李四","status":"draft","views":800}"#.into(),
            text: Some("Rust 所有权系统的深度解析".into()),
            path: vec!["root".into(), "article2".into()],
            order_i64: Some(2),
            created_at_i64: Some(1704240000000), // 2024-01-03
            updated_at_i64: Some(1704326400000),
        },
        mf_search::model::IndexDoc {
            node_id: "article3".into(),
            node_type: "tutorial".into(),
            parent_id: Some("root".into()),
            marks: vec!["featured".into()],
            marks_json: r#"[{"type":"featured","attrs":{}}]"#.into(),
            attrs_flat: vec![
                ("title".into(), "从零开始学 Rust".into()),
                ("author".into(), "王五".into()),
                ("status".into(), "published".into()),
            ],
            attrs_json: r#"{"title":"从零开始学 Rust","author":"王五","status":"published","views":2300}"#.into(),
            text: Some("适合初学者的 Rust 入门教程".into()),
            path: vec!["root".into(), "article3".into()],
            order_i64: Some(3),
            created_at_i64: Some(1704412800000), // 2024-01-05
            updated_at_i64: Some(1704499200000),
        },
    ];

    backend.rebuild_all(docs).await?;

    println!("=== 结构化查询示例（返回完整文档）===\n");

    // 示例 1: 查询所有已发布的文章
    println!("1. 查询所有已发布的内容:");
    let results = search_svc
        .search_docs(SearchQuery {
            attrs: vec![("status".into(), "published".into())],
            limit: 10,
            ..Default::default()
        })
        .await?;

    for doc in &results {
        println!(
            "  - [{}] {}",
            doc.node_id,
            doc.attrs_flat
                .iter()
                .find(|(k, _)| k == "title")
                .map(|(_, v)| v.as_str())
                .unwrap_or("无标题")
        );
        println!(
            "    类型: {}, 作者: {}",
            doc.node_type,
            doc.attrs_flat
                .iter()
                .find(|(k, _)| k == "author")
                .map(|(_, v)| v.as_str())
                .unwrap_or("未知")
        );
        println!("    内容: {}\n", doc.text.as_deref().unwrap_or(""));
    }

    // 示例 2: 全文搜索并获取完整信息
    println!("2. 全文搜索 'Rust' 并获取完整文档:");
    let results = search_svc.search_text_docs("Rust", 10).await?;

    for doc in &results {
        println!("  - ID: {}", doc.node_id);
        println!("    文本: {}", doc.text.as_deref().unwrap_or(""));
        println!("    marks: {:?}", doc.marks);
        println!("    创建时间: {:?}\n", doc.created_at_i64);
    }

    // 示例 3: 按类型查询
    println!("3. 查询所有 'article' 类型的文档:");
    let results = backend
        .search_docs(SearchQuery {
            node_type: Some("article".into()),
            limit: 10,
            ..Default::default()
        })
        .await?;

    println!("  找到 {} 篇文章:", results.len());
    for doc in &results {
        let title = doc
            .attrs_flat
            .iter()
            .find(|(k, _)| k == "title")
            .map(|(_, v)| v.as_str())
            .unwrap_or("无标题");
        println!("    - {}", title);
    }
    println!();

    // 示例 4: 查询带有 featured mark 的内容
    println!("4. 查询精选内容（带 featured mark）:");
    let results = backend
        .search_docs(SearchQuery {
            marks: vec!["featured".into()],
            limit: 10,
            ..Default::default()
        })
        .await?;

    for doc in &results {
        let title = doc
            .attrs_flat
            .iter()
            .find(|(k, _)| k == "title")
            .map(|(_, v)| v.as_str())
            .unwrap_or("无标题");
        println!("  - [精选] {}", title);
    }
    println!();

    // 示例 5: 按时间排序查询
    println!("5. 按创建时间降序查询（最新的内容）:");
    let results = backend
        .search_docs(SearchQuery {
            sort_by: Some("created_at_i64".into()),
            sort_asc: false,
            limit: 3,
            ..Default::default()
        })
        .await?;

    for doc in &results {
        let title = doc
            .attrs_flat
            .iter()
            .find(|(k, _)| k == "title")
            .map(|(_, v)| v.as_str())
            .unwrap_or("无标题");
        println!("  - {} (创建于: {:?})", title, doc.created_at_i64);
    }

    Ok(())
}

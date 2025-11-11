//! 高级查询示例：演示 marks 和 attrs 的完整查询能力

use std::sync::Arc;
use mf_search::{SearchQuery, SqliteBackend, SearchService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建临时索引
    let backend = Arc::new(SqliteBackend::new_in_system_temp().await?);
    let search_svc = SearchService::new(backend.clone());

    // 准备测试数据：包含多种 marks 和复杂属性
    let docs = vec![
        mf_search::model::IndexDoc {
            node_id: "doc1".into(),
            node_type: "paragraph".into(),
            parent_id: Some("root".into()),
            marks: vec!["bold".into(), "link".into()],
            marks_json: r#"[
                {"type":"bold","attrs":{}},
                {"type":"link","attrs":{"href":"https://example.com","title":"Example"}}
            ]"#.into(),
            attrs_flat: vec![("status".into(), "published".into())],
            attrs_json: r#"{"status":"published","priority":1}"#.into(),
            text: Some("带有链接的粗体文本".into()),
            path: vec!["root".into(), "doc1".into()],
            order_i64: Some(1),
            created_at_i64: Some(1000),
            updated_at_i64: Some(1500),
        },
        mf_search::model::IndexDoc {
            node_id: "doc2".into(),
            node_type: "paragraph".into(),
            parent_id: Some("root".into()),
            marks: vec!["link".into(), "color".into()],
            marks_json: r##"[
                {"type":"link","attrs":{"href":"https://rust-lang.org","title":"Rust"}},
                {"type":"color","attrs":{"color":"#ff0000"}}
            ]"##.into(),
            attrs_flat: vec![("status".into(), "draft".into())],
            attrs_json: r#"{"status":"draft","priority":2}"#.into(),
            text: Some("红色链接文本".into()),
            path: vec!["root".into(), "doc2".into()],
            order_i64: Some(2),
            created_at_i64: Some(2000),
            updated_at_i64: Some(2500),
        },
        mf_search::model::IndexDoc {
            node_id: "doc3".into(),
            node_type: "paragraph".into(),
            parent_id: Some("root".into()),
            marks: vec!["bold".into()],
            marks_json: r#"[{"type":"bold","attrs":{}}]"#.into(),
            attrs_flat: vec![("status".into(), "published".into())],
            attrs_json: r#"{"status":"published","priority":1}"#.into(),
            text: Some("普通粗体文本".into()),
            path: vec!["root".into(), "doc3".into()],
            order_i64: Some(3),
            created_at_i64: Some(3000),
            updated_at_i64: Some(3500),
        },
    ];

    backend.rebuild_all(docs).await?;

    println!("=== 高级查询示例 ===\n");

    // 1) 简单 mark 类型查询（只检查是否存在某类型 mark）
    println!("1. 查询所有包含 'bold' mark 的节点:");
    let results = search_svc
        .search(SearchQuery {
            marks: vec!["bold".into()],
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("   结果: {:?}\n", results);

    // 2) 精确 mark 属性查询（查询链接到特定 URL 的节点）
    println!("2. 查询链接到 'https://example.com' 的节点:");
    let results = backend
        .search_ids(SearchQuery {
            mark_attrs: vec![(
                "link".into(),
                "href".into(),
                "https://example.com".into(),
            )],
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("   结果: {:?}\n", results);

    // 3) 查询特定颜色的节点
    println!("3. 查询红色文本节点:");
    let results = backend
        .search_ids(SearchQuery {
            mark_attrs: vec![(
                "color".into(),
                "color".into(),
                String::from("#ff0000"),
            )],
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("   结果: {:?}\n", results);

    // 4) 组合查询：既有 bold mark，又链接到 rust-lang.org
    println!("4. 查询同时包含 'bold' 和链接到 Rust 官网的节点:");
    let results = backend
        .search_ids(SearchQuery {
            marks: vec!["bold".into()],
            mark_attrs: vec![(
                "link".into(),
                "href".into(),
                "https://rust-lang.org".into(),
            )],
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("   结果: {:?}\n", results);

    // 5) 复杂组合：mark + attrs + 全文搜索
    println!("5. 复杂查询：published 状态 + bold mark + 包含'粗体':");
    let results = backend
        .search_ids(SearchQuery {
            text: Some("粗体".into()),
            marks: vec!["bold".into()],
            attrs: vec![("status".into(), "published".into())],
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("   结果: {:?}\n", results);

    Ok(())
}

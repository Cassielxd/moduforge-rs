//! 基础示例：创建临时索引、写入文档并执行查询

use std::sync::Arc;

use mf_search::{IndexService, SearchQuery, SqliteBackend, SearchService};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) 使用系统临时目录创建索引（程序结束自动清理）
    let backend = Arc::new(SqliteBackend::new_in_system_temp().await?);
    let _index_svc = IndexService::new(backend.clone());
    let search_svc = SearchService::new(backend.clone());

    // 2) 准备若干文档（手动构造 IndexDoc，实际项目中可由 NodePool 转换获得）
    let docs = vec![
        mf_search::model::IndexDoc {
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
            created_at_i64: Some(1_000),
            updated_at_i64: Some(1_500),
        },
        mf_search::model::IndexDoc {
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
            created_at_i64: Some(2_000),
            updated_at_i64: Some(2_500),
        },
    ];

    // 3) 全量重建索引
    backend.rebuild_all(docs).await?;

    // 4) 执行查询（全文 + 类型过滤）
    let ids = search_svc
        .search(SearchQuery {
            text: Some("示例".into()),
            node_type: Some("paragraph".into()),
            limit: 10,
            ..Default::default()
        })
        .await?;

    println!("命中节点: {ids:?}");

    // 使用便捷方法：全文搜索
    let ids_text = search_svc.search_text("搜索", 10).await?;
    println!("全文搜索: {ids_text:?}");

    // 也可以按属性/标记过滤
    let ids2 = backend
        .search_ids(SearchQuery {
            attrs: vec![("lang".into(), "en".into())],
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("按属性过滤命中: {ids2:?}");

    // 5) 按 fast field 排序（created_at_i64 降序），获取第一页
    let first_page = backend
        .search_ids(SearchQuery {
            sort_by: Some("created_at_i64".into()),
            sort_asc: false,
            limit: 1,
            ..Default::default()
        })
        .await?;
    println!("按 created_at_i64 降序第一页: {first_page:?}");

    // 6) 获取第二页（使用 offset）
    let second_page = backend
        .search_ids(SearchQuery {
            sort_by: Some("created_at_i64".into()),
            sort_asc: false,
            offset: 1,
            limit: 1,
            ..Default::default()
        })
        .await?;
    println!("按 created_at_i64 降序第二页: {second_page:?}");

    // 7) 范围过滤：created_at_i64 在 [1000, 1500]
    let ranged = backend
        .search_ids(SearchQuery {
            range_field: Some("created_at_i64".into()),
            range_min: Some(1_000),
            range_max: Some(1_500),
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("范围过滤命中: {ranged:?}");

    // 打印索引目录（仅调试查看）
    println!("索引目录: {}", backend.index_dir().display());

    Ok(())
}

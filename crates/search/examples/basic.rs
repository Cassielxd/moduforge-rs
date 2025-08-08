//! 基础示例：创建临时索引、写入文档并执行查询

use std::sync::Arc;

use mf_search::{IndexService, SearchQuery, TantivyBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) 使用系统临时目录创建索引（程序结束自动清理）
    let backend = Arc::new(TantivyBackend::new_in_system_temp()?);
    let _svc = IndexService::new(backend.clone());

    // 2) 准备若干文档（手动构造 IndexDoc，实际项目中可由 NodePool 转换获得）
    let docs = vec![
        mf_search::model::IndexDoc {
            node_id: "n1".into(),
            node_type: "paragraph".into(),
            parent_id: Some("root".into()),
            marks: vec!["bold".into()],
            attrs_flat: vec![("lang".into(), "zh".into())],
            text: Some("Rust 搜索引擎示例".into()),
            path: vec!["root".into(), "n1".into()],
        },
        mf_search::model::IndexDoc {
            node_id: "n2".into(),
            node_type: "paragraph".into(),
            parent_id: Some("root".into()),
            marks: vec![],
            attrs_flat: vec![("lang".into(), "en".into())],
            text: Some("Tantivy backend quick demo".into()),
            path: vec!["root".into(), "n2".into()],
        },
    ];

    // 3) 全量重建索引
    backend.rebuild_all(docs).await?;

    // 4) 执行查询（全文 + 类型过滤）
    let ids = backend
        .search_ids(SearchQuery {
            text: Some("示例".into()),
            node_type: Some("paragraph".into()),
            limit: 10,
            ..Default::default()
        })
        .await?;

    println!("命中节点: {:?}", ids);

    // 也可以按属性/标记过滤
    let ids2 = backend
        .search_ids(SearchQuery {
            attrs: vec![("lang".into(), "en".into())],
            limit: 10,
            ..Default::default()
        })
        .await?;
    println!("按属性过滤命中: {:?}", ids2);

    // 打印索引目录（仅调试查看）
    println!("索引目录: {}", backend.index_dir().display());

    Ok(())
}



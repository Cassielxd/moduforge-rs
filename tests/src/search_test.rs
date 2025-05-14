use moduforge_model::node_type::NodeEnum;
use moduforge_model::{
    node::Node, node_pool::NodePool, types::NodeId, attrs::Attrs,
};
use std::sync::Arc;
use std::time::Instant;
use im::HashMap;
use rayon::prelude::*;

pub type NodePredicate = Box<dyn Fn(&Node) -> bool + Send + Sync>;

/// 生成测试数据
pub fn generate_test_data(size: usize) -> NodeEnum {
    // 创建根节点
    let root_id = "root".to_string();
    let mut root_attrs = im::HashMap::new();
    root_attrs.insert("depth".to_string(), serde_json::json!(0));
    let root_node = Node::new(
        &root_id,
        "document".to_string(),
        Attrs::from(root_attrs),
        vec![],
        vec![],
    );

    // 递归生成子节点
    fn generate_children(
        parent_id: &str,
        depth: usize,
        remaining: &mut usize,
    ) -> Vec<NodeEnum> {
        if *remaining == 0 {
            return vec![];
        }

        let mut children = Vec::new();
        let child_count = (*remaining).min(10); // 每个父节点最多10个子节点
        *remaining -= child_count;

        for i in 0..child_count {
            let node_type = match i % 5 {
                0 => "document",
                1 => "heading",
                2 => "paragraph",
                3 => "list",
                _ => "text",
            };

            let mut attrs = im::HashMap::new();
            attrs.insert("index".to_string(), serde_json::json!(i));
            attrs.insert("depth".to_string(), serde_json::json!(depth));

            let node_id = format!("{}_{}", parent_id, i);
            let node = Node::new(
                &node_id,
                node_type.to_string(),
                Attrs::from(attrs),
                vec![],
                vec![],
            );

            // 递归生成子节点的子节点
            let mut child_remaining = (*remaining).min(100); // 限制每层最多100个节点
            let grandchildren =
                generate_children(&node_id, depth + 1, &mut child_remaining);
            *remaining -= child_remaining;

            children.push(NodeEnum(node, grandchildren));
        }

        children
    }

    // 生成整个树结构
    let mut remaining = size - 1; // 减去根节点
    let children = generate_children(&root_id, 1, &mut remaining);

    // 返回根节点及其子节点
    NodeEnum(root_node, children)
}

#[test]
fn test_query_engine_performance() {
    let test_data = generate_test_data(1000_000);

    // 创建节点池
    let node_pool = NodePool::from(test_data);
    println!("创建节点池");

    // 测试1: 按类型查询
    let config = moduforge_model::node_pool::QueryCacheConfig {
        capacity: 1000,
        enabled: true,
    };
    let start = Instant::now();
    let engine = node_pool.optimized_query(config.clone());
    let duration = start.elapsed();
    println!("构建索引: 用时 {:?}", duration);

    let start = Instant::now();
    let document_nodes = engine.by_type("document");
    let duration = start.elapsed();
    println!(
        "按类型查询 (document): 找到 {} 个节点，用时 {:?}",
        document_nodes.len(),
        duration
    );
    if !document_nodes.is_empty() {
        println!(
            "  First document node: 类型={}, ID={}",
            document_nodes[0].r#type, document_nodes[0].id
        );
    }

    // 测试2: 按深度查询
    let engine = node_pool.optimized_query(config.clone());
    let start = Instant::now();
    let depth_nodes = engine.by_depth(0);
    let duration = start.elapsed();
    println!(
        "按深度查询 (depth 0): 找到 {} 个节点，用时 {:?}",
        depth_nodes.len(),
        duration
    );
    if !depth_nodes.is_empty() {
        println!(
            "  First depth node: 类型={}, ID={}, 深度={}",
            depth_nodes[0].r#type,
            depth_nodes[0].id,
            depth_nodes[0].attrs.get("depth").unwrap_or(&serde_json::json!(0))
        );
    }

    // 测试3: 组合查询
    let engine = node_pool.optimized_query(config.clone());
    let start = Instant::now();
    let document_nodes = engine.by_type("document");
    let results: Vec<_> = document_nodes
        .clone()
        .into_par_iter()
        .filter(|node| {
            node.attrs
                .get("depth")
                .and_then(|v| v.as_u64())
                .map_or(false, |d| d == 0)
        })
        .collect();
    let duration = start.elapsed();
    println!("组合查询: 找到 {} 个节点，用时 {:?}", results.len(), duration);
    if !results.is_empty() {
        println!(
            "  First result: 类型={}, ID={}, 深度={}",
            results[0].r#type,
            results[0].id,
            results[0].attrs.get("depth").unwrap_or(&serde_json::json!(0))
        );
    }

    // 测试4: 并行查询
    let start = Instant::now();
    let parallel_results: Vec<_> = document_nodes
        .clone()
        .into_par_iter()
        .filter(|node| {
            node.attrs
                .get("depth")
                .and_then(|v| v.as_u64())
                .map_or(false, |d| d == 0)
        })
        .collect();
    let duration = start.elapsed();
    println!(
        "并行查询: 找到 {} 个节点，用时 {:?}",
        parallel_results.len(),
        duration
    );
    if !parallel_results.is_empty() {
        println!(
            "  First parallel result: 类型={}, ID={}, 深度={}",
            parallel_results[0].r#type,
            parallel_results[0].id,
            parallel_results[0]
                .attrs
                .get("depth")
                .unwrap_or(&serde_json::json!(0))
        );
    }

    // 测试5: 缓存效果
    let mut engine = node_pool.optimized_query(config.clone());
    let start = Instant::now();
    let conditions: Vec<NodePredicate> = vec![
        Box::new(|node: &Node| node.r#type == "document"),
        Box::new(|node: &Node| {
            node.attrs
                .get("depth")
                .and_then(|v| v.as_u64())
                .map_or(false, |d| d == 0)
        }),
    ];
    let _cached_results = engine.query(conditions);
    let duration = start.elapsed();
    println!(
        "缓存查询: 找到 {} 个节点，用时 {:?}",
        _cached_results.len(),
        duration
    );

    // 测试6: 批量查询
    let start = Instant::now();
    let batch_results = node_pool.parallel_batch_query(5000, |chunk| {
        chunk.iter().filter(|node| node.r#type == "document").cloned().collect()
    });
    let duration = start.elapsed();
    println!(
        "批量查询: 找到 {} 个节点，用时 {:?}",
        batch_results.len(),
        duration
    );
    if !batch_results.is_empty() {
        println!(
            "  First batch result: 类型={}, ID={}",
            batch_results[0].r#type, batch_results[0].id
        );
    }

    // 验证结果
    let document_nodes_clone = document_nodes.clone();
    assert!(!document_nodes_clone.is_empty(), "应该找到文档节点");
    assert!(!depth_nodes.is_empty(), "应该找到深度为0的节点");
    assert!(!results.is_empty(), "应该找到符合组合条件的节点");
    assert!(!parallel_results.is_empty(), "应该找到并行查询的节点");
}

use moduforge_model::{node::Node, node_pool::NodePool, types::NodeId, attrs::Attrs};
use std::sync::Arc;
use std::time::Instant;
use im::HashMap;
use rayon::prelude::*;

type NodePredicate = Box<dyn Fn(&Node) -> bool + Send + Sync>;

/// 生成测试数据
fn generate_test_data(size: usize) -> (Vec<Node>, NodeId) {
    let mut nodes = Vec::with_capacity(size);
    let root_id = "root".to_string();
    
    // 创建根节点
    let mut root_attrs = HashMap::new();
    root_attrs.insert("depth".to_string(), serde_json::json!(0));
    nodes.push(Node::new(
        &root_id,  // 节点ID
        "root".to_string(),  // 节点类型
        Attrs::from(root_attrs),
        vec![],
        vec![],
    ));

    // 生成子节点
    for i in 0..size - 1 {
        let node_type = match i % 5 {
            0 => "document",
            1 => "section",
            2 => "paragraph",
            3 => "image",
            _ => "text",
        };

        let parent_idx = if i < 1000 {
            0 // 前1000个节点直接挂在根节点下
        } else {
            (i - 1000) / 10 + 1 // 其他节点形成树状结构
        };

        let mut attrs = HashMap::new();
        attrs.insert("index".to_string(), serde_json::json!(i));
        attrs.insert("depth".to_string(), serde_json::json!(i / 1000));

        let node_id = format!("node_{}", i);
        let node = Node::new(
            &node_id,  // 节点ID
            node_type.to_string(),  // 节点类型
            Attrs::from(attrs),
            vec![],
            vec![],
        );

        nodes.push(node);
    }

    // 构建父子关系
    for i in 1..size {
        let parent_idx = if i < 1000 {
            0
        } else {
            (i - 1000) / 10 + 1
        };
        let mut content = nodes[parent_idx].content.clone();
        content.push_back(nodes[i].id.clone());
        nodes[parent_idx].content = content;
    }

    (nodes, root_id)
}

#[test]
fn test_query_engine_performance() {

    let (nodes, root_id) = generate_test_data(1000_000);
    println!("创建 {} 个测试节点", nodes.len());
    println!("节点类型分布:");
    let mut type_counts = std::collections::HashMap::new();
    for node in &nodes {
        *type_counts.entry(&node.r#type).or_insert(0) += 1;
    }
    for (type_name, count) in type_counts {
        println!("  {}: {}", type_name, count);
    }

    // 创建节点池
    let node_pool = NodePool::from(nodes, root_id);
    println!("创建节点池");

    // 测试1: 按类型查询
    let config = moduforge_model::node_pool::QueryCacheConfig {
        capacity: 1000,
        enabled: true,
    };
    let mut engine = node_pool.optimized_query(config.clone());
    let start = Instant::now();
    let document_nodes = engine.by_type("document");
    let duration = start.elapsed();
    println!("按类型查询 (document): 找到 {} 个节点，用时 {:?}", document_nodes.len(), duration);
    if !document_nodes.is_empty() {
        println!("  First document node: 类型={}, ID={}", document_nodes[0].r#type, document_nodes[0].id);
    }

    // 测试2: 按深度查询
    let mut engine = node_pool.optimized_query(config.clone());
    let start = Instant::now();
    let depth_nodes = engine.by_depth(0);
    let duration = start.elapsed();
    println!("按深度查询 (depth 0): 找到 {} 个节点，用时 {:?}", depth_nodes.len(), duration);
    if !depth_nodes.is_empty() {
        println!("  First depth node: 类型={}, ID={}, 深度={}", 
            depth_nodes[0].r#type, 
            depth_nodes[0].id,
            depth_nodes[0].attrs.get("depth").unwrap_or(&serde_json::json!(0))
        );
    }

    // 测试3: 组合查询
    let mut engine = node_pool.optimized_query(config.clone());
    let start = Instant::now();
    // 先使用索引获取所有 document 类型的节点
    let document_nodes = engine.by_type("document");
    // 然后在这些节点中过滤深度为0的节点
    let results: Vec<_> = document_nodes.clone().into_par_iter()
        .filter(|node| {
            node.attrs.get("depth")
                .and_then(|v| v.as_u64())
                .map_or(false, |d| d == 0)
        })
        .collect();
    let duration = start.elapsed();
    println!("组合查询: 找到 {} 个节点，用时 {:?}", results.len(), duration);
    if !results.is_empty() {
        println!("  First result: 类型={}, ID={}, 深度={}", 
            results[0].r#type, 
            results[0].id,
            results[0].attrs.get("depth").unwrap_or(&serde_json::json!(0))
        );
    }

    // 测试4: 并行查询
    let start = Instant::now();
    // 同样先使用索引获取所有 document 类型的节点
    let parallel_results: Vec<_> = document_nodes.clone().into_par_iter()
        .filter(|node| {
            node.attrs.get("depth")
                .and_then(|v| v.as_u64())
                .map_or(false, |d| d == 0)
        })
        .collect();
    let duration = start.elapsed();
    println!("并行查询: 找到 {} 个节点，用时 {:?}", parallel_results.len(), duration);
    if !parallel_results.is_empty() {
        println!("  First parallel result: 类型={}, ID={}, 深度={}", 
            parallel_results[0].r#type, 
            parallel_results[0].id,
            parallel_results[0].attrs.get("depth").unwrap_or(&serde_json::json!(0))
        );
    }

    // 测试5: 缓存效果
    let mut engine = node_pool.optimized_query(config.clone());
    let start = Instant::now();
    let conditions: Vec<NodePredicate> = vec![
        Box::new(|node: &Node| node.r#type == "document"),
        Box::new(|node: &Node| {
            node.attrs.get("depth")
                .and_then(|v| v.as_u64())
                .map_or(false, |d| d == 0)
        }),
    ];
    let _cached_results = engine.query(conditions);
    let duration = start.elapsed();
    println!("缓存查询: 找到 {} 个节点，用时 {:?}", _cached_results.len(), duration);

    // 测试6: 批量查询
    let start = Instant::now();
    let batch_results = node_pool.parallel_batch_query(5000, |chunk| {
        chunk.iter()
            .filter(|node| node.r#type == "document")
            .cloned()
            .collect()
    });
    let duration = start.elapsed();
    println!("批量查询: 找到 {} 个节点，用时 {:?}", batch_results.len(), duration);
    if !batch_results.is_empty() {
        println!("  First batch result: 类型={}, ID={}", 
            batch_results[0].r#type, 
            batch_results[0].id
        );
    }

    // 验证结果
    let document_nodes_clone = document_nodes.clone();
    assert!(!document_nodes_clone.is_empty(), "应该找到文档节点");
    assert!(!depth_nodes.is_empty(), "应该找到深度为0的节点");
    assert!(!results.is_empty(), "应该找到符合组合条件的节点");
    assert!(!parallel_results.is_empty(), "应该找到并行查询的节点");
}

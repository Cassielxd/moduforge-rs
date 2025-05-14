use moduforge_model::node_type::NodeEnum;
use moduforge_model::{
    node::Node, node_pool::NodePool, types::NodeId, attrs::Attrs,
};
use std::sync::Arc;
use std::time::Instant;
use im::HashMap;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap as StdHashMap;
use crossbeam::queue::SegQueue;
use std::sync::Mutex;
use im::Vector;

pub type NodePredicate = Box<dyn Fn(&Node) -> bool + Send + Sync>;

/// 生成测试数据
pub fn generate_test_data(size: usize) -> NodeEnum {
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

    // 使用原子计数器
    let node_count = AtomicUsize::new(1);
    let branch = 10;
    
    // 预分配节点存储
    let mut nodes = Vec::with_capacity(size);
    nodes.push((root_id.clone(), 0, root_node.clone()));
    
    // 使用无锁队列存储待处理的节点
    let queue = Arc::new(SegQueue::new());
    queue.push((root_id, 0));
    
    // 使用线程本地存储来减少锁竞争
    let thread_local_nodes: Arc<Mutex<Vec<(String, usize, Node)>>> = Arc::new(Mutex::new(Vec::with_capacity(size)));
    
    // 使用默认的全局线程池
    while node_count.load(Ordering::Relaxed) < size {
        let mut batch = Vec::with_capacity(branch * 2);
        
        // 批量获取待处理的节点
        while let Some((parent_id, depth)) = queue.pop() {
            batch.push((parent_id, depth));
            if batch.len() >= branch * 2 {
                break;
            }
        }
        
        if batch.is_empty() {
            break;
        }
        
        // 并行处理批次
        batch.par_iter().for_each(|(parent_id, depth)| {
            let mut local_nodes = Vec::with_capacity(branch);
            let mut children_ids = Vector::new();
            
            for i in 0..branch {
                let current_count = node_count.load(Ordering::Relaxed);
                if current_count >= size {
                    break;
                }
                
                let node_type = match i % 5 {
                    0 => "document",
                    1 => "heading",
                    2 => "paragraph",
                    3 => "list",
                    _ => "text",
                };
                
                let mut attrs = im::HashMap::new();
                attrs.insert("index".to_string(), serde_json::json!(i));
                attrs.insert("depth".to_string(), serde_json::json!(depth + 1));
                attrs.insert("parent_id".to_string(), serde_json::json!(parent_id.clone()));
                
                let node_id = format!("{}_{}", parent_id, i);
                let node = Node::new(
                    &node_id,
                    node_type.to_string(),
                    Attrs::from(attrs),
                    vec![parent_id.clone()],
                    vec![],
                );
                
                children_ids.push_back(node_id.clone());
                local_nodes.push((node_id.clone(), depth + 1, node));
                
                // 使用原子操作增加计数
                node_count.fetch_add(1, Ordering::Relaxed);
            }
            
            // 批量添加本地节点
            if !local_nodes.is_empty() {
                let mut thread_nodes = thread_local_nodes.lock().unwrap();
                thread_nodes.extend(local_nodes.clone());
                
                // 将新节点添加到队列
                for (node_id, depth, _) in local_nodes {
                    queue.push((node_id, depth));
                }
            }
        });
    }
    
    // 收集所有节点
    let mut all_nodes = thread_local_nodes.lock().unwrap().clone();
    all_nodes.extend(nodes);
    
    // 按深度排序
    all_nodes.par_sort_by_key(|(_, depth, _)| *depth);
    
    // 构建树结构
    let mut node_map: StdHashMap<String, Vec<NodeEnum>> = StdHashMap::new();
    let mut root_children = Vec::new();
    
    // 从叶子节点开始构建树
    for (node_id, depth, node) in all_nodes.into_iter().rev() {
        if depth == 0 {
            continue; // 跳过根节点
        }
        
        let children = node_map.remove(&node_id).unwrap_or_default();
        let node_enum = NodeEnum(node.clone(), children);
        
        if depth == 1 {
            root_children.push(node_enum);
        } else {
            let parent_id = node.attrs.get("parent_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            node_map.entry(parent_id).or_default().push(node_enum);
        }
    }
    
    // 更新根节点
    let mut root_node = root_node;
    root_node.content = Vector::from_iter(root_children.iter().map(|child| child.0.id.clone()));
    
    NodeEnum(root_node, root_children)
}

// 辅助函数：计算树中的节点总数
fn count_nodes(node: &NodeEnum) -> usize {
    1 + node.1.iter().map(|child| count_nodes(child)).sum::<usize>()
}

#[test]
fn test_query_engine_performance() {
    let test_data = generate_test_data(100_0000);

    // 创建节点池
    let node_pool = NodePool::from(test_data);
    println!("创建节点池，节点数量: {}", node_pool.size());

    // 测试1: 按类型查询
    let config = moduforge_model::node_pool::QueryCacheConfig {
        capacity: 1000,
        enabled: true,
    };
    let engine = node_pool.optimized_query(config.clone());

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


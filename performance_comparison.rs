// 性能对比测试代码
// 比较 im-rs 和 Yrs 在历史记录场景下的性能

use std::time::Instant;
use im::{HashMap as ImHashMap, Vector as ImVector};
use yrs::{Doc, Text, Transact};
use serde::{Serialize, Deserialize};

// 模拟的文档节点结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentNode {
    pub id: String,
    pub node_type: String,
    pub content: String,
    pub attributes: ImHashMap<String, String>,
    pub children: ImVector<String>,
}

// 使用 im-rs 的历史管理
#[derive(Clone)]
pub struct ImrsHistoryManager {
    history: ImVector<ImHashMap<String, DocumentNode>>,
    current_version: usize,
}

impl ImrsHistoryManager {
    pub fn new() -> Self {
        Self {
            history: ImVector::new(),
            current_version: 0,
        }
    }

    pub fn add_snapshot(&mut self, nodes: ImHashMap<String, DocumentNode>) {
        self.history.push_back(nodes);
        self.current_version = self.history.len() - 1;
    }

    pub fn get_snapshot(&self, version: usize) -> Option<&ImHashMap<String, DocumentNode>> {
        self.history.get(version)
    }

    pub fn update_node(&mut self, node_id: &str, content: String) {
        if let Some(current) = self.history.get(self.current_version) {
            let mut new_snapshot = current.clone();
            if let Some(node) = new_snapshot.get_mut(node_id) {
                node.content = content;
            }
            self.add_snapshot(new_snapshot);
        }
    }
}

// 使用 Yrs 的历史管理
pub struct YrsHistoryManager {
    doc: Doc,
    snapshots: Vec<Vec<u8>>, // 存储编码后的状态
}

impl YrsHistoryManager {
    pub fn new() -> Self {
        Self {
            doc: Doc::new(),
            snapshots: Vec::new(),
        }
    }

    pub fn add_snapshot(&mut self) {
        let state = self.doc.encode_state_as_update_v1(&yrs::StateVector::default());
        self.snapshots.push(state);
    }

    pub fn update_node(&mut self, node_id: &str, content: String) {
        let txn = self.doc.transact_mut();
        if let Ok(text) = txn.get_text(node_id) {
            text.clear(&txn);
            text.insert(&txn, 0, &content);
        } else {
            // 创建新的文本节点
            let text = txn.get_or_insert_text(node_id);
            text.insert(&txn, 0, &content);
        }
        txn.commit();
        self.add_snapshot();
    }

    pub fn restore_snapshot(&mut self, version: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(state) = self.snapshots.get(version) {
            self.doc = Doc::new();
            let mut txn = self.doc.transact_mut();
            txn.apply_update(yrs::Update::decode_v1(state)?);
            txn.commit();
        }
        Ok(())
    }
}

// 性能测试函数
pub fn benchmark_performance() {
    println!("=== ModuForge-RS 历史记录性能对比测试 ===\n");

    // 测试数据规模
    let node_counts = vec![100, 1000, 5000];
    let operation_counts = vec![10, 50, 100];

    for &node_count in &node_counts {
        for &operation_count in &operation_counts {
            println!("测试规模: {} 节点, {} 操作", node_count, operation_count);
            
            // 测试 im-rs
            let imrs_time = benchmark_imrs(node_count, operation_count);
            
            // 测试 Yrs
            let yrs_time = benchmark_yrs(node_count, operation_count);
            
            println!("  im-rs 耗时: {:?}", imrs_time);
            println!("  Yrs 耗时: {:?}", yrs_time);
            println!("  性能比例: {:.2}x\n", 
                yrs_time.as_nanos() as f64 / imrs_time.as_nanos() as f64);
        }
    }

    // 内存使用对比
    println!("=== 内存使用对比 ===");
    memory_benchmark();

    // 历史查询性能对比
    println!("=== 历史查询性能对比 ===");
    history_query_benchmark();
}

fn benchmark_imrs(node_count: usize, operation_count: usize) -> std::time::Duration {
    let start = Instant::now();
    
    let mut manager = ImrsHistoryManager::new();
    
    // 创建初始节点
    let mut initial_nodes = ImHashMap::new();
    for i in 0..node_count {
        let node = DocumentNode {
            id: format!("node_{}", i),
            node_type: "text".to_string(),
            content: format!("Initial content {}", i),
            attributes: ImHashMap::new(),
            children: ImVector::new(),
        };
        initial_nodes.insert(node.id.clone(), node);
    }
    manager.add_snapshot(initial_nodes);
    
    // 执行更新操作
    for i in 0..operation_count {
        let node_id = format!("node_{}", i % node_count);
        let new_content = format!("Updated content {} - {}", i, chrono::Utc::now().timestamp());
        manager.update_node(&node_id, new_content);
    }
    
    start.elapsed()
}

fn benchmark_yrs(node_count: usize, operation_count: usize) -> std::time::Duration {
    let start = Instant::now();
    
    let mut manager = YrsHistoryManager::new();
    
    // 创建初始节点
    {
        let txn = manager.doc.transact_mut();
        for i in 0..node_count {
            let node_id = format!("node_{}", i);
            let content = format!("Initial content {}", i);
            let text = txn.get_or_insert_text(&node_id);
            text.insert(&txn, 0, &content);
        }
        txn.commit();
    }
    manager.add_snapshot();
    
    // 执行更新操作
    for i in 0..operation_count {
        let node_id = format!("node_{}", i % node_count);
        let new_content = format!("Updated content {} - {}", i, chrono::Utc::now().timestamp());
        manager.update_node(&node_id, new_content);
    }
    
    start.elapsed()
}

fn memory_benchmark() {
    use std::mem;
    
    // im-rs 内存使用
    let mut imrs_manager = ImrsHistoryManager::new();
    let mut initial_nodes = ImHashMap::new();
    
    for i in 0..1000 {
        let node = DocumentNode {
            id: format!("node_{}", i),
            node_type: "text".to_string(),
            content: "Lorem ipsum dolor sit amet".repeat(10), // 较长内容
            attributes: ImHashMap::new(),
            children: ImVector::new(),
        };
        initial_nodes.insert(node.id.clone(), node);
    }
    
    // 添加10个历史版本
    for i in 0..10 {
        imrs_manager.add_snapshot(initial_nodes.clone());
        // 修改一些节点
        if let Some(node) = initial_nodes.get_mut(&format!("node_{}", i * 100)) {
            node.content = format!("Updated version {}", i);
        }
    }
    
    let imrs_size = mem::size_of_val(&imrs_manager) + 
                   imrs_manager.history.len() * mem::size_of::<ImHashMap<String, DocumentNode>>();
    
    // Yrs 内存使用
    let mut yrs_manager = YrsHistoryManager::new();
    {
        let txn = yrs_manager.doc.transact_mut();
        for i in 0..1000 {
            let content = "Lorem ipsum dolor sit amet".repeat(10);
            let text = txn.get_or_insert_text(&format!("node_{}", i));
            text.insert(&txn, 0, &content);
        }
        txn.commit();
    }
    
    // 添加10个历史版本
    for i in 0..10 {
        yrs_manager.add_snapshot();
        let node_id = format!("node_{}", i * 100);
        yrs_manager.update_node(&node_id, format!("Updated version {}", i));
    }
    
    let yrs_size = mem::size_of_val(&yrs_manager) + 
                  yrs_manager.snapshots.iter().map(|s| s.len()).sum::<usize>();
    
    println!("  im-rs 估算内存使用: {} bytes", imrs_size);
    println!("  Yrs 估算内存使用: {} bytes", yrs_size);
    println!("  内存效率比: {:.2}x", imrs_size as f64 / yrs_size as f64);
}

fn history_query_benchmark() {
    // 创建有100个版本的历史
    let mut imrs_manager = ImrsHistoryManager::new();
    let mut yrs_manager = YrsHistoryManager::new();
    
    // 初始化
    let mut nodes = ImHashMap::new();
    for i in 0..100 {
        let node = DocumentNode {
            id: format!("node_{}", i),
            node_type: "text".to_string(),
            content: format!("Content {}", i),
            attributes: ImHashMap::new(),
            children: ImVector::new(),
        };
        nodes.insert(node.id.clone(), node);
    }
    
    // im-rs: 创建100个历史版本
    for version in 0..100 {
        imrs_manager.add_snapshot(nodes.clone());
        // 修改一个节点
        if let Some(node) = nodes.get_mut(&format!("node_{}", version % 100)) {
            node.content = format!("Updated content version {}", version);
        }
    }
    
    // Yrs: 创建100个历史版本
    {
        let txn = yrs_manager.doc.transact_mut();
        for i in 0..100 {
            let text = txn.get_or_insert_text(&format!("node_{}", i));
            text.insert(&txn, 0, &format!("Content {}", i));
        }
        txn.commit();
    }
    
    for version in 0..100 {
        yrs_manager.add_snapshot();
        let node_id = format!("node_{}", version % 100);
        yrs_manager.update_node(&node_id, format!("Updated content version {}", version));
    }
    
    // 测试历史查询性能
    let versions_to_test = vec![10, 30, 50, 70, 90];
    
    for &version in &versions_to_test {
        // im-rs 查询
        let start = Instant::now();
        let _snapshot = imrs_manager.get_snapshot(version);
        let imrs_query_time = start.elapsed();
        
        // Yrs 查询（恢复到特定版本）
        let start = Instant::now();
        let _ = yrs_manager.restore_snapshot(version);
        let yrs_query_time = start.elapsed();
        
        println!("  版本 {} 查询:");
        println!("    im-rs: {:?}", imrs_query_time);
        println!("    Yrs: {:?}", yrs_query_time);
        println!("    比例: {:.2}x", yrs_query_time.as_nanos() as f64 / imrs_query_time.as_nanos() as f64);
    }
}

// 协作场景下的性能对比
pub fn collaboration_benchmark() {
    println!("=== 协作场景性能对比 ===\n");
    
    // 模拟多用户协作
    let user_count = 5;
    let operations_per_user = 20;
    
    println!("模拟 {} 用户，每用户 {} 操作", user_count, operations_per_user);
    
    // im-rs 无法直接支持协作，这里模拟通过序列化同步
    let start = Instant::now();
    simulate_imrs_collaboration(user_count, operations_per_user);
    let imrs_collab_time = start.elapsed();
    
    // Yrs 原生协作支持
    let start = Instant::now();
    simulate_yrs_collaboration(user_count, operations_per_user);
    let yrs_collab_time = start.elapsed();
    
    println!("  im-rs (模拟协作): {:?}", imrs_collab_time);
    println!("  Yrs (原生协作): {:?}", yrs_collab_time);
    println!("  协作效率比: {:.2}x", imrs_collab_time.as_nanos() as f64 / yrs_collab_time.as_nanos() as f64);
}

fn simulate_imrs_collaboration(user_count: usize, operations_per_user: usize) {
    let mut managers = Vec::new();
    
    // 为每个用户创建管理器
    for _ in 0..user_count {
        managers.push(ImrsHistoryManager::new());
    }
    
    // 模拟协作：每次操作后需要序列化和反序列化来同步状态
    for op in 0..operations_per_user {
        for user in 0..user_count {
            let node_id = format!("node_{}_{}", user, op);
            let content = format!("Content from user {} op {}", user, op);
            
            // 获取当前状态
            if let Some(current_state) = managers[user].history.back() {
                let mut new_state = current_state.clone();
                
                // 创建新节点
                let node = DocumentNode {
                    id: node_id.clone(),
                    node_type: "text".to_string(),
                    content,
                    attributes: ImHashMap::new(),
                    children: ImVector::new(),
                };
                new_state.insert(node_id, node);
                
                // 模拟序列化同步（实际场景中需要网络传输）
                let serialized = serde_json::to_string(&new_state).unwrap();
                let deserialized: ImHashMap<String, DocumentNode> = 
                    serde_json::from_str(&serialized).unwrap();
                
                // 更新所有用户的状态
                for manager in &mut managers {
                    manager.add_snapshot(deserialized.clone());
                }
            }
        }
    }
}

fn simulate_yrs_collaboration(user_count: usize, operations_per_user: usize) {
    let mut docs = Vec::new();
    
    // 为每个用户创建文档
    for _ in 0..user_count {
        docs.push(Doc::new());
    }
    
    // 模拟协作：Yrs 的自动同步
    for op in 0..operations_per_user {
        for user in 0..user_count {
            let node_id = format!("node_{}_{}", user, op);
            let content = format!("Content from user {} op {}", user, op);
            
            // 在当前用户的文档中进行操作
            {
                let txn = docs[user].transact_mut();
                let text = txn.get_or_insert_text(&node_id);
                text.insert(&txn, 0, &content);
                txn.commit();
            }
            
            // 生成更新并应用到其他用户的文档
            let update = docs[user].encode_state_as_update_v1(&yrs::StateVector::default());
            
            for (other_user, other_doc) in docs.iter_mut().enumerate() {
                if other_user != user {
                    let mut txn = other_doc.transact_mut();
                    if let Ok(decoded_update) = yrs::Update::decode_v1(&update) {
                        txn.apply_update(decoded_update);
                    }
                    txn.commit();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_comparison() {
        benchmark_performance();
        collaboration_benchmark();
    }

    #[test]
    fn test_memory_efficiency() {
        // 测试结构分享的内存效率
        let mut base_map = ImHashMap::new();
        for i in 0..1000 {
            base_map.insert(format!("key_{}", i), format!("value_{}", i));
        }

        let start_time = Instant::now();
        let mut clones = Vec::new();
        
        // 创建1000个克隆（应该很快由于结构分享）
        for _ in 0..1000 {
            clones.push(base_map.clone());
        }
        
        let clone_time = start_time.elapsed();
        println!("1000个im::HashMap克隆耗时: {:?}", clone_time);
        
        // 修改一个克隆（应该只影响一个副本）
        clones[0].insert("new_key".to_string(), "new_value".to_string());
        
        // 验证其他克隆没有被影响
        assert!(!clones[1].contains_key("new_key"));
        
        println!("结构分享验证通过");
    }
}
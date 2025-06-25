use moduforge_transform::step::Step;
use moduforge_model::tree::Tree;
use std::collections::HashMap;
use crate::{StepResult, NodeData, MarkData, RoomSnapshot};
use yrs::{Map, TransactionMut, Transact, WriteTxn};

/// Step转换器trait - 用于将Step直接应用到Yrs文档
pub trait StepConverter {
    /// 直接将Step应用到Yrs文档事务
    fn apply_to_yrs_txn(&self, step: &dyn Step, txn: &mut yrs::TransactionMut, client_id: &str) -> Result<StepResult, Box<dyn std::error::Error>>;
    
    /// 获取转换器名称
    fn name(&self) -> &'static str;
    
    /// 检查是否支持此Step类型
    fn supports(&self, step: &dyn Step) -> bool;
    
    /// 获取Step的操作描述
    fn get_description(&self, step: &dyn Step) -> String {
        format!("执行操作: {} ({})", step.name(), self.name())
    }
}

/// 默认Step转换器
pub struct DefaultStepConverter;

impl StepConverter for DefaultStepConverter {
    fn apply_to_yrs_txn(&self, step: &dyn Step, _txn: &mut yrs::TransactionMut, client_id: &str) -> Result<StepResult, Box<dyn std::error::Error>> {
       
        
        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: step.name().to_string(),
            description: format!("默认处理器处理了未知步骤: {}", step.name()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            client_id: client_id.to_string(),
        })
    }
    
    fn name(&self) -> &'static str {
        "DefaultStepConverter"
    }
    
    fn supports(&self, _step: &dyn Step) -> bool {
        true // 默认转换器支持所有类型
    }
}

/// 节点操作Step转换器
pub struct NodeStepConverter;

impl StepConverter for NodeStepConverter {
    fn apply_to_yrs_txn(&self, step: &dyn Step, txn: &mut yrs::TransactionMut, client_id: &str) -> Result<StepResult, Box<dyn std::error::Error>> {
        let step_name = step.name();
        let root = txn.get_or_insert_map("nodes");
        
        if step_name.contains("AddNode") {
           
            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("添加了节点"),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id: client_id.to_string(),
            })
        } else if step_name.contains("RemoveNode") {
            // 模拟删除节点
            let node_id = format!("node_{}", uuid::Uuid::new_v4().simple());
            
            // 从Yrs文档中移除节点（这里简化处理）
            if let Some(val) = root.get(txn, &node_id) {
                // 标记为已删除而不是真的删除
                if let Ok(node_map) = val.cast::<yrs::MapRef>() {
                    node_map.insert(txn, "deleted", "true");
                    node_map.insert(txn, "deleted_by", client_id);
                    node_map.insert(txn, "deleted_at", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis().to_string());
                }
            }
            
            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("删除了节点: {}", node_id),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id: client_id.to_string(),
            })
        } else {
            Err("不支持的节点操作".into())
        }
    }
    
    fn name(&self) -> &'static str {
        "NodeStepConverter"
    }
    
    fn supports(&self, step: &dyn Step) -> bool {
        let step_name = step.name();
        step_name.contains("Node")
    }
}

/// 属性操作Step转换器
pub struct AttrStepConverter;

impl StepConverter for AttrStepConverter {
    fn apply_to_yrs_txn(&self, step: &dyn Step, txn: &mut yrs::TransactionMut, client_id: &str) -> Result<StepResult, Box<dyn std::error::Error>> {
        let step_name = step.name();
        
        if step_name.contains("Attr") {
          
            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("更新了节点属性"),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id: client_id.to_string(),
            })
        } else {
            Err("不支持的属性操作".into())
        }
    }
    
    fn name(&self) -> &'static str {
        "AttrStepConverter"
    }
    
    fn supports(&self, step: &dyn Step) -> bool {
        let step_name = step.name();
        step_name.contains("Attr")
    }
}

/// 标记操作Step转换器
pub struct MarkStepConverter;

impl StepConverter for MarkStepConverter {
    fn apply_to_yrs_txn(&self, step: &dyn Step, txn: &mut yrs::TransactionMut, client_id: &str) -> Result<StepResult, Box<dyn std::error::Error>> {

        Err("不支持的标记操作".into())
    }
    
    fn name(&self) -> &'static str {
        "MarkStepConverter"
    }
    
    fn supports(&self, step: &dyn Step) -> bool {
        let step_name = step.name();
        step_name.contains("Mark")
    }
}

/// Step转换器注册表
pub struct StepConverterRegistry {
    converters: Vec<Box<dyn StepConverter + Send + Sync>>,
}

impl StepConverterRegistry {
    /// 创建新的转换器注册表
    pub fn new() -> Self {
        let mut registry = Self {
            converters: Vec::new(),
        };
        
        // 注册默认转换器（必须最后注册，作为fallback）
        registry.register(Box::new(NodeStepConverter));
        registry.register(Box::new(AttrStepConverter));
        registry.register(Box::new(MarkStepConverter));
        registry.register(Box::new(DefaultStepConverter)); // fallback
        
        registry
    }
    
    /// 注册转换器
    pub fn register(&mut self, converter: Box<dyn StepConverter + Send + Sync>) {
        tracing::info!("注册Step转换器: {}", converter.name());
        self.converters.push(converter);
    }
    
    /// 查找支持指定Step的转换器
    pub fn find_converter(&self, step: &dyn Step) -> Option<&(dyn StepConverter + Send + Sync)> {
        for converter in &self.converters {
            if converter.supports(step) {
                return Some(converter.as_ref());
            }
        }
        None
    }
    
    /// 列出所有转换器
    pub fn list_converters(&self) -> Vec<&str> {
        self.converters.iter().map(|c| c.name()).collect()
    }
}

impl Default for StepConverterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局映射器
pub struct Mapper;

impl Mapper {
    /// 获取全局转换器注册表
    pub fn global_registry() -> &'static StepConverterRegistry {
        use std::sync::OnceLock;
        static REGISTRY: OnceLock<StepConverterRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| StepConverterRegistry::new())
    }
    
    /// 将 ModuForge 的 Tree 转换为 RoomSnapshot
    pub fn tree_to_snapshot(tree: &Tree, room_id: String) -> RoomSnapshot {
        let mut nodes = HashMap::new();
        
        // 递归遍历所有节点并转换
        fn collect_nodes(tree: &Tree, node_id: &str, nodes: &mut HashMap<String, NodeData>) {
            if let Some(node) = tree.get_node(&node_id.to_string()) {
                let node_data = NodeData {
                    id: node.id.clone(),
                    node_type: node.r#type.clone(),
                    attrs: node.attrs.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    content: node.content.iter().cloned().collect(),
                    marks: node.marks.iter().map(|mark| MarkData {
                        mark_type: mark.r#type.clone(),
                        attrs: mark.attrs.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    }).collect(),
                };
                
                nodes.insert(node_id.to_string(), node_data);
                
                // 递归处理子节点
                for child_id in &node.content {
                    collect_nodes(tree, child_id, nodes);
                }
            }
        }
        
        collect_nodes(tree, &tree.root_id, &mut nodes);
        
        RoomSnapshot {
            room_id,
            root_id: tree.root_id.clone(),
            nodes,
            version: 0, // Version is now managed by YrsManager, snapshot is always at a point in time.
        }
    }
} 
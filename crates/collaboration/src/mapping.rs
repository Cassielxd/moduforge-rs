use moduforge_transform::{attr_step::AttrStep, mark_step::{AddMarkStep, RemoveMarkStep}, node_step::{AddNodeStep, RemoveNodeStep}, step::Step};
use moduforge_model::{node_pool::NodePool, tree::Tree};
use std::{collections::HashMap, sync::Arc};
use crate::{StepResult, NodeData, MarkData, RoomSnapshot};
use yrs::{Map, Transact, MapPrelim, ArrayPrelim, Array, TransactionMut, WriteTxn};

/// Step转换器trait - 用于将Step直接应用到Yrs文档
pub trait StepConverter {
    /// 直接将Step应用到Yrs文档事务
    fn apply_to_yrs_txn(
        &self,
        doc: Arc<NodePool>,
        step: &dyn Step,
        txn: &mut yrs::TransactionMut,
        client_id: &str,
    ) -> Result<StepResult, Box<dyn std::error::Error>>;

    /// 获取转换器名称
    fn name(&self) -> &'static str;

    /// 检查是否支持此Step类型
    fn supports(
        &self,
        step: &dyn Step,
    ) -> bool;

    /// 获取Step的操作描述
    fn get_description(
        &self,
        step: &dyn Step,
    ) -> String {
        format!("执行操作: {} ({})", step.name(), self.name())
    }

    /// 添加标记到Yrs数组的公共方法
    fn add_mark_to_array(
        &self,
        marks_array: &yrs::ArrayRef,
        txn: &mut yrs::TransactionMut,
        mark: &moduforge_model::mark::Mark,
    ) {
        let mark_map = marks_array.push_back(txn, MapPrelim::default());
        mark_map.insert(txn, "type", mark.r#type.clone());
        let mark_attrs_map = mark_map.insert(txn, "attrs", MapPrelim::default());
        for (key, value) in mark.attrs.iter() {
            mark_attrs_map.insert(txn, key.clone(), value.to_string());
        }
    }
    
    /// 获取或创建标记数组的公共方法
    fn get_or_create_marks_array(
        &self,
        node_data_map: &yrs::MapRef,
        txn: &mut yrs::TransactionMut,
    ) -> yrs::ArrayRef {
        if let Some(yrs::Value::YArray(array)) = node_data_map.get(txn, "marks") {
            array
        } else {
            node_data_map.insert(txn, "marks", ArrayPrelim::default())
        }
    }

    fn get_or_create_node_data_map(
        &self,
        nodes_map: &yrs::MapRef,
        txn: &mut yrs::TransactionMut,
        node_id: &str,
    ) -> yrs::MapRef {
        if let Some(yrs::Value::YMap(map)) = nodes_map.get(txn, node_id) {
            map
        } else {
            nodes_map.insert(txn, node_id.to_string(), MapPrelim::default())
        }
    }
    fn get_or_create_node_attrs_map(
        &self,
        node_data_map: &yrs::MapRef,
        txn: &mut yrs::TransactionMut,
    ) -> yrs::MapRef {
        if let Some(yrs::Value::YMap(map)) = node_data_map.get(txn, "attrs") {
            map
        } else {
            node_data_map.insert(txn, "attrs", MapPrelim::default())
        }
    }
}

/// 默认Step转换器
pub struct DefaultStepConverter;

impl StepConverter for DefaultStepConverter {
    fn apply_to_yrs_txn(
        &self,
        doc: Arc<NodePool>,
        step: &dyn Step,
        _txn: &mut yrs::TransactionMut,
        client_id: &str,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
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

    fn supports(
        &self,
        _step: &dyn Step,
    ) -> bool {
        true // 默认转换器支持所有类型
    }
}

/// 节点操作Step转换器
pub struct NodeStepConverter;

impl StepConverter for NodeStepConverter {
    fn apply_to_yrs_txn(
        &self,
        doc: Arc<NodePool>,
        step: &dyn Step,
        txn: &mut yrs::TransactionMut,
        client_id: &str,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        if let Some(add_step) = step.downcast_ref::<AddNodeStep>() {
            let mut all_node_ids = Vec::new();
            for node_enum in &add_step.nodes {
                all_node_ids.extend(AddNodeStep::collect_node_ids(node_enum));
            }
            let nodes_map = txn.get_or_insert_map("nodes");

            for node_id in &all_node_ids {
                if let Some(node) = doc.get_node(node_id) {
                    //  1. 插入节点到 yrs 文档
                    let node_data_map = self.get_or_create_node_data_map(&nodes_map, txn, &node.id);
                    // 2. 填充节点基本信息
                    node_data_map.insert(txn, "type", node.r#type.clone());
                    // 3. 填充节点属性
                    let attrs_map = node_data_map.insert(
                        txn,
                        "attrs",
                        MapPrelim::default(),
                    );
                    for (key, value) in node.attrs.iter() {
                        attrs_map.insert(txn, key.clone(), value.to_string());
                    }
                    // 4. 填充节点内容
                    let content_array = node_data_map.insert(
                        txn,
                        "content",
                        ArrayPrelim::default(),
                    );
                    for child_id in &node.content {
                        content_array.push_back(txn, child_id.clone());
                    }
                    // 5. 填充节点标记
                    let marks_array = node_data_map.insert(
                        txn,
                        "marks",
                        ArrayPrelim::default(),
                    );
                    for mark in &node.marks {
                        self.add_mark_to_array(&marks_array, txn, mark);
                    }
                }
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("添加了 {} 个节点", all_node_ids.len()),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id: client_id.to_string(),
            });
        } else if step.name().contains("remove_node_step") {
            if let Some(remove_step) = step.downcast_ref::<RemoveNodeStep>() {
                let nodes_map = txn.get_or_insert_map("nodes");
                for node_id in &remove_step.node_ids {
                    nodes_map.remove(txn, node_id);
                }
                // 此处为简化后的占位逻辑
            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("删除了 {} 个节点", remove_step.node_ids.len()),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id: client_id.to_string(),
            });
            }
            
        }

        Err("不支持的节点操作".into())
    }

    fn name(&self) -> &'static str {
        "NodeStepConverter"
    }

    fn supports(
        &self,
        step: &dyn Step,
    ) -> bool {
        step.name().contains("add_node_step")
            || step.name().contains("remove_node_step")
    }
}

/// 属性操作Step转换器
pub struct AttrStepConverter;

impl StepConverter for AttrStepConverter {
    fn apply_to_yrs_txn(
        &self,
        _doc: Arc<NodePool>,
        step: &dyn Step,
        txn: &mut yrs::TransactionMut,
        client_id: &str,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        if let Some(attr_step) = step.downcast_ref::<AttrStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");

            let node_data_map = self.get_or_create_node_data_map(&nodes_map, txn, &attr_step.id);

            let attrs_map = self.get_or_create_node_attrs_map(&node_data_map, txn);

            for (key, value) in attr_step.values.iter() {
                if attrs_map.contains_key(txn, key) {
                    attrs_map.try_update(txn, key.clone(), value.to_string());
                } else {
                    attrs_map.insert(txn, key.clone(), value.to_string());
                }
            }

            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("更新了节点 {} 的 {} 个属性", attr_step.id, attr_step.values.len()),
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

    fn supports(
        &self,
        step: &dyn Step,
    ) -> bool {
        step.name().contains("attr_step")
    }
}

/// 标记操作Step转换器
pub struct MarkStepConverter;

impl MarkStepConverter {
    
}

impl StepConverter for MarkStepConverter {
    fn apply_to_yrs_txn(
        &self,
        doc: Arc<NodePool>,
        step: &dyn Step,
        txn: &mut yrs::TransactionMut,
        client_id: &str,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        if let Some(add_mark_step) = step.downcast_ref::<AddMarkStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let node = doc.get_node(&add_mark_step.id).unwrap();
            
            // 获取或创建节点数据Map
            let node_data_map = self.get_or_create_node_data_map(&nodes_map, txn, &node.id);
            
            // 获取或创建标记数组
            let marks_array = self.get_or_create_marks_array(&node_data_map, txn);
            // 清除现有标记
            marks_array.remove_range(txn, 0, marks_array.len(txn));
            // 添加新标记
            for mark in &node.marks {
                self.add_mark_to_array(&marks_array, txn, mark);
            }
            
            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("添加了 {} 个标记到节点 {}", add_mark_step.marks.len(), add_mark_step.id),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id: client_id.to_string(),
            });
        } else if let Some(remove_mark_step) = step.downcast_ref::<RemoveMarkStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let node = doc.get_node(&remove_mark_step.id).unwrap();
            
            let node_data_map = self.get_or_create_node_data_map(&nodes_map, txn, &remove_mark_step.id);
            
            // 获取标记数组
            let marks_array = self.get_or_create_marks_array(&node_data_map, txn);
            let marks = doc.get_node(&node.id).unwrap().marks.clone();
            
            // 清除现有标记
            marks_array.remove_range(txn, 0, marks_array.len(txn));
            // 重新添加不需要删除的标记
            for mark in marks {
                if !remove_mark_step.mark_types.contains(&mark.r#type) {
                    self.add_mark_to_array(&marks_array, txn, &mark);
                }
            }
            
            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("从节点 {} 删除了 {} 个标记", remove_mark_step.id, remove_mark_step.mark_types.len()),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id: client_id.to_string(),
            });
        }

        Err("不支持的标记操作".into())
    }

    fn name(&self) -> &'static str {
        "MarkStepConverter"
    }

    fn supports(
        &self,
        step: &dyn Step,
    ) -> bool {
        let step_name = step.name();
        step_name.contains("add_mark_step") || step_name.contains("remove_mark_step")
    }
}

/// Step转换器注册表
pub struct StepConverterRegistry {
    converters: Vec<Box<dyn StepConverter + Send + Sync>>,
}

impl StepConverterRegistry {
    /// 创建新的转换器注册表
    pub fn new() -> Self {
        let mut registry = Self { converters: Vec::new() };

        // 注册默认转换器（必须最后注册，作为fallback）
        registry.register(Box::new(NodeStepConverter));
        registry.register(Box::new(AttrStepConverter));
        registry.register(Box::new(MarkStepConverter));
        registry.register(Box::new(DefaultStepConverter)); // fallback

        registry
    }

    /// 注册转换器
    pub fn register(
        &mut self,
        converter: Box<dyn StepConverter + Send + Sync>,
    ) {
        tracing::info!("注册Step转换器: {}", converter.name());
        self.converters.push(converter);
    }

    /// 查找支持指定Step的转换器
    pub fn find_converter(
        &self,
        step: &dyn Step,
    ) -> Option<&(dyn StepConverter + Send + Sync)> {
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
    pub fn tree_to_snapshot(
        tree: &Tree,
        room_id: String,
    ) -> RoomSnapshot {
        let mut nodes = HashMap::new();

        // 递归遍历所有节点并转换
        fn collect_nodes(
            tree: &Tree,
            node_id: &str,
            nodes: &mut HashMap<String, NodeData>,
        ) {
            if let Some(node) = tree.get_node(&node_id.to_string()) {
                let node_data = NodeData {
                    id: node.id.clone(),
                    node_type: node.r#type.clone(),
                    attrs: node
                        .attrs
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                    content: node.content.iter().cloned().collect(),
                    marks: node
                        .marks
                        .iter()
                        .map(|mark| MarkData {
                            mark_type: mark.r#type.clone(),
                            attrs: mark
                                .attrs
                                .iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect(),
                        })
                        .collect(),
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

    /// 将 ModuForge 的 Tree 转换为一个临时的 yrs::Doc (用于同步)
    pub fn tree_to_yrs_doc(tree: &Tree) -> yrs::Doc {
        let doc = yrs::Doc::new();
        {
            // Scope the transaction so it's dropped before we return the doc
            let mut txn = doc.transact_mut();
            let nodes_map = txn.get_or_insert_map("nodes");

            // Use an iterative approach to avoid recursion issues and stack overflows
            let mut queue = std::collections::VecDeque::new();
            if !tree.root_id.is_empty() {
                queue.push_back(tree.root_id.clone());
            }

            let mut visited = std::collections::HashSet::new();
            if !tree.root_id.is_empty() {
                visited.insert(tree.root_id.clone());
            }

            while let Some(node_id) = queue.pop_front() {
                if let Some(node) = tree.get_node(&node_id) {
                    let node_data_map = nodes_map.insert(
                        &mut txn,
                        node.id.clone(),
                        MapPrelim::default(),
                    );

                    node_data_map.insert(&mut txn, "type", node.r#type.clone());

                    let attrs_map = node_data_map.insert(
                        &mut txn,
                        "attrs",
                        MapPrelim::default(),
                    );
                    for (key, value) in node.attrs.iter() {
                        attrs_map.insert(
                            &mut txn,
                            key.clone(),
                            value.to_string(),
                        );
                    }

                    let content_array = node_data_map.insert(
                        &mut txn,
                        "content",
                        ArrayPrelim::default(),
                    );
                    for child_id in &node.content {
                        content_array.push_back(&mut txn, child_id.clone());
                        if visited.insert(child_id.clone()) {
                            queue.push_back(child_id.clone());
                        }
                    }
                }
            }
        }
        doc
    }
}

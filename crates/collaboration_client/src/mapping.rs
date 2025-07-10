use mf_transform::{
    attr_step::AttrStep,
    mark_step::{AddMarkStep, RemoveMarkStep},
    node_step::{AddNodeStep, RemoveNodeStep},
    step::Step,
};
use mf_model::{node::Node, tree::Tree};
use std::{any::TypeId};

use crate::{types::StepResult, utils::Utils};
use yrs::{
    types::{array::ArrayRef, map::MapRef, Value},
    Array, ArrayPrelim, Map, MapPrelim, ReadTxn, Transact, TransactionMut,
    WriteTxn,
};

/// 将 `Step` 转换为 Yrs 事务的 Trait
/// 这个 Trait 是动态安全的
pub trait StepConverter: Send + Sync {
    /// 将步骤应用到 Yrs 文档事务中
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>>;
    /// 将 Yrs 变更转换成 ModuForge step
    fn apply_yrs_to_step(&self) {}

    /// 返回转换器的名称
    fn name(&self) -> &'static str;

    /// 检查此转换器是否支持给定的步骤类型
    fn supports(
        &self,
        step: &dyn Step,
    ) -> bool;

    /// 获取步骤的操作描述
    fn get_description(
        &self,
        step: &dyn Step,
    ) -> String {
        format!("Executing operation: {} ({})", step.name(), self.name())
    }
}

/// 默认的步骤转换器，用于不支持的步骤
pub struct DefaultStepConverter;

impl StepConverter for DefaultStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: step.name().to_string(),
            description: format!("默认处理程序处理未知步骤: {}", step.name()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            client_id: txn
                .origin()
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        })
    }

    fn name(&self) -> &'static str {
        "DefaultStepConverter"
    }

    fn supports(
        &self,
        _step: &dyn Step,
    ) -> bool {
        true // Default converter supports all types as a fallback.
    }
}

/// 节点相关步骤的转换器
pub struct NodeStepConverter;

impl NodeStepConverter {
    fn insert_node_data(
        &self,
        txn: &mut TransactionMut,
        nodes_map: &MapRef,
        node: &Node,
    ) {
        let node_data_map =
            Utils::get_or_create_node_data_map(nodes_map, txn, &node.id);
        // 插入节点类型
        node_data_map.insert(txn, "type", node.r#type.clone());
        // 插入节点属性
        let attrs_map =
            node_data_map.insert(txn, "attrs", MapPrelim::<yrs::Any>::new());
        for (key, value) in node.attrs.iter() {
            attrs_map.insert(
                txn,
                key.clone(),
                Utils::json_value_to_yrs_any(value),
            );
        }
        // 插入节点内容
        let content_array = node_data_map.insert(
            txn,
            "content",
            ArrayPrelim::from(Vec::<yrs::Any>::new()),
        );
        for child_id in &node.content {
            content_array
                .push_back(txn, yrs::Any::String(child_id.clone().into()));
        }
        // 插入节点标记
        let marks_array = node_data_map.insert(
            txn,
            "marks",
            ArrayPrelim::from(Vec::<yrs::Any>::new()),
        );
        for mark in &node.marks {
            Utils::add_mark_to_array(&marks_array, txn, mark);
        }
    }

    /// 递归插入节点及其所有子节点
    fn insert_node_enum_recursive(
        &self,
        txn: &mut TransactionMut,
        nodes_map: &MapRef,
        node_enum: &mf_model::node_type::NodeEnum,
    ) {
        // 插入当前节点
        self.insert_node_data(txn, nodes_map, &node_enum.0);

        // 递归插入所有子节点
        for child_enum in &node_enum.1 {
            self.insert_node_enum_recursive(txn, nodes_map, child_enum);
        }
    }
}

impl StepConverter for NodeStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        let client_id =
            txn.origin().as_ref().map(|s| s.to_string()).unwrap_or_default();

        if let Some(add_step) = step.downcast_ref::<AddNodeStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let all_nodes = &add_step.nodes;
            let parent_id = add_step.parent_id.clone();

            // 如果parent_id 不是根节点需要 修改 parent_id节点的 content 数组
            // 获取根节点 id 从 meta 区域
            let meta_map = txn.get_or_insert_map("meta");
            let root_id = meta_map.get(txn, "root_id");
            if let Some(root_id) = root_id
                && root_id.to_string(txn) != parent_id
            {
                let node_data_map = Utils::get_or_create_node_data_map(
                    &nodes_map, txn, &parent_id,
                );
                let content_array =
                    Utils::get_or_create_content_array(&node_data_map, txn);
                for node_enum in all_nodes {
                    content_array.push_back(
                        txn,
                        yrs::Any::String(node_enum.0.id.clone().into()),
                    );
                }
            }

            for node_enum in all_nodes {
                self.insert_node_enum_recursive(txn, &nodes_map, node_enum);
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("添加 {} 个节点", all_nodes.len()),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            });
        } else if let Some(remove_step) = step.downcast_ref::<RemoveNodeStep>()
        {
            let nodes_map = txn.get_or_insert_map("nodes");
            for node_id in &remove_step.node_ids {
                nodes_map.remove(txn, &node_id.to_string());
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!(
                    "Removed {} nodes",
                    remove_step.node_ids.len()
                ),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            });
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
        step.type_id() == TypeId::of::<AddNodeStep>()
            || step.type_id() == TypeId::of::<RemoveNodeStep>()
    }
}

/// 属性相关步骤的转换器
pub struct AttrStepConverter;

impl StepConverter for AttrStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        let client_id =
            txn.origin().as_ref().map(|s| s.to_string()).unwrap_or_default();
        if let Some(attr_step) = step.downcast_ref::<AttrStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let node_data_map = Utils::get_or_create_node_data_map(
                &nodes_map,
                txn,
                &attr_step.id,
            );
            // 获取或创建节点属性映射
            let attrs_map =
                Utils::get_or_create_node_attrs_map(&node_data_map, txn);
            // 更新节点属性
            for (key, value) in attr_step.values.iter() {
                attrs_map.insert(
                    txn,
                    key.clone(),
                    Utils::json_value_to_yrs_any(value),
                );
            }

            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!(
                    "更新 {} 个属性 for node {}",
                    attr_step.values.len(),
                    attr_step.id
                ),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
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
        step.type_id() == TypeId::of::<AttrStep>()
    }
}

/// 标记相关步骤的转换器
pub struct MarkStepConverter;

impl MarkStepConverter {
    /// 从 Yrs 数组中删除标记
    fn remove_mark_from_array(
        &self,
        marks_array: &ArrayRef,
        txn: &mut TransactionMut,
        mark_type_to_remove: &str,
    ) {
        let len = marks_array.len(txn);
        for i in (0..len).rev() {
            if let Some(Value::YMap(mark_map)) = marks_array.get(txn, i) {
                if let Some(mark_type_value) = mark_map.get(txn, "type") {
                    let mark_type = match mark_type_value {
                        Value::YText(_text_ref) => {
                            // For now, skip text refs as we can't easily extract string
                            continue;
                        },
                        Value::Any(any) => any.to_string(),
                        _ => continue,
                    };
                    if mark_type == mark_type_to_remove {
                        marks_array.remove(txn, i);
                        return;
                    }
                }
            }
        }
    }
}

impl StepConverter for MarkStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        let client_id =
            txn.origin().as_ref().map(|s| s.to_string()).unwrap_or_default();

        if let Some(add_mark_step) = step.downcast_ref::<AddMarkStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            // 获取或创建节点数据映射
            let node_data_map = Utils::get_or_create_node_data_map(
                &nodes_map,
                txn,
                &add_mark_step.id,
            );
            // 获取或创建标记数组
            let marks_array =
                Utils::get_or_create_marks_array(&node_data_map, txn);
            // 添加标记
            for mark in &add_mark_step.marks {
                Utils::add_mark_to_array(&marks_array, txn, mark);
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!(
                    "添加 {} 个标记 to node {}",
                    add_mark_step.marks.len(),
                    add_mark_step.id
                ),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            });
        } else if let Some(remove_mark_step) =
            step.downcast_ref::<RemoveMarkStep>()
        {
            let nodes_map = txn.get_or_insert_map("nodes");
            let node_data_map = Utils::get_or_create_node_data_map(
                &nodes_map,
                txn,
                &remove_mark_step.id,
            );
            let marks_array =
                Utils::get_or_create_marks_array(&node_data_map, txn);
            // 删除标记
            for mark_type in &remove_mark_step.mark_types {
                self.remove_mark_from_array(&marks_array, txn, mark_type);
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!(
                    "Removed {} marks from node {}",
                    remove_mark_step.mark_types.len(),
                    remove_mark_step.id
                ),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
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
        step.type_id() == TypeId::of::<AddMarkStep>()
            || step.type_id() == TypeId::of::<RemoveMarkStep>()
    }
}

/// 所有步骤转换器的注册表
pub struct StepConverterRegistry {
    converters: Vec<Box<dyn StepConverter>>,
}

impl Default for StepConverterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StepConverterRegistry {
    /// 创建一个新的注册表，包含所有默认的转换器
    pub fn new() -> Self {
        let mut registry = Self { converters: Vec::new() };

        registry.register(Box::new(NodeStepConverter));
        registry.register(Box::new(AttrStepConverter));
        registry.register(Box::new(MarkStepConverter));
        registry.register(Box::new(DefaultStepConverter));

        registry
    }

    /// 注册一个新的转换器
    pub fn register(
        &mut self,
        converter: Box<dyn StepConverter>,
    ) {
        tracing::info!("🔄 注册步骤转换器: {}", converter.name());
        self.converters.push(converter);
    }

    /// 查找支持给定步骤的转换器
    pub fn find_converter(
        &self,
        step: &dyn Step,
    ) -> Option<&(dyn StepConverter)> {
        for converter in &self.converters {
            if converter.supports(step) {
                return Some(converter.as_ref());
            }
        }
        None
    }
}

/// 全局映射器，用于处理转换
#[derive(Debug)]
pub struct Mapper;

impl Mapper {
    /// Gets the global singleton instance of the converter registry.
    pub fn global_registry() -> &'static StepConverterRegistry {
        use std::sync::OnceLock;
        static REGISTRY: OnceLock<StepConverterRegistry> = OnceLock::new();
        REGISTRY.get_or_init(StepConverterRegistry::new)
    }

    /// 将 Yrs 文档转换为 ModuForge Tree
    /// 这是从协作状态重建文档树的关键方法
    pub fn yrs_doc_to_tree(
        doc: &yrs::Doc
    ) -> Result<Tree, Box<dyn std::error::Error>> {
        use mf_model::types::NodeId;
        use std::collections::HashMap;

        let root_id = Utils::get_root_id_from_yrs_doc(doc)?;
        let txn = doc.transact();
        let nodes_map =
            txn.get_map("nodes").ok_or("Yrs 文档中没有找到 nodes 映射")?;
        let mut tree_nodes = HashMap::new();
        let mut parent_map = HashMap::new();

        Utils::build_tree_nodes_from_yrs(
            &root_id,
            &nodes_map,
            &txn,
            &mut tree_nodes,
            &mut parent_map,
            None,
        )?;

        let root_node = tree_nodes
            .get(&NodeId::from(root_id))
            .ok_or("根节点不存在")?
            .as_ref()
            .clone();
        let root_enum =
            Utils::build_node_enum_from_map(&root_node, &tree_nodes);
        Ok(Tree::from(root_enum))
    }

    /// 将 Tree 直接写入 Yrs 文档
    /// 用于初始化协作文档
    pub fn tree_to_yrs_doc(
        tree: &Tree,
        doc: &yrs::Doc,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut txn = doc.transact_mut();
        let nodes_map = txn.get_or_insert_map("nodes");

        // 清空现有数据
        nodes_map.clear(&mut txn);

        // 使用现有的转换器系统
        let registry = Self::global_registry();

        // 获取根节点的所有子树
        if let Some(root_tree) = tree.all_children(&tree.root_id, None) {
            use mf_transform::{step::Step, node_step::AddNodeStep};

            // 创建一个 AddNodeStep 来添加整个子树
            let add_step = AddNodeStep {
                parent_id: tree.root_id.clone(),
                nodes: vec![root_tree],
            };

            // 使用现有的转换器应用步骤
            if let Some(converter) =
                registry.find_converter(&add_step as &dyn Step)
            {
                if let Err(e) =
                    converter.apply_to_yrs_txn(&add_step as &dyn Step, &mut txn)
                {
                    return Err(
                        format!("Failed to sync tree to Yrs: {}", e).into()
                    );
                }
            } else {
                return Err("No converter found for AddNodeStep".into());
            }
        }

        Ok(())
    }

    /// 获取 Yrs 文档的版本信息
    pub fn get_yrs_doc_version(doc: &yrs::Doc) -> u64 {
        let txn = doc.transact();
        txn.state_vector().len() as u64
    }

    /// 检查 Yrs 文档是否为空
    pub fn is_yrs_doc_empty(doc: &yrs::Doc) -> bool {
        let txn = doc.transact();
        let nodes_map = txn.get_map("nodes");
        nodes_map.map_or(true, |map| map.len(&txn) == 0)
    }
}

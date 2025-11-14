/// 简化版转换器实现
/// 不使用宏，直接实现 TypedStepConverter trait
use yrs::{Array, ArrayPrelim, Map, MapPrelim, TransactionMut, WriteTxn};
use mf_transform::{
    step::Step,
    node_step::{AddNodeStep, RemoveNodeStep},
    attr_step::AttrStep,
    mark_step::{AddMarkStep, RemoveMarkStep},
};
use mf_model::node::Node;

use crate::mapping_v2::{
    typed_converter::{TypedStepConverter, ConversionContext},
    error::{ConversionError, ConversionResult},
};
use crate::{types::StepResult, utils::Utils};

// ================================
// 节点添加转换器
// ================================

#[derive(Debug, Default, Clone)]
pub struct SimpleNodeAddConverter;

impl TypedStepConverter<AddNodeStep> for SimpleNodeAddConverter {
    fn convert_typed(
        &self,
        step: &AddNodeStep,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult> {
        // 验证步骤
        if step.nodes.is_empty() {
            return Err(ConversionError::validation_failed(
                "AddNodeStep",
                "节点列表不能为空",
            ));
        }

        let nodes_map = txn.get_or_insert_map("nodes");

        // 获取根节点ID
        let meta_map = txn.get_or_insert_map("meta");
        let root_id = meta_map.get(txn, "root_id");

        // 如果不是根节点，需要更新父节点的content数组
        if let Some(root_id) = root_id {
            if root_id.to_string(txn) != *step.parent_id {
                let parent_node_data = Utils::get_or_create_node_data_map(
                    &nodes_map,
                    txn,
                    &step.parent_id,
                );
                let content_array =
                    Utils::get_or_create_content_array(&parent_node_data, txn);

                for node_enum in &step.nodes {
                    content_array.push_back(
                        txn,
                        yrs::Any::String(node_enum.0.id.clone().into()),
                    );
                }
            }
        }

        // 插入所有节点
        let mut inserted_count = 0;
        for node_enum in &step.nodes {
            insert_node_recursive(&nodes_map, txn, node_enum)?;
            inserted_count += 1;
        }

        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: step.name().to_string(),
            description: format!(
                "成功添加 {} 个节点到父节点 {}",
                inserted_count, step.parent_id
            ),
            timestamp: context.timestamp,
            client_id: context.client_id.clone(),
        })
    }

    fn validate_step(
        &self,
        step: &AddNodeStep,
        _context: &ConversionContext,
    ) -> ConversionResult<()> {
        if step.nodes.is_empty() {
            return Err(ConversionError::validation_failed(
                "AddNodeStep",
                "节点列表不能为空",
            ));
        }

        Ok(())
    }

    fn converter_name() -> &'static str {
        "SimpleNodeAddConverter"
    }

    fn step_type_name() -> &'static str {
        "AddNodeStep"
    }

    fn priority() -> u8 {
        10
    }

    fn supports_concurrent_execution() -> bool {
        true
    }
}

// ================================
// 节点删除转换器
// ================================

#[derive(Debug, Default, Clone)]
pub struct SimpleNodeRemoveConverter;

impl TypedStepConverter<RemoveNodeStep> for SimpleNodeRemoveConverter {
    fn convert_typed(
        &self,
        step: &RemoveNodeStep,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult> {
        let nodes_map = txn.get_or_insert_map("nodes");
        let mut removed_count = 0;

        for node_id in &step.node_ids {
            if nodes_map.get(txn, node_id).is_some() {
                nodes_map.remove(txn, node_id);
                removed_count += 1;
            }
        }

        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: step.name().to_string(),
            description: format!("成功删除 {removed_count} 个节点"),
            timestamp: context.timestamp,
            client_id: context.client_id.clone(),
        })
    }

    fn validate_step(
        &self,
        step: &RemoveNodeStep,
        _context: &ConversionContext,
    ) -> ConversionResult<()> {
        if step.node_ids.is_empty() {
            return Err(ConversionError::validation_failed(
                "RemoveNodeStep",
                "节点ID列表不能为空",
            ));
        }

        Ok(())
    }

    fn converter_name() -> &'static str {
        "SimpleNodeRemoveConverter"
    }

    fn step_type_name() -> &'static str {
        "RemoveNodeStep"
    }

    fn priority() -> u8 {
        10
    }

    fn supports_concurrent_execution() -> bool {
        true
    }
}

// ================================
// 属性转换器
// ================================

#[derive(Debug, Default, Clone)]
pub struct SimpleAttrConverter;

impl TypedStepConverter<AttrStep> for SimpleAttrConverter {
    fn convert_typed(
        &self,
        step: &AttrStep,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult> {
        if step.values.is_empty() {
            return Err(ConversionError::validation_failed(
                "AttrStep",
                "属性值不能为空",
            ));
        }

        let nodes_map = txn.get_or_insert_map("nodes");
        let node_data_map =
            Utils::get_or_create_node_data_map(&nodes_map, txn, &step.id);
        let attrs_map =
            Utils::get_or_create_node_attrs_map(&node_data_map, txn);

        for (key, value) in &step.values {
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
                "更新节点 {} 的 {} 个属性",
                step.id,
                step.values.keys().len()
            ),
            timestamp: context.timestamp,
            client_id: context.client_id.clone(),
        })
    }

    fn validate_step(
        &self,
        step: &AttrStep,
        _context: &ConversionContext,
    ) -> ConversionResult<()> {
        if step.values.is_empty() {
            return Err(ConversionError::validation_failed(
                "AttrStep",
                "属性值不能为空",
            ));
        }

        Ok(())
    }

    fn converter_name() -> &'static str {
        "SimpleAttrConverter"
    }

    fn step_type_name() -> &'static str {
        "AttrStep"
    }

    fn priority() -> u8 {
        10
    }

    fn supports_concurrent_execution() -> bool {
        true
    }
}

// ================================
// 标记添加转换器
// ================================

#[derive(Debug, Default, Clone)]
pub struct SimpleMarkAddConverter;

impl TypedStepConverter<AddMarkStep> for SimpleMarkAddConverter {
    fn convert_typed(
        &self,
        step: &AddMarkStep,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult> {
        let nodes_map = txn.get_or_insert_map("nodes");
        let node_data_map =
            Utils::get_or_create_node_data_map(&nodes_map, txn, &step.id);
        let marks_array = Utils::get_or_create_marks_array(&node_data_map, txn);

        for mark in &step.marks {
            Utils::add_mark_to_array(&marks_array, txn, mark);
        }

        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: step.name().to_string(),
            description: format!(
                "为节点 {} 添加 {} 个标记",
                step.id,
                step.marks.len()
            ),
            timestamp: context.timestamp,
            client_id: context.client_id.clone(),
        })
    }

    fn validate_step(
        &self,
        step: &AddMarkStep,
        _context: &ConversionContext,
    ) -> ConversionResult<()> {
        if step.marks.is_empty() {
            return Err(ConversionError::validation_failed(
                "AddMarkStep",
                "标记列表不能为空",
            ));
        }

        Ok(())
    }

    fn converter_name() -> &'static str {
        "SimpleMarkAddConverter"
    }

    fn step_type_name() -> &'static str {
        "AddMarkStep"
    }

    fn priority() -> u8 {
        10
    }

    fn supports_concurrent_execution() -> bool {
        true
    }
}

// ================================
// 标记删除转换器
// ================================

#[derive(Debug, Default, Clone)]
pub struct SimpleMarkRemoveConverter;

impl TypedStepConverter<RemoveMarkStep> for SimpleMarkRemoveConverter {
    fn convert_typed(
        &self,
        step: &RemoveMarkStep,
        txn: &mut TransactionMut,
        context: &ConversionContext,
    ) -> ConversionResult<StepResult> {
        let nodes_map = txn.get_or_insert_map("nodes");
        let node_data_map =
            Utils::get_or_create_node_data_map(&nodes_map, txn, &step.id);
        let marks_array = Utils::get_or_create_marks_array(&node_data_map, txn);

        let mut removed_count = 0;
        for mark_type in &step.mark_types {
            if remove_mark_by_type(&marks_array, txn, mark_type)? {
                removed_count += 1;
            }
        }

        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: step.name().to_string(),
            description: format!(
                "从节点 {} 删除 {} 个标记",
                step.id, removed_count
            ),
            timestamp: context.timestamp,
            client_id: context.client_id.clone(),
        })
    }

    fn validate_step(
        &self,
        step: &RemoveMarkStep,
        _context: &ConversionContext,
    ) -> ConversionResult<()> {
        if step.mark_types.is_empty() {
            return Err(ConversionError::validation_failed(
                "RemoveMarkStep",
                "标记类型列表不能为空",
            ));
        }

        Ok(())
    }

    fn converter_name() -> &'static str {
        "SimpleMarkRemoveConverter"
    }

    fn step_type_name() -> &'static str {
        "RemoveMarkStep"
    }

    fn priority() -> u8 {
        10
    }

    fn supports_concurrent_execution() -> bool {
        true
    }
}

// ================================
// 辅助函数
// ================================

/// 递归插入节点
fn insert_node_recursive(
    nodes_map: &yrs::types::map::MapRef,
    txn: &mut TransactionMut,
    node_enum: &mf_model::node_definition::NodeTree,
) -> ConversionResult<()> {
    // 插入当前节点
    insert_single_node(nodes_map, txn, &node_enum.0)?;

    // 递归插入子节点
    for child_enum in &node_enum.1 {
        insert_node_recursive(nodes_map, txn, child_enum)?;
    }

    Ok(())
}

/// 插入单个节点
fn insert_single_node(
    nodes_map: &yrs::types::map::MapRef,
    txn: &mut TransactionMut,
    node: &Node,
) -> ConversionResult<()> {
    let node_data_map =
        Utils::get_or_create_node_data_map(nodes_map, txn, &node.id);

    // 设置节点类型
    node_data_map.insert(txn, "type", node.r#type.clone());

    // 设置属性
    let attrs_map =
        node_data_map.insert(txn, "attrs", MapPrelim::<yrs::Any>::new());
    for (key, value) in node.attrs.iter() {
        attrs_map.insert(txn, key.clone(), Utils::json_value_to_yrs_any(value));
    }

    // 设置内容
    let content_array = node_data_map.insert(
        txn,
        "content",
        ArrayPrelim::from(Vec::<yrs::Any>::new()),
    );
    for child_id in &node.content {
        content_array.push_back(txn, yrs::Any::String(child_id.clone().into()));
    }

    // 设置标记
    let marks_array = node_data_map.insert(
        txn,
        "marks",
        ArrayPrelim::from(Vec::<yrs::Any>::new()),
    );
    for mark in &node.marks {
        Utils::add_mark_to_array(&marks_array, txn, mark);
    }

    Ok(())
}

/// 按类型删除标记
fn remove_mark_by_type(
    marks_array: &yrs::types::array::ArrayRef,
    txn: &mut TransactionMut,
    mark_type: &str,
) -> ConversionResult<bool> {
    let len = marks_array.len(txn);

    for i in (0..len).rev() {
        if let Some(yrs::types::Value::YMap(mark_map)) = marks_array.get(txn, i)
        {
            if let Some(type_value) = mark_map.get(txn, "type") {
                let current_mark_type = match type_value {
                    yrs::types::Value::Any(any) => any.to_string(),
                    _ => continue,
                };

                if current_mark_type == mark_type {
                    marks_array.remove(txn, i);
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

// 自动注册所有转换器
#[ctor::ctor]
fn register_simple_converters() {
    use crate::mapping_v2::converter_registry::register_global_converter;

    register_global_converter::<AddNodeStep, SimpleNodeAddConverter>();
    register_global_converter::<RemoveNodeStep, SimpleNodeRemoveConverter>();
    register_global_converter::<AttrStep, SimpleAttrConverter>();
    register_global_converter::<AddMarkStep, SimpleMarkAddConverter>();
    register_global_converter::<RemoveMarkStep, SimpleMarkRemoveConverter>();

    tracing::info!("✅ 已注册所有简化版转换器");
}

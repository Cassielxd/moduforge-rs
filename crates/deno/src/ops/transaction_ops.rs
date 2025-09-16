use deno_core::op2;
use deno_core::OpState;
use mf_state::{State, transaction::Transaction};
use mf_model::types::NodeId;
use mf_model::node_type::NodeEnum;
use mf_model::mark::Mark;
use serde_json::Value;
use std::sync::Arc;
use crate::runtime::context::{ModuForgeContext, get_context_from_opstate};

/// 创建新的事务
#[op2]
#[smi]
pub fn op_transaction_new(state: &mut OpState) -> Result<u32, String> {
    let mut context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let transaction_id = context.create_transaction();
    Ok(transaction_id)
}

/// 设置节点属性
#[op2]
pub fn op_transaction_set_node_attribute(
    state: &mut OpState,
    #[smi] transaction_id: u32,
    #[smi] node_id: u32,
    #[string] attributes_json: String,
) -> Result<(), String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    // 解析属性 JSON
    let attributes: std::collections::HashMap<String, Value> =
        serde_json::from_str(&attributes_json)
            .map_err(|e| format!("Invalid attributes JSON: {}", e))?;

    let mut attr_map = imbl::HashMap::new();
    for (key, value) in attributes {
        attr_map.insert(key, value);
    }

    // 从上下文获取事务并执行操作
    if let Some(mut transaction_ref) = context.get_transaction_mut(transaction_id) {
        transaction_ref.set_node_attribute(NodeId::new(node_id as u64), attr_map)
            .map_err(|e| format!("Failed to set node attribute: {}", e))?;
        Ok(())
    } else {
        Err(format!("Transaction {} not found", transaction_id))
    }
}

/// 添加子节点
#[op2]
pub fn op_transaction_add_node(
    #[smi] transaction_id: u32,
    #[smi] parent_id: u32,
    #[string] nodes_json: String,
) -> Result<(), String> {
    // 解析节点 JSON
    let node_data: Vec<serde_json::Value> =
        serde_json::from_str(&nodes_json)
            .map_err(|e| format!("Invalid nodes JSON: {}", e))?;

    // 转换为 NodeEnum 类型需要更复杂的逻辑
    // 这里简化处理

    Ok(())
}

/// 删除节点
#[op2]
pub fn op_transaction_remove_node(
    #[smi] transaction_id: u32,
    #[smi] parent_id: u32,
    #[string] node_ids_json: String,
) -> Result<(), String> {
    let node_ids: Vec<u32> =
        serde_json::from_str(&node_ids_json)
            .map_err(|e| format!("Invalid node IDs JSON: {}", e))?;

    let node_id_vec: Vec<NodeId> = node_ids.into_iter()
        .map(|id| NodeId::new(id as u64))
        .collect();

    // 从 OpState 获取事务并执行删除
    // transaction.remove_node(NodeId::new(parent_id as u64), node_id_vec)?;

    Ok(())
}

/// 添加标记
#[op2]
pub fn op_transaction_add_mark(
    #[smi] transaction_id: u32,
    #[smi] node_id: u32,
    #[string] marks_json: String,
) -> Result<(), String> {
    // 解析标记 JSON 并创建 Mark 对象
    let mark_data: Vec<serde_json::Value> =
        serde_json::from_str(&marks_json)
            .map_err(|e| format!("Invalid marks JSON: {}", e))?;

    // 转换为 Mark 类型
    let marks: Result<Vec<Mark>, String> = mark_data.into_iter()
        .map(|data| {
            Mark::from_json(data)
                .map_err(|e| format!("Invalid mark data: {}", e))
        })
        .collect();

    let marks = marks?;

    // 从 OpState 获取事务并添加标记
    // transaction.add_mark(NodeId::new(node_id as u64), marks)?;

    Ok(())
}

/// 删除标记
#[op2]
pub fn op_transaction_remove_mark(
    #[smi] transaction_id: u32,
    #[smi] node_id: u32,
    #[string] mark_types_json: String,
) -> Result<(), String> {
    let mark_types: Vec<String> =
        serde_json::from_str(&mark_types_json)
            .map_err(|e| format!("Invalid mark types JSON: {}", e))?;

    // 从 OpState 获取事务并删除标记
    // transaction.remove_mark(NodeId::new(node_id as u64), mark_types)?;

    Ok(())
}

/// 设置事务元数据
#[op2]
pub fn op_transaction_set_meta(
    #[smi] transaction_id: u32,
    #[string] key: String,
    #[string] value_json: String,
) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(&value_json)
            .map_err(|e| format!("Invalid value JSON: {}", e))?;

    // 从 OpState 获取事务并设置元数据
    // transaction.set_meta(key, value);

    Ok(())
}

/// 获取事务元数据
#[op2]
#[string]
pub fn op_transaction_get_meta(
    #[smi] transaction_id: u32,
    #[string] key: String,
) -> Option<String> {
    // 从 OpState 获取事务并获取元数据
    // transaction.get_meta::<serde_json::Value>(&key)
    //     .and_then(|value| serde_json::to_string(&value).ok())
    None
}
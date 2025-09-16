use deno_core::op2;
use deno_core::OpState;
use mf_state::State;
use mf_model::types::NodeId;
use crate::runtime::context::{ModuForgeContext, get_context_from_opstate};

/// 获取节点属性
#[op2]
#[string]
pub fn op_node_get_attribute(
    state: &mut OpState,
    #[smi] node_id: u32,
    #[string] attr_name: String,
) -> Result<Option<String>, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let node_id = NodeId::new(node_id as u64);
    let doc = context.current_state.doc();

    let result = doc.get_node(&node_id)
        .and_then(|node| {
            node.attrs.get(&attr_name)
                .and_then(|value| serde_json::to_string(value).ok())
        });

    Ok(result)
}

/// 获取节点的所有子节点 ID
#[op2]
#[string]
pub fn op_node_get_children(
    state: &mut OpState,
    #[smi] node_id: u32,
) -> Result<String, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let node_id = NodeId::new(node_id as u64);
    let doc = context.current_state.doc();

    let children_ids: Vec<u32> = doc.get_node(&node_id)
        .map(|node| {
            node.content.iter()
                .map(|child| child.id().value() as u32)
                .collect()
        })
        .unwrap_or_default();

    Ok(serde_json::to_string(&children_ids).unwrap_or_else(|_| "[]".to_string()))
}

/// 获取节点的父节点 ID
#[op2(fast)]
#[smi]
pub fn op_node_get_parent(
    state: &mut OpState,
    #[smi] node_id: u32,
) -> Result<Option<u32>, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let node_id = NodeId::new(node_id as u64);
    let doc = context.current_state.doc();

    let result = doc.get_parent(&node_id)
        .map(|parent_id| parent_id.value() as u32);

    Ok(result)
}

/// 通过 ID 查找节点是否存在
#[op2(fast)]
pub fn op_node_find_by_id(
    state: &mut OpState,
    #[smi] node_id: u32,
) -> Result<bool, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let node_id = NodeId::new(node_id as u64);
    let doc = context.current_state.doc();

    Ok(doc.get_node(&node_id).is_some())
}

/// 获取节点的基本信息
#[op2]
#[string]
pub fn op_node_get_info(
    state: &mut OpState,
    #[smi] node_id: u32,
) -> Result<Option<String>, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let node_id = NodeId::new(node_id as u64);
    let doc = context.current_state.doc();

    let result = doc.get_node(&node_id)
        .and_then(|node| {
            let info = serde_json::json!({
                "id": node_id.value(),
                "type": node.node_type,
                "attrs": node.attrs,
                "marks": node.marks,
                "childrenCount": node.content.len()
            });
            serde_json::to_string(&info).ok()
        });

    Ok(result)
}
use deno_core::op2;
use deno_core::OpState;
use mf_state::{State, resource::Resource};
use mf_model::types::NodeId;
use std::sync::Arc;
use crate::runtime::context::{ModuForgeContext, get_context_from_opstate};

/// 获取状态版本号
#[op2(fast)]
#[smi]
pub fn op_state_get_version(state: &mut OpState) -> Result<u32, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    Ok(context.current_state.version as u32)
}

/// 检查状态字段是否存在
#[op2(fast)]
pub fn op_state_has_field(
    state: &mut OpState,
    #[string] field_name: &str,
) -> Result<bool, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    Ok(context.current_state.has_field(field_name))
}

/// 获取状态字段（返回序列化的 JSON）
#[op2]
#[string]
pub fn op_state_get_field(
    state: &mut OpState,
    #[string] field_name: &str,
) -> Result<Option<String>, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let result = context.current_state.get_field(field_name)
        .and_then(|resource| {
            // 尝试将 Resource 序列化为 JSON
            // 这里需要 Resource trait 支持序列化
            serde_json::to_string(&**resource).ok()
        });

    Ok(result)
}

/// 获取当前文档的根节点 ID
#[op2(fast)]
#[smi]
pub fn op_state_get_doc(state: &mut OpState) -> Result<u32, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    // 获取文档根节点的 ID
    Ok(context.current_state.doc().root_id().value() as u32)
}

/// 获取 Schema 信息（简化版）
#[op2]
#[string]
pub fn op_state_get_schema(state: &mut OpState) -> Result<String, String> {
    let context = get_context_from_opstate(state)
        .map_err(|e| format!("Failed to get context: {}", e))?;

    let schema = context.current_state.schema();
    let result = serde_json::json!({
        "name": schema.name,
        "version": schema.version,
        "nodeTypes": schema.node_types.keys().collect::<Vec<_>>(),
        "markTypes": schema.mark_types.keys().collect::<Vec<_>>()
    }).to_string();

    Ok(result)
}
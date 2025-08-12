use mf_model::{mark::Mark, node::Node, node_type::NodeEnum, node_pool::NodePool, types::NodeId};
use std::sync::Arc;

/// 扁平化后的索引文档（写入后端的基础结构）
#[derive(Debug, Clone)]
pub struct IndexDoc {
    pub node_id: String,
    pub node_type: String,
    pub parent_id: Option<String>,
    pub marks: Vec<String>,
    pub attrs_flat: Vec<(String, String)>,
    pub text: Option<String>,
    pub path: Vec<String>,
    // 常用 fast fields（i64）
    pub order_i64: Option<i64>,
    pub created_at_i64: Option<i64>,
    pub updated_at_i64: Option<i64>,
}

impl IndexDoc {
    /// 从节点与池快照构建索引文档
    pub fn from_node(pool: &NodePool, node: &Arc<Node>) -> Self {
        let parent_id = pool.parent_id(&node.id).cloned();
        let marks = node.marks.iter().map(|m: &Mark| m.r#type.clone()).collect();
        let mut attrs_flat = Vec::with_capacity(node.attrs.attrs.len());
        for (k, v) in node.attrs.attrs.iter() {
            attrs_flat.push((k.clone(), flatten_value(v)));
        }

        // 路径（根到当前）
        let path: Vec<String> = pool.get_node_path(&node.id).into_iter().map(|id| id.to_string()).collect();

        let text = extract_text(node);

        // 提取常用 fast fields（若存在且为数值）
        let order_i64 = extract_i64(node, "order");
        let created_at_i64 = extract_i64(node, "created_at");
        let updated_at_i64 = extract_i64(node, "updated_at");

        IndexDoc {
            node_id: node.id.to_string(),
            node_type: node.r#type.clone(),
            parent_id: parent_id.map(|id| id.to_string()),
            marks,
            attrs_flat,
            text,
            path,
            order_i64,
            created_at_i64,
            updated_at_i64,
        }
    }
}

/// 将属性值转为扁平字符串（便于倒排过滤）
fn flatten_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => serde_json::to_string(v).unwrap_or_default(),
    }
}

/// 提取用于全文字段的文本（约定: 优先 text/title/content）
fn extract_text(node: &Node) -> Option<String> {
    for key in ["text", "title", "content"] {
        if let Some(serde_json::Value::String(s)) = node.attrs.get(key) {
            if !s.is_empty() {
                return Some(s.clone());
            }
        }
    }
    None
}

/// 从节点属性中提取 i64 数值（仅当为数值或可解析为整数的字符串时）
fn extract_i64(node: &Node, key: &str) -> Option<i64> {
    node.attrs.get(key).and_then(|v| match v {
        serde_json::Value::Number(n) => n.as_i64().or_else(|| n.as_u64().and_then(|u| i64::try_from(u).ok())),
        serde_json::Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    })
}

/// 收集 NodeEnum 中所有节点 id（包含子树）
pub fn collect_node_ids_from_enum(node_enum: &NodeEnum) -> Vec<NodeId> {
    let mut ids: Vec<NodeId> = vec![node_enum.0.id.clone()];
    for child in &node_enum.1 {
        ids.extend(collect_node_ids_from_enum(child));
    }
    ids
}



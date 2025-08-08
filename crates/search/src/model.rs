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
        let path = pool.get_node_path(&node.id).into_iter().collect();

        let text = extract_text(node);

        IndexDoc {
            node_id: node.id.clone(),
            node_type: node.r#type.clone(),
            parent_id,
            marks,
            attrs_flat,
            text,
            path,
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

/// 收集 NodeEnum 中所有节点 id（包含子树）
pub fn collect_node_ids_from_enum(node_enum: &NodeEnum) -> Vec<NodeId> {
    let mut ids: Vec<NodeId> = vec![node_enum.0.id.clone()];
    for child in &node_enum.1 {
        ids.extend(collect_node_ids_from_enum(child));
    }
    ids
}



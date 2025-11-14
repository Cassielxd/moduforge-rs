use mf_model::{
    mark::Mark, node::Node, node_definition::NodeTree, node_pool::NodePool,
    types::NodeId,
};
use serde::Serialize;
use mf_model::rpds::{HashTrieMapSync, VectorSync};

/// 扁平化后的索引文档（写入后端的基础结构）
#[derive(Debug, Clone, Serialize)]
pub struct IndexDoc {
    pub node_id: String,
    pub node_type: String,
    pub parent_id: Option<String>,
    /// mark 类型列表（用于简单的 "包含某 mark" 查询）
    pub marks: Vec<String>,
    /// 完整的 marks JSON（用于带属性的精确查询）
    pub marks_json: String,
    /// 扁平化的顶层属性（用于简单查询）
    pub attrs_flat: Vec<(String, String)>,
    /// 完整的 attrs JSON（用于嵌套属性查询）
    pub attrs_json: String,
    pub text: Option<String>,
    pub path: Vec<String>,
    // 常用 fast fields（i64）
    pub order_i64: Option<i64>,
    pub created_at_i64: Option<i64>,
    pub updated_at_i64: Option<i64>,
}
impl IndexDoc {
    /// 从索引文档转换回 Node
    ///
    /// 注意：
    /// - content 字段会是空的，因为 IndexDoc 不保存子节点信息
    /// - 如果需要完整的树结构，需要从 NodePool 中重建
    pub fn to_node(&self) -> anyhow::Result<Node> {
        // 反序列化 marks_json
        let marks: VectorSync<Mark> =
            serde_json::from_str(&self.marks_json).unwrap_or_default();

        // 反序列化 attrs_json
        let attrs_map: HashTrieMapSync<String, serde_json::Value> =
            serde_json::from_str(&self.attrs_json).unwrap_or_default();

        let node = Node {
            id: self.node_id.as_str().into(),
            r#type: self.node_type.clone(),
            attrs: mf_model::attrs::Attrs::from(attrs_map),
            content: VectorSync::new_sync(), // IndexDoc 不保存子节点信息
            marks: marks.into(),
        };

        Ok(node)
    }

    /// 从节点与池快照构建索引文档
    pub fn from_node(
        pool: &NodePool,
        node: &Node,
    ) -> Self {
        let parent_id = pool.parent_id(&node.id).cloned();

        // 提取 mark 类型列表（用于简单查询）
        let marks: Vec<String> =
            node.marks.iter().map(|m: &Mark| m.r#type.clone()).collect();

        // 序列化完整的 marks（用于带属性的查询）
        let marks_json = serde_json::to_string(&node.marks)
            .unwrap_or_else(|_| "[]".to_string());

        // 提取扁平化的顶层属性（用于简单查询）
        let mut attrs_flat = Vec::with_capacity(node.attrs.attrs.keys().len());
        for (k, v) in node.attrs.attrs.iter() {
            attrs_flat.push((k.clone(), flatten_value(v)));
        }

        // 序列化完整的 attrs（用于嵌套属性查询）
        let attrs_json = serde_json::to_string(&node.attrs.attrs)
            .unwrap_or_else(|_| "{}".to_string());

        // 路径（根到当前）
        let path: Vec<String> = pool
            .get_node_path(&node.id)
            .into_iter()
            .map(|id| id.to_string())
            .collect();

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
            marks_json,
            attrs_flat,
            attrs_json,
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
fn extract_i64(
    node: &Node,
    key: &str,
) -> Option<i64> {
    node.attrs.get(key).and_then(|v| match v {
        serde_json::Value::Number(n) => n
            .as_i64()
            .or_else(|| n.as_u64().and_then(|u| i64::try_from(u).ok())),
        serde_json::Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    })
}

/// 收集 NodeEnum 中所有节点 id（包含子树）
pub fn collect_node_ids_from_enum(node_enum: &NodeTree) -> Vec<NodeId> {
    let mut ids: Vec<NodeId> = vec![node_enum.0.id.clone()];
    for child in &node_enum.1 {
        ids.extend(collect_node_ids_from_enum(child));
    }
    ids
}

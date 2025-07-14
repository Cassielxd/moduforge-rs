use super::attrs::Attrs;
use super::mark::Mark;
use super::types::NodeId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
/// 基础节点定义，任何数据都可以认为是节点
///
/// # 属性
///
/// * `id` - 节点唯一标识符
/// * `type` - 节点类型
/// * `attrs` - 节点属性，一般用于元数据的存储
/// * `content` - 子节点列表
/// * `marks` - 节点标记列表
///
/// # 示例
///
/// ```
/// use mf_rs::model::node::Node;
/// use mf_rs::model::attrs::Attrs;
///
/// let node = Node::new(
///     "node1",
///     "paragraph".to_string(),
///     Attrs::default(),
///     vec![],
///     vec![],
/// );
/// ```

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "i")]
    pub id: NodeId,
    #[serde(rename = "t")]
    pub r#type: String,
    #[serde(rename = "a")]
    pub attrs: Attrs,
    #[serde(rename = "c")]
    pub content: im::Vector<NodeId>, // 使用im::Vector替代Arc<Vec>
    #[serde(rename = "m")]
    pub marks: im::Vector<Mark>,
}
unsafe impl Send for Node {}
unsafe impl Sync for Node {}

impl Node {
    /// 创建一个新的节点实例
    ///
    /// # 参数
    ///
    /// * `id` - 节点ID，字符串引用
    /// * `type` - 节点类型
    /// * `attrs` - 节点属性
    /// * `content` - 子节点ID列表
    /// * `marks` - 节点标记列表
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `Node` 实例
    pub fn new(
        id: &str, // 接受字符串引用
        r#type: String,
        attrs: Attrs,
        content: Vec<NodeId>,
        marks: Vec<Mark>,
    ) -> Self {
        Node {
            id: id.into(), // 转换为Arc<str>
            r#type,
            attrs,
            content: content.into(),
            marks: marks.into(),
        }
    }
    /// 获取子节点数量
    ///
    /// # 返回值
    ///
    /// 返回节点包含的子节点数量
    pub fn child_count(&self) -> usize {
        self.content.len()
    }
    /// 更新节点属性
    ///
    /// # 参数
    ///
    /// * `new_values` - 新的属性值
    ///
    /// # 返回值
    pub fn update_attr(&self, new_values: im::HashMap<String, Value>) -> Self {
        let mut new_node = self.clone();
        let new_attrs = self.attrs.update(new_values);
        new_node.attrs = new_attrs;
        new_node
    }
    pub fn insert_content_at_index(&self, index: usize, node_id: &str) -> Self {
        let mut new_node = self.clone();
        new_node.content.insert(index, node_id.into());
        new_node
    }
    pub fn insert_contents(&self, node_ids: &Vec<String>) -> Self {
        let mut new_node = self.clone();
        for node_id in node_ids {
            new_node.content.push_back(node_id.into());
        }
        new_node
    }
    pub fn insert_content(&self, node_id: &str) -> Self {
        let mut new_node = self.clone();
        new_node.content.push_back(node_id.into());
        new_node
    }

    pub fn remove_mark_by_name(&self, mark_name: &str) -> Self {
        let mut new_node = self.clone();
        new_node.marks = new_node
            .marks
            .iter()
            .filter(|&m| m.r#type != mark_name)
            .cloned()
            .collect();
        new_node
    }
    pub fn remove_mark(&self, mark_types: &[String]) -> Self {
        let mut new_node = self.clone();
        new_node.marks = new_node
            .marks
            .iter()
            .filter(|&m| !mark_types.contains(&m.r#type))
            .cloned()
            .collect();
        new_node
    }
    pub fn add_marks(&self, marks: &Vec<Mark>) -> Self {
        let mark_types =
            marks.iter().map(|m| m.r#type.clone()).collect::<Vec<String>>();
        let mut new_node = self.clone();
        //如果存在相同类型的mark，则覆盖
        new_node.marks = new_node
            .marks
            .iter()
            .filter(|m| !mark_types.contains(&m.r#type))
            .cloned()
            .collect();
        new_node.marks.extend(marks.iter().map(|m| m.clone()));
        new_node
    }
}

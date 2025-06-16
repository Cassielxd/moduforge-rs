use super::attrs::Attrs;
use super::mark::Mark;
use super::types::NodeId;
use serde::{Deserialize, Serialize};
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
/// use moduforge_rs::model::node::Node;
/// use moduforge_rs::model::attrs::Attrs;
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
}

use super::attrs::Attrs;
use super::mark::Mark;
use super::types::NodeId;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
/**
 * 基础节点定义 任何数据都可以认为是节点
 * @property id 节点id
 * @property type 节点类型
 * @property attrs 节点属性 一般用于元数据的存储
 * @property content 子节点
 * @property marks 节点标记
 * @author string<348040933@qq.com>
 */

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Decode, Encode)]
pub struct Node {
    pub id: NodeId,
    pub r#type: String,
    #[bincode(with_serde)]
    pub attrs: Attrs,
    #[bincode(with_serde)]
    pub content: im::Vector<NodeId>, // 使用im::Vector替代Arc<Vec>
    #[bincode(with_serde)]
    pub marks: im::Vector<Mark>,
}
unsafe impl Send for Node {}
unsafe impl Sync for Node {}

impl Node {
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

    pub fn child_count(&self) -> usize {
        self.content.len()
    }
}

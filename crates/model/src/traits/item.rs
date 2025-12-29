use std::fmt::{Debug, Display};
use std::hash::Hash;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

/// 数据容器中的基本单元
///
/// 这是所有数据单元的基础 trait，`Node` 是其默认实现。
///
/// # 示例
///
/// ```ignore
/// use mf_model::traits::DataItem;
///
/// // Node 实现了 DataItem
/// let node = Node::new("id1", "paragraph".to_string(), /* ... */);
/// assert_eq!(node.type_name(), "paragraph");
/// ```
pub trait DataItem: Send + Sync + Clone + Debug + Serialize + for<'de> Deserialize<'de> {
    /// 单元的唯一标识符类型
    type Id: Send + Sync + Clone + Hash + Eq + Debug + Display;

    /// 获取单元的类型名称（如 "paragraph", "table", "entity"）
    fn type_name(&self) -> &str;

    /// 获取唯一标识符
    fn id(&self) -> &Self::Id;

    /// 获取所有属性（如果有）
    ///
    /// 返回 `None` 表示该数据单元不支持属性
    fn attributes(&self) -> Option<&HashMap<String, Value>> {
        None
    }

    /// 设置属性（返回新实例，保持不可变性）
    ///
    /// # 参数
    ///
    /// * `attrs` - 新的属性映射
    ///
    /// # 返回值
    ///
    /// 返回设置了新属性的数据单元副本
    fn with_attributes(&self, attrs: HashMap<String, Value>) -> Self;
}

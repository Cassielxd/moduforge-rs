use super::item::DataItem;
use std::fmt::Debug;

/// 数据容器 trait
///
/// 定义数据容器的通用接口，`NodePool` 是其默认实现。
///
/// # 类型参数
///
/// * `Item` - 容器中存储的数据单元类型
/// * `InnerState` - 容器的内部可变状态类型（用于事务处理）
///
/// # 示例
///
/// ```ignore
/// use mf_model::traits::DataContainer;
/// use mf_model::NodePool;
///
/// let pool: NodePool = /* ... */;
/// assert!(pool.contains(&node_id));
/// ```
pub trait DataContainer: Send + Sync + Clone + Debug {
    /// 容器中的数据单元类型
    type Item: DataItem;

    /// 内部可变状态类型（如 `Tree`）
    ///
    /// 用于事务处理时的草稿状态
    type InnerState: Clone + Debug + Send + Sync;

    /// 获取数据单元
    ///
    /// # 参数
    ///
    /// * `id` - 数据单元的唯一标识符
    ///
    /// # 返回值
    ///
    /// 如果存在返回数据单元的引用，否则返回 `None`
    fn get(&self, id: &<Self::Item as DataItem>::Id) -> Option<&Self::Item>;

    /// 检查是否包含指定的数据单元
    ///
    /// # 参数
    ///
    /// * `id` - 数据单元的唯一标识符
    fn contains(&self, id: &<Self::Item as DataItem>::Id) -> bool;

    /// 获取容器中数据单元的总数
    fn size(&self) -> usize;

    /// 获取容器的唯一标识符
    fn key(&self) -> &str;

    /// 获取所有数据单元（用于迭代）
    ///
    /// # 注意
    ///
    /// 此方法可能在大型容器上性能较差，仅用于需要遍历所有数据的场景
    fn items(&self) -> Vec<&Self::Item>;

    /// 获取内部状态的不可变引用
    ///
    /// 用于访问容器的内部数据结构
    fn inner(&self) -> &Self::InnerState;
}

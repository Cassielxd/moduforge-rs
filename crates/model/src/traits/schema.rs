use super::container::DataContainer;
use std::fmt::Debug;

/// Schema 定义 trait
///
/// 定义数据结构的约束和规则，`Schema` 是其默认实现。
///
/// # 类型参数
///
/// * `Container` - 关联的数据容器类型
/// * `ItemDefinition` - 数据单元的类型定义
///
/// # 示例
///
/// ```ignore
/// use mf_model::traits::SchemaDefinition;
/// use mf_model::Schema;
///
/// let schema: Schema = /* ... */;
/// if let Some(def) = schema.get_definition("paragraph") {
///     println!("Found definition: {:?}", def);
/// }
/// ```
pub trait SchemaDefinition: Send + Sync + Clone + Debug {
    /// 关联的数据容器类型
    type Container: DataContainer;

    /// 数据单元类型定义
    ///
    /// 例如：对于树形文档，这是 `NodeDefinition`
    type ItemDefinition: Send + Sync + Clone + Debug;

    /// 获取 Schema 的名称
    fn name(&self) -> &str;

    /// 获取指定类型的定义
    ///
    /// # 参数
    ///
    /// * `type_name` - 类型名称（如 "paragraph", "table"）
    ///
    /// # 返回值
    ///
    /// 如果存在该类型定义返回引用，否则返回 `None`
    fn get_definition(&self, type_name: &str) -> Option<&Self::ItemDefinition>;

    /// 获取所有类型定义
    ///
    /// # 返回值
    ///
    /// 所有类型定义的引用列表
    fn definitions(&self) -> Vec<&Self::ItemDefinition>;

    /// 验证容器中的数据是否符合 Schema
    ///
    /// # 参数
    ///
    /// * `container` - 要验证的数据容器
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 验证通过
    /// * `Err(errors)` - 验证失败，包含所有错误信息
    fn validate(&self, container: &Self::Container) -> Result<(), Vec<String>>;

    /// 验证单个数据单元是否符合定义
    ///
    /// # 参数
    ///
    /// * `item` - 要验证的数据单元
    /// * `definition` - 类型定义
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 验证通过
    /// * `Err(message)` - 验证失败，包含错误信息
    fn validate_item(
        &self,
        item: &<Self::Container as DataContainer>::Item,
        definition: &Self::ItemDefinition,
    ) -> Result<(), String>;
}

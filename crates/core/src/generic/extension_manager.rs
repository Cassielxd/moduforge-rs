//! 扩展管理器的泛型定义
//!
//! 此模块包含 ExtensionManager 的泛型版本。

use std::sync::Arc;

use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::plugin::PluginGeneric;

use crate::{extension::OpFnGeneric, ForgeResult};

/// 扩展管理器（泛型版本）
///
/// 负责管理扩展插件、Schema 和操作函数
pub struct ExtensionManagerGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    plugins: Vec<Arc<PluginGeneric<C, S>>>,
    schema: Arc<S>,
    op_fns: OpFnGeneric<C, S>,
}

impl<C, S> ExtensionManagerGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 创建新的扩展管理器实例
    ///
    /// # 参数
    /// * `plugins` - 插件列表
    /// * `schema` - Schema 实例
    /// * `op_fns` - 操作函数列表
    pub fn new(
        plugins: Vec<Arc<PluginGeneric<C, S>>>,
        schema: Arc<S>,
        op_fns: OpFnGeneric<C, S>,
    ) -> Self {
        Self {
            plugins,
            schema,
            op_fns,
        }
    }

    /// 获取Schema引用
    pub fn get_schema(&self) -> Arc<S> {
        self.schema.clone()
    }

    /// 获取所有插件
    pub fn get_plugins(&self) -> &Vec<Arc<PluginGeneric<C, S>>> {
        &self.plugins
    }

    /// 获取操作函数列表
    pub fn get_op_fns(&self) -> &OpFnGeneric<C, S> {
        &self.op_fns
    }

    /// 重新加载扩展
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub fn reload_extensions(&mut self) -> ForgeResult<()> {
        // 泛型版本的重新加载逻辑需要由具体实现提供
        // 这里提供默认的空实现
        Ok(())
    }
}

impl<C, S> Clone for ExtensionManagerGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    fn clone(&self) -> Self {
        Self {
            plugins: self.plugins.clone(),
            schema: self.schema.clone(),
            op_fns: self.op_fns.clone(),
        }
    }
}

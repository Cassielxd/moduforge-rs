//! 运行时统一接口的泛型定义
//!
//! 此模块定义了 RuntimeTraitGeneric,为所有运行时实现提供统一接口。
//! 支持任意 DataContainer 和 SchemaDefinition 组合。

use std::sync::Arc;

use async_trait::async_trait;
use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::{
    state::StateGeneric,
    transaction::{CommandGeneric, TransactionGeneric},
};

use crate::{config::ForgeConfig, types::RuntimeOptions, ForgeResult};

/// 运行时统一接口（泛型版本）
///
/// 定义了所有运行时实现必须提供的核心功能。
/// 所有方法都是异步的,以支持不同的执行模型。
///
/// # 设计原则
///
/// 1. **接口最小化**: 只包含核心必需方法
/// 2. **异步优先**: 所有方法异步,兼容三种运行时
/// 3. **错误统一**: 使用 ForgeResult 统一错误处理
/// 4. **状态不可变**: 返回 Arc<StateGeneric<C, S>> 避免克隆
/// 5. **完全泛型**: 支持任意 DataContainer 和 SchemaDefinition 组合
///
/// # 使用示例
///
/// ```rust
/// use mf_core::{ForgeRuntime, RuntimeTrait};
/// use mf_model::{node_pool::NodePool, schema::Schema};
///
/// async fn process_with_runtime<C, S>(
///     runtime: &mut dyn RuntimeTraitGeneric<C, S>
/// ) -> ForgeResult<()>
/// where
///     C: DataContainer + 'static,
///     S: SchemaDefinition<Container = C> + 'static,
/// {
///     let state = runtime.get_state().await?;
///     let mut tr = runtime.get_tr().await?;
///
///     // ... 修改事务 ...
///
///     runtime.dispatch(tr).await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait RuntimeTraitGeneric<C, S>: Send
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    // ==================== 核心事务操作 ====================

    /// 分发事务到运行时处理
    ///
    /// 这是运行时的核心方法,负责:
    /// 1. 执行前置中间件
    /// 2. 应用事务到状态
    /// 3. 触发插件的 append_transaction
    /// 4. 执行后置中间件
    /// 5. 更新状态和历史
    /// 6. 广播事件
    ///
    /// # 参数
    /// * `transaction` - 要处理的事务
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 处理结果
    async fn dispatch(
        &mut self,
        transaction: TransactionGeneric<C, S>,
    ) -> ForgeResult<()>;

    /// 分发事务(包含元信息)
    ///
    /// 与 dispatch 相同,但可以附加描述和元数据,用于:
    /// - 历史记录的可读性
    /// - 审计日志
    /// - 撤销/重做时的显示
    ///
    /// # 参数
    /// * `transaction` - 要处理的事务
    /// * `description` - 事务描述(用于历史记录)
    /// * `meta` - 元数据(JSON格式)
    async fn dispatch_with_meta(
        &mut self,
        transaction: TransactionGeneric<C, S>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()>;

    /// 执行命令
    ///
    /// 命令是对事务的高级封装,提供:
    /// - 类型安全的操作
    /// - 可复用的业务逻辑
    /// - 更好的测试性
    ///
    /// # 参数
    /// * `command` - 要执行的命令
    async fn command(
        &mut self,
        command: Arc<dyn CommandGeneric<C, S>>,
    ) -> ForgeResult<()>;

    /// 执行命令(包含元信息)
    async fn command_with_meta(
        &mut self,
        command: Arc<dyn CommandGeneric<C, S>>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()>;

    // ==================== 状态访问 ====================

    /// 获取当前状态
    ///
    /// 返回不可变状态引用,避免克隆开销。
    /// 状态是快照,读取时不会被其他操作修改。
    async fn get_state(&self) -> ForgeResult<Arc<StateGeneric<C, S>>>;

    /// 获取新事务
    ///
    /// 基于当前状态创建新事务,用于构建编辑操作。
    async fn get_tr(&self) -> ForgeResult<TransactionGeneric<C, S>>;

    /// 获取文档
    ///
    /// 快捷方法,等同于 `get_state().await?.doc()`
    async fn doc(&self) -> ForgeResult<Arc<C>> {
        Ok(self.get_state().await?.doc())
    }

    /// 获取Schema
    ///
    /// 返回文档结构定义
    async fn get_schema(&self) -> ForgeResult<Arc<S>>;

    // ==================== 历史管理 ====================

    /// 撤销操作
    ///
    /// 回退到上一个历史状态
    async fn undo(&mut self) -> ForgeResult<()>;

    /// 重做操作
    ///
    /// 前进到下一个历史状态
    async fn redo(&mut self) -> ForgeResult<()>;

    /// 跳转到指定历史位置
    ///
    /// # 参数
    /// * `steps` - 跳转步数(正数前进,负数后退)
    async fn jump(
        &mut self,
        steps: isize,
    ) -> ForgeResult<()>;

    // ==================== 配置管理 ====================

    /// 获取运行时配置
    fn get_config(&self) -> &ForgeConfig;

    /// 更新运行时配置
    fn update_config(
        &mut self,
        config: ForgeConfig,
    );

    /// 获取运行时选项
    fn get_options(&self) -> &RuntimeOptions;

    // ==================== 生命周期管理 ====================

    /// 销毁运行时
    ///
    /// 清理所有资源,包括:
    /// - 停止事件循环
    /// - 关闭Actor系统(如果适用)
    /// - 释放异步资源(如果适用)
    async fn destroy(&mut self) -> ForgeResult<()>;
}

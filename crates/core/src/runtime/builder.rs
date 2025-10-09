//! 统一运行时构建器
//!
//! 提供统一的运行时创建接口，支持：
//! 1. 自动检测系统资源并选择最优运行时
//! 2. 手动指定运行时类型
//! 3. 使用配置文件创建运行时
//!
//! # 使用示例
//!
//! ## 完全自动（推荐）
//! ```rust
//! use mf_core::runtime::builder::ForgeRuntimeBuilder;
//!
//! // 自动检测系统资源，选择最优运行时和配置
//! let runtime = ForgeRuntimeBuilder::auto().await?;
//! ```
//!
//! ## 手动指定类型
//! ```rust
//! use mf_core::runtime::builder::ForgeRuntimeBuilder;
//! use mf_core::config::RuntimeType;
//!
//! // 明确使用Actor运行时
//! let runtime = ForgeRuntimeBuilder::with_type(RuntimeType::Actor).await?;
//! ```
//!
//! ## 使用配置
//! ```rust
//! use mf_core::runtime::builder::ForgeRuntimeBuilder;
//! use mf_core::config::{ForgeConfig, RuntimeType, RuntimeConfig};
//!
//! let config = ForgeConfig {
//!     runtime: RuntimeConfig {
//!         runtime_type: RuntimeType::Async,
//!     },
//!     ..Default::default()
//! };
//! let runtime = ForgeRuntimeBuilder::from_config(config, None).await?;
//! ```

use crate::{
    config::{ForgeConfig, RuntimeType},
    debug::info,
    runtime::{
        adaptive::AdaptiveRuntimeSelector, actor_runtime::ForgeActorRuntime,
        async_runtime::ForgeAsyncRuntime, runtime::ForgeRuntime,
        runtime_trait::RuntimeTrait, system_detector::SystemResources,
    },
    types::RuntimeOptions,
    ForgeResult,
};

/// 统一运行时构建器
///
/// 提供多种创建运行时的方式：
/// - `auto()`: 完全自动，根据系统资源选择
/// - `with_type()`: 手动指定运行时类型
/// - `from_config()`: 从配置创建
pub struct ForgeRuntimeBuilder;

impl ForgeRuntimeBuilder {
    /// 🎯 完全自动创建 - 检测系统资源并创建最优运行时
    ///
    /// 这是推荐的创建方式，会：
    /// 1. 自动检测CPU核心数和内存大小
    /// 2. 根据系统资源选择最优运行时类型
    /// 3. 生成优化的配置参数
    /// 4. 输出检测和配置信息
    ///
    /// # 参数
    /// * `options` - 可选的运行时选项（为None时使用默认值）
    ///
    /// # 返回值
    /// * `ForgeResult<Box<dyn RuntimeTrait>>` - 运行时实例或错误
    ///
    /// # 示例
    /// ```rust
    /// // 使用默认选项
    /// let runtime = ForgeRuntimeBuilder::auto(None).await?;
    ///
    /// // 使用自定义选项
    /// let options = RuntimeOptions::default();
    /// let runtime = ForgeRuntimeBuilder::auto(Some(options)).await?;
    ///
    /// // 输出示例：
    /// // 🖥️  系统资源: 8 核心, 16 GB 内存 (高性能)
    /// // ⚡ 运行时类型: Actor
    /// // 📊 并发任务数: 6
    /// // 💾 队列大小: 1600
    /// ```
    pub async fn auto(
        options: Option<RuntimeOptions>
    ) -> ForgeResult<Box<dyn RuntimeTrait>> {
        // 1. 检测系统资源
        let resources = SystemResources::detect();

        // 2. 生成自适应配置
        let config = AdaptiveRuntimeSelector::generate_config(&resources);

        // 3. 输出配置信息
        info!(
            "🖥️  系统资源: {} 核心 / {} 线程, {} GB 内存 ({})",
            resources.cpu_cores,
            resources.cpu_threads,
            resources.total_memory_mb / 1024,
            resources.tier_description()
        );
        info!("⚡ 运行时类型: {:?}", config.runtime.runtime_type);
        info!("📊 并发任务数: {}", config.processor.max_concurrent_tasks);
        info!("💾 队列大小: {}", config.processor.max_queue_size);

        // 4. 创建运行时
        Self::from_config(config, options).await
    }

    /// 使用指定的运行时类型创建
    ///
    /// 仍然会检测系统资源并优化配置，但强制使用指定的运行时类型。
    ///
    /// # 参数
    /// * `runtime_type` - 指定的运行时类型
    ///
    /// # 返回值
    /// * `ForgeResult<Box<dyn RuntimeTrait>>` - 运行时实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::config::RuntimeType;
    ///
    /// // 强制使用Actor运行时，但配置参数仍然自适应
    /// let runtime = ForgeRuntimeBuilder::with_type(RuntimeType::Actor).await?;
    /// ```
    pub async fn with_type(
        runtime_type: RuntimeType,
        options: Option<RuntimeOptions>,
    ) -> ForgeResult<Box<dyn RuntimeTrait>> {
        let resources = SystemResources::detect();
        let mut config = AdaptiveRuntimeSelector::generate_config(&resources);
        config.runtime.runtime_type = runtime_type;

        info!("⚡ 使用指定运行时: {:?}", runtime_type);
        Self::from_config(config, options).await
    }

    /// 从配置创建运行时
    ///
    /// 如果配置中的运行时类型为 `Auto`，会自动检测系统资源。
    ///
    /// # 参数
    /// * `config` - Forge配置
    /// * `options` - 可选的运行时选项（为None时使用默认值）
    ///
    /// # 返回值
    /// * `ForgeResult<Box<dyn RuntimeTrait>>` - 运行时实例或错误
    ///
    /// # 示例
    /// ```rust
    /// let config = ForgeConfig {
    ///     runtime: RuntimeConfig {
    ///         runtime_type: RuntimeType::Auto, // 自动检测
    ///     },
    ///     ..Default::default()
    /// };
    /// let runtime = ForgeRuntimeBuilder::from_config(config, None).await?;
    /// ```
    pub async fn from_config(
        config: ForgeConfig,
        options: Option<RuntimeOptions>,
    ) -> ForgeResult<Box<dyn RuntimeTrait>> {
        let options = options.unwrap_or_default();

        // 如果是Auto，检测系统资源并选择运行时
        let runtime_type = match config.runtime.runtime_type {
            RuntimeType::Auto => {
                let resources = SystemResources::detect();
                AdaptiveRuntimeSelector::select_runtime(&resources)
            },
            rt => rt,
        };

        Self::create_with_type(runtime_type, options, config).await
    }

    /// 内部方法：根据运行时类型创建实例
    async fn create_with_type(
        runtime_type: RuntimeType,
        options: RuntimeOptions,
        config: ForgeConfig,
    ) -> ForgeResult<Box<dyn RuntimeTrait>> {
        match runtime_type {
            RuntimeType::Sync => Ok(Box::new(
                ForgeRuntime::create_with_config(options, config).await?,
            )),
            RuntimeType::Async => Ok(Box::new(
                ForgeAsyncRuntime::create_with_config(options, config).await?,
            )),
            RuntimeType::Actor => Ok(Box::new(
                ForgeActorRuntime::create_with_config(options, config).await?,
            )),
            RuntimeType::Auto => {
                unreachable!("Auto should be resolved before this point")
            },
        }
    }
}

/// 为RuntimeTrait添加辅助方法
pub trait RuntimeExt {
    /// 获取运行时类型描述
    fn runtime_type_name(&self) -> &'static str;
}

impl RuntimeExt for Box<dyn RuntimeTrait> {
    fn runtime_type_name(&self) -> &'static str {
        // 简单返回"Runtime"，因为trait object无法直接判断具体类型
        // 如需准确类型，可在RuntimeTrait中添加type_name方法
        "Runtime"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auto_creation() {
        // 测试自动创建
        let result = ForgeRuntimeBuilder::auto(None).await;

        // 应该成功创建
        assert!(result.is_ok(), "自动创建运行时应该成功");
    }

    #[tokio::test]
    async fn test_with_type_creation() {
        // 测试指定类型创建
        let result =
            ForgeRuntimeBuilder::with_type(RuntimeType::Sync, None).await;

        assert!(result.is_ok(), "指定类型创建应该成功");
    }

    #[tokio::test]
    async fn test_from_config_auto() {
        let config = ForgeConfig {
            runtime: crate::config::RuntimeConfig {
                runtime_type: RuntimeType::Auto,
            },
            ..Default::default()
        };

        let result = ForgeRuntimeBuilder::from_config(config, None).await;
        assert!(result.is_ok(), "从Auto配置创建应该成功");
    }

    #[tokio::test]
    async fn test_from_config_sync() {
        let config = ForgeConfig {
            runtime: crate::config::RuntimeConfig {
                runtime_type: RuntimeType::Sync,
            },
            ..Default::default()
        };

        let result = ForgeRuntimeBuilder::from_config(config, None).await;
        assert!(result.is_ok(), "从Sync配置创建应该成功");
    }
}

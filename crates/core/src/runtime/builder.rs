//! 统一运行时构建器
//!
//! 提供统一、流畅的运行时创建接口，支持：
//! 1. 自动检测系统资源并选择最优运行时
//! 2. 手动指定运行时类型
//! 3. 使用配置文件创建运行时
//! 4. 链式配置构建
//!
//! # 设计原则
//!
//! - **简单优先**：最常见的用例应该最简单
//! - **渐进式配置**：从简单到复杂，逐步添加配置
//! - **类型安全**：编译期捕获配置错误
//! - **统一接口**：所有创建方式返回统一的运行时类型
//!
//! # 使用示例
//!
//! ## 1. 最简单的方式（推荐）
//! ```rust
//! use mf_core::ForgeRuntimeBuilder;
//!
//! // 自动检测系统资源，选择最优运行时和配置
//! let runtime = ForgeRuntimeBuilder::new().build().await?;
//! ```
//!
//! ## 2. 指定运行时类型
//! ```rust
//! use mf_core::{ForgeRuntimeBuilder, RuntimeType};
//!
//! // 明确使用 Actor 运行时
//! let runtime = ForgeRuntimeBuilder::new()
//!     .runtime_type(RuntimeType::Actor)
//!     .build()
//!     .await?;
//! ```
//!
//! ## 3. 链式配置
//! ```rust
//! use mf_core::{ForgeRuntimeBuilder, RuntimeType, Extensions};
//!
//! let runtime = ForgeRuntimeBuilder::new()
//!     .runtime_type(RuntimeType::Async)
//!     .max_concurrent_tasks(20)
//!     .queue_size(5000)
//!     .enable_monitoring(true)
//!     .history_limit(1000)
//!     .extension(my_extension)
//!     .build()
//!     .await?;
//! ```
//!
//! ## 4. 从配置文件
//! ```rust
//! use mf_core::ForgeRuntimeBuilder;
//!
//! let runtime = ForgeRuntimeBuilder::from_config_file("config.toml")
//!     .await?
//!     .build()
//!     .await?;
//! ```
//!
//! ## 5. 从 XML Schema
//! ```rust
//! use mf_core::ForgeRuntimeBuilder;
//!
//! let runtime = ForgeRuntimeBuilder::new()
//!     .schema_path("schema/document.xml")
//!     .build()
//!     .await?;
//! ```

use crate::{
    config::{ForgeConfig, RuntimeType, Environment},
    debug::info,
    runtime::{
        adaptive::AdaptiveRuntimeSelector, actor_runtime::ForgeActorRuntime,
        async_runtime::ForgeAsyncRuntime, runtime::ForgeRuntime,
        runtime_trait::RuntimeTraitGeneric, system_detector::SystemResources,
    },
    types::{RuntimeOptions, Extensions, Content, EditorOptionsBuilder},
    ForgeResult,
};
use std::sync::Arc;

/// 统一运行时构建器
///
/// 提供流畅的链式 API 来配置和创建运行时。
///
/// # 设计特点
///
/// 1. **自动推断**：未指定的配置项会根据系统资源自动优化
/// 2. **类型安全**：配置错误在编译期捕获
/// 3. **灵活组合**：可以混合使用不同的配置方式
/// 4. **统一返回**：始终返回 `AnyRuntime` 枚举，避免 trait object 开销
///
/// # 示例
///
/// ```rust
/// // 最简单的用法
/// let runtime = ForgeRuntimeBuilder::new().build().await?;
///
/// // 完全自定义
/// let runtime = ForgeRuntimeBuilder::new()
///     .runtime_type(RuntimeType::Actor)
///     .environment(Environment::Production)
///     .max_concurrent_tasks(20)
///     .build()
///     .await?;
/// ```
#[derive(Default)]
pub struct ForgeRuntimeBuilder {
    // 核心配置
    runtime_type: Option<RuntimeType>,
    environment: Option<Environment>,

    // 运行时选项
    content: Option<Content>,
    extensions: Vec<Extensions>,
    history_limit: Option<usize>,
    event_handlers: Vec<
        Arc<dyn crate::event::EventHandler<crate::event::Event> + Send + Sync>,
    >,

    // 性能配置
    max_concurrent_tasks: Option<usize>,
    queue_size: Option<usize>,
    enable_monitoring: Option<bool>,
    middleware_timeout_ms: Option<u64>,

    // Schema 配置
    schema_paths: Vec<String>,

    // 完整配置（如果提供）
    full_config: Option<ForgeConfig>,
}

impl ForgeRuntimeBuilder {
    /// 创建新的构建器实例
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    // ==================== 核心配置方法 ====================

    /// 设置运行时类型
    ///
    /// 如果不设置，将根据系统资源自动选择。
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .runtime_type(RuntimeType::Actor);
    /// ```
    pub fn runtime_type(
        mut self,
        runtime_type: RuntimeType,
    ) -> Self {
        self.runtime_type = Some(runtime_type);
        self
    }

    /// 设置运行环境
    ///
    /// 不同环境有不同的默认配置。
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .environment(Environment::Production);
    /// ```
    pub fn environment(
        mut self,
        environment: Environment,
    ) -> Self {
        self.environment = Some(environment);
        self
    }

    // ==================== 内容和扩展配置 ====================

    /// 设置初始内容
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .content(Content::Json(json_data));
    /// ```
    pub fn content(
        mut self,
        content: Content,
    ) -> Self {
        self.content = Some(content);
        self
    }

    /// 添加扩展
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .extension(Extensions::N(my_node));
    /// ```
    pub fn extension(
        mut self,
        extension: Extensions,
    ) -> Self {
        self.extensions.push(extension);
        self
    }

    /// 批量添加扩展
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .extensions(vec![ext1, ext2, ext3]);
    /// ```
    pub fn extensions(
        mut self,
        extensions: Vec<Extensions>,
    ) -> Self {
        self.extensions.extend(extensions);
        self
    }

    /// 从 XML Schema 文件加载扩展
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .schema_path("schema/document.xml");
    /// ```
    pub fn schema_path(
        mut self,
        path: impl Into<String>,
    ) -> Self {
        self.schema_paths.push(path.into());
        self
    }

    /// 从多个 XML Schema 文件加载扩展
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .schema_paths(vec!["schema/doc.xml", "schema/ui.xml"]);
    /// ```
    pub fn schema_paths(
        mut self,
        paths: Vec<String>,
    ) -> Self {
        self.schema_paths.extend(paths);
        self
    }

    // ==================== 性能配置 ====================

    /// 设置最大并发任务数
    ///
    /// 如果不设置，将根据 CPU 核心数自动计算。
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .max_concurrent_tasks(20);
    /// ```
    pub fn max_concurrent_tasks(
        mut self,
        count: usize,
    ) -> Self {
        self.max_concurrent_tasks = Some(count);
        self
    }

    /// 设置任务队列大小
    ///
    /// 如果不设置，将根据可用内存自动计算。
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .queue_size(5000);
    /// ```
    pub fn queue_size(
        mut self,
        size: usize,
    ) -> Self {
        self.queue_size = Some(size);
        self
    }

    /// 启用或禁用性能监控
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .enable_monitoring(true);
    /// ```
    pub fn enable_monitoring(
        mut self,
        enable: bool,
    ) -> Self {
        self.enable_monitoring = Some(enable);
        self
    }

    /// 设置中间件超时时间（毫秒）
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .middleware_timeout_ms(1000);
    /// ```
    pub fn middleware_timeout_ms(
        mut self,
        timeout: u64,
    ) -> Self {
        self.middleware_timeout_ms = Some(timeout);
        self
    }

    // ==================== 历史和事件配置 ====================

    /// 设置历史记录限制
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .history_limit(1000);
    /// ```
    pub fn history_limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.history_limit = Some(limit);
        self
    }

    /// 添加事件处理器
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .event_handler(my_handler);
    /// ```
    pub fn event_handler(
        mut self,
        handler: Arc<
            dyn crate::event::EventHandler<crate::event::Event> + Send + Sync,
        >,
    ) -> Self {
        self.event_handlers.push(handler);
        self
    }

    // ==================== 高级配置 ====================

    /// 使用完整的 ForgeConfig
    ///
    /// 这会覆盖之前设置的所有配置项。
    ///
    /// # 示例
    /// ```rust
    /// let config = ForgeConfig::for_environment(Environment::Production);
    /// let builder = ForgeRuntimeBuilder::new()
    ///     .with_config(config);
    /// ```
    pub fn with_config(
        mut self,
        config: ForgeConfig,
    ) -> Self {
        self.full_config = Some(config);
        self
    }

    /// 从 JSON 配置文件加载
    ///
    /// # 示例
    /// ```rust
    /// let builder = ForgeRuntimeBuilder::from_config_file("config.json").await?;
    /// ```
    pub async fn from_config_file(path: &str) -> ForgeResult<Self> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            crate::error::error_utils::storage_error(format!(
                "Failed to read config file: {}",
                e
            ))
        })?;

        let config: ForgeConfig =
            serde_json::from_str(&content).map_err(|e| {
                crate::error::error_utils::config_error(format!(
                    "Failed to parse JSON config: {}",
                    e
                ))
            })?;

        Ok(Self::new().with_config(config))
    }

    // ==================== 构建方法 ====================

    /// 构建运行时实例
    ///
    /// 这是最终的构建方法，会：
    /// 1. 合并所有配置
    /// 2. 检测系统资源（如果需要）
    /// 3. 创建运行时实例
    ///
    /// # 返回值
    /// * `ForgeResult<AnyRuntime>` - 运行时实例或错误
    ///
    /// # 示例
    /// ```rust
    /// let runtime = ForgeRuntimeBuilder::new()
    ///     .runtime_type(RuntimeType::Async)
    ///     .max_concurrent_tasks(20)
    ///     .build()
    ///     .await?;
    /// ```
    pub async fn build(self) -> ForgeResult<AnyRuntime> {
        // 1. 构建最终配置
        let (config, options) = self.build_config_and_options().await?;

        // 2. 确定运行时类型
        let runtime_type = match config.runtime.runtime_type {
            RuntimeType::Auto => {
                let resources = SystemResources::detect();
                let selected =
                    AdaptiveRuntimeSelector::select_runtime(&resources);

                info!(
                    "🖥️  系统资源: {} 核心 / {} 线程, {} GB 内存 ({})",
                    resources.cpu_cores,
                    resources.cpu_threads,
                    resources.total_memory_mb / 1024,
                    resources.tier_description()
                );
                info!("⚡ 自动选择运行时: {:?}", selected);

                selected
            },
            rt => {
                info!("⚡ 使用指定运行时: {:?}", rt);
                rt
            },
        };

        // 3. 创建运行时实例
        Self::create_runtime(runtime_type, options, config).await
    }

    /// 构建配置和选项
    async fn build_config_and_options(
        self
    ) -> ForgeResult<(ForgeConfig, RuntimeOptions)> {
        // 如果提供了完整配置，使用它作为基础
        let mut config = self.full_config.unwrap_or_else(|| {
            // 否则，根据环境创建默认配置
            match self.environment {
                Some(env) => ForgeConfig::for_environment(env),
                None => {
                    // 检测系统资源并生成自适应配置
                    let resources = SystemResources::detect();
                    AdaptiveRuntimeSelector::generate_config(&resources)
                },
            }
        });

        // 应用用户指定的配置覆盖
        if let Some(rt) = self.runtime_type {
            config.runtime.runtime_type = rt;
        }
        if let Some(tasks) = self.max_concurrent_tasks {
            config.processor.max_concurrent_tasks = tasks;
        }
        if let Some(size) = self.queue_size {
            config.processor.max_queue_size = size;
        }
        if let Some(enable) = self.enable_monitoring {
            config.performance.enable_monitoring = enable;
        }
        if let Some(timeout) = self.middleware_timeout_ms {
            config.performance.middleware_timeout_ms = timeout;
        }

        // 如果指定了 schema 路径，添加到配置中
        if !self.schema_paths.is_empty() {
            config.extension.xml_schema_paths = self.schema_paths;
        }

        // 构建 RuntimeOptions
        let mut options_builder = EditorOptionsBuilder::new();

        if let Some(content) = self.content {
            options_builder = options_builder.content(content);
        }

        options_builder = options_builder.extensions(self.extensions);

        if let Some(limit) = self.history_limit {
            options_builder = options_builder.history_limit(limit);
        }

        options_builder = options_builder.event_handlers(self.event_handlers);

        let options = options_builder.build();

        Ok((config, options))
    }

    /// 创建运行时实例
    async fn create_runtime(
        runtime_type: RuntimeType,
        options: RuntimeOptions,
        config: ForgeConfig,
    ) -> ForgeResult<AnyRuntime> {
        match runtime_type {
            RuntimeType::Sync => {
                let runtime =
                    ForgeRuntime::create_with_config(options, config).await?;
                Ok(AnyRuntime::Sync(runtime))
            },
            RuntimeType::Async => {
                let runtime =
                    ForgeAsyncRuntime::create_with_config(options, config)
                        .await?;
                Ok(AnyRuntime::Async(runtime))
            },
            RuntimeType::Actor => {
                let runtime =
                    ForgeActorRuntime::create_with_config(options, config)
                        .await?;
                Ok(AnyRuntime::Actor(runtime))
            },
            RuntimeType::Auto => {
                unreachable!("Auto should be resolved before this point")
            },
        }
    }
}

/// 统一的运行时枚举
///
/// 相比 `Box<dyn RuntimeTrait>`，这个枚举：
/// - 避免了动态分发的性能开销
/// - 保留了具体类型信息
/// - 支持类型特化优化
/// - 可以添加运行时特有的方法
///
/// # 示例
/// ```rust
/// let runtime = ForgeRuntimeBuilder::new().build().await?;
///
/// match &runtime {
///     AnyRuntime::Sync(rt) => println!("Using sync runtime"),
///     AnyRuntime::Async(rt) => println!("Using async runtime"),
///     AnyRuntime::Actor(rt) => println!("Using actor runtime"),
/// }
/// ```
pub enum AnyRuntime {
    Sync(ForgeRuntime),
    Async(ForgeAsyncRuntime),
    Actor(ForgeActorRuntime),
}

impl AnyRuntime {
    /// 获取运行时类型
    pub fn runtime_type(&self) -> RuntimeType {
        match self {
            Self::Sync(_) => RuntimeType::Sync,
            Self::Async(_) => RuntimeType::Async,
            Self::Actor(_) => RuntimeType::Actor,
        }
    }

    /// 尝试获取 Sync 运行时的引用
    pub fn as_sync(&self) -> Option<&ForgeRuntime> {
        match self {
            Self::Sync(rt) => Some(rt),
            _ => None,
        }
    }

    /// 尝试获取 Async 运行时的引用
    pub fn as_async(&self) -> Option<&ForgeAsyncRuntime> {
        match self {
            Self::Async(rt) => Some(rt),
            _ => None,
        }
    }

    /// 尝试获取 Actor 运行时的引用
    pub fn as_actor(&self) -> Option<&ForgeActorRuntime> {
        match self {
            Self::Actor(rt) => Some(rt),
            _ => None,
        }
    }

    /// 尝试获取 Sync 运行时的可变引用
    pub fn as_sync_mut(&mut self) -> Option<&mut ForgeRuntime> {
        match self {
            Self::Sync(rt) => Some(rt),
            _ => None,
        }
    }

    /// 尝试获取 Async 运行时的可变引用
    pub fn as_async_mut(&mut self) -> Option<&mut ForgeAsyncRuntime> {
        match self {
            Self::Async(rt) => Some(rt),
            _ => None,
        }
    }

    /// 尝试获取 Actor 运行时的可变引用
    pub fn as_actor_mut(&mut self) -> Option<&mut ForgeActorRuntime> {
        match self {
            Self::Actor(rt) => Some(rt),
            _ => None,
        }
    }
}

// 为 AnyRuntime 提供便捷方法（委托给具体的运行时）
impl AnyRuntime {
    /// 分发事务
    pub async fn dispatch(
        &mut self,
        transaction: mf_state::Transaction,
    ) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => rt.dispatch(transaction).await,
            Self::Async(rt) => rt.dispatch(transaction).await,
            Self::Actor(rt) => rt.dispatch(transaction).await,
        }
    }

    /// 分发事务（带元信息）
    pub async fn dispatch_with_meta(
        &mut self,
        transaction: mf_state::Transaction,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => {
                rt.dispatch_with_meta(transaction, description, meta).await
            },
            Self::Async(rt) => {
                rt.dispatch_with_meta(transaction, description, meta).await
            },
            Self::Actor(rt) => {
                rt.dispatch_with_meta(transaction, description, meta).await
            },
        }
    }

    /// 执行命令
    pub async fn command(
        &mut self,
        command: Arc<dyn mf_state::transaction::Command>,
    ) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => rt.command(command).await,
            Self::Async(rt) => rt.command(command).await,
            Self::Actor(rt) => rt.command(command).await,
        }
    }

    /// 执行命令（带元信息）
    pub async fn command_with_meta(
        &mut self,
        command: Arc<dyn mf_state::transaction::Command>,
        description: String,
        meta: serde_json::Value,
    ) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => {
                rt.command_with_meta(command, description, meta).await
            },
            Self::Async(rt) => {
                rt.command_with_meta(command, description, meta).await
            },
            Self::Actor(rt) => {
                rt.command_with_meta(command, description, meta).await
            },
        }
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> ForgeResult<Arc<mf_state::State>> {
        match self {
            Self::Sync(rt) => Ok(Arc::clone(rt.get_state())),
            Self::Async(rt) => rt.get_state().await,
            Self::Actor(rt) => rt.get_state().await,
        }
    }

    /// 获取新事务
    pub async fn get_tr(&self) -> ForgeResult<mf_state::Transaction> {
        match self {
            Self::Sync(rt) => Ok(rt.get_tr()),
            Self::Async(rt) => rt.get_tr().await,
            Self::Actor(rt) => rt.get_tr().await,
        }
    }

    /// 获取文档
    pub async fn doc(&self) -> ForgeResult<Arc<mf_model::NodePool>> {
        match self {
            Self::Sync(rt) => Ok(rt.doc()),
            Self::Async(rt) => rt.doc().await,
            Self::Actor(rt) => rt.doc().await,
        }
    }

    /// 获取 Schema
    pub async fn schema(&self) -> ForgeResult<Arc<mf_model::Schema>> {
        match self {
            Self::Sync(rt) => Ok(rt.get_schema()),
            Self::Async(rt) => rt.get_schema().await,
            Self::Actor(rt) => rt.get_schema().await,
        }
    }

    /// 撤销
    pub async fn undo(&mut self) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => {
                rt.undo();
                Ok(())
            },
            Self::Async(rt) => rt.undo().await,
            Self::Actor(rt) => rt.undo().await,
        }
    }

    /// 重做
    pub async fn redo(&mut self) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => {
                rt.redo();
                Ok(())
            },
            Self::Async(rt) => rt.redo().await,
            Self::Actor(rt) => rt.redo().await,
        }
    }

    /// 跳转到指定历史位置
    pub async fn jump(
        &mut self,
        steps: isize,
    ) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => {
                rt.jump(steps);
                Ok(())
            },
            Self::Async(rt) => rt.jump(steps).await,
            Self::Actor(rt) => rt.jump(steps).await,
        }
    }

    /// 获取运行时配置
    pub fn get_config(&self) -> &crate::config::ForgeConfig {
        match self {
            Self::Sync(rt) => rt.get_config(),
            Self::Async(rt) => rt.get_config(),
            Self::Actor(rt) => rt.get_config(),
        }
    }

    /// 更新运行时配置
    pub fn update_config(
        &mut self,
        config: crate::config::ForgeConfig,
    ) {
        match self {
            Self::Sync(rt) => rt.update_config(config),
            Self::Async(rt) => rt.update_config(config),
            Self::Actor(rt) => rt.update_config(config),
        }
    }

    /// 获取运行时选项
    ///
    /// 注意：Actor 运行时返回默认选项，因为它不直接持有 options
    pub fn get_options(&self) -> crate::types::RuntimeOptions {
        match self {
            Self::Sync(rt) => rt.get_options().clone(),
            Self::Async(rt) => rt.get_options().clone(),
            Self::Actor(rt) => rt.get_options(),
        }
    }

    /// 销毁运行时
    pub async fn destroy(&mut self) -> ForgeResult<()> {
        match self {
            Self::Sync(rt) => rt.destroy().await,
            Self::Async(rt) => rt.destroy().await,
            Self::Actor(rt) => rt.destroy().await,
        }
    }
}

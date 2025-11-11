//! 开发环境追踪初始化模块
//!
//! 此模块提供统一的追踪初始化接口，仅在开发环境启用。
//! 生产环境下所有追踪代码会被编译器完全优化掉，实现零开销。

// ============================================================================
// tokio-console 支持（实时异步任务监控）
// ============================================================================

#[cfg(feature = "dev-console")]
pub mod tokio_console {
    //! tokio-console 实时监控模块
    //!
    //! 提供实时的异步任务监控和调试功能，无需手动添加 instrument 注解。
    //!
    //! # 使用方法
    //!
    //! 1. 启用 feature：
    //! ```bash
    //! cargo run --features dev-console
    //! ```
    //!
    //! 2. 在代码中初始化：
    //! ```rust,ignore
    //! #[cfg(feature = "dev-console")]
    //! mf_core::tracing_init::tokio_console::init()?;
    //! ```
    //!
    //! 3. 启动 tokio-console 客户端：
    //! ```bash
    //! tokio-console
    //! ```
    //!
    //! # 注意事项
    //!
    //! - tokio-console 会监听 `127.0.0.1:6669` 端口
    //! - 不要在生产环境启用此 feature，会有性能开销
    //! - 与其他 tracing 初始化函数互斥，只能选择一个

    /// 初始化 tokio-console 订阅者
    ///
    /// 这会启动一个后台服务器，监听 `127.0.0.1:6669`，
    /// 供 tokio-console 客户端连接。
    ///
    /// # 返回值
    /// * `Ok(())` - 初始化成功
    /// * `Err(anyhow::Error)` - 初始化失败
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// #[cfg(feature = "dev-console")]
    /// {
    ///     use mf_core::tracing_init::tokio_console;
    ///     tokio_console::init()?;
    ///     tracing::info!("tokio-console 已启动，请运行 'tokio-console' 连接");
    /// }
    /// ```
    pub fn init() -> anyhow::Result<()> {
        console_subscriber::init();
        tracing::info!("🔍 tokio-console 已启动");
        tracing::info!("📡 监听地址: 127.0.0.1:6669");
        tracing::info!("💡 运行 'tokio-console' 命令连接到监控界面");
        tracing::info!("📚 文档: https://docs.rs/tokio-console");
        Ok(())
    }

    /// 使用自定义配置初始化 tokio-console
    ///
    /// # 参数
    /// * `server_addr` - 服务器监听地址，例如 "127.0.0.1:6669"
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// #[cfg(feature = "dev-console")]
    /// {
    ///     use mf_core::tracing_init::tokio_console;
    ///     tokio_console::init_with_config("0.0.0.0:6669")?;
    /// }
    /// ```
    pub fn init_with_config(server_addr: &str) -> anyhow::Result<()> {
        let builder = console_subscriber::ConsoleLayer::builder()
            .server_addr(server_addr.parse()?);

        builder.init();

        tracing::info!("🔍 tokio-console 已启动（自定义配置）");
        tracing::info!("📡 监听地址: {}", server_addr);
        tracing::info!("💡 运行 'tokio-console' 命令连接到监控界面");
        Ok(())
    }
}

// ============================================================================
// 开发环境追踪（Chrome Tracing、Perfetto 等）
// ============================================================================

#[cfg(feature = "dev-tracing")]
pub mod dev_tracing {
    use tracing_subscriber::{
        fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
    };
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// 全局追踪 ID 计数器
    static TRACE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// 生成唯一的追踪 ID
    ///
    /// 用于标识特定的方法调用，便于过滤和追踪完整的调用链路
    pub fn generate_trace_id() -> u64 {
        TRACE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// 检查是否应该追踪特定的方法
    ///
    /// 通过环境变量 `TRACE_METHODS` 控制，多个方法用逗号分隔
    ///
    /// # 示例
    ///
    /// ```bash
    /// TRACE_METHODS=dispatch,command cargo dev
    /// ```
    pub fn should_trace(method_name: &str) -> bool {
        if let Ok(trace_methods) = std::env::var("TRACE_METHODS") {
            if trace_methods == "*" {
                return true; // 追踪所有方法
            }
            trace_methods.split(',').any(|m| m.trim() == method_name)
        } else {
            true // 默认追踪所有（向后兼容）
        }
    }

    /// 检查是否应该追踪特定的 tr_id
    ///
    /// 通过环境变量 `TRACE_TR_ID` 控制
    ///
    /// # 示例
    ///
    /// ```bash
    /// TRACE_TR_ID=daaca572-1234-5678-9abc-def012345678 cargo dev
    /// ```
    pub fn should_trace_tr_id(tr_id: &str) -> bool {
        if let Ok(target_tr_id) = std::env::var("TRACE_TR_ID") {
            tr_id.starts_with(&target_tr_id)
        } else {
            true // 默认追踪所有
        }
    }

    /// 追踪输出格式
    #[derive(Debug, Clone)]
    pub enum TraceFormat {
        /// 控制台输出（带颜色，适合开发调试）
        Console,
        /// JSON 格式（适合日志分析）
        Json,
        /// Chrome Tracing 格式（适合性能可视化，推荐）
        #[cfg(feature = "dev-tracing-chrome")]
        Chrome,
        /// Perfetto 格式（适合性能可视化）
        #[cfg(feature = "dev-tracing-perfetto")]
        Perfetto,
    }

    /// Chrome Tracing Guard（需要保持到程序结束）
    #[cfg(feature = "dev-tracing-chrome")]
    pub struct ChromeTracingGuard {
        _guard: tracing_chrome::FlushGuard,
    }

    #[cfg(feature = "dev-tracing-chrome")]
    impl Drop for ChromeTracingGuard {
        fn drop(&mut self) {
            tracing::info!("🔄 正在刷新 Chrome Tracing 数据...");
        }
    }

    /// 追踪配置
    #[derive(Debug, Clone)]
    pub struct TraceConfig {
        /// 输出格式
        pub format: TraceFormat,
        /// 输出路径（None 表示输出到 stdout）
        pub output_path: Option<PathBuf>,
        /// 最大日志级别
        pub max_level: tracing::Level,
        /// 是否显示目标模块
        pub with_target: bool,
        /// 是否显示线程 ID
        pub with_thread_ids: bool,
        /// 是否显示文件名和行号
        pub with_file_line: bool,
    }

    impl Default for TraceConfig {
        fn default() -> Self {
            Self {
                format: TraceFormat::Console,
                output_path: None,
                max_level: tracing::Level::DEBUG,
                with_target: true,
                with_thread_ids: true,
                with_file_line: true,
            }
        }
    }

    impl TraceConfig {
        /// 创建控制台输出配置
        pub fn console() -> Self {
            Self { format: TraceFormat::Console, ..Default::default() }
        }

        /// 创建 JSON 输出配置
        pub fn json(output_path: impl Into<PathBuf>) -> Self {
            Self {
                format: TraceFormat::Json,
                output_path: Some(output_path.into()),
                ..Default::default()
            }
        }

        /// 创建 Chrome Tracing 输出配置
        #[cfg(feature = "dev-tracing-chrome")]
        pub fn chrome(output_path: impl Into<PathBuf>) -> Self {
            Self {
                format: TraceFormat::Chrome,
                output_path: Some(output_path.into()),
                ..Default::default()
            }
        }

        /// 创建 Perfetto 输出配置
        #[cfg(feature = "dev-tracing-perfetto")]
        pub fn perfetto(output_path: impl Into<PathBuf>) -> Self {
            Self {
                format: TraceFormat::Perfetto,
                output_path: Some(output_path.into()),
                ..Default::default()
            }
        }

        /// 设置最大日志级别
        pub fn with_max_level(
            mut self,
            level: tracing::Level,
        ) -> Self {
            self.max_level = level;
            self
        }
    }

    /// 追踪初始化结果
    pub enum TracingGuard {
        /// 无需 guard（Console/JSON/Perfetto）
        None,
        /// Chrome Tracing guard（需要保持到程序结束）
        #[cfg(feature = "dev-tracing-chrome")]
        Chrome(ChromeTracingGuard),
    }

    /// 初始化全局追踪
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use mf_core::tracing_init::dev_tracing::{init_tracing, TraceConfig};
    ///
    /// // 控制台输出
    /// let _guard = init_tracing(TraceConfig::console()).unwrap();
    ///
    /// // JSON 文件输出
    /// let _guard = init_tracing(TraceConfig::json("./logs/trace.json")).unwrap();
    ///
    /// // Chrome Tracing（需要保持 guard 到程序结束）
    /// let _guard = init_tracing(TraceConfig::chrome("./logs/trace.json")).unwrap();
    /// // ... 程序运行 ...
    /// // guard 在这里 drop，确保数据被刷新
    /// ```
    pub fn init_tracing(config: TraceConfig) -> anyhow::Result<TracingGuard> {
        match config.format {
            TraceFormat::Console => {
                init_console_tracing(config)?;
                Ok(TracingGuard::None)
            },
            TraceFormat::Json => {
                init_json_tracing(config)?;
                Ok(TracingGuard::None)
            },
            #[cfg(feature = "dev-tracing-chrome")]
            TraceFormat::Chrome => init_chrome_tracing(config),
            #[cfg(feature = "dev-tracing-perfetto")]
            TraceFormat::Perfetto => {
                init_perfetto_tracing(config)?;
                Ok(TracingGuard::None)
            },
        }
    }

    /// 初始化控制台追踪
    fn init_console_tracing(config: TraceConfig) -> anyhow::Result<()> {
        use tracing_subscriber::fmt::time::ChronoLocal;

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(config.max_level.as_str()));

        let fmt_layer = fmt::layer()
            .with_target(config.with_target)
            .with_thread_ids(config.with_thread_ids)
            .with_file(config.with_file_line)
            .with_line_number(config.with_file_line)
            .with_ansi(true)
            .with_timer(ChronoLocal::new("%H:%M:%S%.3f".to_string()))
            .with_span_events(fmt::format::FmtSpan::CLOSE) // 显示 span 关闭时的耗时
            .pretty();

        tracing_subscriber::registry().with(env_filter).with(fmt_layer).init();

        tracing::info!("🔍 开发追踪已启用（控制台模式）");
        tracing::info!("📊 日志级别: {}", config.max_level);
        tracing::info!("⏱️  显示 span 执行时间");
        Ok(())
    }

    /// 初始化 JSON 追踪
    fn init_json_tracing(config: TraceConfig) -> anyhow::Result<()> {
        let path = config
            .output_path
            .unwrap_or_else(|| PathBuf::from("./logs/trace.json"));

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::File::create(&path)?;

        let env_filter = EnvFilter::new(config.max_level.as_str());

        let fmt_layer = fmt::layer()
            .json()
            .with_writer(file)
            .with_target(config.with_target)
            .with_thread_ids(config.with_thread_ids)
            .with_file(config.with_file_line)
            .with_line_number(config.with_file_line);

        tracing_subscriber::registry().with(env_filter).with(fmt_layer).init();

        tracing::info!("🔍 开发追踪已启用（JSON 模式）");
        tracing::info!("📁 输出文件: {}", path.display());
        tracing::info!("📊 日志级别: {}", config.max_level);
        Ok(())
    }

    /// 初始化 Chrome Tracing 追踪
    #[cfg(feature = "dev-tracing-chrome")]
    fn init_chrome_tracing(
        config: TraceConfig
    ) -> anyhow::Result<TracingGuard> {
        let path = config
            .output_path
            .unwrap_or_else(|| PathBuf::from("./logs/trace.json"));

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 创建 Chrome Tracing layer（传递路径而不是文件对象）
        let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file(&path)
            .include_args(true) // 包含 span 参数
            .build();

        // 同时输出到控制台（简化版）
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(config.max_level.as_str()));

        let fmt_layer = fmt::layer()
            .with_target(config.with_target)
            .with_thread_ids(config.with_thread_ids)
            .with_ansi(true)
            .compact();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(chrome_layer)
            .init();

        tracing::info!("🔍 开发追踪已启用（Chrome Tracing 模式）");
        tracing::info!("📁 输出文件: {}", path.display());
        tracing::info!("📊 日志级别: {}", config.max_level);
        tracing::info!(
            "🌐 查看方式: 在 Chrome 浏览器中访问 chrome://tracing 并加载文件"
        );
        tracing::info!("📦 包含信息: Span 时序、参数、线程 ID、进程 ID");
        tracing::info!(
            "⚠️  重要: 请保持返回的 guard 直到程序结束，以确保数据被正确写入"
        );

        // 返回 guard，调用者需要保持它直到程序结束
        Ok(TracingGuard::Chrome(ChromeTracingGuard { _guard: guard }))
    }

    /// 初始化 Perfetto 追踪
    #[cfg(feature = "dev-tracing-perfetto")]
    fn init_perfetto_tracing(config: TraceConfig) -> anyhow::Result<()> {
        use std::fs::File;

        let path = config
            .output_path
            .unwrap_or_else(|| PathBuf::from("./logs/trace.perfetto"));

        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 创建文件用于 Perfetto 输出
        let file = File::create(&path)?;
        let perfetto_layer = tracing_perfetto::PerfettoLayer::new(file)
            .with_debug_annotations(true);

        // 同时输出到控制台（带时间戳）
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(config.max_level.as_str()));

        let fmt_layer = fmt::layer()
            .with_target(config.with_target)
            .with_thread_ids(config.with_thread_ids)
            .with_ansi(true)
            .compact();

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(perfetto_layer)
            .init();

        // 记录系统信息到追踪
        log_system_info();

        tracing::info!("🔍 开发追踪已启用（Perfetto 模式）");
        tracing::info!("📁 输出文件: {}", path.display());
        tracing::info!("📊 使用 https://ui.perfetto.dev/ 查看追踪数据");
        tracing::info!("📊 日志级别: {}", config.max_level);
        tracing::info!("💡 Perfetto 包含: span 时序、线程信息、系统资源");

        Ok(())
    }

    /// 记录系统信息到追踪（用于 Perfetto 分析）
    #[cfg(feature = "dev-tracing-perfetto")]
    fn log_system_info() {
        use std::thread;

        let process_id = std::process::id();
        let thread_id = thread::current().id();
        let thread_name =
            thread::current().name().unwrap_or("main").to_string();

        tracing::info!(
            process_id = process_id,
            thread_id = ?thread_id,
            thread_name = %thread_name,
            "系统信息"
        );

        // 记录 CPU 核心数
        let cpu_count = num_cpus::get();
        tracing::info!(cpu_count = cpu_count, "CPU 信息");
    }

    /// 快速初始化（使用默认配置）
    pub fn init_default() -> anyhow::Result<TracingGuard> {
        init_tracing(TraceConfig::default())
    }
}

// 生产环境：空实现，零开销
#[cfg(not(feature = "dev-tracing"))]
pub mod dev_tracing {
    use std::path::PathBuf;

    /// 空 Guard（生产环境）
    pub struct TracingGuard;

    #[derive(Debug, Clone)]
    pub enum TraceFormat {
        Console,
        Json,
    }

    #[derive(Debug, Clone)]
    pub struct TraceConfig {
        pub format: TraceFormat,
        pub output_path: Option<PathBuf>,
        pub max_level: (),
        pub with_target: bool,
        pub with_thread_ids: bool,
        pub with_file_line: bool,
    }

    impl Default for TraceConfig {
        fn default() -> Self {
            Self {
                format: TraceFormat::Console,
                output_path: None,
                max_level: (),
                with_target: false,
                with_thread_ids: false,
                with_file_line: false,
            }
        }
    }

    impl TraceConfig {
        pub fn console() -> Self {
            Self::default()
        }
        pub fn json(_path: impl Into<PathBuf>) -> Self {
            Self::default()
        }
        pub fn with_max_level(
            self,
            _level: (),
        ) -> Self {
            self
        }
    }

    /// 生产环境：什么都不做
    pub fn init_tracing(_config: TraceConfig) -> anyhow::Result<TracingGuard> {
        Ok(TracingGuard)
    }

    pub fn init_default() -> anyhow::Result<TracingGuard> {
        Ok(TracingGuard)
    }
}

// ============================================================================
// 便捷宏定义
// ============================================================================

/// 创建一个带唯一追踪 ID 的 span
///
/// 用于追踪特定方法调用的完整执行链路
///
/// # 示例
///
/// ```rust
/// use moduforge_core::traced_span;
///
/// pub async fn my_method(&self) -> Result<()> {
///     let _span = traced_span!("my_method");
///     // 这个方法内的所有子调用都会继承 trace_id
///     self.do_something().await?;
///     Ok(())
/// }
/// ```
///
/// 然后可以通过 grep 过滤特定的 trace_id：
///
/// ```bash
/// cargo dev 2>&1 | grep "trace_id=42"
/// ```
#[cfg(feature = "dev-tracing")]
#[macro_export]
macro_rules! traced_span {
    ($name:expr) => {{
        let trace_id = $crate::tracing_init::dev_tracing::generate_trace_id();
        tracing::info_span!($name, trace_id = trace_id)
    }};
    ($name:expr, $($field:tt)*) => {{
        let trace_id = $crate::tracing_init::dev_tracing::generate_trace_id();
        tracing::info_span!($name, trace_id = trace_id, $($field)*)
    }};
}

#[cfg(not(feature = "dev-tracing"))]
#[macro_export]
macro_rules! traced_span {
    ($name:expr) => {{}};
    ($name:expr, $($field:tt)*) => {{}};
}

/// 条件追踪宏 - 只在环境变量指定时才创建 span
///
/// 通过 `TRACE_METHODS` 环境变量控制要追踪的方法
///
/// # 示例
///
/// ```rust
/// use moduforge_core::trace_if_enabled;
///
/// pub async fn dispatch(&mut self, tr: Transaction) -> Result<()> {
///     let _span = trace_if_enabled!("dispatch", tr_id = %tr.id);
///     // 只有在 TRACE_METHODS=dispatch 时才会追踪
///     Ok(())
/// }
/// ```
///
/// 使用方式：
///
/// ```bash
/// # 只追踪 dispatch 方法
/// TRACE_METHODS=dispatch cargo dev
///
/// # 追踪多个方法
/// TRACE_METHODS=dispatch,command,apply_inner cargo dev
///
/// # 追踪所有方法
/// TRACE_METHODS=* cargo dev
/// ```
#[cfg(feature = "dev-tracing")]
#[macro_export]
macro_rules! trace_if_enabled {
    ($method:expr) => {{
        if $crate::tracing_init::dev_tracing::should_trace($method) {
            Some(tracing::info_span!($method).entered())
        } else {
            None
        }
    }};
    ($method:expr, $($field:tt)*) => {{
        if $crate::tracing_init::dev_tracing::should_trace($method) {
            Some(tracing::info_span!($method, $($field)*).entered())
        } else {
            None
        }
    }};
}

#[cfg(not(feature = "dev-tracing"))]
#[macro_export]
macro_rules! trace_if_enabled {
    ($method:expr) => {
        None
    };
    ($method:expr, $($field:tt)*) => {
        None
    };
}

// Copyright 2018-2025 the Deno authors. MIT license.

use std::cell::RefCell;
use std::thread;

use deno_core::ModuleSpecifier;
use deno_core::v8;
use deno_telemetry::OtelConfig;
use deno_terminal::colors;
use serde::Serialize;

/// 该工作器的执行模式。某些模式可能具有隐式行为。
#[derive(Copy, Clone)]
pub enum WorkerExecutionMode {
    /// 无特殊行为。
    None,

    /// 在工作器中运行。
    Worker,
    /// `deno run`
    Run,
    /// `deno repl`
    Repl,
    /// `deno eval`
    Eval,
    /// `deno test`
    Test,
    /// `deno bench`
    Bench,
    /// `deno serve`
    ServeMain {
        worker_count: usize,
    },
    ServeWorker {
        worker_index: usize,
    },
    /// `deno jupyter`
    Jupyter,
    /// `deno deploy`
    Deploy,
}

impl WorkerExecutionMode {
    pub fn discriminant(&self) -> u8 {
        match self {
            WorkerExecutionMode::None => 0,
            WorkerExecutionMode::Worker => 1,
            WorkerExecutionMode::Run => 2,
            WorkerExecutionMode::Repl => 3,
            WorkerExecutionMode::Eval => 4,
            WorkerExecutionMode::Test => 5,
            WorkerExecutionMode::Bench => 6,
            WorkerExecutionMode::ServeMain { .. }
            | WorkerExecutionMode::ServeWorker { .. } => 7,
            WorkerExecutionMode::Jupyter => 8,
            WorkerExecutionMode::Deploy => 9,
        }
    }
}

/// 在工作器中打印诊断日志消息、警告或错误时使用的日志级别。
///
/// 注意：这与log crate的日志级别是分离的，此crate中的Rust代码
/// 将遵循该值。要指定该值，请使用`log::set_max_level`。
#[derive(Debug, Default, Clone, Copy)]
pub enum WorkerLogLevel {
    // 警告：确保与JS值保持同步（搜索LogLevel）。
    Error = 1,
    Warn = 2,
    #[default]
    Info = 3,
    Debug = 4,
}

impl From<log::Level> for WorkerLogLevel {
    fn from(value: log::Level) -> Self {
        match value {
            log::Level::Error => WorkerLogLevel::Error,
            log::Level::Warn => WorkerLogLevel::Warn,
            log::Level::Info => WorkerLogLevel::Info,
            log::Level::Debug => WorkerLogLevel::Debug,
            log::Level::Trace => WorkerLogLevel::Debug,
        }
    }
}

/// MainWorker和WebWorker的通用启动选项
#[derive(Clone)]
pub struct BootstrapOptions {
    pub deno_version: String,
    /// 在JS运行时中设置`Deno.args`。
    pub args: Vec<String>,
    pub cpu_count: usize,
    pub log_level: WorkerLogLevel,
    pub enable_op_summary_metrics: bool,
    pub enable_testing_features: bool,
    pub locale: String,
    pub location: Option<ModuleSpecifier>,
    pub color_level: deno_terminal::colors::ColorLevel,
    // --unstable-* 标志
    pub unstable_features: Vec<i32>,
    pub user_agent: String,
    pub inspect: bool,
    /// 如果这是一个`deno compile`编译的可执行文件。
    pub is_standalone: bool,
    pub has_node_modules_dir: bool,
    pub argv0: Option<String>,
    pub node_debug: Option<String>,
    pub node_ipc_fd: Option<i64>,
    pub mode: WorkerExecutionMode,
    pub no_legacy_abort: bool,
    // 由`deno serve`使用
    pub serve_port: Option<u16>,
    pub serve_host: Option<String>,
    pub auto_serve: bool,
    pub otel_config: OtelConfig,
    pub close_on_idle: bool,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        let cpu_count =
            thread::available_parallelism().map(|p| p.get()).unwrap_or(1);

        // 此版本不正确，因为它是deno_runtime的版本，
        // 实现者应该提供一个合理的用户代理
        let runtime_version = env!("CARGO_PKG_VERSION");
        let user_agent = format!("Deno/{runtime_version}");

        Self {
            deno_version: runtime_version.to_string(),
            user_agent,
            cpu_count,
            color_level: colors::get_color_level(),
            enable_op_summary_metrics: false,
            enable_testing_features: false,
            log_level: Default::default(),
            locale: "en".to_string(),
            location: Default::default(),
            unstable_features: Default::default(),
            inspect: false,
            args: Default::default(),
            is_standalone: false,
            auto_serve: false,
            has_node_modules_dir: false,
            argv0: None,
            node_debug: None,
            node_ipc_fd: None,
            mode: WorkerExecutionMode::None,
            no_legacy_abort: false,
            serve_port: Default::default(),
            serve_host: Default::default(),
            otel_config: Default::default(),
            close_on_idle: false,
        }
    }
}

/// 这是我们用来将上述`BootstrapOptions`结构体的内容
/// 序列化为V8形式的结构体。虽然`serde_v8`不如手动编码快，
/// 但在序列化像这样的大元组时它"足够快"，不会出现在火焰图上。
///
/// 注意：这里的一些字段来自进程和环境，
/// 不是来自底层的`BootstrapOptions`。
///
/// 保持与`99_main.js`同步。
#[derive(Serialize)]
struct BootstrapV8<'a>(
    // deno版本
    &'a str,
    // 位置
    Option<&'a str>,
    // 细粒度不稳定标志
    &'a [i32],
    // 检查
    bool,
    // 启用测试功能
    bool,
    // 是否有node_modules目录
    bool,
    // argv0参数
    Option<&'a str>,
    // node调试
    Option<&'a str>,
    // 模式
    i32,
    // 服务端口
    u16,
    // 服务主机
    Option<&'a str>,
    // 服务是主服务
    bool,
    // 服务工作器数量
    Option<usize>,
    // OTEL配置
    Box<[u8]>,
    // 空闲时关闭
    bool,
    // 是独立模式
    bool,
    // 自动服务
    bool,
);

impl BootstrapOptions {
    /// 返回此结构体的v8等价物。
    pub fn as_v8<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> v8::Local<'s, v8::Value> {
        let scope = RefCell::new(scope);
        let ser = deno_core::serde_v8::Serializer::new(&scope);

        let bootstrap = BootstrapV8(
            &self.deno_version,
            self.location.as_ref().map(|l| l.as_str()),
            self.unstable_features.as_ref(),
            self.inspect,
            self.enable_testing_features,
            self.has_node_modules_dir,
            self.argv0.as_deref(),
            self.node_debug.as_deref(),
            self.mode.discriminant() as _,
            self.serve_port.unwrap_or_default(),
            self.serve_host.as_deref(),
            matches!(self.mode, WorkerExecutionMode::ServeMain { .. }),
            match self.mode {
                WorkerExecutionMode::ServeMain { worker_count } => {
                    Some(worker_count)
                },
                WorkerExecutionMode::ServeWorker { worker_index } => {
                    Some(worker_index)
                },
                _ => None,
            },
            self.otel_config.as_v8(),
            self.close_on_idle,
            self.is_standalone,
            self.auto_serve,
        );

        bootstrap.serialize(ser).unwrap()
    }
}

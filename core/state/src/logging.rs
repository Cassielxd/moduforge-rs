use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self},
    EnvFilter,
    prelude::*,
};

/// 初始化日志系统
///
/// # Arguments
/// * `log_level` - 日志级别，默认为 INFO
/// * `log_file` - 日志文件路径，如果为 None 则只输出到控制台
pub fn init_logging(
    log_level: Option<Level>,
    log_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    // 设置日志级别
    let level = log_level.unwrap_or(Level::INFO);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("info,{}", level.as_str().to_lowercase()))
    });

    // 创建控制台输出
    let console_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .with_file(true)
        .with_line_number(true);

    // 创建订阅者
    let subscriber =
        tracing_subscriber::registry().with(env_filter).with(console_layer);

    // 如果指定了日志文件，添加文件输出
    if let Some(log_file) = log_file {
        let directory =
            log_file.parent().unwrap_or(&PathBuf::from(".")).to_path_buf();
        let file_appender =
            tracing_appender::rolling::RollingFileAppender::builder()
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .filename_prefix("moduforge")
                .filename_suffix("log")
                .build(&directory)?;

        let file_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_writer(file_appender);

        subscriber.with(file_layer).init();
    } else {
        subscriber.init();
    }

    Ok(())
}

/// 获取当前日志级别
pub fn get_log_level() -> Level {
    tracing::level_filters::LevelFilter::current()
        .into_level()
        .unwrap_or(Level::INFO)
}

/// 设置日志级别
pub fn set_log_level(level: Level) {
    tracing_subscriber::filter::LevelFilter::from_level(level);
}

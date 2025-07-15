use metrics::{counter, gauge, histogram};

/// 已提交任务总数
pub const TASKS_SUBMITTED_TOTAL: &str = "core.tasks.submitted.total";
/// 已处理任务总数
pub const TASKS_PROCESSED_TOTAL: &str = "core.tasks.processed.total";
/// 任务处理耗时（秒）
pub const TASK_PROCESSING_DURATION_SECONDS: &str =
    "core.task.processing.duration.seconds";
/// 当前任务队列大小
pub const QUEUE_SIZE: &str = "core.queue.size";
/// 当前正在处理的任务数
pub const PROCESSING_TASKS: &str = "core.processing.tasks";
/// 任务重试总数
pub const TASKS_RETRIED_TOTAL: &str = "core.tasks.retried.total";

// 编辑器/运行时 指标
/// 编辑器创建耗时（秒）
pub const EDITOR_CREATION_DURATION_SECONDS: &str =
    "core.editor.creation.duration.seconds";
/// 已执行命令总数
pub const COMMANDS_EXECUTED_TOTAL: &str = "core.commands.executed.total";
/// 已分发事务总数
pub const TRANSACTIONS_DISPATCHED_TOTAL: &str =
    "core.transactions.dispatched.total";
/// 中间件执行耗时（秒）
pub const MIDDLEWARE_EXECUTION_DURATION_SECONDS: &str =
    "core.middleware.execution.duration.seconds";
/// 历史操作（撤销/重做）总数
pub const HISTORY_OPERATIONS_TOTAL: &str = "core.history.operations.total";
/// 已分发事件总数
pub const EVENTS_EMITTED_TOTAL: &str = "core.events.emitted.total";

// 插件管理器 指标
/// 插件管理器创建耗时（秒）
pub const EXTENSION_MANAGER_CREATION_DURATION_SECONDS: &str =
    "core.extension_manager.creation.duration.seconds";
/// 已加载扩展总数
pub const EXTENSIONS_LOADED_TOTAL: &str = "core.extensions.loaded.total";
/// 已加载插件总数
pub const PLUGINS_LOADED_TOTAL: &str = "core.plugins.loaded.total";
/// XML解析耗时（秒）
pub const XML_PARSING_DURATION_SECONDS: &str = "core.xml.parsing.duration.seconds";

pub fn register_metrics() {
    //
}

pub fn task_submitted() {
    counter!(TASKS_SUBMITTED_TOTAL).increment(1);
}

pub fn task_processed(status: &str) {
    counter!(TASKS_PROCESSED_TOTAL, "status" => status.to_string())
        .increment(1);
}

pub fn task_processing_duration(duration: std::time::Duration) {
    histogram!(TASK_PROCESSING_DURATION_SECONDS).record(duration.as_secs_f64());
}

pub fn set_queue_size(size: usize) {
    gauge!(QUEUE_SIZE).set(size as f64);
}

pub fn increment_processing_tasks() {
    gauge!(PROCESSING_TASKS).increment(1.0);
}

pub fn decrement_processing_tasks() {
    gauge!(PROCESSING_TASKS).decrement(1.0);
}

pub fn task_retried() {
    counter!(TASKS_RETRIED_TOTAL).increment(1);
}

pub fn editor_creation_duration(duration: std::time::Duration) {
    histogram!(EDITOR_CREATION_DURATION_SECONDS).record(duration.as_secs_f64());
}

pub fn command_executed(name: &str) {
    counter!(COMMANDS_EXECUTED_TOTAL, "command_name" => name.to_string())
        .increment(1);
}

pub fn transaction_dispatched() {
    counter!(TRANSACTIONS_DISPATCHED_TOTAL).increment(1);
}

pub fn middleware_execution_duration(
    duration: std::time::Duration,
    mw_type: &str,
    mw_name: &str,
) {
    histogram!(
        MIDDLEWARE_EXECUTION_DURATION_SECONDS,
        "type" => mw_type.to_string(),
        "middleware_name" => mw_name.to_string()
    )
    .record(duration.as_secs_f64());
}

pub fn history_operation(op: &str) {
    counter!(HISTORY_OPERATIONS_TOTAL, "operation" => op.to_string())
        .increment(1);
}

pub fn event_emitted(event_type: &str) {
    counter!(EVENTS_EMITTED_TOTAL, "event_type" => event_type.to_string())
        .increment(1);
}

pub fn extension_manager_creation_duration(duration: std::time::Duration) {
    histogram!(EXTENSION_MANAGER_CREATION_DURATION_SECONDS)
        .record(duration.as_secs_f64());
}

pub fn extensions_loaded(count: u64) {
    counter!(EXTENSIONS_LOADED_TOTAL).increment(count);
}

pub fn plugins_loaded(count: u64) {
    counter!(PLUGINS_LOADED_TOTAL).increment(count);
}

pub fn xml_parsing_duration(duration: std::time::Duration) {
    histogram!(XML_PARSING_DURATION_SECONDS).record(duration.as_secs_f64());
}

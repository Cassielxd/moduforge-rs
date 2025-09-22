// Copyright 2018-2025 the Deno authors. MIT license.
use std::fmt::Debug;
use std::str::FromStr;

use deno_core::unsync::MaskFutureAsSend;
#[cfg(tokio_unstable)]
use tokio_metrics::RuntimeMonitor;

/// tokio的默认配置。在未来，根据平台和/或CPU布局，
/// 此方法可能具有不同的默认值。
const fn tokio_configuration() -> (u32, u32, usize) {
    (61, 31, 1024)
}

fn tokio_env<T: FromStr>(
    name: &'static str,
    default: T,
) -> T
where
    <T as FromStr>::Err: Debug,
{
    match std::env::var(name) {
        Ok(value) => value.parse().unwrap(),
        Err(_) => default,
    }
}

pub fn create_basic_runtime() -> tokio::runtime::Runtime {
    let (event_interval, global_queue_interval, max_io_events_per_tick) =
        tokio_configuration();

    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .event_interval(tokio_env("DENO_TOKIO_EVENT_INTERVAL", event_interval))
        .global_queue_interval(tokio_env(
            "DENO_TOKIO_GLOBAL_QUEUE_INTERVAL",
            global_queue_interval,
        ))
        .max_io_events_per_tick(tokio_env(
            "DENO_TOKIO_MAX_IO_EVENTS_PER_TICK",
            max_io_events_per_tick,
        ))
        // This limits the number of threads for blocking operations (like for
        // synchronous fs ops) or CPU bound tasks like when we run dprint in
        // parallel for deno fmt.
        // The default value is 512, which is an unhelpfully large thread pool. We
        // don't ever want to have more than a couple dozen threads.
        .max_blocking_threads(if cfg!(windows) {
            // on windows, tokio uses blocking tasks for child process IO, make sure
            // we have enough available threads for other tasks to run
            4 * std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(8)
        } else {
            32
        })
        .build()
        .unwrap()
}

#[inline(always)]
fn create_and_run_current_thread_inner<F, R>(
    future: F,
    metrics_enabled: bool,
) -> R
where
    F: std::future::Future<Output = R> + 'static,
    R: Send + 'static,
{
    let rt = create_basic_runtime();

    // 由于这是主要的future，我们希望在调试模式下将其装箱，因为它往往相当大，
    // 编译器不会优化重复的复制。我们还使这个运行时工厂函数为#[inline(always)]
    // 以避免在堆栈上保持未装箱、未使用的future。

    #[cfg(debug_assertions)]
    // 安全性：这保证在当前线程执行器上运行
    let future = Box::pin(unsafe { MaskFutureAsSend::new(future) });

    #[cfg(not(debug_assertions))]
    // 安全性：这保证在当前线程执行器上运行
    let future = unsafe { MaskFutureAsSend::new(future) };

    #[cfg(tokio_unstable)]
    let join_handle = if metrics_enabled {
        rt.spawn(async move {
            let metrics_interval: u64 =
                std::env::var("DENO_TOKIO_METRICS_INTERVAL")
                    .ok()
                    .and_then(|val| val.parse().ok())
                    .unwrap_or(1000);
            let handle = tokio::runtime::Handle::current();
            let runtime_monitor = RuntimeMonitor::new(&handle);
            tokio::spawn(async move {
                #[allow(clippy::print_stderr)]
                for interval in runtime_monitor.intervals() {
                    eprintln!("{:#?}", interval);
                    // wait 500ms
                    tokio::time::sleep(std::time::Duration::from_millis(
                        metrics_interval,
                    ))
                    .await;
                }
            });
            future.await
        })
    } else {
        rt.spawn(future)
    };

    #[cfg(not(tokio_unstable))]
    let join_handle = rt.spawn(future);

    let r = rt.block_on(join_handle).unwrap().into_inner();
    // 强制关闭运行时 - 此时我们已完成JS代码的执行，
    // 但可能还有一些被创建并后来"取消引用"的突出阻塞任务。
    // 它们不会自己终止，所以我们在此时强制终止Tokio运行时。
    rt.shutdown_background();
    r
}

#[inline(always)]
pub fn create_and_run_current_thread<F, R>(future: F) -> R
where
    F: std::future::Future<Output = R> + 'static,
    R: Send + 'static,
{
    create_and_run_current_thread_inner(future, false)
}

#[inline(always)]
pub fn create_and_run_current_thread_with_maybe_metrics<F, R>(future: F) -> R
where
    F: std::future::Future<Output = R> + 'static,
    R: Send + 'static,
{
    let metrics_enabled = std::env::var("DENO_TOKIO_METRICS").ok().is_some();
    create_and_run_current_thread_inner(future, metrics_enabled)
}

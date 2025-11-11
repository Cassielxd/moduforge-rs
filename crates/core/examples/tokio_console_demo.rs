//! tokio-console 实时监控演示
//!
//! 此示例演示如何使用 tokio-console 实时监控异步任务。
//!
//! # 运行步骤
//!
//! 1. 启用 dev-console feature 运行此示例：
//! ```bash
//! cargo run --example tokio_console_demo --features dev-console
//! ```
//!
//! 2. 在另一个终端安装并运行 tokio-console 客户端：
//! ```bash
//! # 安装 tokio-console（只需一次）
//! cargo install tokio-console
//!
//! # 连接到监控服务器
//! tokio-console
//! ```
//!
//! 3. 在 tokio-console 界面中你可以看到：
//!    - 所有运行中的异步任务
//!    - 任务的状态（运行/等待/空闲）
//!    - 任务的执行时间统计
//!    - 任务的唤醒次数
//!    - 资源使用情况
//!
//! # 监控内容
//!
//! 此示例会创建多种类型的异步任务：
//! - 快速任务（立即完成）
//! - 慢速任务（模拟耗时操作）
//! - 周期性任务（定时执行）
//! - 并发任务（多个任务同时运行）
//! - Actor 系统任务（如果使用 Actor 运行时）

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

#[cfg(feature = "dev-console")]
use mf_core::tracing_init::tokio_console;

/// 模拟快速任务
async fn fast_task(id: u32) {
    info!("快速任务 {} 开始", id);
    sleep(Duration::from_millis(10)).await;
    info!("快速任务 {} 完成", id);
}

/// 模拟慢速任务
async fn slow_task(id: u32) {
    info!("慢速任务 {} 开始", id);
    sleep(Duration::from_secs(2)).await;
    info!("慢速任务 {} 完成", id);
}

/// 模拟周期性任务
async fn periodic_task(
    id: u32,
    interval_ms: u64,
) {
    info!("周期性任务 {} 启动，间隔 {}ms", id, interval_ms);
    for i in 0..10 {
        sleep(Duration::from_millis(interval_ms)).await;
        info!("周期性任务 {} - 第 {} 次执行", id, i + 1);
    }
    info!("周期性任务 {} 完成", id);
}

/// 模拟 CPU 密集型任务
async fn cpu_intensive_task(id: u32) {
    info!("CPU 密集型任务 {} 开始", id);

    // 模拟计算密集型操作
    tokio::task::spawn_blocking(move || {
        let mut sum = 0u64;
        for i in 0..10_000_000 {
            sum = sum.wrapping_add(i);
        }
        info!("CPU 密集型任务 {} 计算结果: {}", id, sum);
    })
    .await
    .unwrap();

    info!("CPU 密集型任务 {} 完成", id);
}

/// 模拟有依赖关系的任务链
async fn task_chain(id: u32) {
    info!("任务链 {} 开始", id);

    // 第一步
    info!("任务链 {} - 步骤 1: 准备数据", id);
    sleep(Duration::from_millis(100)).await;

    // 第二步
    info!("任务链 {} - 步骤 2: 处理数据", id);
    sleep(Duration::from_millis(200)).await;

    // 第三步
    info!("任务链 {} - 步骤 3: 保存结果", id);
    sleep(Duration::from_millis(150)).await;

    info!("任务链 {} 完成", id);
}

/// 模拟可能阻塞的任务
async fn potentially_blocking_task(id: u32) {
    warn!("⚠️  潜在阻塞任务 {} 开始（这会在 tokio-console 中显示为警告）", id);

    // 故意在异步上下文中进行同步阻塞操作（不推荐）
    // tokio-console 会检测到这个问题
    std::thread::sleep(Duration::from_millis(500));

    warn!("⚠️  潜在阻塞任务 {} 完成", id);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 tokio-console
    #[cfg(feature = "dev-console")]
    {
        tokio_console::init()?;
        info!("🚀 tokio-console 演示程序启动");
        info!("📊 请在另一个终端运行 'tokio-console' 查看实时监控");
        info!("");
    }

    #[cfg(not(feature = "dev-console"))]
    {
        // 如果没有启用 dev-console feature，使用普通的日志
        tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
        warn!("⚠️  未启用 dev-console feature");
        warn!("请使用以下命令运行：");
        warn!("cargo run --example tokio_console_demo --features dev-console");
        return Ok(());
    }

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("第 1 阶段：快速任务（10个并发）");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut handles = vec![];
    for i in 0..10 {
        handles.push(tokio::spawn(fast_task(i)));
    }
    for handle in handles {
        handle.await?;
    }

    sleep(Duration::from_secs(1)).await;

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("第 2 阶段：慢速任务（3个并发）");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut handles = vec![];
    for i in 0..3 {
        handles.push(tokio::spawn(slow_task(i)));
    }
    for handle in handles {
        handle.await?;
    }

    sleep(Duration::from_secs(1)).await;

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("第 3 阶段：周期性任务（3个不同频率）");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut handles = vec![];
    handles.push(tokio::spawn(periodic_task(1, 100)));
    handles.push(tokio::spawn(periodic_task(2, 200)));
    handles.push(tokio::spawn(periodic_task(3, 300)));

    for handle in handles {
        handle.await?;
    }

    sleep(Duration::from_secs(1)).await;

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("第 4 阶段：CPU 密集型任务（2个）");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut handles = vec![];
    for i in 0..2 {
        handles.push(tokio::spawn(cpu_intensive_task(i)));
    }
    for handle in handles {
        handle.await?;
    }

    sleep(Duration::from_secs(1)).await;

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("第 5 阶段：任务链（3个串行任务）");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut handles = vec![];
    for i in 0..3 {
        handles.push(tokio::spawn(task_chain(i)));
    }
    for handle in handles {
        handle.await?;
    }

    sleep(Duration::from_secs(1)).await;

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("第 6 阶段：潜在阻塞任务（演示问题检测）");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let handle = tokio::spawn(potentially_blocking_task(1));
    handle.await?;

    sleep(Duration::from_secs(1)).await;

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("✅ 所有任务完成！");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("💡 提示：");
    info!("  - 在 tokio-console 中按 'h' 查看帮助");
    info!("  - 按 't' 切换到任务视图");
    info!("  - 按 'r' 切换到资源视图");
    info!("  - 按 'q' 退出");
    info!("");
    info!("程序将在 10 秒后退出，请在 tokio-console 中查看统计信息...");

    sleep(Duration::from_secs(10)).await;

    Ok(())
}

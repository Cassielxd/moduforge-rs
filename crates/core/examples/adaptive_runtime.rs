//! 自适应运行时示例
//!
//! 演示如何使用自适应运行时配置功能：
//! 1. 完全自动模式 - 检测系统资源并自动选择运行时
//! 2. 查看系统资源信息
//! 3. 查看自动生成的配置
//! 4. 手动指定运行时类型

use mf_core::{
    AdaptiveRuntimeSelector, ForgeRuntimeBuilder, RuntimeType, SystemResources,
    ForgeResult,
};

#[tokio::main]
async fn main() -> ForgeResult<()> {
    println!("=== ModuForge 自适应运行时示例 ===\n");

    // 示例1: 检测系统资源
    example_detect_system_resources();

    // 示例2: 查看自动生成的配置
    example_show_adaptive_config();

    // 示例3: 完全自动模式（推荐）
    example_auto_runtime().await?;

    // 示例4: 手动指定运行时类型
    example_manual_runtime_type().await?;

    // 示例5: 使用运行时执行操作
    example_use_runtime().await?;

    Ok(())
}

/// 示例1: 检测系统资源
fn example_detect_system_resources() {
    println!("📊 示例1: 检测系统资源\n");

    let resources = SystemResources::detect();

    println!("系统配置信息:");
    println!("  CPU 物理核心数: {}", resources.cpu_cores);
    println!("  CPU 逻辑线程数: {}", resources.cpu_threads);
    println!(
        "  总内存: {} GB ({} MB)",
        resources.total_memory_mb / 1024,
        resources.total_memory_mb
    );
    println!(
        "  可用内存: {} GB ({} MB)",
        resources.available_memory_mb / 1024,
        resources.available_memory_mb
    );
    println!(
        "  资源等级: {} ({})",
        resources.resource_tier(),
        resources.tier_description()
    );

    let recommended = AdaptiveRuntimeSelector::select_runtime(&resources);
    println!("  推荐运行时: {:?}", recommended);

    println!();
}

/// 示例2: 查看自动生成的配置
fn example_show_adaptive_config() {
    println!("⚙️  示例2: 自动生成的配置\n");

    let resources = SystemResources::detect();
    let config = AdaptiveRuntimeSelector::generate_config(&resources);

    println!("基于系统资源的优化配置:");
    println!("  运行时类型: {:?}", config.runtime.runtime_type);
    println!("  最大并发任务数: {}", config.processor.max_concurrent_tasks);
    println!("  任务队列大小: {}", config.processor.max_queue_size);
    println!("  任务超时: {:?}", config.processor.task_timeout);
    println!("  中间件超时: {} ms", config.performance.middleware_timeout_ms);
    println!(
        "  任务接收超时: {} ms",
        config.performance.task_receive_timeout_ms
    );
    println!(
        "  性能监控: {}",
        if config.performance.enable_monitoring { "启用" } else { "禁用" }
    );
    println!(
        "  指标采样率: {}%",
        (config.performance.metrics_sampling_rate * 100.0) as u32
    );
    println!("  事件队列大小: {}", config.event.max_queue_size);
    println!("  历史记录条数: {}", config.history.max_entries);
    println!("  缓存条目数: {}", config.cache.max_entries);

    println!();
}

/// 示例3: 完全自动模式
async fn example_auto_runtime() -> ForgeResult<()> {
    println!("🎯 示例3: 完全自动模式（推荐）\n");

    println!("创建运行时（自动检测系统资源）...");
    let mut runtime = ForgeRuntimeBuilder::auto(None).await?;

    println!("✅ 运行时创建成功！");
    println!();

    // 清理
    runtime.destroy().await?;
    Ok(())
}

/// 示例4: 手动指定运行时类型
async fn example_manual_runtime_type() -> ForgeResult<()> {
    println!("🔧 示例4: 手动指定运行时类型\n");

    // 强制使用Async运行时
    println!("创建运行时（强制使用Async类型）...");
    let mut runtime =
        ForgeRuntimeBuilder::with_type(RuntimeType::Async, None).await?;

    println!("✅ Async运行时创建成功！");
    println!();

    // 清理
    runtime.destroy().await?;
    Ok(())
}

/// 示例5: 使用运行时执行操作
async fn example_use_runtime() -> ForgeResult<()> {
    println!("💻 示例5: 使用运行时执行操作\n");

    // 创建运行时
    let mut runtime = ForgeRuntimeBuilder::auto(None).await?;

    // 获取当前状态
    let state = runtime.get_state().await?;
    println!("当前状态版本: {}", state.version);

    // 创建事务
    let tr = runtime.get_tr().await?;
    println!("事务创建成功，步骤数: {}", tr.steps.len());

    // 清理
    runtime.destroy().await?;
    println!("✅ 运行时已销毁");
    println!();

    Ok(())
}

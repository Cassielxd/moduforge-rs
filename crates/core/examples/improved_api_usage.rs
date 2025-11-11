//! 改进的 API 使用示例
//!
//! 展示新的统一、流畅的运行时构建 API

use mf_core::{
    ForgeRuntimeBuilder, RuntimeType, Environment, types::Content, ForgeResult,
};

#[tokio::main]
async fn main() -> ForgeResult<()> {
    println!("=== ModuForge 改进的 API 使用示例 ===\n");

    // 示例 1: 最简单的用法（推荐）
    example_1_simplest().await?;

    // 示例 2: 指定运行时类型
    example_2_specify_runtime().await?;

    // 示例 3: 链式配置
    example_3_fluent_config().await?;

    // 示例 4: 从配置文件加载
    // example_4_from_config_file().await?;

    // 示例 5: 使用 XML Schema
    example_5_with_schema().await?;

    // 示例 6: 生产环境配置
    example_6_production().await?;

    // 示例 7: 运行时类型匹配
    example_7_runtime_matching().await?;

    Ok(())
}

/// 示例 1: 最简单的用法
///
/// 自动检测系统资源，选择最优运行时和配置
async fn example_1_simplest() -> ForgeResult<()> {
    println!("📝 示例 1: 最简单的用法\n");

    let mut runtime = ForgeRuntimeBuilder::new().build().await?;

    println!("✅ 运行时创建成功");
    println!("   类型: {:?}", runtime.runtime_type());

    // 使用运行时
    let state = runtime.get_state().await?;
    println!("   文档节点数: {}", state.doc().size());

    println!();
    Ok(())
}

/// 示例 2: 指定运行时类型
///
/// 手动指定使用哪种运行时
async fn example_2_specify_runtime() -> ForgeResult<()> {
    println!("📝 示例 2: 指定运行时类型\n");

    // 明确使用 Async 运行时
    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Async)
        .build()
        .await?;

    println!("✅ 运行时创建成功");
    println!("   类型: {:?}", runtime.runtime_type());

    // 可以获取具体类型的引用
    if let Some(async_rt) = runtime.as_async() {
        println!("   这是一个异步运行时");
    }

    println!();
    Ok(())
}

/// 示例 3: 链式配置
///
/// 使用流畅的链式 API 配置运行时
async fn example_3_fluent_config() -> ForgeResult<()> {
    println!("📝 示例 3: 链式配置\n");

    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Async)
        .max_concurrent_tasks(20)
        .queue_size(5000)
        .enable_monitoring(true)
        .middleware_timeout_ms(1000)
        .history_limit(1000)
        .build()
        .await?;

    println!("✅ 运行时创建成功（自定义配置）");
    println!("   类型: {:?}", runtime.runtime_type());

    println!();
    Ok(())
}

/// 示例 4: 从配置文件加载
///
/// 从 TOML 或 JSON 配置文件加载配置
#[allow(dead_code)]
async fn example_4_from_config_file() -> ForgeResult<()> {
    println!("📝 示例 4: 从配置文件加载\n");

    // 假设有一个 config.toml 文件
    let mut runtime = ForgeRuntimeBuilder::from_config_file("config.toml")
        .await?
        .build()
        .await?;

    println!("✅ 从配置文件创建成功");
    println!("   类型: {:?}", runtime.runtime_type());

    println!();
    Ok(())
}

/// 示例 5: 使用 XML Schema
///
/// 从 XML Schema 文件加载节点和标记定义
async fn example_5_with_schema() -> ForgeResult<()> {
    println!("📝 示例 5: 使用 XML Schema\n");

    // 注意：这需要实际的 schema 文件存在
    let result =
        ForgeRuntimeBuilder::new().schema_path("schema/main.xml").build().await;

    match result {
        Ok(mut runtime) => {
            println!("✅ 使用 Schema 创建成功");
            println!("   类型: {:?}", runtime.runtime_type());

            let schema = runtime.schema().await?;
            println!("   Schema 已加载");
        },
        Err(e) => {
            println!("⚠️  Schema 文件不存在或加载失败: {}", e);
            println!("   这是正常的，如果没有 schema 文件");
        },
    }

    println!();
    Ok(())
}

/// 示例 6: 生产环境配置
///
/// 使用生产环境的预设配置
async fn example_6_production() -> ForgeResult<()> {
    println!("📝 示例 6: 生产环境配置\n");

    let mut runtime = ForgeRuntimeBuilder::new()
        .environment(Environment::Production)
        .runtime_type(RuntimeType::Actor) // 生产环境使用 Actor 运行时
        .enable_monitoring(true) // 启用监控
        .build()
        .await?;

    println!("✅ 生产环境运行时创建成功");
    println!("   类型: {:?}", runtime.runtime_type());

    println!();
    Ok(())
}

/// 示例 7: 运行时类型匹配
///
/// 展示如何根据运行时类型执行不同的操作
async fn example_7_runtime_matching() -> ForgeResult<()> {
    println!("📝 示例 7: 运行时类型匹配\n");

    let mut runtime = ForgeRuntimeBuilder::new().build().await?;

    // 使用 match 进行类型匹配
    match &runtime {
        mf_core::runtime::builder::AnyRuntime::Sync(rt) => {
            println!("✅ 使用同步运行时");
            println!("   适合: 简单场景、低配机器");
        },
        mf_core::runtime::builder::AnyRuntime::Async(rt) => {
            println!("✅ 使用异步运行时");
            println!("   适合: 中等并发、标准配置");
        },
        mf_core::runtime::builder::AnyRuntime::Actor(rt) => {
            println!("✅ 使用 Actor 运行时");
            println!("   适合: 高并发、高配机器");
        },
    }

    // 或者使用辅助方法
    if let Some(_async_rt) = runtime.as_async() {
        println!("   这是异步运行时的特定操作");
    }

    println!();
    Ok(())
}

/// 示例 8: 对比旧 API 和新 API
#[allow(dead_code)]
fn api_comparison() {
    println!("=== API 对比 ===\n");

    println!("旧 API（复杂）:");
    println!("```rust");
    println!("// 方式 1: 直接创建");
    println!("let runtime = ForgeRuntime::create(options).await?;");
    println!();
    println!("// 方式 2: 使用构建器");
    println!("let runtime = ForgeRuntimeBuilder::auto(None).await?;");
    println!();
    println!("// 方式 3: 指定类型");
    println!(
        "let runtime = ForgeRuntimeBuilder::with_type(RuntimeType::Actor, None).await?;"
    );
    println!();
    println!("// 方式 4: 从配置");
    println!(
        "let runtime = ForgeRuntimeBuilder::from_config(config, Some(options)).await?;"
    );
    println!("```\n");

    println!("新 API（统一、流畅）:");
    println!("```rust");
    println!("// 最简单");
    println!("let runtime = ForgeRuntimeBuilder::new().build().await?;");
    println!();
    println!("// 指定类型");
    println!("let runtime = ForgeRuntimeBuilder::new()");
    println!("    .runtime_type(RuntimeType::Actor)");
    println!("    .build().await?;");
    println!();
    println!("// 完全自定义");
    println!("let runtime = ForgeRuntimeBuilder::new()");
    println!("    .runtime_type(RuntimeType::Async)");
    println!("    .max_concurrent_tasks(20)");
    println!("    .enable_monitoring(true)");
    println!("    .build().await?;");
    println!("```\n");

    println!("优势:");
    println!("✅ 统一的入口点（ForgeRuntimeBuilder::new()）");
    println!("✅ 流畅的链式 API");
    println!("✅ 类型安全的配置");
    println!("✅ 返回具体的枚举类型而非 trait object");
    println!("✅ 支持渐进式配置");
}

//! 开发环境追踪功能演示
//!
//! 此示例演示如何使用 ModuForge-RS 的开发追踪功能。
//!
//! # 运行方式
//!
//! ```bash
//! # 控制台输出（默认）
//! cargo run --example tracing_demo --features dev-tracing
//!
//! # JSON 文件输出
//! TRACE_FORMAT=json cargo run --example tracing_demo --features dev-tracing
//!
//! # Perfetto 可视化（需要 dev-tracing-perfetto feature）
//! TRACE_FORMAT=perfetto cargo run --example tracing_demo --features dev-tracing-perfetto
//! ```

use mf_core::{
    tracing_init::dev_tracing::{init_tracing, TraceConfig},
    runtime::builder::{ForgeRuntimeBuilder, AnyRuntime},
    RuntimeType,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化追踪系统
    #[cfg(feature = "dev-tracing")]
    {
        let trace_format = std::env::var("TRACE_FORMAT")
            .unwrap_or_else(|_| "perfetto".to_string());

        let config = match trace_format.as_str() {
            "json" => {
                println!("📊 使用 JSON 格式输出到 ./logs/trace.json");
                TraceConfig::json("./logs/trace.json")
                    .with_max_level(tracing::Level::DEBUG)
            },
            #[cfg(feature = "dev-tracing-perfetto")]
            "perfetto" => {
                println!("📊 使用 Perfetto 格式输出到 ./logs/trace.perfetto");
                println!("📊 使用 https://ui.perfetto.dev/ 查看追踪数据");
                TraceConfig::perfetto("./logs/trace.perfetto")
                    .with_max_level(tracing::Level::DEBUG)
            },
            _ => {
                println!("📊 使用控制台输出");
                TraceConfig::console().with_max_level(tracing::Level::DEBUG)
            },
        };

        init_tracing(config)?;
    }

    #[cfg(not(feature = "dev-tracing"))]
    {
        println!("⚠️  追踪功能未启用");
        println!("💡 使用 --features dev-tracing 启用追踪");
        return Ok(());
    }

    println!("\n🚀 开始演示追踪功能...\n");

    // 2. 创建运行时（会被追踪）
    #[cfg(feature = "dev-tracing")]
    tracing::info!("创建 Sync 运行时");

    let mut runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Sync)
        .build()
        .await?;

    #[cfg(feature = "dev-tracing")]
    tracing::info!("运行时创建成功");

    // 3. 执行一些操作（会被追踪）
    #[cfg(feature = "dev-tracing")]
    tracing::info!("开始执行事务操作");

    let tr = match &mut runtime {
        AnyRuntime::Sync(rt) => rt.get_tr(),
        AnyRuntime::Async(rt) => rt.get_tr(),
        AnyRuntime::Actor(rt) => rt.get_tr().await?,
    };

    // 分发事务
    runtime.dispatch(tr).await?;

    #[cfg(feature = "dev-tracing")]
    tracing::info!("事务执行完成");

    // 4. 再执行几个操作，生成更多追踪数据
    for i in 1..=3 {
        #[cfg(feature = "dev-tracing")]
        tracing::info!("执行第 {} 个事务", i);

        let tr = match &mut runtime {
            AnyRuntime::Sync(rt) => rt.get_tr(),
            AnyRuntime::Async(rt) => rt.get_tr(),
            AnyRuntime::Actor(rt) => rt.get_tr().await?,
        };

        runtime
            .dispatch_with_meta(
                tr,
                format!("测试事务 {}", i),
                serde_json::json!({ "index": i }),
            )
            .await?;

        #[cfg(feature = "dev-tracing")]
        tracing::debug!("第 {} 个事务完成", i);
    }

    // 5. 销毁运行时（会被追踪）
    #[cfg(feature = "dev-tracing")]
    tracing::info!("销毁运行时");

    runtime.destroy().await?;

    #[cfg(feature = "dev-tracing")]
    tracing::info!("运行时销毁完成");

    println!("\n✅ 追踪演示完成！\n");

    #[cfg(feature = "dev-tracing")]
    {
        println!("📊 追踪数据已生成");
        println!("💡 查看追踪数据：");
        println!("   - 控制台模式：直接查看上面的输出");
        println!("   - JSON 模式：查看 ./logs/trace.json");
        #[cfg(feature = "dev-tracing-perfetto")]
        println!(
            "   - Perfetto 模式：访问 https://ui.perfetto.dev/ 并上传 ./logs/trace.perfetto"
        );
    }

    Ok(())
}

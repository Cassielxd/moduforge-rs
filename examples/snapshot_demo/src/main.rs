//! 快照启动演示
//! 
//! 演示如何使用快照实现毫秒级启动

use std::time::Instant;
use mf_core::{ForgeRuntime, types::RuntimeOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ModuForge 快照启动演示 ===\n");

    // 1. 传统启动方式测试
    println!("📊 测试传统启动方式...");
    let traditional_start = Instant::now();
    let mut traditional_time = None;
    
    match ForgeRuntime::create(RuntimeOptions::default()).await {
        Ok(_traditional_runtime) => {
            let time = traditional_start.elapsed();
            traditional_time = Some(time);
            println!("✅ 传统启动完成，耗时: {:?}\n", time);
        },
        Err(e) => {
            println!("⚠️ 传统启动失败: {}", e);
            println!("这可能是因为缺少默认 schema 配置\n");
        }
    }

    // 2. 快照启动方式测试
    println!("🚀 测试快照启动方式...");
    let snapshot_start = Instant::now();
    
    // 尝试从快照启动，如果失败则回退到传统方式
    let snapshot_path = "target/snapshots/demo_snapshot.bin";
    let _snapshot_runtime = match ForgeRuntime::from_snapshot(snapshot_path, None).await {
        Ok(runtime) => {
            let snapshot_time = snapshot_start.elapsed();
            println!("✅ 快照启动完成，耗时: {:?}", snapshot_time);
            if let Some(traditional_time) = traditional_time {
                println!("🎯 性能提升: {:.2}x 倍", 
                    traditional_time.as_secs_f64() / snapshot_time.as_secs_f64());
            }
            runtime
        },
        Err(e) => {
            println!("⚠️  快照加载失败: {}", e);
            println!("🔄 回退到传统启动方式...");
            let fallback_runtime = ForgeRuntime::create(RuntimeOptions::default()).await?;
            let fallback_time = snapshot_start.elapsed();
            println!("✅ 回退启动完成，耗时: {:?}", fallback_time);
            fallback_runtime
        }
    };

    // 3. 快照 + 回退方式测试
    println!("\n🛡️  测试智能回退启动...");
    let smart_start = Instant::now();
    
    match ForgeRuntime::from_snapshot_or_fallback(
        snapshot_path, 
        RuntimeOptions::default()
    ).await {
        Ok(_smart_runtime) => {
            let smart_time = smart_start.elapsed();
            println!("✅ 智能启动完成，耗时: {:?}", smart_time);
        },
        Err(e) => {
            println!("⚠️ 智能启动失败: {}", e);
        }
    }

    // 4. 总结
    println!("\n📈 快照机制演示完成!");
    
    println!("\n🎉 演示完成！");
    Ok(())
}
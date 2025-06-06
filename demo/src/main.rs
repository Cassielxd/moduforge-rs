use anyhow::Result;
use tokio;
use moduforge_demo::simple_demo::run_simple_demo;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt::init();
    
    // 运行简化演示
    run_simple_demo().await?;
    
    Ok(())
}

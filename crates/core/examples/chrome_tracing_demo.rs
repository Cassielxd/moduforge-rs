//! Chrome Tracing 演示示例
//!
//! 运行方式：
//! ```bash
//! cargo run --example chrome_tracing_demo --features dev-tracing-chrome
//! ```
//!
//! 查看方式：
//! 1. 打开 Chrome 浏览器
//! 2. 访问 chrome://tracing
//! 3. 点击 "Load" 加载 logs/trace.json 文件

use mf_core::tracing_init::dev_tracing::{init_tracing, TraceConfig};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, debug};

#[derive(Debug)]
struct Transaction {
    id: String,
    data: Vec<u8>,
}

impl Transaction {
    fn new(
        id: &str,
        size: usize,
    ) -> Self {
        Self { id: id.to_string(), data: vec![0u8; size] }
    }
}

struct Runtime {
    name: String,
}

impl Runtime {
    fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id,
        data_size = transaction.data.len(),
        runtime = %self.name
    )))]
    async fn dispatch(
        &self,
        transaction: Transaction,
    ) -> Result<(), String> {
        info!("开始处理事务");

        self.validate(&transaction).await?;
        self.apply(&transaction).await?;
        self.notify(&transaction).await?;

        info!("事务处理完成");
        Ok(())
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn validate(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("验证事务数据");
        sleep(Duration::from_millis(10)).await;

        if transaction.data.is_empty() {
            return Err("数据为空".to_string());
        }

        debug!("验证通过");
        Ok(())
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn apply(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("应用事务");

        // 模拟数据库操作
        self.update_database(transaction).await?;

        // 模拟缓存更新
        self.update_cache(transaction).await?;

        debug!("应用完成");
        Ok(())
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn update_database(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("更新数据库");
        sleep(Duration::from_millis(50)).await;
        debug!("数据库更新完成");
        Ok(())
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn update_cache(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("更新缓存");
        sleep(Duration::from_millis(5)).await;
        debug!("缓存更新完成");
        Ok(())
    }

    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn notify(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("发送通知");
        sleep(Duration::from_millis(15)).await;
        debug!("通知发送完成");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 Chrome Tracing（保持 guard 直到程序结束）
    #[cfg(feature = "dev-tracing-chrome")]
    let _guard = init_tracing(TraceConfig::chrome("./logs/trace.json"))?;

    info!("🚀 Chrome Tracing 演示开始");
    info!("📊 将生成 3 个事务的追踪数据");
    info!("");

    let runtime = Runtime::new("demo-runtime");

    // 处理多个事务
    for i in 1..=3 {
        let tr = Transaction::new(&format!("tx-{:03}", i), 1024 * i);

        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("处理事务 {}", i);

        if let Err(e) = runtime.dispatch(tr).await {
            tracing::error!("事务处理失败: {}", e);
        }

        // 事务之间的间隔
        sleep(Duration::from_millis(20)).await;
    }

    info!("");
    info!("✅ 演示完成！");
    info!("");
    info!("📁 追踪文件已生成: logs/trace.json");
    info!("");
    info!("🌐 查看方式：");
    info!("   1. 打开 Chrome 浏览器");
    info!("   2. 访问 chrome://tracing");
    info!("   3. 点击 'Load' 按钮");
    info!("   4. 选择 logs/trace.json 文件");
    info!("");
    info!("💡 提示：");
    info!("   - 使用 W/A/S/D 键移动视图");
    info!("   - 使用鼠标滚轮缩放");
    info!("   - 点击 span 查看详细信息");
    info!("   - 可以看到每个操作的耗时和嵌套关系");

    info!("");
    info!("🔄 正在刷新追踪数据到文件...");

    // guard 在这里 drop，确保数据被正确写入
    Ok(())
}

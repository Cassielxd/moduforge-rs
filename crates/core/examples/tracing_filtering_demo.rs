//! 追踪过滤演示
//!
//! 演示如何只追踪特定方法调用的完整执行链路
//!
//! # 运行方式
//!
//! ```bash
//! # 1. 追踪所有方法（默认）
//! cargo run --example tracing_filtering_demo --features dev-tracing
//!
//! # 2. 只追踪 process_transaction 方法
//! TRACE_METHODS=process_transaction cargo run --example tracing_filtering_demo --features dev-tracing
//!
//! # 3. 只追踪多个方法
//! TRACE_METHODS=process_transaction,apply_changes cargo run --example tracing_filtering_demo --features dev-tracing
//!
//! # 4. 使用 grep 过滤特定 trace_id
//! cargo run --example tracing_filtering_demo --features dev-tracing 2>&1 | grep "trace_id=1"
//!
//! # 5. 使用 grep 过滤特定 tr_id
//! cargo run --example tracing_filtering_demo --features dev-tracing 2>&1 | grep "tr_id=\"tx-001\""
//! ```

use mf_core::tracing_init::dev_tracing::{init_tracing, TraceConfig};
use mf_core::{traced_span, trace_if_enabled};
use tracing::{info, debug};

/// 模拟一个事务结构
#[derive(Debug)]
struct Transaction {
    id: String,
    data: String,
}

/// 模拟的业务逻辑层
struct BusinessLogic;

impl BusinessLogic {
    /// 处理事务 - 这是我们想要追踪的主要方法
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id,
        data_size = transaction.data.len()
    )))]
    async fn process_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<(), String> {
        info!("开始处理事务");

        // 验证事务
        self.validate_transaction(&transaction).await?;

        // 应用变更
        self.apply_changes(&transaction).await?;

        // 通知其他系统
        self.notify_systems(&transaction).await?;

        info!("事务处理完成");
        Ok(())
    }

    /// 验证事务
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn validate_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("验证事务数据");

        // 模拟验证逻辑
        if transaction.data.is_empty() {
            return Err("事务数据为空".to_string());
        }

        debug!("事务验证通过");
        Ok(())
    }

    /// 应用变更
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn apply_changes(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("应用变更到状态");

        // 模拟数据库操作
        self.update_database(&transaction).await?;

        // 模拟缓存更新
        self.update_cache(&transaction).await?;

        debug!("变更应用完成");
        Ok(())
    }

    /// 更新数据库
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn update_database(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("写入数据库: {}", transaction.data);
        // 模拟 I/O 延迟
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        Ok(())
    }

    /// 更新缓存
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn update_cache(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("更新缓存");
        Ok(())
    }

    /// 通知其他系统
    #[cfg_attr(feature = "dev-tracing", tracing::instrument(skip(self, transaction), fields(
        tr_id = %transaction.id
    )))]
    async fn notify_systems(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        debug!("发送通知");
        Ok(())
    }
}

/// 演示使用 traced_span 宏
async fn demo_traced_span() {
    // 创建一个带唯一 trace_id 的 span
    let _span = traced_span!("demo_traced_span", operation = "example");

    info!("这是一个带 trace_id 的操作");

    // 子操作会继承 trace_id
    sub_operation_1().await;
    sub_operation_2().await;
}

#[cfg_attr(feature = "dev-tracing", tracing::instrument)]
async fn sub_operation_1() {
    debug!("子操作 1");
}

#[cfg_attr(feature = "dev-tracing", tracing::instrument)]
async fn sub_operation_2() {
    debug!("子操作 2");
}

/// 演示使用 trace_if_enabled 宏
async fn demo_conditional_tracing() {
    // 只在 TRACE_METHODS 包含 "conditional_method" 时才追踪
    let _span = trace_if_enabled!("conditional_method", test = "value");

    info!("这个方法只在指定时才会被追踪");

    conditional_sub_operation().await;
}

#[cfg_attr(feature = "dev-tracing", tracing::instrument)]
async fn conditional_sub_operation() {
    debug!("条件追踪的子操作");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化追踪
    init_tracing(TraceConfig::console())?;

    println!("\n🔍 追踪过滤演示");
    println!("================\n");

    // 演示 1: 处理多个事务
    println!("📊 演示 1: 处理多个事务");
    println!("提示: 使用 grep 过滤特定 tr_id");
    println!(
        "例如: cargo run --example tracing_filtering_demo --features dev-tracing 2>&1 | grep 'tx-001'\n"
    );

    let logic = BusinessLogic;

    let tx1 =
        Transaction { id: "tx-001".to_string(), data: "数据 A".to_string() };

    let tx2 =
        Transaction { id: "tx-002".to_string(), data: "数据 B".to_string() };

    let tx3 =
        Transaction { id: "tx-003".to_string(), data: "数据 C".to_string() };

    // 处理三个事务
    logic.process_transaction(tx1).await?;
    logic.process_transaction(tx2).await?;
    logic.process_transaction(tx3).await?;

    println!("\n");

    // 演示 2: 使用 traced_span
    println!("📊 演示 2: 使用 traced_span 宏");
    println!("提示: 使用 grep 过滤特定 trace_id");
    println!(
        "例如: cargo run --example tracing_filtering_demo --features dev-tracing 2>&1 | grep 'trace_id=0'\n"
    );

    demo_traced_span().await;

    println!("\n");

    // 演示 3: 条件追踪
    println!("📊 演示 3: 条件追踪");
    println!("提示: 使用 TRACE_METHODS 环境变量控制");
    println!(
        "例如: TRACE_METHODS=conditional_method cargo run --example tracing_filtering_demo --features dev-tracing\n"
    );

    demo_conditional_tracing().await;

    println!("\n✅ 演示完成！\n");

    println!("💡 过滤技巧:");
    println!("  1. 按事务 ID:  cargo run ... 2>&1 | grep 'tr_id=\"tx-001\"'");
    println!("  2. 按追踪 ID:  cargo run ... 2>&1 | grep 'trace_id=0'");
    println!(
        "  3. 按方法名:   TRACE_METHODS=process_transaction cargo run ..."
    );
    println!("  4. 按模块:     RUST_LOG=business_logic=debug cargo run ...");
    println!("  5. 只看错误:   RUST_LOG=error cargo run ...");

    Ok(())
}

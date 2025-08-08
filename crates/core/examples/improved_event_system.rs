//! 改进的事件系统示例
//!
//! 展示使用高效并发数据结构的事件系统性能优化

use std::{sync::Arc, time::Duration};
use mf_core::{
    config::EventConfig,
    event::{EventBus, EventHandler},
    ForgeResult,
};
use async_trait::async_trait;
use tokio::time::Instant;

// 示例事件类型
#[derive(Debug, Clone)]
pub enum TestEvent {
    UserAction(String),
    SystemEvent(u64),
    PerformanceMetric { name: String, value: f64 },
}

// 快速事件处理器
#[derive(Debug)]
struct FastHandler {
    id: String,
}

#[async_trait]
impl EventHandler<TestEvent> for FastHandler {
    async fn handle(
        &self,
        event: &TestEvent,
    ) -> ForgeResult<()> {
        // 模拟快速处理
        tokio::time::sleep(Duration::from_millis(1)).await;
        println!("FastHandler[{}] 处理事件: {:?}", self.id, event);
        Ok(())
    }
}

// 慢速事件处理器
#[derive(Debug)]
struct SlowHandler {
    id: String,
}

#[async_trait]
impl EventHandler<TestEvent> for SlowHandler {
    async fn handle(
        &self,
        event: &TestEvent,
    ) -> ForgeResult<()> {
        // 模拟慢速处理
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("SlowHandler[{}] 处理事件: {:?}", self.id, event);
        Ok(())
    }
}

// 可能失败的处理器
#[derive(Debug)]
struct FlakyHandler {
    id: String,
    failure_rate: f64,
}

#[async_trait]
impl EventHandler<TestEvent> for FlakyHandler {
    async fn handle(
        &self,
        event: &TestEvent,
    ) -> ForgeResult<()> {
        if rand::random::<f64>() < self.failure_rate {
            return Err(mf_core::error::error_utils::event_error(format!(
                "FlakyHandler[{}] 模拟失败",
                self.id
            )));
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("FlakyHandler[{}] 成功处理事件: {:?}", self.id, event);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> ForgeResult<()> {
    println!("=== 改进的事件系统性能测试 ===\n");

    // 1. 创建高性能配置
    let config = EventConfig {
        max_queue_size: 10000,
        handler_timeout: Duration::from_secs(2),
        enable_persistence: false,
        batch_size: 100,
        max_concurrent_handlers: 20,
    };

    // 2. 创建事件总线
    let event_bus = EventBus::with_config(config);

    println!("事件总线配置:");
    println!("  - 队列大小: {}", event_bus.get_config().max_queue_size);
    println!("  - 处理器超时: {:?}", event_bus.get_config().handler_timeout);
    println!(
        "  - 最大并发处理器: {}",
        event_bus.get_config().max_concurrent_handlers
    );

    // 3. 添加各种类型的事件处理器
    println!("\n=== 添加事件处理器 ===");

    let mut handler_ids = Vec::new();

    // 添加快速处理器
    for i in 0..5 {
        let handler = Arc::new(FastHandler { id: format!("fast-{}", i) });
        let id = event_bus.add_event_handler(handler)?;
        handler_ids.push(id);
        println!("添加快速处理器: ID={}", id);
    }

    // 添加慢速处理器
    for i in 0..3 {
        let handler = Arc::new(SlowHandler { id: format!("slow-{}", i) });
        let id = event_bus.add_event_handler(handler)?;
        handler_ids.push(id);
        println!("添加慢速处理器: ID={}", id);
    }

    // 批量添加不稳定处理器
    let flaky_handlers: Vec<Arc<dyn EventHandler<TestEvent> + Send + Sync>> = (0..3)
        .map(|i| {
            Arc::new(FlakyHandler {
                id: format!("flaky-{}", i),
                failure_rate: 0.3, // 30% 失败率
            }) as Arc<dyn EventHandler<TestEvent> + Send + Sync>
        })
        .collect();

    let flaky_ids = event_bus.add_event_handlers(flaky_handlers)?;
    handler_ids.extend(flaky_ids.iter());
    println!("批量添加不稳定处理器: {:?}", flaky_ids);

    println!("总处理器数量: {}", event_bus.handler_count());

    // 4. 启动事件循环
    event_bus.start_event_loop();

    // 等待事件循环启动
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 5. 性能测试 - 发送大量事件
    println!("\n=== 性能测试 ===");
    let start_time = Instant::now();
    let event_count = 1000;

    for i in 0..event_count {
        let event = match i % 3 {
            0 => TestEvent::UserAction(format!("action-{}", i)),
            1 => TestEvent::SystemEvent(i as u64),
            2 => TestEvent::PerformanceMetric {
                name: format!("metric-{}", i),
                value: i as f64 * 0.1,
            },
            _ => unreachable!(), // i % 3 只能是 0, 1, 2
        };

        event_bus.broadcast(event).await?;

        // 每100个事件打印一次进度
        if i % 100 == 0 {
            println!("已发送 {} 个事件", i);
        }
    }

    let send_duration = start_time.elapsed();
    println!("发送 {} 个事件耗时: {:?}", event_count, send_duration);
    println!(
        "发送速率: {:.2} 事件/秒",
        event_count as f64 / send_duration.as_secs_f64()
    );

    // 6. 等待事件处理完成
    println!("\n=== 等待事件处理完成 ===");
    tokio::time::sleep(Duration::from_secs(3)).await;

    // 7. 获取性能报告
    let report = event_bus.get_performance_report();
    println!("\n=== 性能报告 ===");
    println!("已处理事件总数: {}", report.total_events_processed);
    println!("活跃处理器数量: {}", report.active_handlers_count);
    println!("处理失败总数: {}", report.total_processing_failures);
    println!("处理超时总数: {}", report.total_processing_timeouts);
    println!("成功率: {:.2}%", report.success_rate);
    println!("处理器注册表大小: {}", report.handler_registry_size);

    // 8. 动态管理处理器
    println!("\n=== 动态处理器管理 ===");

    // 移除一些处理器
    let remove_count = 3;
    let removed_ids = &handler_ids[0..remove_count];
    let removed = event_bus.remove_event_handlers(removed_ids)?;
    println!("移除了 {} 个处理器", removed);
    println!("剩余处理器数量: {}", event_bus.handler_count());

    // 发送更多事件测试
    for i in 0..100 {
        let event = TestEvent::UserAction(format!("after-removal-{}", i));
        event_bus.broadcast(event).await?;
    }

    tokio::time::sleep(Duration::from_secs(1)).await;

    // 最终报告
    let final_report = event_bus.get_performance_report();
    println!("\n=== 最终性能报告 ===");
    println!("总处理事件数: {}", final_report.total_events_processed);
    println!("最终成功率: {:.2}%", final_report.success_rate);

    // 9. 清理
    println!("\n=== 清理资源 ===");
    event_bus.clear_handlers()?;
    println!("已清空所有处理器");
    println!("最终处理器数量: {}", event_bus.handler_count());

    println!("\n=== 测试完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_handler_management() -> ForgeResult<()> {
        let event_bus = EventBus::with_config(EventConfig::default());

        // 并发添加处理器
        let mut handles = Vec::new();
        for i in 0..10 {
            let bus = event_bus.clone();
            let handle = tokio::spawn(async move {
                let handler =
                    Arc::new(FastHandler { id: format!("concurrent-{}", i) });
                bus.add_event_handler(handler)
            });
            handles.push(handle);
        }

        // 等待所有添加完成
        let mut handler_ids = Vec::new();
        for handle in handles {
            let id = handle.await.unwrap()?;
            handler_ids.push(id);
        }

        assert_eq!(event_bus.handler_count(), 10);

        // 并发移除处理器
        let removed = event_bus.remove_event_handlers(&handler_ids)?;
        assert_eq!(removed, 10);
        assert_eq!(event_bus.handler_count(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_processing_stats() -> ForgeResult<()> {
        let event_bus = EventBus::with_config(EventConfig::default());

        // 添加处理器
        let handler = Arc::new(FastHandler { id: "test".to_string() });
        event_bus.add_event_handler(handler)?;

        // 启动事件循环
        event_bus.start_event_loop();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // 发送事件
        for i in 0..10 {
            let event = TestEvent::UserAction(format!("test-{}", i));
            event_bus.broadcast(event).await?;
        }

        // 等待处理完成
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 检查统计
        let stats = event_bus.get_stats();
        assert!(
            stats.events_processed.load(std::sync::atomic::Ordering::Relaxed)
                >= 10
        );
        assert_eq!(
            stats.active_handlers.load(std::sync::atomic::Ordering::Relaxed),
            1
        );

        Ok(())
    }
}

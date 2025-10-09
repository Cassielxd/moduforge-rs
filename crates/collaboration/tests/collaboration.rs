use std::sync::Arc;

use mf_collab::{Result, SyncService, YrsManager, CollaborationServer};

#[tokio::test]
async fn test_collaboration() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🚀 启动 ModuForge-RS Yrs 同步服务演示");

    // 1. 创建核心服务
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));

    // 2. 创建协作服务器，使用现有的 sync_service
    let collaboration_server = CollaborationServer::with_sync_service(
        yrs_manager.clone(),
        sync_service.clone(),
        8080,
    );
    // 5. 启动 WebSocket 服务器
    tokio::spawn(async move {
        collaboration_server.start().await;
    });
    // 等待用户中断
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("📴 收到停止信号，正在关闭服务...");
        }
    }
    Ok(())
}

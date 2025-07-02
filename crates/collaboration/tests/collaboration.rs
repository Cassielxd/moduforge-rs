use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use mf_state::{transaction::Command, Transaction};
use mf_transform::TransformResult;
use mf_collab::{
    Result, SyncService, YrsMiddleware, YrsManager, CollaborationServer,
    RoomStatus,
};
use tokio::time::{sleep, Duration};
use tracing_subscriber;
use tokio::sync::Mutex;

use mf_core::{
    middleware::MiddlewareStack,
    node::Node,
    runtime::ForgeRuntime,
    types::{Extensions, RuntimeOptions},
};
use mf_macro::node;
use mf_model::{node_type::NodeEnum};

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
    // 3. 创建并配置 ForgeRuntime (Editor) - 使用Mutex包装以支持可变访问
    let room_id = "demo-room".to_string();
    let runtime = Arc::new(Mutex::new(
        build_runtime(sync_service.clone(), room_id.clone()).await,
    ));
    // 4. 关键时机：在启动服务器前，使用现有的文档数据初始化房间
    // 这是推荐的初始化时机 - 在任何客户端连接之前
    {
        let runtime_guard = runtime.lock().await;
        let tree = runtime_guard.doc().get_inner().clone();

        // 初始化房间并同步现有的 Tree 数据
        if let Err(e) =
            collaboration_server.init_room_with_data(&room_id, &tree).await
        {
            tracing::error!("Failed to initialize room with data: {}", e);
        } else {
            tracing::info!("✅ 房间 '{}' 已成功使用现有数据初始化", room_id);
        }
    }
    // 5. 启动 WebSocket 服务器
    tokio::spawn(async move {
        collaboration_server.start().await;
    });

    // 6. 启动一个任务来模拟命令，以触发中间件
    let runtime_clone_for_commands = runtime.clone();
    tokio::spawn(async move {
        let mut counter = 0;
        loop {
            sleep(Duration::from_secs(5)).await;
            counter += 1;

            tracing::info!("🔄 执行第 {} 次测试命令", counter);

            // 执行测试命令 - 这会触发 YrsMiddleware
            tracing::info!("🔒 准备获取 runtime lock");
            let mut runtime_guard = runtime_clone_for_commands.lock().await;
            tracing::info!("🔓 已获取 runtime lock，准备执行 command");

            match runtime_guard.command(Arc::new(TestCommand)).await {
                Ok(_) => tracing::info!("✅ 测试命令执行成功"),
                Err(e) => tracing::error!("❌ 测试命令执行失败: {}", e),
            }

            tracing::info!("🔓 准备释放 runtime lock");
            // runtime_guard 在这里自动释放
        }
    });

    // 等待用户中断
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("📴 收到停止信号，正在关闭服务...");
        }
    }
    Ok(())
}

/// 测试房间下线功能
#[tokio::test]
async fn test_room_offline() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🧪 测试房间下线功能");

    // 1. 创建服务
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
    let server = CollaborationServer::with_sync_service(
        yrs_manager.clone(),
        sync_service.clone(),
        8080,
    );

    // 2. 创建测试房间
    let test_rooms = vec!["room1", "room2", "room3"];
    for room_id in &test_rooms {
        let runtime =
            build_runtime(sync_service.clone(), room_id.to_string()).await;
        let tree = runtime.doc().get_inner().clone();

        // 初始化房间
        server.init_room_with_data(room_id, &tree).await?;
        tracing::info!("✅ 测试房间 '{}' 初始化完成", room_id);
    }

    // 3. 检查房间状态
    let active_rooms = server.get_active_rooms();
    tracing::info!("📊 当前活跃房间: {:?}", active_rooms);

    let room_stats = server.get_rooms_stats().await;
    for room_info in &room_stats {
        tracing::info!("📋 房间信息: {:?}", room_info);
    }

    // 4. 测试单个房间下线
    tracing::info!("🔄 测试单个房间下线");
    let result = server.offline_room("room1", true).await?;
    tracing::info!("📊 房间 'room1' 下线结果: {}", result);

    // 检查房间状态
    let status = sync_service.get_room_status("room1").await;
    tracing::info!("📊 房间 'room1' 状态: {:?}", status);
    assert_eq!(status, RoomStatus::NotExists);

    // 5. 测试批量下线
    tracing::info!("🔄 测试批量房间下线");
    let remaining_rooms = vec!["room2".to_string(), "room3".to_string()];
    let batch_results = server.offline_rooms(&remaining_rooms, true).await?;
    tracing::info!("📊 批量下线结果: {:?}", batch_results);

    // 6. 验证所有房间已下线
    let final_active_rooms = server.get_active_rooms();
    tracing::info!("📊 最终活跃房间: {:?}", final_active_rooms);
    assert!(final_active_rooms.is_empty());

    tracing::info!("✅ 房间下线功能测试完成");
    Ok(())
}

/// 测试条件下线功能
#[tokio::test]
async fn test_conditional_offline() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🧪 测试条件下线功能");

    // 1. 创建服务
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
    let server = CollaborationServer::with_sync_service(
        yrs_manager.clone(),
        sync_service.clone(),
        8080,
    );

    // 2. 创建多个房间
    let test_rooms = vec!["empty-room1", "empty-room2", "active-room"];
    for room_id in &test_rooms {
        let runtime =
            build_runtime(sync_service.clone(), room_id.to_string()).await;
        let tree = runtime.doc().get_inner().clone();

        server.init_room_with_data(room_id, &tree).await?;
    }

    // 3. 模拟空房间（没有客户端连接）
    tracing::info!("🔄 测试下线空房间");
    let empty_rooms = server.offline_empty_rooms(true).await?;
    tracing::info!("📊 下线的空房间: {:?}", empty_rooms);

    // 4. 测试服务器完全关闭
    tracing::info!("🔄 测试服务器关闭");
    server.shutdown(true).await?;

    let final_rooms = server.get_active_rooms();
    tracing::info!("📊 关闭后剩余房间: {:?}", final_rooms);
    assert!(final_rooms.is_empty());

    tracing::info!("✅ 条件下线功能测试完成");
    Ok(())
}

/// 测试房间不存在时的错误处理
#[tokio::test]
async fn test_room_not_found_error() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🧪 测试房间不存在错误处理");

    // 1. 创建服务
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
    let server = CollaborationServer::with_sync_service(
        yrs_manager.clone(),
        sync_service.clone(),
        8080,
    );

    // 2. 验证房间不存在
    let non_existent_room = "non-existent-room";
    assert!(!yrs_manager.room_exists(non_existent_room));

    let status = sync_service.get_room_status(non_existent_room).await;
    assert_eq!(status, RoomStatus::NotExists);

    // 3. 验证房间信息为 None
    let room_info = sync_service.get_room_info(non_existent_room).await;
    assert!(room_info.is_none());

    tracing::info!("✅ 房间不存在错误处理测试完成");
    Ok(())
}

/// 测试房间存在性检查
#[tokio::test]
async fn test_room_existence_check() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🧪 测试房间存在性检查");

    // 1. 创建服务
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
    let server = CollaborationServer::with_sync_service(
        yrs_manager.clone(),
        sync_service.clone(),
        8080,
    );

    let room_id = "test-room";

    // 2. 初始状态：房间不存在
    assert!(!yrs_manager.room_exists(room_id));
    let status = sync_service.get_room_status(room_id).await;
    assert_eq!(status, RoomStatus::NotExists);

    // 3. 创建房间
    let runtime =
        build_runtime(sync_service.clone(), room_id.to_string()).await;
    let tree = runtime.doc().get_inner().clone();
    server.init_room_with_data(room_id, &tree).await?;

    // 4. 验证房间现在存在
    assert!(yrs_manager.room_exists(room_id));
    let status = sync_service.get_room_status(room_id).await;
    assert_eq!(status, RoomStatus::Initialized);

    // 5. 验证可以获取房间信息
    let room_info = sync_service.get_room_info(room_id).await;
    assert!(room_info.is_some());

    if let Some(info) = room_info {
        assert_eq!(info.room_id, room_id);
        assert_eq!(info.status, RoomStatus::Initialized);
        tracing::info!("📊 房间信息: {:?}", info);
    }

    // 6. 下线房间
    server.offline_room(room_id, false).await?;

    // 7. 验证房间已不存在
    assert!(!yrs_manager.room_exists(room_id));
    let final_status = sync_service.get_room_status(room_id).await;
    assert_eq!(final_status, RoomStatus::NotExists);

    tracing::info!("✅ 房间存在性检查测试完成");
    Ok(())
}

/// 测试 HTTP 接口功能
#[tokio::test]
async fn test_http_endpoints() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🧪 测试 HTTP 接口功能");

    // 1. 创建服务
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
    let server = CollaborationServer::with_sync_service(
        yrs_manager.clone(),
        sync_service.clone(),
        8081, // 使用不同的端口避免冲突
    );

    let room_id = "test-http-room";

    // 2. 创建房间
    let runtime =
        build_runtime(sync_service.clone(), room_id.to_string()).await;
    let tree = runtime.doc().get_inner().clone();
    server.init_room_with_data(room_id, &tree).await?;

    // 3. 验证房间存在
    assert!(yrs_manager.room_exists(room_id));

    // 4. 测试房间检查 API（模拟）
    let exists = yrs_manager.room_exists(room_id);
    assert!(exists, "房间应该存在");

    let non_existent_exists = yrs_manager.room_exists("non-existent-room");
    assert!(!non_existent_exists, "不存在的房间应该返回 false");

    // 5. 测试房间状态 API
    let room_info = sync_service.get_room_info(room_id).await;
    assert!(room_info.is_some(), "应该能获取房间信息");

    if let Some(info) = room_info {
        assert_eq!(info.room_id, room_id);
        assert_eq!(info.status, RoomStatus::Initialized);
        tracing::info!("📊 房间信息: {:?}", info);
    }

    // 6. 测试健康检查 API（模拟）
    let active_rooms = sync_service.get_active_rooms();
    let room_stats = sync_service.get_rooms_stats().await;

    assert!(
        active_rooms.contains(&room_id.to_string()),
        "活跃房间列表应包含测试房间"
    );
    assert!(!room_stats.is_empty(), "房间统计不应为空");

    tracing::info!("📊 活跃房间数: {}", active_rooms.len());
    tracing::info!("📊 房间统计数: {}", room_stats.len());

    // 7. 下线房间
    server.offline_room(room_id, false).await?;

    // 8. 验证房间已下线
    assert!(!yrs_manager.room_exists(room_id));
    let final_room_info = sync_service.get_room_info(room_id).await;
    assert!(final_room_info.is_none(), "下线后应无法获取房间信息");

    tracing::info!("✅ HTTP 接口测试完成");
    Ok(())
}

/// 测试房间预检查逻辑
#[tokio::test]
async fn test_room_precheck_logic() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🧪 测试房间预检查逻辑");

    // 1. 创建服务
    let yrs_manager = Arc::new(YrsManager::new());
    let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
    let server = CollaborationServer::with_sync_service(
        yrs_manager.clone(),
        sync_service.clone(),
        8082,
    );

    // 2. 测试场景：房间不存在
    let non_existent_room = "non-existent-room";
    assert!(!yrs_manager.room_exists(non_existent_room));

    let room_status = sync_service.get_room_status(non_existent_room).await;
    assert_eq!(room_status, RoomStatus::NotExists);

    // 3. 测试场景：创建房间后存在
    let existing_room = "existing-room";
    let runtime =
        build_runtime(sync_service.clone(), existing_room.to_string()).await;
    let tree = runtime.doc().get_inner().clone();
    server.init_room_with_data(existing_room, &tree).await?;

    assert!(yrs_manager.room_exists(existing_room));
    let status = sync_service.get_room_status(existing_room).await;
    assert_eq!(status, RoomStatus::Initialized);

    // 4. 测试场景：房间信息获取
    let room_info = sync_service.get_room_info(existing_room).await;
    assert!(room_info.is_some());

    if let Some(info) = room_info {
        assert_eq!(info.room_id, existing_room);
        assert_eq!(info.status, RoomStatus::Initialized);
        assert_eq!(info.client_count, 0); // 没有客户端连接
        tracing::info!(
            "📊 房间信息: 节点数={}, 客户端数={}",
            info.node_count,
            info.client_count
        );
    }

    // 5. 清理
    server.offline_room(existing_room, false).await?;
    assert!(!yrs_manager.room_exists(existing_room));

    tracing::info!("✅ 房间预检查逻辑测试完成");
    Ok(())
}

pub const DWGC_STR: &str = "DWGC";
pub const DXGC_STR: &str = "DXGC";
pub const GCXM_STR: &str = "GCXM";

fn nodes() -> Vec<Node> {
    let mut GCXM: Node = node!(GCXM_STR, "工程项目", &format!("{}+", DXGC_STR));
    GCXM.set_top_node();
    let DXGC: Node =
        node!(DXGC_STR, "单项工程", &format!("({}|{})+", DWGC_STR, DXGC_STR));
    let DWGC: Node = node!(DWGC_STR, "单位工程");
    vec![GCXM, DXGC, DWGC]
}

async fn build_runtime(
    sync_service: Arc<SyncService>,
    room_id: String,
) -> ForgeRuntime {
    let nodes = nodes();
    let mut runtime_options = RuntimeOptions::default();
    for node in nodes {
        runtime_options = runtime_options.add_extension(Extensions::N(node));
    }
    // 3. 设置中间件以连接 Runtime 和 SyncService
    let mut middleware_stack = MiddlewareStack::new();
    middleware_stack
        .add(YrsMiddleware { sync_service: sync_service.clone(), room_id });
    runtime_options = runtime_options.set_middleware_stack(middleware_stack);
    ForgeRuntime::create(runtime_options).await.unwrap()
}

#[derive(Debug)]
struct TestCommand;

#[async_trait]
impl Command for TestCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        let root_id = tr.doc().root_id().clone();
        let dxgc = tr.schema.nodes[DXGC_STR].clone();
        let dxgc_node = dxgc.create(None, None, Vec::new(), None);
        tr.add_node(root_id, vec![NodeEnum(dxgc_node, vec![])])?;
        Ok(())
    }
    fn name(&self) -> String {
        "TestCommand".to_string()
    }
}

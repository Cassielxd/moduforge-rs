use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use moduforge_state::{transaction::Command, Transaction};
use moduforge_transform::TransformResult;
use moduforge_collaboration::{
    Result, SyncService, YrsMiddleware, YrsManager, WebSocketServer,
};
use tokio::time::{sleep, Duration};
use tracing_subscriber;
use tokio::sync::Mutex;

use moduforge_core::{
    middleware::MiddlewareStack,
    node::Node,
    runtime::ForgeRuntime,
    types::{Extensions, RuntimeOptions},
};
use moduforge_macros::node;
use moduforge_model::{node_type::NodeEnum};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    tracing::info!("🚀 启动 ModuForge-RS Yrs 同步服务演示");

    // 1. 创建核心服务
    let yrs_manager = Arc::new(YrsManager::new());
    let ws_server = Arc::new(WebSocketServer::new(yrs_manager.clone()));
    let sync_service =
        Arc::new(SyncService::new(yrs_manager, ws_server.clone()));

    // 2. 创建并配置 ForgeRuntime (Editor) - 使用Mutex包装以支持可变访问
    let room_id = "demo-room".to_string();
    let runtime = Arc::new(Mutex::new(
        build_runtime(sync_service.clone(), room_id.clone()).await,
    ));

    // 4. 启动 WebSocket 服务器
    let ws_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let _ =
        ws_server.start(ws_addr, sync_service.clone(), runtime.clone()).await;

    // 5. 启动一个任务来模拟命令，以触发中间件
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

use std::net::SocketAddr;
use std::sync::Arc;
use async_trait::async_trait;
use moduforge_state::{transaction::Command, Transaction};
use moduforge_transform::TransformResult;
use moduforge_transmission::{Result, SyncService, YrsMiddleware};
use tokio::time::{sleep, Duration};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 启动 ModuForge-RS Yrs 同步服务演示");

    // 创建同步服务
    let sync_service = Arc::new(SyncService::new());

    // 启动 WebSocket 服务器
    let ws_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let _ws_handle = sync_service.start(ws_addr).await?;

    // 设置房间ID
    let room_id = "demo-room".to_string();
    let mut editor_for_commands = build_editor(sync_service.clone(), room_id).await; // 创建专门用于命令的editor
    tokio::spawn(async move {
        let mut counter = 0;
        loop {
            sleep(Duration::from_secs(30)).await;
            counter += 1;
            
            tracing::info!("🔄 执行第 {} 次测试命令", counter);
            
            // 执行测试命令 - 这会触发中间件
            match editor_for_commands.command(Arc::new(TestCommand)).await {
                Ok(_) => {
                    tracing::info!("✅ 测试命令执行成功");
                }
                Err(e) => {
                    tracing::error!("❌ 测试命令执行失败: {}", e);
                }
            }
        }
    });

    // 等待用户中断
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("📴 收到停止信号，正在关闭服务...");
        }
        _ = sleep(Duration::from_secs(600)) => {
            tracing::info!("⏰ 演示时间到，自动关闭服务");
        }
    }
    
    tracing::info!("👋 演示结束");
    Ok(())
}

pub const DWGC_STR: &str = "DWGC";
pub const DXGC_STR: &str = "DXGC";
pub const GCXM_STR: &str = "GCXM";

use moduforge_core::{middleware::MiddlewareStack, node::Node, runtime::ForgeRuntime, types::{Extensions, RuntimeOptions}};
use moduforge_macros::node;
use moduforge_model::{node_type::NodeEnum, schema::AttributeSpec};

fn nodes() -> Vec<Node>{
    let mut GCXM: Node = node!(GCXM_STR, "工程项目", &format!("{}+", DXGC_STR));
    GCXM.set_top_node();
    let DXGC: Node = node!(DXGC_STR, "单项工程", &format!("({}|{})+", DWGC_STR, DXGC_STR));
    let DWGC: Node = node!(DWGC_STR, "单位工程");
    vec![GCXM, DXGC, DWGC]
}


async fn build_editor(sync_service: Arc<SyncService>, room_id: String) -> ForgeRuntime{
    let nodes = nodes();
    let mut runtime_options = RuntimeOptions::default();
    for node in nodes {
        runtime_options = runtime_options.add_extension(Extensions::N(node));
    }
    let mut middleware_stack = MiddlewareStack::new();
    middleware_stack.add(YrsMiddleware{sync_service, room_id });
    runtime_options =runtime_options.set_middleware_stack(middleware_stack);
    let runtime = ForgeRuntime::create(runtime_options).await.unwrap();
    runtime
}
#[derive(Debug)]
struct TestCommand;
#[async_trait]
impl Command for TestCommand{
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
       let root_id = tr.doc().root_id().clone();
       let dxgc = tr.schema.nodes[DXGC_STR].clone();
       let dxgc_node = dxgc.create(None, None, Vec::new(), None);
       tr.add_node(root_id, vec![NodeEnum(dxgc_node,vec![])])?;
        Ok(())
    }
    fn name(&self) -> String {
        "TestCommand".to_string()
    }
}
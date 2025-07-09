use std::sync::Arc;
use crate::{YrsManager, SyncService};
use crate::sync_service::{RoomInfo, RoomStatus};
use warp::ws::{WebSocket, Ws};
use warp::{Filter, Rejection, Reply};
use yrs_warp::broadcast::BroadcastGroup;
use yrs_warp::ws::{WarpSink, WarpStream};
use tokio::sync::Mutex;
use futures_util::StreamExt;
use serde_json::json;

/// 自定义错误类型用于房间不存在的情况
#[derive(Debug)]
pub struct RoomNotFoundError {
    room_id: String,
}

impl warp::reject::Reject for RoomNotFoundError {}

impl RoomNotFoundError {
    pub fn new(room_id: String) -> Self {
        Self { room_id }
    }

    pub fn room_id(&self) -> &str {
        &self.room_id
    }
}

/// YrsManager 的包装器，用于处理动态房间创建和广播组管理
#[derive(Clone)]
pub struct CollaborationServer {
    yrs_manager: Arc<YrsManager>,
    sync_service: Arc<SyncService>,
    port: u16,
}

impl CollaborationServer {
    pub fn new(
        yrs_manager: Arc<YrsManager>,
        port: u16,
    ) -> Self {
        let sync_service = Arc::new(SyncService::new(yrs_manager.clone()));
        Self { yrs_manager, sync_service, port }
    }

    /// 使用现有的 SyncService 创建服务器
    pub fn with_sync_service(
        yrs_manager: Arc<YrsManager>,
        sync_service: Arc<SyncService>,
        port: u16,
    ) -> Self {
        Self { yrs_manager, sync_service, port }
    }

    /// 自定义错误处理器
    pub async fn handle_rejection(
        err: Rejection
    ) -> Result<impl Reply, std::convert::Infallible> {
        if let Some(room_error) = err.find::<RoomNotFoundError>() {
            let error_response = json!({
                "error": "ROOM_NOT_FOUND",
                "message": format!("房间 '{}' 不存在", room_error.room_id()),
                "room_id": room_error.room_id(),
                "code": 404
            });

            let reply = warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::NOT_FOUND,
            );

            return Ok(reply.into_response());
        }

        // 处理其他错误
        if err.is_not_found() {
            let error_response = json!({
                "error": "NOT_FOUND",
                "message": "请求的资源不存在",
                "code": 404
            });

            let reply = warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::NOT_FOUND,
            );

            return Ok(reply.into_response());
        }

        // 默认错误处理
        let error_response = json!({
            "error": "INTERNAL_SERVER_ERROR",
            "message": "服务器内部错误",
            "code": 500
        });

        let reply = warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        );

        Ok(reply.into_response())
    }

    /// 房间下线 - 优雅关闭房间
    /// 1. 通知所有客户端房间即将关闭
    /// 2. 等待客户端完成当前操作
    /// 3. 保存数据（可选）
    /// 4. 清理资源
    pub async fn offline_room(
        &self,
        room_id: &str,
        save_data: bool,
    ) -> crate::Result<bool> {
        tracing::info!("🔄 开始下线房间: {}", room_id);

        // 1. 检查房间状态
        let room_status = self.sync_service.get_room_status(room_id).await;
        match room_status {
            RoomStatus::NotExists => {
                tracing::warn!("❌ 尝试下线不存在的房间: {}", room_id);
                return Ok(false);
            },
            RoomStatus::Offline => {
                tracing::info!("ℹ️ 房间 {} 已经下线", room_id);
                return Ok(true);
            },
            _ => {},
        }

        // 2. 获取房间信息
        if let Some(room_info) = self.sync_service.get_room_info(room_id).await
        {
            tracing::info!(
                "📊 房间信息 - 节点数: {}, 客户端数: {}",
                room_info.node_count,
                room_info.client_count
            );
        }

        // 3. 执行下线操作
        match self.sync_service.offline_room(room_id, save_data).await {
            Ok(snapshot) => {
                if let Some(_snapshot) = snapshot {
                    tracing::info!("💾 房间 {} 数据已保存", room_id);
                }
                tracing::info!("✅ 房间 {} 成功下线", room_id);
                Ok(true)
            },
            Err(e) => {
                tracing::error!("❌ 房间 {} 下线失败: {}", room_id, e);
                Err(e)
            },
        }
    }

    /// 强制房间下线 - 紧急情况使用
    pub async fn force_offline_room(
        &self,
        room_id: &str,
    ) -> crate::Result<bool> {
        tracing::warn!("⚠️ 强制下线房间: {}", room_id);

        match self.sync_service.force_offline_room(room_id).await {
            Ok(success) => {
                if success {
                    tracing::info!("✅ 房间 {} 强制下线成功", room_id);
                } else {
                    tracing::error!("❌ 房间 {} 强制下线失败", room_id);
                }
                Ok(success)
            },
            Err(e) => {
                tracing::error!(
                    "❌ 强制下线房间 {} 时发生错误: {}",
                    room_id,
                    e
                );
                Err(e)
            },
        }
    }

    /// 批量下线房间
    pub async fn offline_rooms(
        &self,
        room_ids: &[String],
        save_data: bool,
    ) -> crate::Result<Vec<(String, bool)>> {
        tracing::info!("🔄 批量下线 {} 个房间", room_ids.len());

        let mut results = Vec::new();

        for room_id in room_ids {
            match self.offline_room(room_id, save_data).await {
                Ok(success) => results.push((room_id.clone(), success)),
                Err(e) => {
                    tracing::error!("❌ 下线房间 {} 失败: {}", room_id, e);
                    results.push((room_id.clone(), false));
                },
            }
        }

        let successful_count =
            results.iter().filter(|(_, success)| *success).count();
        tracing::info!(
            "📊 批量下线完成: {}/{} 个房间成功下线",
            successful_count,
            room_ids.len()
        );

        Ok(results)
    }

    /// 获取所有活跃房间列表
    pub fn get_active_rooms(&self) -> Vec<String> {
        self.sync_service.get_active_rooms()
    }

    /// 获取房间统计信息
    pub async fn get_rooms_stats(&self) -> Vec<RoomInfo> {
        self.sync_service.get_rooms_stats().await
    }

    /// 根据条件下线房间
    /// 例如：下线空闲时间超过指定时间的房间、下线没有客户端的房间等
    pub async fn offline_rooms_by_condition<F>(
        &self,
        condition: F,
        save_data: bool,
    ) -> crate::Result<Vec<String>>
    where
        F: Fn(&RoomInfo) -> bool,
    {
        let room_stats = self.get_rooms_stats().await;
        let rooms_to_offline: Vec<String> = room_stats
            .into_iter()
            .filter(|info| condition(info))
            .map(|info| info.room_id)
            .collect();

        if rooms_to_offline.is_empty() {
            tracing::info!("🔍 没有房间满足下线条件");
            return Ok(vec![]);
        }

        tracing::info!("🎯 找到 {} 个房间满足下线条件", rooms_to_offline.len());

        let results = self.offline_rooms(&rooms_to_offline, save_data).await?;
        let successful_rooms: Vec<String> = results
            .into_iter()
            .filter_map(
                |(room_id, success)| if success { Some(room_id) } else { None },
            )
            .collect();

        Ok(successful_rooms)
    }

    /// 下线空房间（没有客户端连接的房间）
    pub async fn offline_empty_rooms(
        &self,
        save_data: bool,
    ) -> crate::Result<Vec<String>> {
        tracing::info!("🔍 搜索并下线空房间");

        self.offline_rooms_by_condition(
            |room_info| room_info.client_count == 0,
            save_data,
        )
        .await
    }

    /// 下线长时间未活动的房间
    pub async fn offline_inactive_rooms(
        &self,
        inactive_duration: std::time::Duration,
        save_data: bool,
    ) -> crate::Result<Vec<String>> {
        let now = std::time::SystemTime::now();
        tracing::info!(
            "🔍 搜索并下线超过 {:?} 未活动的房间",
            inactive_duration
        );

        self.offline_rooms_by_condition(
            |room_info| {
                now.duration_since(room_info.last_activity).unwrap_or_default()
                    > inactive_duration
            },
            save_data,
        )
        .await
    }

    /// 服务器完全关闭 - 下线所有房间
    pub async fn shutdown(
        &self,
        save_all_data: bool,
    ) -> crate::Result<()> {
        tracing::info!("🔴 开始服务器关闭流程");

        let all_rooms = self.get_active_rooms();
        if all_rooms.is_empty() {
            tracing::info!("ℹ️ 没有活跃房间需要关闭");
            return Ok(());
        }

        tracing::info!("📊 准备关闭 {} 个房间", all_rooms.len());

        // 批量下线所有房间
        let results = self.offline_rooms(&all_rooms, save_all_data).await?;
        let successful_count =
            results.iter().filter(|(_, success)| *success).count();

        tracing::info!(
            "✅ 服务器关闭完成: {}/{} 个房间成功下线",
            successful_count,
            all_rooms.len()
        );

        if successful_count != all_rooms.len() {
            tracing::warn!("⚠️ 部分房间下线失败，可能需要手动清理");
        }

        Ok(())
    }

    /// 启动 WebSocket 服务器
    pub async fn start(self) {
        let server = self.clone(); // 克隆 self 以移动到过滤器

        // WebSocket 路由（带错误处理）
        let ws_route = warp::path("collaboration")
            .and(warp::path::param::<String>()) // Expect a room_id in the path, e.g., /collaboration/my-room-name
            .and(warp::ws())
            .and(warp::addr::remote()) // 这里添加
            .and(warp::any().map(move || server.clone()))
            .and_then(Self::ws_handler);

        // HTTP 房间检查路由
        let server_for_http = self.clone();
        let room_check_route = warp::path("collaboration")
            .and(warp::path("room-check"))
            .and(warp::path::param::<String>()) // room_id
            .and(warp::get())
            .and(warp::any().map(move || server_for_http.clone()))
            .and_then(Self::room_check_handler);

        // 健康检查路由
        let server_for_health = self.clone();
        let health_route = warp::path("health")
            .and(warp::get())
            .and(warp::any().map(move || server_for_health.clone()))
            .and_then(Self::health_check_handler);

        // 房间状态路由
        let server_for_status = self.clone();
        let room_status_route = warp::path("collaboration")
            .and(warp::path("rooms"))
            .and(warp::path::param::<String>()) // room_id
            .and(warp::path("status"))
            .and(warp::get())
            .and(warp::any().map(move || server_for_status.clone()))
            .and_then(Self::room_status_handler);

        // 合并所有路由并添加全局错误处理
        let routes = ws_route
            .or(room_check_route)
            .or(health_route)
            .or(room_status_route)
            .recover(Self::handle_rejection) // 移到这里，对所有路由应用错误处理
            .with(
                warp::cors()
                    .allow_any_origin()
                    .allow_headers(vec!["content-type"])
                    .allow_methods(vec![
                        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS",
                    ]),
            );

        let addr = ([0, 0, 0, 0], self.port);
        tracing::info!(
            "🌐 协作服务器启动于 http://{}:{}",
            addr.0.iter().map(|&o| o.to_string()).collect::<Vec<_>>().join("."),
            addr.1
        );
        tracing::info!(
            "📡 WebSocket: ws://{}:{}/collaboration/{{room_id}}",
            addr.0.iter().map(|&o| o.to_string()).collect::<Vec<_>>().join("."),
            addr.1
        );
        tracing::info!(
            "🔍 房间检查: http://{}:{}/collaboration/room-check/{{room_id}}",
            addr.0.iter().map(|&o| o.to_string()).collect::<Vec<_>>().join("."),
            addr.1
        );
        tracing::info!(
            "💚 健康检查: http://{}:{}/health",
            addr.0.iter().map(|&o| o.to_string()).collect::<Vec<_>>().join("."),
            addr.1
        );
        tracing::info!(
            "📊 房间状态: http://{}:{}/collaboration/rooms/{{room_id}}/status",
            addr.0.iter().map(|&o| o.to_string()).collect::<Vec<_>>().join("."),
            addr.1
        );

        warp::serve(routes).run(addr).await;
    }

    /// WebSocket connection handler with room initialization.
    async fn ws_handler(
        room_id: String,
        ws: Ws,
        remote_addr: Option<std::net::SocketAddr>,
        server: CollaborationServer,
    ) -> Result<impl Reply, Rejection> {
        let yrs_manager = server.yrs_manager.clone();
        // 获取已存在的 awareness（不创建新的）
        let awareness_ref = yrs_manager.get_or_create_awareness(&room_id);
        Ok(ws.on_upgrade(move |socket| async move {
            tracing::info!("✅ 客户端成功连接到现有房间: {}", room_id);
            let client_addr = remote_addr
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            // The buffer capacity can be adjusted as needed. 128 is a reasonable default.
            let bcast = Arc::new(BroadcastGroup::new(awareness_ref, 128).await);
            Self::peer(socket, bcast, room_id.clone(), client_addr).await;
        }))
    }

    /// 处理单个客户端连接（基于官方示例）
    async fn peer(
        ws: WebSocket,
        bcast: Arc<BroadcastGroup>,
        room_id: String,
        client_addr: String,
    ) {
        let (sink, stream) = ws.split();
        let sink = Arc::new(Mutex::new(WarpSink::from(sink)));
        let stream = WarpStream::from(stream);
        // 增加客户端连接的详细日志
        tracing::info!(
            "🔗 新客户端连接到房间: {} (地址: {})",
            room_id,
            client_addr
        );

        let sub = bcast.subscribe(sink, stream);

        match sub.completed().await {
            Ok(_) => {
                tracing::info!(
                    "✅ 客户端正常断开连接 - 房间: {} (地址: {})",
                    room_id,
                    client_addr
                );
            },
            Err(e) => {
                // 根据错误类型提供更详细的错误信息
                let error_msg = format!("{}", e);

                if error_msg.contains("failed to deserialize message") {
                    tracing::warn!(
                        "⚠️ 客户端发送了无效数据包 - 房间: {}, 错误: {}",
                        room_id,
                        error_msg
                    );
                    tracing::debug!(
                        "💡 这通常是由网络中断或客户端异常关闭导致的，属于正常现象"
                    );
                } else if error_msg.contains("unexpected end of buffer") {
                    tracing::warn!(
                        "⚠️ 数据包不完整 - 房间: {}, 可能是网络传输中断",
                        room_id
                    );
                } else if error_msg.contains("connection closed")
                    || error_msg.contains("broken pipe")
                {
                    tracing::info!(
                        "🔌 客户端连接意外断开 - 房间: {} ({})",
                        room_id,
                        error_msg
                    );
                } else {
                    tracing::error!(
                        "❌ 客户端连接异常 - 房间: {}, 错误: {}",
                        room_id,
                        error_msg
                    );
                }
            },
        }
    }

    /// 获取 SyncService 的引用，用于外部操作
    pub fn sync_service(&self) -> &Arc<SyncService> {
        &self.sync_service
    }

    /// HTTP 房间检查处理器
    async fn room_check_handler(
        room_id: String,
        server: CollaborationServer,
    ) -> Result<impl Reply, Rejection> {
        tracing::debug!("🔍 检查房间是否存在: {}", room_id);

        let exists = server.yrs_manager.room_exists(&room_id);

        if exists {
            let room_info = server.sync_service.get_room_info(&room_id).await;

            let response = json!({
                "exists": true,
                "room_id": room_id,
                "status": "available",
                "info": room_info
            });

            tracing::debug!("✅ 房间 {} 存在", room_id);
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        } else {
            let response = json!({
                "exists": false,
                "room_id": room_id,
                "status": "not_found",
                "message": format!("房间 '{}' 不存在", room_id)
            });

            tracing::debug!("❌ 房间 {} 不存在", room_id);
            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
    }

    /// 健康检查处理器
    async fn health_check_handler(
        server: CollaborationServer
    ) -> Result<impl Reply, Rejection> {
        let room_stats = server.sync_service.get_rooms_stats().await;
        let active_rooms = server.sync_service.get_active_rooms();

        let response = json!({
            "status": "healthy",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "service": "ModuForge Collaboration Server",
            "version": env!("CARGO_PKG_VERSION"),
            "statistics": {
                "active_rooms": active_rooms.len(),
                "total_rooms": room_stats.len(),
                "rooms": active_rooms
            }
        });

        Ok(warp::reply::with_status(
            warp::reply::json(&response),
            warp::http::StatusCode::OK,
        ))
    }

    /// 房间状态处理器
    async fn room_status_handler(
        room_id: String,
        server: CollaborationServer,
    ) -> Result<impl Reply, Rejection> {
        tracing::debug!("📊 获取房间状态: {}", room_id);

        if let Some(room_info) =
            server.sync_service.get_room_info(&room_id).await
        {
            let response = json!({
                "room_id": room_id,
                "status": room_info.status,
                "node_count": room_info.node_count,
                "client_count": room_info.client_count,
                "last_activity": room_info.last_activity
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                "available": true
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::OK,
            ))
        } else {
            let response = json!({
                "room_id": room_id,
                "status": "not_found",
                "available": false,
                "message": format!("房间 '{}' 不存在", room_id)
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
    }
}

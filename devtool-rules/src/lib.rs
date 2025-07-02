use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::path::Path;
use std::thread::available_parallelism;
use tokio_util::task::LocalPoolHandle;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_status::SetStatus;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use mf_engine::model::DecisionContent;
use mf_engine::{DecisionEngine, EvaluationError, EvaluationOptions};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use zip::ZipArchive;

const IS_DEVELOPMENT: bool = cfg!(debug_assertions);
const STATIC_RESOURCES_URL: &str =
    "https://pricing-dev.oss-cn-hangzhou.aliyuncs.com/static/static.zip"; // 替换为实际的静态资源URL

pub async fn start_dev_server() {
    // 确保静态资源存在
    if let Err(e) = ensure_static_resources().await {
        tracing::error!("确保静态资源失败: {}", e);
    }

    let local_pool = LocalPoolHandle::new(
        available_parallelism().map(Into::into).unwrap_or(1),
    );

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    "devtool_rules_backend=info,tower_http=info".into()
                }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host_address =
        IS_DEVELOPMENT.then_some("127.0.0.1").unwrap_or("0.0.0.0");
    let listener_address = format!("{host_address}:3000");

    let app = Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/simulate",
            post(simulate).layer(DefaultBodyLimit::max(16 * 1024 * 1024)),
        )
        .layer(Extension(local_pool))
        .nest_service("/", serve_dir_service());

    let listener =
        tokio::net::TcpListener::bind(listener_address).await.unwrap();
    let compression_layer = CompressionLayer::new().gzip(true).br(true);

    tracing::info!("🚀 Listening on http://{}", listener.local_addr().unwrap());

    let mut app_with_layers =
        app.layer(TraceLayer::new_for_http()).layer(compression_layer);
    if let Ok(_) = env::var("CORS_PERMISSIVE") {
        app_with_layers = app_with_layers.layer(CorsLayer::permissive())
    }

    axum::serve(listener, app_with_layers).await.unwrap();
}

async fn download_static_resources(
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let work_dir = env::current_dir()?;
    let zip_path = work_dir.join("static.zip");

    // 下载文件
    tracing::info!("开始下载静态资源: {}", STATIC_RESOURCES_URL);
    let response = reqwest::get(STATIC_RESOURCES_URL).await?;

    if !response.status().is_success() {
        return Err(
            format!("下载失败，HTTP状态码: {}", response.status()).into()
        );
    }

    let bytes = response.bytes().await?;
    tracing::info!("下载完成，文件大小: {} 字节", bytes.len());

    // 保存zip文件
    let mut file = File::create(&zip_path)?;
    file.write_all(&bytes)?;
    tracing::info!("ZIP文件已保存到: {:?}", zip_path);

    Ok(zip_path)
}

fn extract_static_resources(
    zip_path: &PathBuf
) -> Result<(), Box<dyn std::error::Error>> {
    let work_dir = env::current_dir()?;

    // 解压文件
    tracing::info!("开始解压文件: {:?}", zip_path);
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    tracing::info!("ZIP文件包含 {} 个文件", archive.len());

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = work_dir.join(file.name());

        tracing::info!("正在解压: {} -> {:?}", file.name(), outpath);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
            tracing::info!("创建目录: {:?}", outpath);
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                    tracing::info!("创建父目录: {:?}", parent);
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
            tracing::info!("解压文件完成: {:?}", outpath);
        }
    }

    // 清理zip文件
    tracing::info!("清理临时ZIP文件: {:?}", zip_path);
    fs::remove_file(zip_path)?;

    tracing::info!("所有文件解压完成");
    Ok(())
}

async fn ensure_static_resources() -> Result<(), Box<dyn std::error::Error>> {
    let work_dir = env::current_dir()?;
    let static_path = work_dir.join("static");

    if static_path.exists() {
        tracing::info!("静态资源目录已存在: {:?}", static_path);
        return Ok(());
    }

    tracing::info!("静态资源不存在，开始下载...");
    let zip_path = download_static_resources().await?;
    extract_static_resources(&zip_path)?;
    tracing::info!("静态资源下载并解压成功");

    Ok(())
}

fn serve_dir_service() -> ServeDir<SetStatus<ServeFile>> {
    let work_dir = env::current_dir()
        .ok()
        .map_or("static".to_string(), |dir| dir.to_string_lossy().to_string());
    let static_path = Path::new(&work_dir).join("static");
    let index_path = static_path.join("index.html");
    tracing::info!("提供静态文件: {:?}", static_path.display());
    ServeDir::new(static_path).not_found_service(ServeFile::new(index_path))
}

async fn health() -> (StatusCode, String) {
    (StatusCode::OK, String::from("healthy"))
}

#[derive(Deserialize, Serialize)]
struct SimulateRequest {
    context: Value,
    content: DecisionContent,
}

async fn simulate(
    Extension(local_pool): Extension<LocalPoolHandle>,
    Json(payload): Json<SimulateRequest>,
) -> Result<Json<Value>, SimulateError> {
    let engine = DecisionEngine::default();
    let decision = engine.create_decision(payload.content.into());

    let result = local_pool
        .spawn_pinned(move || async move {
            decision
                .evaluate_with_opts(
                    payload.context.into(),
                    EvaluationOptions { trace: Some(true), max_depth: None },
                )
                .await
                .map(serde_json::to_value)
        })
        .await
        .expect("Thread failed to join")?
        .map_err(|_| {
            SimulateError::from(Box::new(EvaluationError::DepthLimitExceeded))
        })?;

    Ok(Json(result))
}

struct SimulateError(Box<EvaluationError>);

impl IntoResponse for SimulateError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            serde_json::to_string(&self.0).unwrap_or_default(),
        )
            .into_response()
    }
}

impl From<Box<EvaluationError>> for SimulateError {
    fn from(value: Box<EvaluationError>) -> Self {
        Self(value)
    }
}

use std::future::Future;
use std::ops::{Deref, DerefMut};

use axum::{routing::MethodRouter, Router};
use tokio::signal;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::info;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
        },
        _ = terminate => {},
    }
}

pub struct App {
    app: Router,
    addr: String,
}

impl App {
    pub fn new(
        app: Router,
        addr: String,
    ) -> Self {
        Self { app, addr }
    }
    pub async fn run_with_graceful_shutdown<F>(
        self,
        signal: F,
    ) where
        F: Future<Output = ()> + Send + 'static,
    {
        info!("listen:{}", self.addr.clone());
        let cors = CorsLayer::new()
            // 允许来自 localhost:8080 的请求
            .allow_origin(AllowOrigin::any())
            // 允许 GET, POST, PUT, DELETE, OPTIONS 方法
            .allow_methods(Any)
            // 允许自定义头部，包括 agency_code
            .allow_headers(Any)
            // 允许凭证（cookies 等）
            .allow_credentials(false);
        let listener = tokio::net::TcpListener::bind(self.addr).await.unwrap();
        axum::serve(listener, self.app.layer(cors))
            .with_graceful_shutdown(signal)
            .await
            .unwrap();
    }
    pub fn run(self) {
        tokio::spawn(async move {
            self.run_with_graceful_shutdown(shutdown_signal()).await;
        });
    }
}

impl Deref for App {
    type Target = Router;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl DerefMut for App {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.app
    }
}

pub struct AppBuilder {
    root: Router,
    addr: String,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self { root: Router::new(), addr: "127.0.0.1:20008".to_string() }
    }
    pub fn build(self) -> App {
        App::new(self.root, self.addr)
    }
    pub fn next(
        mut self,
        path: &str,
        router: Router,
    ) -> Self {
        self.root = self.root.nest(path, router);
        self
    }
    pub fn route(
        mut self,
        path: &str,
        method: MethodRouter,
    ) -> Self {
        self.root = self.root.route(path, method);
        self
    }
    pub fn addr(
        mut self,
        addr: String,
    ) -> Self {
        self.addr = addr;
        self
    }
}

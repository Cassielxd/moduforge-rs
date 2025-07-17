use axum::Router;

use crate::controller::{fbfx_csxm, gcxm};

pub fn build_app() -> Router {
    Router::new()
        .nest("/gcxm", gcxm::build_app()) //工程项目
        .nest("/fbfx_csxm", fbfx_csxm::build_app()) //分部分项 措施项目
}

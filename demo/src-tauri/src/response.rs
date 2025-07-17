use anyhow::anyhow;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::{error::AppError, ResponseResult};
#[derive(Serialize)]
pub struct Res<T> {
    pub code: usize,
    pub msg: Option<String>,
    pub data: Option<T>,
}
impl<T> Res<T> {
    pub fn success(data: T) -> ResponseResult<T> {
        Ok(Res {
            code: 200,
            msg: None,
            data: Some(data),
        })
    }
    pub fn error(msg: String) -> ResponseResult<T> {
        Err(AppError(anyhow!(msg)))
    }
}

impl<T> IntoResponse for Res<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

#[macro_export]
macro_rules! res {
    // 成功响应，带数据
    ($data:expr) => {
        Res::success($data)
    };
    // 错误响应，带消息
    (err $msg:expr) => {
        Res::error($msg.to_string())
    };
    // 自定义错误码和消息
    (err $code:expr, $msg:expr) => {
        Res {
            code: $code,
            msg: Some($msg.to_string()),
            data: None,
        }
    };
}

use std::future::Future;

use anyhow::anyhow;

use crate::loader::{DecisionLoader, LoaderError, LoaderResponse};

/// 默认加载器总是失败
#[derive(Default, Debug)]
pub struct NoopLoader;

impl DecisionLoader for NoopLoader {
    fn load<'a>(&'a self, key: &'a str) -> impl Future<Output = LoaderResponse> + 'a {
        async move {
            Err(LoaderError::Internal {
                key: key.to_string(),
                source: anyhow!("没有提供默认加载器"),
            }
            .into())
        }
    }
}

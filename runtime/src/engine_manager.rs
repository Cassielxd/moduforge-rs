use std::{
    collections::HashMap,
    env::current_dir,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use serde_json::Error;
use zen_engine::{
    DecisionEngine, Variable,
    loader::{FilesystemLoader, FilesystemLoaderOptions},
    model::DecisionContent,
};

use crate::types::Engine;

/// 规则引擎管理器
pub struct EngineManager {
    pub engine: Engine,
}
impl EngineManager {
    pub fn create(root: Option<PathBuf>) -> EngineManager {
        let loader = FilesystemLoader::new(FilesystemLoaderOptions {
            keep_in_memory: true,
            root: root
                .unwrap_or(current_dir().unwrap())
                .as_path()
                .to_string_lossy()
                .to_string(),
        });
        EngineManager {
            engine: Arc::new(DecisionEngine::default().with_loader(Arc::new(loader))),
        }
    }
    pub async fn evaluate<K, T>(&self, key: K, context: Variable) -> Result<T, Error>
    where
        K: AsRef<str>,
        T: serde::de::DeserializeOwned,
    {
        let response = self.engine.evaluate(key, context).await.unwrap();
        serde_json::from_value::<T>(response.result.to_value())
    }
}

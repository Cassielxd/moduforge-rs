use std::sync::Arc;

use moduforge_engine::get_engine;
use serde_json::Error;
use zen_engine::{
    handler::custom_node_adapter::NoopCustomNode, loader::FilesystemLoader, DecisionEngine, Variable,
};

pub struct EngineManager {
    pub engine: Arc<DecisionEngine<FilesystemLoader, NoopCustomNode>>,
}
impl EngineManager {
    pub fn create() -> EngineManager {
        EngineManager {
            engine: Arc::new(get_engine()),
        }
    }
    pub async fn evaluate<K,T>(&self, key: K,context: Variable)->Result<T,Error>
    where
        K: AsRef<str>,T: serde::de::DeserializeOwned,
    {
        let response =self.engine.evaluate(key, context).await.unwrap();
        let result = serde_json::from_value::<T>(response.result.to_value());
        result
    }
}

use std::sync::Arc;

use serde_json::Error;
use zen_engine::{
    handler::custom_node_adapter::NoopCustomNode, loader::MemoryLoader, model::DecisionContent,
    DecisionEngine, Variable,
};

/// 规则引擎管理器
pub struct EngineManager {
    pub engine: Arc<DecisionEngine<MemoryLoader, NoopCustomNode>>,
    pub loader: Arc<MemoryLoader>,
}
impl EngineManager {
    pub fn create() -> EngineManager {
        let memory_loader = Arc::new(MemoryLoader::default());
        EngineManager {
            engine: Arc::new(DecisionEngine::default().with_loader(memory_loader.clone())),
            loader: memory_loader,
        }
    }
    /// 添加决策
    pub fn add_decision(&self, key: &str, content: DecisionContent) {
        self.loader.add(key, content);
    }
    /// 删除决策
    pub fn remove_decision(&self, key: &str) -> bool {
        self.loader.remove(key)
    }

    pub async fn evaluate<K, T>(&self, key: K, context: Variable) -> Result<T, Error>
    where
        K: AsRef<str>,
        T: serde::de::DeserializeOwned,
    {
        let response = self.engine.evaluate(key, context).await.unwrap();
        let result = serde_json::from_value::<T>(response.result.to_value());
        result
    }
}

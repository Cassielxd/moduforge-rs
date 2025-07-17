// 增量数据存储

use std::sync::Arc;

use async_trait::async_trait;
use mf_collab_client::{utils::Utils, AwarenessRef};
use mf_state::{plugin::StateField, resource::Resource, State, StateConfig, Transaction};



pub struct CollabState;
impl Resource for CollabState {}

/// 权限状态字段管理器
#[derive(Debug)]
pub struct CollabStateField{
    awareness: AwarenessRef,
}

impl CollabStateField{
    pub fn new(awareness: AwarenessRef) -> Self {
        Self { awareness }
    }
}

#[async_trait]
impl StateField for CollabStateField {
    async fn init(&self, _config: &StateConfig, _instance: &State) -> Arc<dyn Resource> {
        Arc::new(CollabState)
    }
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _: &State,
    ) -> Arc<dyn Resource> {
        Utils::apply_transaction_to_yrs(self.awareness.clone(), tr).await;
        value
    }
}

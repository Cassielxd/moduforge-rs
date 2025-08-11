use std::{collections::HashMap, sync::Arc};

use mf_transform::{
    attr_step::AttrStep,
    mark_step::AddMarkStep,
    node_step::{AddNodeStep, MoveNodeStep, RemoveNodeStep},
    step::Step,
};
use std::fmt::Debug;

pub trait StepFactory: Send + Sync + Debug {
    fn create_from_bytes(&self, bytes: &[u8]) -> Arc<dyn Step>;
}

#[derive(Debug)]
pub struct StepFactoryRegistry {
    factories: HashMap<String, Arc<dyn StepFactory>>,
}

impl Default for StepFactoryRegistry {
    fn default() -> Self { Self::new() }
}

impl StepFactoryRegistry {
    pub fn new() -> Self {
        let mut registry = StepFactoryRegistry { factories: HashMap::new() };
        // 属性更新
        registry.register("attr_step", Arc::new(AttrStepFactory));
        // 添加标记
        registry.register("add_mark_step", Arc::new(AddMarkStepFactory));
        // 添加节点
        registry.register("add_node_step", Arc::new(AddNodeStepFactory));
        // 删除节点
        registry.register("remove_node_step", Arc::new(RemoveNodeStepFactory));
        // 移动节点
        registry.register("move_node_step", Arc::new(MoveNodeStepFactory));
        registry
    }

    pub fn register(&mut self, type_id: &str, factory: Arc<dyn StepFactory>) {
        self.factories.insert(type_id.to_string(), factory);
    }

    pub fn create(&self, type_id: &str, bytes: &[u8]) -> Arc<dyn Step> {
        self.factories
            .get(type_id)
            .expect("Unknown step type")
            .create_from_bytes(bytes)
    }
}

#[derive(Debug)]
pub struct AttrStepFactory;
impl StepFactory for AttrStepFactory {
    fn create_from_bytes(&self, bytes: &[u8]) -> Arc<dyn Step> {
        let step: AttrStep = serde_json::from_slice(bytes).unwrap();
        Arc::new(step)
    }
}

#[derive(Debug)]
pub struct AddMarkStepFactory;
impl StepFactory for AddMarkStepFactory {
    fn create_from_bytes(&self, bytes: &[u8]) -> Arc<dyn Step> {
        let step: AddMarkStep = serde_json::from_slice(bytes).unwrap();
        Arc::new(step)
    }
}

#[derive(Debug)]
pub struct AddNodeStepFactory;
impl StepFactory for AddNodeStepFactory {
    fn create_from_bytes(&self, bytes: &[u8]) -> Arc<dyn Step> {
        let step: AddNodeStep = serde_json::from_slice(bytes).unwrap();
        Arc::new(step)
    }
}

#[derive(Debug)]
pub struct RemoveNodeStepFactory;
impl StepFactory for RemoveNodeStepFactory {
    fn create_from_bytes(&self, bytes: &[u8]) -> Arc<dyn Step> {
        let step: RemoveNodeStep = serde_json::from_slice(bytes).unwrap();
        Arc::new(step)
    }
}

#[derive(Debug)]
pub struct MoveNodeStepFactory;
impl StepFactory for MoveNodeStepFactory {
    fn create_from_bytes(&self, bytes: &[u8]) -> Arc<dyn Step> {
        let step: MoveNodeStep = serde_json::from_slice(bytes).unwrap();
        Arc::new(step)
    }
}



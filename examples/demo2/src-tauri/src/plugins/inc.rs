// 增量数据存储

use std::sync::Arc;

use async_trait::async_trait;
use mf_model::{attrs::Attrs, mark::Mark, node::Node, NodeId};
use mf_state::{
    plugin::{PluginMetadata, PluginTrait, StateField},
    resource::Resource,
    State, StateConfig, Transaction,
};
use mf_transform::{
    attr_step::AttrStep,
    mark_step::{AddMarkStep, RemoveMarkStep},
    node_step::{AddNodeStep, RemoveNodeStep},
};
use serde::{Deserialize, Serialize};

pub struct IncState;
impl Resource for IncState {}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Operations(pub Vec<Operation>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    RemoveMark(NodeId, Vec<String>),
    AddMark(NodeId, Vec<Mark>),
    UpdateAttrs(NodeId, Attrs),
    UpdateNode(Vec<Arc<Node>>),
    RemoveNode(Vec<NodeId>),
}
impl Resource for Operations {}
/// 权限状态字段管理器
#[derive(Debug)]
pub struct IncStateField;

const INC_DATA_KEY: &str = "inc_data";

impl IncStateField {
    ///收集增量的数据更新
    pub fn collect_tr(
        tr: &Transaction,
        new_state: &State,
    ) {
        if tr.steps.is_empty() {
            return;
        }
        let manager = new_state.resource_manager();
        let mut operations: Vec<Operation> = Vec::new();
        //清除老的增量数据
        //收集增量数据
        for (index, step) in tr.steps.iter().enumerate() {
            // 添加节点
            if let Some(add_step) = step.downcast_ref::<AddNodeStep>() {
                let mut node_ids = Vec::new();
                for node_enum in add_step.nodes.iter() {
                    node_ids.extend(AddNodeStep::collect_node_ids(node_enum));
                }
                let mut nodes = Vec::new();
                for node_id in node_ids.iter() {
                    let node = tr.doc().get_node(node_id);
                    if let Some(node) = node {
                        nodes.push(node);
                    }
                }
                if !nodes.is_empty() {
                    operations.push(Operation::UpdateNode(nodes));
                }
            }
            // 删除节点
            if step.downcast_ref::<RemoveNodeStep>().is_some() {
                let mut node_ids = Vec::new();
                // 获取反向操作的节点 id 由于删除 有可能删除多个节点 并包含子节点 只拿RemoveNodeStep 中的 id是不够的
                if let Some(add_step) =
                    tr.invert_steps[index].downcast_ref::<AddNodeStep>()
                {
                    for node_enum in add_step.nodes.iter() {
                        node_ids
                            .extend(AddNodeStep::collect_node_ids(node_enum));
                    }
                }
                if !node_ids.is_empty() {
                    operations.push(Operation::RemoveNode(node_ids));
                }
            }
            // 更新节点
            if let Some(attr_step) = step.downcast_ref::<AttrStep>() {
                let node = tr.doc().get_node(&attr_step.id);
                if let Some(node) = node {
                    operations.push(Operation::UpdateAttrs(
                        attr_step.id.clone(),
                        node.attrs.clone(),
                    ));
                }
            }
            // 添加标记
            if let Some(add_mark_step) = step.downcast_ref::<AddMarkStep>() {
                let node = tr.doc().get_node(&add_mark_step.id);
                if let Some(_) = node {
                    operations.push(Operation::AddMark(
                        add_mark_step.id.clone(),
                        add_mark_step.marks.clone(),
                    ));
                }
            }
            // 删除标记
            if let Some(remove_mark_step) =
                step.downcast_ref::<RemoveMarkStep>()
            {
                let node = tr.doc().get_node(&remove_mark_step.id);
                if let Some(_) = node {
                    operations.push(Operation::RemoveMark(
                        remove_mark_step.id.clone(),
                        remove_mark_step.mark_types.clone(),
                    ));
                }
            }
        }
        if !operations.is_empty() {
            manager
                .resource_table
                .add(INC_DATA_KEY.to_string(), Operations(operations));
        }
    }
}

#[async_trait]
impl StateField for IncStateField {
    type Value = IncState;

    async fn init(
        &self,
        _config: &StateConfig,
        _instance: &State,
    ) -> Arc<Self::Value> {
        Arc::new(IncState)
    }
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<Self::Value>,
        _old_state: &State,
        new_state: &State,
    ) -> Arc<Self::Value> {
        IncStateField::collect_tr(tr, new_state);
        value
    }
}

#[derive(Debug)]
pub struct IncStatePlugin;

impl PluginTrait for IncStatePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "inc_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "增量数据插件".to_string(),
            author: "collab".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }
}

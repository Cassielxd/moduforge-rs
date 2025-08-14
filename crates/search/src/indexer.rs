use crate::backend::IndexMutation;
use crate::model::IndexDoc;
use crate::step_registry::{global_registry, StepIndexContext};
use mf_model::{Node, NodeId};
use mf_model::{node_pool::NodePool, node_type::NodeEnum};
use mf_transform::step::Step;
use mf_transform::{attr_step::AttrStep, mark_step::{AddMarkStep, RemoveMarkStep}, node_step::{AddNodeStep, MoveNodeStep, RemoveNodeStep}};
use std::sync::Arc;
use serde::Deserialize;

/// 将单个 Step 翻译为增量索引变更
/// - 删除：使用 pool_before 收集子树
/// - 新增/修改/移动：使用 pool_after 生成文档
pub fn mutations_from_step(
    pool_before: &NodePool,
    pool_after: &NodePool,
    step: &Arc<dyn Step>,
) -> Vec<IndexMutation> {
    // 1) 先用注册表（支持扩展 Step 类型）
    if let Some(reg) = Some(global_registry().read().unwrap()) {
        let ctx = StepIndexContext { pool_before, pool_after };
        if let Some(muts) = reg.index_step(step.as_ref(), &ctx) {
            return muts;
        }
    }
    // 2) fallback：通过 downcast 处理内置步骤
    if let Some(s) = step.downcast_ref::<AttrStep>() {
        // 属性变化：使用 Upsert 目标节点
        if let Some(node) = pool_after.get_node(&s.id) {
            return vec![IndexMutation::Upsert(IndexDoc::from_node(pool_after, &node))];
        }
        return vec![];
    }

    if let Some(s) = step.downcast_ref::<AddMarkStep>() {
        if let Some(node) = pool_after.get_node(&s.id) {
            return vec![IndexMutation::Upsert(IndexDoc::from_node(pool_after, &node))];
        }
        return vec![];
    }

    if let Some(s) = step.downcast_ref::<RemoveMarkStep>() {
        if let Some(node) = pool_after.get_node(&s.id) {
            return vec![IndexMutation::Upsert(IndexDoc::from_node(pool_after, &node))];
        }
        return vec![];
    }

    if let Some(s) = step.downcast_ref::<AddNodeStep>() {
        // 新增子树：为所有节点生成 Add
        let mut muts = Vec::new();
        for ne in &s.nodes {
            collect_adds_for_node_enum(pool_after, ne, &mut muts);
        }
        return muts;
    }

    if let Some(s) = step.downcast_ref::<RemoveNodeStep>() {
        // 删除节点及子树：DeleteMany
        let mut all_ids: Vec<NodeId> = Vec::new();
        for id in &s.node_ids {
            // 使用变更前的 pool 收集删除子树
            if let Some(enum_subtree) = pool_before.get_inner().all_children(id, None) {
                all_ids.extend(collect_ids_from_enum(&enum_subtree));
            } else {
                all_ids.push(id.clone());
            }
        }
        return vec![IndexMutation::DeleteManyById(all_ids.into_iter().map(|id| id.to_string()).collect())];
    }

    if step.downcast_ref::<MoveNodeStep>().is_some() {
        // MoveNodeStep 字段为私有，改为通过序列化结果读取
        if let Some(bytes) = Step::serialize(step.as_ref()) {
            if let Ok(ms) = serde_json::from_slice::<MoveNodeSerde>(&bytes) {
                let mut muts = Vec::new();
                if let Some(enum_subtree) = pool_after.get_inner().all_children(&ms.node_id.clone(), None) {
                    collect_upserts_for_enum(pool_after, &enum_subtree, &mut muts);
                } else if let Some(node) = pool_after.get_node(&ms.node_id) {
                    muts.push(IndexMutation::Upsert(IndexDoc::from_node(pool_after, &node)));
                }
                return muts;
            }
        }
        return Vec::new();
    }

    Vec::new()
}

#[derive(Deserialize)]
struct MoveNodeSerde {
    #[serde(rename = "source_parent_id")] 
    _source_parent_id: NodeId,
    #[serde(rename = "target_parent_id")] 
    _target_parent_id: NodeId,
    node_id: NodeId,
    #[serde(rename = "position")] 
    _position: Option<usize>,
}

fn collect_ids_from_enum(ne: &NodeEnum) -> Vec<NodeId> {
    let mut ids = vec![ne.0.id.clone()];
    for c in &ne.1 { ids.extend(collect_ids_from_enum(c)); }
    ids
}

fn collect_adds_for_node_enum(
    pool: &NodePool,
    ne: &NodeEnum,
    out: &mut Vec<IndexMutation>,
) {
    let node = Arc::new(ne.0.clone());
    out.push(IndexMutation::Add(IndexDoc::from_node(pool, &node)));
    for c in &ne.1 { collect_adds_for_node_enum(pool, c, out); }
}

fn collect_upserts_for_enum(
    pool: &NodePool,
    ne: &NodeEnum,
    out: &mut Vec<IndexMutation>,
) {
    let node = Arc::new(ne.0.clone());
    out.push(IndexMutation::Upsert(IndexDoc::from_node(pool, &node)));
    for c in &ne.1 { collect_upserts_for_enum(pool, c, out); }
}



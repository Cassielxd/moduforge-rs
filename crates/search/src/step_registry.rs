use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};

use mf_model::node_pool::NodePool;
use mf_model::NodeId;
use mf_transform::step::Step;

use crate::backend::IndexMutation;
use crate::model::IndexDoc;

/// 步骤转换上下文（提供前后池以便获取节点/父链等）
pub struct StepIndexContext<'a> {
    pub pool_before: &'a NodePool,
    pub pool_after: &'a NodePool,
}

/// 类型安全的 Step→索引增量 转换器
pub trait TypedStepIndexer<T>: Send + Sync + 'static
where
    T: Step + 'static,
{
    fn index_step(
        &self,
        step: &T,
        ctx: &StepIndexContext,
    ) -> Vec<IndexMutation>;
    fn name() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }
}

/// 类型擦除的转换器
struct ErasedIndexer {
    type_id: TypeId,
    index_fn: fn(&dyn Any, &StepIndexContext) -> Vec<IndexMutation>,
}

impl ErasedIndexer {
    fn new<T, C>() -> Self
    where
        T: Step + 'static,
        C: TypedStepIndexer<T> + Default + 'static,
    {
        Self {
            type_id: TypeId::of::<T>(),
            index_fn: |step_any, ctx| {
                let converter = C::default();
                let step = step_any
                    .downcast_ref::<T>()
                    .expect("Type mismatch in step indexer");
                converter.index_step(step, ctx)
            },
        }
    }

    fn try_index(
        &self,
        step: &dyn Step,
        ctx: &StepIndexContext,
    ) -> Option<Vec<IndexMutation>> {
        if step.type_id() != self.type_id {
            return None;
        }
        Some((self.index_fn)(step as &dyn Any, ctx))
    }
}

/// 转换器注册表
#[derive(Default)]
pub struct StepIndexerRegistry {
    by_type: HashMap<TypeId, Arc<ErasedIndexer>>,
}

impl StepIndexerRegistry {
    pub fn new() -> Self {
        Self { by_type: HashMap::new() }
    }

    pub fn register<T, C>(&mut self) -> &mut Self
    where
        T: Step + 'static,
        C: TypedStepIndexer<T> + Default + 'static,
    {
        let type_id = TypeId::of::<T>();
        self.by_type
            .entry(type_id)
            .or_insert_with(|| Arc::new(ErasedIndexer::new::<T, C>()));
        self
    }

    pub fn index_step(
        &self,
        step: &dyn Step,
        ctx: &StepIndexContext,
    ) -> Option<Vec<IndexMutation>> {
        let type_id = step.type_id();
        self.by_type.get(&type_id).and_then(|e| e.try_index(step, ctx))
    }
}

static GLOBAL: OnceLock<RwLock<StepIndexerRegistry>> = OnceLock::new();

pub fn global_registry() -> &'static RwLock<StepIndexerRegistry> {
    GLOBAL.get_or_init(|| RwLock::new(StepIndexerRegistry::new()))
}

pub fn register_step_indexer<T, C>()
where
    T: Step + 'static,
    C: TypedStepIndexer<T> + Default + 'static,
{
    let mut reg = global_registry().write().unwrap();
    reg.register::<T, C>();
}

// ---------------- 内置步骤的默认转换器 ----------------
use mf_transform::attr_step::AttrStep;
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};
use mf_transform::node_step::{AddNodeStep, RemoveNodeStep, MoveNodeStep};
use mf_model::node_type::NodeEnum;
use serde::Deserialize;

#[derive(Default)]
struct AttrIndexer;
impl TypedStepIndexer<AttrStep> for AttrIndexer {
    fn index_step(
        &self,
        step: &AttrStep,
        ctx: &StepIndexContext,
    ) -> Vec<IndexMutation> {
        if let Some(node) = ctx.pool_after.get_node(&step.id) {
            vec![IndexMutation::Upsert(IndexDoc::from_node(
                ctx.pool_after,
                &node,
            ))]
        } else {
            Vec::new()
        }
    }
}

#[derive(Default)]
struct AddMarkIndexer;
impl TypedStepIndexer<AddMarkStep> for AddMarkIndexer {
    fn index_step(
        &self,
        step: &AddMarkStep,
        ctx: &StepIndexContext,
    ) -> Vec<IndexMutation> {
        if let Some(node) = ctx.pool_after.get_node(&step.id) {
            vec![IndexMutation::Upsert(IndexDoc::from_node(
                ctx.pool_after,
                &node,
            ))]
        } else {
            Vec::new()
        }
    }
}

#[derive(Default)]
struct RemoveMarkIndexer;
impl TypedStepIndexer<RemoveMarkStep> for RemoveMarkIndexer {
    fn index_step(
        &self,
        step: &RemoveMarkStep,
        ctx: &StepIndexContext,
    ) -> Vec<IndexMutation> {
        if let Some(node) = ctx.pool_after.get_node(&step.id) {
            vec![IndexMutation::Upsert(IndexDoc::from_node(
                ctx.pool_after,
                &node,
            ))]
        } else {
            Vec::new()
        }
    }
}

#[derive(Default)]
struct AddNodeIndexer;
impl TypedStepIndexer<AddNodeStep> for AddNodeIndexer {
    fn index_step(
        &self,
        step: &AddNodeStep,
        ctx: &StepIndexContext,
    ) -> Vec<IndexMutation> {
        let mut muts = Vec::new();
        for ne in &step.nodes {
            collect_adds_for_node_enum(ctx.pool_after, ne, &mut muts);
        }
        muts
    }
}

#[derive(Default)]
struct RemoveNodeIndexer;
impl TypedStepIndexer<RemoveNodeStep> for RemoveNodeIndexer {
    fn index_step(
        &self,
        step: &RemoveNodeStep,
        ctx: &StepIndexContext,
    ) -> Vec<IndexMutation> {
        let mut all_ids: Vec<String> = Vec::new();
        for id in &step.node_ids {
            if let Some(enum_subtree) =
                ctx.pool_before.get_inner().all_children(id, None)
            {
                all_ids.extend(
                    collect_ids_from_enum(&enum_subtree)
                        .into_iter()
                        .map(|id| id.to_string()),
                );
            } else {
                all_ids.push(id.to_string());
            }
        }
        vec![IndexMutation::DeleteManyById(all_ids)]
    }
}

#[derive(Default)]
struct MoveNodeIndexer;
impl TypedStepIndexer<MoveNodeStep> for MoveNodeIndexer {
    fn index_step(
        &self,
        step: &MoveNodeStep,
        ctx: &StepIndexContext,
    ) -> Vec<IndexMutation> {
        // MoveNodeStep 的字段为私有，使用序列化提取 node_id
        #[derive(Deserialize)]
        struct MoveNodeSerde {
            node_id: NodeId,
        }
        if let Some(bytes) = Step::serialize(step) {
            if let Ok(ms) = serde_json::from_slice::<MoveNodeSerde>(&bytes) {
                let mut muts = Vec::new();
                if let Some(enum_subtree) =
                    ctx.pool_after.get_inner().all_children(&ms.node_id, None)
                {
                    collect_upserts_for_enum(
                        ctx.pool_after,
                        &enum_subtree,
                        &mut muts,
                    );
                } else if let Some(node) = ctx.pool_after.get_node(&ms.node_id)
                {
                    muts.push(IndexMutation::Upsert(IndexDoc::from_node(
                        ctx.pool_after,
                        &node,
                    )));
                }
                return muts;
            }
        }
        Vec::new()
    }
}

fn collect_ids_from_enum(ne: &NodeEnum) -> Vec<NodeId> {
    let mut ids = vec![ne.0.id.clone()];
    for c in &ne.1 {
        ids.extend(collect_ids_from_enum(c));
    }
    ids
}

fn collect_adds_for_node_enum(
    pool: &NodePool,
    ne: &NodeEnum,
    out: &mut Vec<IndexMutation>,
) {
    let node = std::sync::Arc::new(ne.0.clone());
    out.push(IndexMutation::Add(IndexDoc::from_node(pool, &node)));
    for c in &ne.1 {
        collect_adds_for_node_enum(pool, c, out);
    }
}

fn collect_upserts_for_enum(
    pool: &NodePool,
    ne: &NodeEnum,
    out: &mut Vec<IndexMutation>,
) {
    let node = std::sync::Arc::new(ne.0.clone());
    out.push(IndexMutation::Upsert(IndexDoc::from_node(pool, &node)));
    for c in &ne.1 {
        collect_upserts_for_enum(pool, c, out);
    }
}

use std::sync::OnceLock as _OnceLock;
static DEFAULTS: _OnceLock<()> = _OnceLock::new();

pub fn ensure_default_step_indexers() {
    DEFAULTS.get_or_init(|| {
        register_step_indexer::<AttrStep, AttrIndexer>();
        register_step_indexer::<AddMarkStep, AddMarkIndexer>();
        register_step_indexer::<RemoveMarkStep, RemoveMarkIndexer>();
        register_step_indexer::<AddNodeStep, AddNodeIndexer>();
        register_step_indexer::<RemoveNodeStep, RemoveNodeIndexer>();
        register_step_indexer::<MoveNodeStep, MoveNodeIndexer>();
    });
}

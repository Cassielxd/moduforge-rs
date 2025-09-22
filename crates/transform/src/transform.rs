use std::sync::{Arc, RwLock};

use mf_model::{node_pool::NodePool, schema::Schema, tree::Tree};

use crate::TransformResult;

use super::step::{Step, StepResult};

/// 延迟计算的文档状态
#[derive(Debug, Clone)]
pub enum LazyDoc {
    /// 原始文档，未进行任何修改
    Original(Arc<NodePool>),
    /// 需要重新计算的状态，包含基础文档和待应用的步骤
    Pending { base: Arc<NodePool>, steps: imbl::Vector<Arc<dyn Step>> },
    /// 已计算的最新状态
    Computed(Arc<NodePool>),
}

#[derive(Debug, Clone)]
pub struct Transform {
    draft: Arc<RwLock<Tree>>,
}

impl Transform {
    pub fn new(tree: Tree) -> Transform {
        Transform { draft: Arc::new(RwLock::new(tree)) }
    }
    pub fn new_shared(draft: Arc<RwLock<Tree>>) -> Transform {
        Transform { draft }
    }
    pub fn get_draft(&self) -> Arc<RwLock<Tree>> {
        self.draft.clone()
    }
}

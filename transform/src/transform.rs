use std::{fmt, sync::Arc};

use moduforge_model::{node_pool::NodePool, schema::Schema};

use crate::{draft::Draft, patch::Patch};

use super::step::{Step, StepResult};

// 定义 TransformError 结构体
#[derive(Debug)]
pub struct TransformError {
    message: String,
}
impl fmt::Display for TransformError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for TransformError {}

impl TransformError {
    pub fn new(message: String) -> Self {
        TransformError { message }
    }
}
#[derive(Debug, Clone)]
pub struct Transform {
    /// 当前文档状态
    pub doc: Arc<NodePool>,
    /// 文档的草稿状态，用于临时修改
    pub draft: Draft,
    /// 存储所有操作步骤
    pub steps: im::Vector<Arc<dyn Step>>,
    /// 存储每个步骤对应的补丁列表
    pub patches: im::Vector<Vec<Patch>>,
    /// 文档的模式定义
    pub schema: Arc<Schema>,
}
impl Transform {
    pub fn step(
        &mut self,
        step: Arc<dyn Step>,
    ) -> Result<(), TransformError> {
        let result = step.apply(&mut self.draft, self.schema.clone())?;
        match result.failed {
            Some(message) => Err(TransformError::new(message)),
            None => {
                self.add_step(step, result);
                Ok(())
            },
        }
    }
    /// 检查文档是否被修改
    pub fn doc_changed(&self) -> bool {
        !self.steps.is_empty()
    }
    /// 添加一个步骤及其结果到事务中
    pub fn add_step(
        &mut self,
        step: Arc<dyn Step>,
        result: StepResult,
    ) {
        self.steps.push_back(step);
        self.patches.push_back(result.patches);
        self.doc = result.doc.unwrap();
    }
    
}

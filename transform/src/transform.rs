use std::{fmt, sync::Arc};

use moduforge_model::{node_pool::NodePool, schema::Schema, tree::Tree};

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
    pub draft: Tree,
    /// 存储所有操作步骤
    pub steps: im::Vector<Arc<dyn Step>>,
    /// 文档的模式定义
    pub schema: Arc<Schema>,
}
impl Transform {
    pub fn new(
        doc: Arc<NodePool>,
        schema: Arc<Schema>,
    ) -> Transform {
        Transform {
            doc: doc.clone(),
            draft: doc.get_inner().as_ref().clone(),
            steps: im::Vector::new(),
            schema,
        }
    }
    pub fn step(
        &mut self,
        step: Arc<dyn Step>,
    ) -> Result<(), TransformError> {
        let result: StepResult =
            step.apply(&mut self.draft, self.schema.clone())?;
        match result.failed {
            Some(message) => Err(TransformError::new(message)),
            None => {
                self.add_step(step);
                Ok(())
            },
        }
    }
    /// 检查文档是否被修改
    pub fn doc_changed(&self) -> bool {
        !self.steps.is_empty()
    }
    /// 添加一个步骤及其结果到事务中
    fn add_step(
        &mut self,
        step: Arc<dyn Step>,
    ) {
        self.steps.push_back(step);
        self.doc = NodePool::new(Arc::new(self.draft.clone()));
    }
}

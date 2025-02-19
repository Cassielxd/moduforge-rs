use std::{fmt, sync::Arc};

use crate::model::node_pool::NodePool;

use super::step::{Step, StepResult};

// 定义 TransformError 结构体
#[derive(Debug)]
pub struct TransformError {
    message: String,
}
impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for TransformError {}

impl TransformError {
    pub fn new(message: String) -> Self {
        TransformError { message }
    }
}

pub trait Transform {
    fn step(&mut self, step: Box<dyn Step>) -> Result<(), TransformError>;
    fn doc_changed(&self) -> bool;
    fn add_step(&mut self, step: Box<dyn Step>, result: StepResult);
}

use std::{fmt, sync::Arc};

use crate::model::{node_pool::NodePool, schema::Schema};

use super::step::Step;


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
    fn new(message: String) -> Self {
        TransformError { message }
    }
}

pub struct Transform {
    pub steps: Vec<Box<dyn Step>>,
    pub docs: Vec<Arc<NodePool>>,
    pub doc: Arc<NodePool>,
    schema: Arc<Schema>,
}
impl Transform {
    pub fn new(doc: Arc<NodePool>, schema: Arc<Schema>) -> Transform {
        Transform {
            steps: vec![],
            docs: vec![],
            doc,
            schema,
        }
    }
    fn before(&self) -> &NodePool {
        self.docs.get(0).unwrap_or(&self.doc)
    }

    fn step(&mut self, step: Box<dyn Step>) -> Result<(), TransformError> {
        let result = step.apply(self.doc.clone(), self.schema.clone());
        match result.failed {
            Some(message) => Err(TransformError::new(message)),
            None => {
                self.add_step(step, result.doc.unwrap());
                Ok(())
            }
        }
    }

    fn doc_changed(&self) -> bool {
        !self.steps.is_empty()
    }

    fn add_step(&mut self, step: Box<dyn Step>, doc: Arc<NodePool>) {
        self.docs.push(self.doc.clone());
        self.steps.push(step);
        self.doc = doc;
    }
}

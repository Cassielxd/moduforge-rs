//! ModuForge-RS 数据转换模块
//!
//! 该模块负责处理文档的转换操作，包括：
//! - 节点操作（添加、移动、删除、替换）
//! - 标记操作
//! - 属性更新
//! - 批量操作
//! - 补丁应用
//!
//! 主要组件：
//! - `attr_step`: 属性步骤，处理属性更新操作
//! - `draft`: 草稿系统，管理文档的临时状态
//! - `mark_step`: 标记步骤，处理标记的添加和删除
//! - `node_step`: 节点步骤，处理节点的各种操作
//! - `patch`: 补丁系统，用于增量更新
//! - `step`: 步骤定义，定义转换操作的基本接口
//! - `transform`: 转换系统，协调各种转换操作
//!
//! 核心类型：
//! - `ConcreteStep`: 具体步骤枚举，表示所有可能的转换操作
//! - `PatchStep`: 补丁步骤，用于应用补丁
//! - `BatchStep`: 批量步骤，用于执行多个转换操作

use std::sync::Arc;

use attr_step::AttrStep;
use draft::Draft;
use mark_step::AddMarkStep;
use node_step::{AddNodeStep, MoveNodeStep, RemoveNodeStep, ReplaceNodeStep};
use patch::Patch;
use serde::{Deserialize, Serialize};
use step::{Step, StepResult};
use transform::TransformError;

use moduforge_model::schema::Schema;
pub mod attr_step;
pub mod draft;
pub mod mark_step;
pub mod node_step;
pub mod patch;
pub mod step;
pub mod transform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConcreteStep {
    UpdateAttrs(AttrStep),
    AddNodeStep(AddNodeStep),
    AddMarkStep(AddMarkStep),
    RemoveNodeStep(RemoveNodeStep),
    PatchStep(PatchStep),
    MoveNodeStep(MoveNodeStep),
    ReplaceNodeStep(ReplaceNodeStep),
    BatchStep(BatchStep),
}
impl Step for ConcreteStep {
    fn apply(
        &self,
        dart: &mut Draft,
        schema: std::sync::Arc<moduforge_model::schema::Schema>,
    ) -> Result<step::StepResult, transform::TransformError> {
        match self {
            ConcreteStep::UpdateAttrs(attr_step) => {
                attr_step.apply(dart, schema)
            },
            ConcreteStep::AddNodeStep(add_node_step) => {
                add_node_step.apply(dart, schema)
            },
            ConcreteStep::AddMarkStep(add_mark_step) => {
                add_mark_step.apply(dart, schema)
            },
            ConcreteStep::RemoveNodeStep(remove_node_step) => {
                remove_node_step.apply(dart, schema)
            },
            ConcreteStep::PatchStep(patch_step) => {
                patch_step.apply(dart, schema)
            },
            ConcreteStep::MoveNodeStep(move_node_step) => {
                move_node_step.apply(dart, schema)
            },
            ConcreteStep::BatchStep(batch_step) => {
                batch_step.apply(dart, schema)
            },
            ConcreteStep::ReplaceNodeStep(replace_node_step) => {
                replace_node_step.apply(dart, schema)
            },
        }
    }
    fn to_concrete(&self) -> ConcreteStep {
        self.clone()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchStep {
    pub patches: Vec<Patch>,
}
impl Step for PatchStep {
    fn apply(
        &self,
        dart: &mut Draft,
        _: std::sync::Arc<moduforge_model::schema::Schema>,
    ) -> Result<step::StepResult, transform::TransformError> {
        match dart.apply_patches(&self.patches) {
            Ok(()) => Ok(dart.commit()),
            Err(err) => Err(TransformError::new(err.to_string())),
        }
    }

    fn to_concrete(&self) -> ConcreteStep {
        ConcreteStep::PatchStep(self.clone())
    }
}
/// 批量操作步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchStep {
    steps: Vec<ConcreteStep>,
}

impl BatchStep {
    pub fn new(steps: Vec<ConcreteStep>) -> Self {
        BatchStep { steps }
    }
}
impl Step for BatchStep {
    fn apply(
        &self,
        dart: &mut Draft,
        schema: Arc<Schema>,
    ) -> Result<StepResult, TransformError> {
        dart.begin = true;
        for step in &self.steps {
            let schema = schema.clone();
            let result = match step {
                ConcreteStep::UpdateAttrs(attr_step) => {
                    attr_step.apply(dart, schema)
                },
                ConcreteStep::AddNodeStep(add_node_step) => {
                    add_node_step.apply(dart, schema)
                },
                ConcreteStep::AddMarkStep(add_mark_step) => {
                    add_mark_step.apply(dart, schema)
                },
                ConcreteStep::RemoveNodeStep(remove_node_step) => {
                    remove_node_step.apply(dart, schema)
                },
                ConcreteStep::PatchStep(patch_step) => {
                    patch_step.apply(dart, schema)
                },
                ConcreteStep::MoveNodeStep(move_node_step) => {
                    move_node_step.apply(dart, schema)
                },
                ConcreteStep::ReplaceNodeStep(replace_node_step) => {
                    replace_node_step.apply(dart, schema)
                },
                ConcreteStep::BatchStep(batch_setp) => {
                    batch_setp.apply(dart, schema)
                },
            };
            match result {
                Ok(result) => {
                    if let Some(message) = result.failed {
                        return Ok(StepResult::fail(message));
                    }
                    // 继续执行下一个步骤
                },
                Err(err) => return Err(err),
            }
        }
        dart.begin = false;
        // 所有步骤执行成功，提交更改
        Ok(dart.commit())
    }

    fn to_concrete(&self) -> ConcreteStep {
        ConcreteStep::BatchStep(self.clone())
    }
}

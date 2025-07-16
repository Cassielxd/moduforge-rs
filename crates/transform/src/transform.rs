use std::{sync::Arc};

use mf_model::{node_pool::NodePool, schema::Schema, tree::Tree};

use crate::TransformResult;

use super::step::{Step, StepResult};

/// 延迟计算的文档状态
#[derive(Debug, Clone)]
enum LazyDoc {
    /// 原始文档，未进行任何修改
    Original(Arc<NodePool>),
    /// 需要重新计算的状态，包含基础文档和待应用的步骤
    Pending { base: Arc<NodePool>, steps: imbl::Vector<Arc<dyn Step>> },
    /// 已计算的最新状态
    Computed(Arc<NodePool>),
}

#[derive(Debug, Clone)]
pub struct Transform {
    /// 原始文档状态
    pub base_doc: Arc<NodePool>,
    /// 延迟计算的当前文档状态
    lazy_doc: LazyDoc,
    /// 文档的草稿状态，用于临时修改 (Copy-on-Write)
    draft: Option<Tree>,
    /// 存储所有操作步骤
    pub steps: imbl::Vector<Arc<dyn Step>>,
    /// 存储所有反向操作步骤
    pub invert_steps: imbl::Vector<Arc<dyn Step>>,
    /// 文档的模式定义
    pub schema: Arc<Schema>,
    /// 标记是否需要重新计算文档状态
    needs_recompute: bool,
}

impl Transform {
    pub fn new(
        doc: Arc<NodePool>,
        schema: Arc<Schema>,
    ) -> Transform {
        Transform {
            base_doc: doc.clone(),
            lazy_doc: LazyDoc::Original(doc),
            draft: None,
            steps: imbl::Vector::new(),
            invert_steps: imbl::Vector::new(),
            schema,
            needs_recompute: false,
        }
    }

    /// 获取当前文档状态，使用延迟计算
    pub fn doc(&self) -> Arc<NodePool> {
        match &self.lazy_doc {
            LazyDoc::Original(doc) => doc.clone(),
            LazyDoc::Computed(doc) => doc.clone(),
            LazyDoc::Pending { base, steps } => {
                // 延迟计算：只有在需要时才重新计算文档状态
                self.compute_doc_state(base.clone(), steps.clone())
            },
        }
    }

    /// 获取草稿状态，使用 Copy-on-Write
    fn get_draft(&mut self) -> &mut Tree {
        if self.draft.is_none() {
            // 只有在第一次修改时才克隆
            self.draft = Some(self.base_doc.get_inner().as_ref().clone());
        }
        self.draft.as_mut().unwrap()
    }

    pub fn step(
        &mut self,
        step: Arc<dyn Step>,
    ) -> TransformResult<()> {
        let schema = self.schema.clone();
        let draft = self.get_draft();
        let result: StepResult = step.apply(draft, schema)?;

        match result.failed {
            Some(message) => Err(anyhow::anyhow!(message)),
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
        // 生成反向步骤
        if let Some(invert_step) = step.invert(&self.base_doc.get_inner()) {
            self.invert_steps.push_back(invert_step);
        }

        self.steps.push_back(step.clone());

        // 标记需要延迟重新计算，而不是立即计算
        self.lazy_doc = LazyDoc::Pending {
            base: self.base_doc.clone(),
            steps: self.steps.clone(),
        };
        self.needs_recompute = true;
    }

    /// 强制重新计算文档状态（私有方法）
    fn compute_doc_state(
        &self,
        base: Arc<NodePool>,
        steps: imbl::Vector<Arc<dyn Step>>,
    ) -> Arc<NodePool> {
        if steps.is_empty() {
            return base;
        }

        // 只有在真正需要时才进行计算
        if let Some(ref draft) = self.draft {
            NodePool::new(Arc::new(draft.clone()))
        } else {
            base
        }
    }

    /// 批量应用步骤（优化版本）
    pub fn apply_steps_batch(
        &mut self,
        steps: Vec<Arc<dyn Step>>,
    ) -> TransformResult<()> {
        let schema = self.schema.clone();
        let base_doc_inner = self.base_doc.get_inner().clone();

        // 收集反向步骤
        let mut new_invert_steps = Vec::new();
        for step in &steps {
            if let Some(invert_step) = step.invert(&base_doc_inner) {
                new_invert_steps.push(invert_step);
            }
        }

        let draft = self.get_draft();

        // 批量应用，减少中间状态创建
        for step in &steps {
            let result = step.apply(draft, schema.clone())?;
            if let Some(message) = result.failed {
                return Err(anyhow::anyhow!(message));
            }
        }

        // 更新步骤列表
        for step in steps {
            self.steps.push_back(step);
        }
        for invert_step in new_invert_steps {
            self.invert_steps.push_back(invert_step);
        }

        // 只在最后更新状态
        self.lazy_doc = LazyDoc::Pending {
            base: self.base_doc.clone(),
            steps: self.steps.clone(),
        };
        self.needs_recompute = true;

        Ok(())
    }

    /// 提交更改，将当前状态设为新的基础状态
    /// 保留历史记录（steps 和 invert_steps）以支持回滚功能
    pub fn commit(&mut self) {
        if self.needs_recompute && self.draft.is_some() {
            let new_doc =
                NodePool::new(Arc::new(self.draft.as_ref().unwrap().clone()));
            self.base_doc = new_doc.clone();
            self.lazy_doc = LazyDoc::Computed(new_doc);
            self.draft = None;
            // 保留 steps 和 invert_steps 用于历史记录和回滚
            self.needs_recompute = false;
        }
    }

    /// 回滚所有未提交的更改
    pub fn rollback(&mut self) {
        self.lazy_doc = LazyDoc::Original(self.base_doc.clone());
        self.draft = None;
        self.steps = imbl::Vector::new();
        self.invert_steps = imbl::Vector::new();
        self.needs_recompute = false;
    }

    /// 回滚指定数量的步骤
    pub fn rollback_steps(
        &mut self,
        count: usize,
    ) -> TransformResult<()> {
        if count > self.invert_steps.len() {
            return Err(anyhow::anyhow!("回滚步骤数量超出历史记录"));
        }

        let schema = self.schema.clone();

        // 收集要应用的反向步骤
        let mut invert_steps_to_apply = Vec::new();
        for _ in 0..count {
            if let Some(invert_step) = self.invert_steps.pop_back() {
                invert_steps_to_apply.push(invert_step);
                self.steps.pop_back();
            }
        }

        let draft = self.get_draft();

        // 应用反向步骤
        for invert_step in invert_steps_to_apply {
            invert_step.apply(draft, schema.clone())?;
        }

        self.needs_recompute = true;
        Ok(())
    }

    /// 清除历史记录（释放内存）
    pub fn clear_history(&mut self) {
        self.steps = imbl::Vector::new();
        self.invert_steps = imbl::Vector::new();
    }

    /// 获取历史记录大小
    pub fn history_size(&self) -> usize {
        self.steps.len()
    }
}

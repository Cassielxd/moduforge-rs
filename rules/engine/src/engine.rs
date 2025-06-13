use std::future::Future;
use std::sync::Arc;

use crate::decision::Decision;
use crate::handler::custom_node_adapter::{CustomNodeAdapter, NoopCustomNode};
use crate::handler::graph::DecisionGraphResponse;
use crate::loader::{
    ClosureLoader, DecisionLoader, LoaderResponse, LoaderResult, NoopLoader,
};
use crate::model::DecisionContent;
use crate::EvaluationError;
use moduforge_rules_expression::variable::Variable;
use moduforge_rules_expression::functions::custom::CustomFunctionRegistry;

/// 决策引擎结构体，用于生成和评估决策
#[derive(Debug, Clone)]
pub struct DecisionEngine<Loader, CustomNode>
where
    Loader: DecisionLoader + 'static,
    CustomNode: CustomNodeAdapter + 'static,
{
    loader: Arc<Loader>,
    adapter: Arc<CustomNode>,
}

#[derive(Debug, Default)]
pub struct EvaluationOptions {
    pub trace: Option<bool>,
    pub max_depth: Option<u8>,
}

impl Default for DecisionEngine<NoopLoader, NoopCustomNode> {
    fn default() -> Self {
        Self {
            loader: Arc::new(NoopLoader::default()),
            adapter: Arc::new(NoopCustomNode::default()),
        }
    }
}

impl<L: DecisionLoader + 'static, A: CustomNodeAdapter + 'static>
    DecisionEngine<L, A>
{
    pub fn new(
        loader: Arc<L>,
        adapter: Arc<A>,
    ) -> Self {
        Self { loader, adapter }
    }

    pub fn with_adapter<CustomNode>(
        self,
        adapter: Arc<CustomNode>,
    ) -> DecisionEngine<L, CustomNode>
    where
        CustomNode: CustomNodeAdapter,
    {
        DecisionEngine { loader: self.loader, adapter }
    }

    pub fn with_loader<Loader>(
        self,
        loader: Arc<Loader>,
    ) -> DecisionEngine<Loader, A>
    where
        Loader: DecisionLoader,
    {
        DecisionEngine { loader, adapter: self.adapter }
    }

    pub fn with_closure_loader<F, O>(
        self,
        loader: F,
    ) -> DecisionEngine<ClosureLoader<F>, A>
    where
        F: Fn(String) -> O + Sync + Send,
        O: Future<Output = LoaderResponse> + Send,
    {
        DecisionEngine {
            loader: Arc::new(ClosureLoader::new(loader)),
            adapter: self.adapter,
        }
    }

    /// 使用 key 评估决策
    pub async fn evaluate<K>(
        &self,
        key: K,
        context: Variable,
    ) -> Result<DecisionGraphResponse, Box<EvaluationError>>
    where
        K: AsRef<str>,
    {
        self.evaluate_with_opts(key, context, Default::default()).await
    }
    pub async fn evaluate_with_state_and_opts<K>(
        &self,
        key: K,
        context: Variable,
        state: Arc<moduforge_state::State>,
        options: EvaluationOptions,
    ) -> Result<DecisionGraphResponse, Box<EvaluationError>>
    where
        K: AsRef<str>,
    {
        // 设置 thread_local State
        CustomFunctionRegistry::set_current_state(Some(state));

        // 执行评估
        let result = self.evaluate_with_opts(key, context, options).await;

        // 清理 thread_local State
        CustomFunctionRegistry::set_current_state(None);

        result
    }
    /// 使用 State 评估决策 - 通过 thread_local 设置 State
    pub async fn evaluate_with_state<K>(
        &self,
        key: K,
        context: Variable,
        state: Arc<moduforge_state::State>,
    ) -> Result<DecisionGraphResponse, Box<EvaluationError>>
    where
        K: AsRef<str>,
    {
        // 设置 thread_local State
        CustomFunctionRegistry::set_current_state(Some(state));

        // 执行评估
        let result = self.evaluate(key, context).await;

        // 清理 thread_local State
        CustomFunctionRegistry::set_current_state(None);

        result
    }

    /// 使用 key 评估决策，并使用高级选项
    pub async fn evaluate_with_opts<K>(
        &self,
        key: K,
        context: Variable,
        options: EvaluationOptions,
    ) -> Result<DecisionGraphResponse, Box<EvaluationError>>
    where
        K: AsRef<str>,
    {
        let content = self.loader.load(key.as_ref()).await?;
        let decision = self.create_decision(content);
        decision.evaluate_with_opts(context, options).await
    }

    /// 从 DecisionContent 创建决策，用于更简单的绑定创建
    pub fn create_decision(
        &self,
        content: Arc<DecisionContent>,
    ) -> Decision<L, A> {
        Decision::from(content)
            .with_loader(self.loader.clone())
            .with_adapter(self.adapter.clone())
    }

    /// 根据 loader 获取决策
    pub async fn get_decision(
        &self,
        key: &str,
    ) -> LoaderResult<Decision<L, A>> {
        let content = self.loader.load(key).await?;
        Ok(self.create_decision(content))
    }

    pub fn loader(&self) -> &L {
        self.loader.as_ref()
    }

    pub fn adapter(&self) -> &A {
        self.adapter.as_ref()
    }
}

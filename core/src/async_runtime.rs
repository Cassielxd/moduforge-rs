use std::{
    ops::{Deref, DerefMut},
    sync::{Arc},
};

use crate::{
    error_utils,
    event::Event,
    flow::{FlowEngine, ProcessorResult},
    types::EditorOptions,
    EditorResult,
};
use moduforge_state::{
    debug,
    state::TransactionResult,
    transaction::{Command, Transaction},
    State,
};
use crate::runtime::Editor;
/// Editor 结构体代表编辑器的核心功能实现
/// 负责管理文档状态、事件处理、插件系统和存储等核心功能
pub struct AsyncEditor {
    base: Editor,
    flow_engine: FlowEngine,
}
impl Deref for AsyncEditor {
    type Target = Editor;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for AsyncEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl AsyncEditor {
    /// 创建新的编辑器实例
    /// options: 编辑器配置选项
    pub async fn create(
        options: EditorOptions
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let base = Editor::create(options).await?;
        Ok(AsyncEditor { base, flow_engine: FlowEngine::new()? })
    }

    pub async fn command(
        &mut self,
        command: Arc<dyn Command>,
    ) -> EditorResult<()> {
        debug!("正在执行命令: {}", command.name());
        let mut tr = self.get_tr();
        tr.transaction(command).await;
        self.dispatch(tr).await
    }

    pub async fn dispatch(
        &mut self,
        transaction: Transaction,
    ) -> EditorResult<()> {
        // 保存当前事务的副本，用于中间件处理
        let mut current_transaction = transaction;
        self.run_before_middleware(&mut current_transaction).await?;

        // 使用 flow_engine 提交事务
        let (_id, mut rx) = self
            .flow_engine
            .submit_transaction((
                self.base.get_state().clone(),
                current_transaction,
            ))
            .await?;

        // 等待任务结果
        let Some(task_result) = rx.recv().await else {
            return Ok(());
        };

        // 获取处理结果
        let Some(ProcessorResult { result: Some(result), .. }) =
            task_result.output
        else {
            return Ok(());
        };

        // 更新编辑器状态
        let mut current_state = None;
        let mut transactions = Vec::new();
        transactions.extend(result.transactions);
        // 检查最后一个事务是否改变了文档
        if let Some(tr) = transactions.last() {
            if tr.doc_changed() {
                current_state = Some(Arc::new(result.state));
            }
        }
        // 执行后置中间件链，允许中间件在事务应用后执行额外操作
        self.run_after_middleware(&mut current_state, &mut transactions)
            .await?;
        if let Some(state) = current_state {
            self.base.update_state(state.clone()).await?;
            self.base
                .emmit_event(Event::TrApply(Arc::new(transactions), state))
                .await?;
        }
        Ok(())
    }

    pub async fn run_before_middleware(
        &mut self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        debug!("执行前置中间件链");
        for middleware in &self.base.get_middleware_stack().middlewares {
            let timeout = std::time::Duration::from_millis(500);
            if let Err(e) = tokio::time::timeout(
                timeout,
                middleware.before_dispatch(transaction),
            )
            .await
            {
                return Err(error_utils::middleware_error(format!(
                    "中间件执行超时: {}",
                    e
                )));
            }
        }
        Ok(())
    }
    pub async fn run_after_middleware(
        &mut self,
        state: &mut Option<Arc<State>>,
        transactions: &mut Vec<Transaction>,
    ) -> EditorResult<()> {
        debug!("执行后置中间件链");
        for middleware in &self.base.get_middleware_stack().middlewares {
            let timeout = std::time::Duration::from_millis(500);
            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(state.clone()),
            )
            .await
            {
                Ok(result) => result?,
                Err(e) => {
                    return Err(error_utils::middleware_error(format!(
                        "中间件执行超时: {}",
                        e
                    )));
                },
            };

            if let Some(transaction) = middleware_result.additional_transaction
            {
                let (_id, mut rx) = self
                    .flow_engine
                    .submit_transaction((
                        self.base.get_state().clone(),
                        transaction,
                    ))
                    .await?;
                let Some(task_result) = rx.recv().await else {
                    return Ok(());
                };
                let Some(ProcessorResult { result: Some(result), .. }) =
                    task_result.output
                else {
                    return Ok(());
                };
                let TransactionResult { state: new_state, transactions: trs } =
                    result;
                *state = Some(Arc::new(new_state));
                transactions.extend(trs);
            }
        }
        Ok(())
    }
}

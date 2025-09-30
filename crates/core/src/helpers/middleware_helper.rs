//! 中间件执行框架辅助模块
//!
//! 提供统一的中间件执行逻辑，包括：
//! - before_dispatch 中间件链执行
//! - after_dispatch 中间件链执行
//! - 超时处理
//! - 错误处理
//! - 性能指标记录

use crate::{
    config::ForgeConfig,
    debug::debug,
    error::{error_utils, ForgeResult},
    metrics,
    middleware::MiddlewareStack,
    types::RuntimeOptions,
};
use mf_state::{state::State, transaction::Transaction};
use std::sync::Arc;
use std::time::Instant;

/// 中间件执行辅助器
pub struct MiddlewareHelper;

impl MiddlewareHelper {
    /// 执行前置中间件链
    ///
    /// # 参数
    /// * `transaction` - 可变的事务引用
    /// * `middleware_stack` - 中间件栈
    /// * `config` - Forge配置（用于获取超时设置）
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub async fn run_before_middleware(
        transaction: &mut Transaction,
        middleware_stack: &MiddlewareStack,
        config: &ForgeConfig,
    ) -> ForgeResult<()> {
        debug!("执行前置中间件链");

        let timeout = std::time::Duration::from_millis(
            config.performance.middleware_timeout_ms,
        );

        for middleware in &middleware_stack.middlewares {
            let start_time = Instant::now();

            match tokio::time::timeout(
                timeout,
                middleware.before_dispatch(transaction),
            )
            .await
            {
                Ok(Ok(())) => {
                    // 中间件执行成功
                    metrics::middleware_execution_duration(
                        start_time.elapsed(),
                        "before",
                        middleware.name().as_str(),
                    );
                    continue;
                },
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行失败: {}",
                        e
                    )));
                },
                Err(_) => {
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行超时（{}ms）",
                        config.performance.middleware_timeout_ms
                    )));
                },
            }
        }

        Ok(())
    }

    /// 执行后置中间件链
    ///
    /// # 参数
    /// * `state` - 可变的状态选项引用
    /// * `transactions` - 可变的事务列表引用
    /// * `middleware_stack` - 中间件栈
    /// * `config` - Forge配置（用于获取超时设置）
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub async fn run_after_middleware(
        state: &mut Option<Arc<State>>,
        transactions: &mut Vec<Arc<Transaction>>,
        middleware_stack: &MiddlewareStack,
        config: &ForgeConfig,
    ) -> ForgeResult<()> {
        debug!("执行后置中间件链");

        let timeout = std::time::Duration::from_millis(
            config.performance.middleware_timeout_ms,
        );

        for middleware in &middleware_stack.middlewares {
            let start_time = Instant::now();

            let middleware_result = match tokio::time::timeout(
                timeout,
                middleware.after_dispatch(state.clone(), transactions),
            )
            .await
            {
                Ok(Ok(result)) => {
                    metrics::middleware_execution_duration(
                        start_time.elapsed(),
                        "after",
                        middleware.name().as_str(),
                    );
                    result
                },
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行失败: {}",
                        e
                    )));
                },
                Err(_) => {
                    return Err(error_utils::middleware_error(format!(
                        "后置中间件执行超时（{}ms）",
                        config.performance.middleware_timeout_ms
                    )));
                },
            };

            // 处理中间件返回的额外事务
            // 注意：原始实现中，middleware_result 是 Option<Transaction>
            // 如果返回了 Transaction，需要由调用者进一步处理
            // 这里我们暂时保留这个返回值，由调用者决定如何处理
            if middleware_result.is_some() {
                // 中间件返回了额外的事务，但这里无法直接处理
                // 因为需要通过 flow_engine 提交，这是运行时特定的逻辑
                // 所以这部分逻辑保留在 ForgeRuntime 中
            }
        }

        Ok(())
    }
}

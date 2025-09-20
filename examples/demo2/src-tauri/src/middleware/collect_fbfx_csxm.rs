use std::sync::Arc;

use async_trait::async_trait;
use mf_core::{middleware::Middleware, ForgeResult};
use mf_state::{State, Transaction};

/// 收集 分部分项 措施项目 汇总 中间件
/// 当 编辑区 分部分项 措施项目节点 更新后需要 收集 分部分项 措施项目 汇总
#[derive(Debug)]
pub struct CollectFbfxCsxmMiddleware;

#[async_trait]
impl Middleware for CollectFbfxCsxmMiddleware {
    /// 返回中间件的名称
    fn name(&self) -> String {
        "collect_fbfx_csxm".to_string()
    }

    /// 在核心分发之后处理结果
    /// 返回一个可能包含需要额外处理的事务的 MiddlewareResult
    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Arc<Transaction>],
    ) -> ForgeResult<Option<Transaction>> {
        println!("分部分项 措施项目 汇总");
        for tr in transactions {
            if let Some(_) = tr.get_meta::<Vec<String>>("de_ids") {
                //汇总对应的定额 价格 向上汇总
            }
        }
        Ok(None)
    }
}

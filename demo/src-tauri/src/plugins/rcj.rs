use async_trait::async_trait;
use mf_state::{plugin::PluginTrait, State, Transaction};

/*
人材机 插件
分部分项 数据插入后需要 触发人材机的计算
在此方法里 拿到 分部分项 的 meta 数据 找到对应的 定额节点  新增对应的人材机节点
并设置 meta 用作 单价构成 插件流转

*/
#[derive(Debug)]
pub struct RcjPlugin;

#[async_trait]
impl PluginTrait for RcjPlugin {
    async fn append_transaction(
        &self,
        _trs: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> anyhow::Result<Option<Transaction>> {
        // 计算 人材机 价格 并回填 设置meta 用作 单价构成 插件流转
        Ok(None)
    }
}

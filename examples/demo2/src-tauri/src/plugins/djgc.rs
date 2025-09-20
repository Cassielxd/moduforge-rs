use std::sync::Arc;

use async_trait::async_trait;
use mf_state::{plugin::PluginTrait, State, Transaction};
use mf_state::plugin::PluginMetadata;
/*
单价构成 插件
人材机 数据插入后需要 触发单价构成的计算
在此方法里 拿到 人材机 的 meta 数据 找到对应的 分部分项节点  新增对应的人材机节点
并设置 meta 用作 单价构成 插件流转

*/
#[derive(Debug)]
pub struct DjgcPlugin;

#[async_trait]
impl PluginTrait for DjgcPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata{
            name: "djgc".to_string(),
            version: "1.0.0".to_string(),
            description: "单价构成插件".to_string(),
            author: "".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }
    async fn append_transaction(
        &self,
        _trs: &[Arc<Transaction>],
        _old_state: &Arc<State>,
        _new_state: &Arc<State>,
    ) -> anyhow::Result<Option<Transaction>> {
        // 拿到人材机 meate 并计算 单价构成数据 并回填 设置meta 分部分项插件回填

        Ok(None)
    }
}

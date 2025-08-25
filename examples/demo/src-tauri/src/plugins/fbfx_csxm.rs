use std::collections::HashMap;

use async_trait::async_trait;

use mf_state::{plugin::PluginTrait, State, Transaction};
use rand::Rng;

use crate::commands::AddRequest;

/*
分部分项 措施项目 插件
 */
#[derive(Debug)]
pub struct FbfxCsxmPlugin;

#[async_trait]
impl PluginTrait for FbfxCsxmPlugin {
    async fn append_transaction(
        &self,
        trs: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> anyhow::Result<Option<Transaction>> {
        let mut rng = rand::rng();
        for tr in trs {
            if let Some(data) = tr.get_meta::<AddRequest>("insert_fbfx_csxm") {
                // 模拟 计算当前节点 价格
                let mut tr = new_state.tr();
                tr.set_node_attribute(
                    data.id.clone().unwrap(),
                    HashMap::from([
                        (
                            "sbfTotal".to_string(),
                            serde_json::json!(rng.random_range(1000..10000)),
                        ),
                        (
                            "sbfPrice".to_string(),
                            serde_json::json!(rng.random_range(100..1000)),
                        ),
                    ])
                    .into(),
                )?;
                //标记 当前节点 对应的 定额 节点 id 用于后续汇总使用
                tr.set_meta("de_ids", vec![data.id.clone().unwrap()]);
                return Ok(Some(tr));
            }
        }
        Ok(None)
    }
}

use async_trait::async_trait;
use mf_state::{transaction::Command, Transaction};
use mf_transform::TransformResult;
use serde::{Deserialize, Serialize};

use crate::commands::{AddRequest, DeleteNodeRequest, ShareCommand};

// 插入分部分项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsertFbfxCsxmCommand {
    pub data: AddRequest,
}

#[async_trait]
impl Command for InsertFbfxCsxmCommand {
    async fn execute(&self, tr: &mut Transaction) -> TransformResult<()> {
        tr.set_meta("insert_fbfx_csxm", self.data.clone());
        self.add_node(tr, &self.data).await
    }

    fn name(&self) -> String {
        "insert_fbfx_csxm".to_string()
    }
}

#[async_trait]
impl ShareCommand for InsertFbfxCsxmCommand {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteFbfxCsxmCommand {
    pub data: DeleteNodeRequest,
}

#[async_trait]
impl Command for DeleteFbfxCsxmCommand {
    async fn execute(&self, tr: &mut Transaction) -> TransformResult<()> {
        // 设置 meta 高度后续删除 的是 措施项目节点 todo!()
        self.delete_node(tr, &self.data).await
    }

    fn name(&self) -> String {
        "delete_fbfx_csxm".to_string()
    }
}

#[async_trait]
impl ShareCommand for DeleteFbfxCsxmCommand {}

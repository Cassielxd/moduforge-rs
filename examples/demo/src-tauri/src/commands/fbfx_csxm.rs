use async_trait::async_trait;
use mf_state::{transaction::CommandGeneric, Transaction};
use mf_model::{node_pool::NodePool, schema::Schema};
use mf_transform::TransformResult;
use serde::{Deserialize, Serialize};

use crate::commands::{AddRequest, DeleteNodeRequest, ShareCommand};

// 插入分部分项
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsertFbfxCsxmCommand {
    pub data: AddRequest,
}

#[async_trait]
impl CommandGeneric<NodePool, Schema> for InsertFbfxCsxmCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
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
impl CommandGeneric<NodePool, Schema> for DeleteFbfxCsxmCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        self.delete_node(tr, &self.data).await
    }

    fn name(&self) -> String {
        "delete_fbfx_csxm".to_string()
    }
}

#[async_trait]
impl ShareCommand for DeleteFbfxCsxmCommand {}

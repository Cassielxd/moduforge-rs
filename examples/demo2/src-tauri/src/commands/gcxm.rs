use std::collections::HashMap;

use async_trait::async_trait;
use mf_model::types::NodeId;
use mf_model::{node_pool::NodePool, schema::Schema};
use mf_state::{transaction::CommandGeneric, Transaction};
use mf_transform::TransformResult;
use serde::{Deserialize, Serialize};

use crate::{
    commands::{AddMarkRequest, AddRequest, DeleteNodeRequest, ShareCommand},
    marks::FOOTNOTE_STR,
};
#[derive(Debug, Clone)]
pub struct InsertChildCammand {
    pub data: AddRequest,
}

#[async_trait]
impl CommandGeneric<NodePool, Schema> for InsertChildCammand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        self.add_node(tr, &self.data).await
    }
    fn name(&self) -> String {
        "insert_gcxm_child".to_string()
    }
}
#[async_trait]
impl ShareCommand for InsertChildCammand {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddFootNoteCammand {
    pub editor_name: String,
    pub id: NodeId,
    pub footnote: String,
}

#[async_trait]
impl CommandGeneric<NodePool, Schema> for AddFootNoteCammand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        let mark = tr.schema.marks.get(FOOTNOTE_STR).unwrap().create(Some(
            &HashMap::from_iter(vec![(
                "value".to_string(),
                self.footnote.clone().into(),
            )]),
        ));
        self.add_mark(
            tr,
            &AddMarkRequest {
                editor_name: self.editor_name.clone(),
                id: self.id.clone(),
                marks: vec![mark],
            },
        )
        .await
    }
    fn name(&self) -> String {
        "add_gcxm_footnote".to_string()
    }
}
#[async_trait]
impl ShareCommand for AddFootNoteCammand {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteGcxmCammand {
    pub data: DeleteNodeRequest,
}

#[async_trait]
impl CommandGeneric<NodePool, Schema> for DeleteGcxmCammand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        self.delete_node(tr, &self.data).await
    }
    fn name(&self) -> String {
        "delete_gcxm".to_string()
    }
}

#[async_trait]
impl ShareCommand for DeleteGcxmCammand {}

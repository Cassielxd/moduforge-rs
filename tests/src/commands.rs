use std::sync::Arc;

use async_trait::async_trait;
use moduforge_state::{
    transaction::{Command, Transaction},
};
use moduforge_transform::transform::TransformError;
use moduforge_core::impl_command;

#[derive(Clone, Default, Debug)]
pub struct MyCommand;
impl MyCommand {
    pub fn new() -> Arc<MyCommand> {
        Arc::new(MyCommand)
    }
}

#[async_trait]
impl Command for MyCommand {
    fn name(&self) -> String {
        "cassie".to_string()
    }
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> Result<(), TransformError> {
        //  数据库的查询
        tr.add_node(
            tr.doc().inner.root_id.to_string(),
            vec![tr.schema.nodes.get("DW").unwrap().create(
                None,
                None,
                vec![],
                None,
            )],
        );

        Ok(())
    }
}

impl_command!(
    MyCommand1,
    async |tr: &mut Transaction| -> Result<(), TransformError> {
        tr.add_node(
            tr.doc().inner.root_id.to_string(),
            vec![tr.schema.nodes.get("DW").unwrap().create(
                None,
                None,
                vec![],
                None,
            )],
        );
        Ok(())
    },
    "MyCommand1"
);

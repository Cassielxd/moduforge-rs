use std::sync::Arc;

use async_trait::async_trait;
use moduforge_model::node_type::NodeEnum;
use moduforge_state::transaction::{Command, Transaction};
use moduforge_transform::{node_step::AddNodeStep, transform::TransformError};
use moduforge_macros::impl_command;

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
        let root = tr.doc().root();
        let node: NodeEnum = NodeEnum::from(
            root.as_ref().clone(),
            vec![tr.schema.nodes.get("DW").unwrap().create(
                None,
                None,
                vec![],
                None,
            )],
        );
        tr.add_node(node);
        Ok(())
    }
}

impl_command!(
    MyCommand1,
    async |tr: &mut Transaction| -> Result<(), TransformError> {
        let root = tr.doc().root();
        let node: NodeEnum = NodeEnum::from(
            root.as_ref().clone(),
            vec![tr.schema.nodes.get("DW").unwrap().create(
                None,
                None,
                vec![],
                None,
            )],
        );
        tr.add_node(node);
        Ok(())
    },
    "MyCommand1"
);

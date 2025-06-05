use std::sync::Arc;

use async_trait::async_trait;
use moduforge_model::node_type::NodeEnum;
use moduforge_state::{
    transaction::{Command, Transaction},
};
use moduforge_macros::impl_command;
use moduforge_transform::TransformResult;

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
    ) -> TransformResult<()> {
        //  数据库的查询
        let root = tr.doc().root();
        let node = tr.schema.nodes.get("DW").unwrap().create(
            None,
            None,
            vec![],
            None,
        );
        let _ = tr.add_node(root.id.clone(), vec![NodeEnum(node, vec![])]);
        Ok(())
    }
}

impl_command!(
    MyCommand1,
    async |tr: &mut Transaction| -> TransformResult<()> {
        let root = tr.doc().root();
        let node = tr.schema.nodes.get("DW").unwrap().create(
            None,
            None,
            vec![],
            None,
        );
        let _ = tr.add_node(root.id.clone(), vec![NodeEnum(node, vec![])]);
        Ok(())
    },
    "MyCommand1"
);

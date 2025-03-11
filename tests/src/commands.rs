use std::sync::Arc;

use async_trait::async_trait;
use moduforge_core::{
  state::transaction::{Command, Transaction},
  transform::transform::TransformError,
};

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
      tr.schema.nodes.get("DW").unwrap().create(None, None, vec![], None),
    );

    Ok(())
  }
}

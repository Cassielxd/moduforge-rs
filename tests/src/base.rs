use moduforge_runtime::{node::Node, types::Extensions};

use crate::ext::get_extension;

pub fn get_base() -> Vec<Extensions> {
  let mut extensions = vec![];

  let mut top_node = Node::default();
  top_node
    .set_top_node()
    .set_name("doc")
    .set_content("DW+")
    .set_desc("顶级节点")
    .set_attr("name", None);
  extensions.push(Extensions::N(top_node));
  let mut dw = Node::default();
  dw.set_name("DW").set_desc("页面");
  extensions.push(Extensions::N(dw));
  extensions.push(Extensions::E(get_extension()));
  extensions
}

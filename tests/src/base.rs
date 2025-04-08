use moduforge_core::{node, types::Extensions};

use crate::ext::get_extension;

pub fn get_base() -> Vec<Extensions> {
    let mut extensions = vec![];

    let top_node = node!("doc", "顶级节点", "DW+", "name"=>"doc".into());
    extensions.push(Extensions::N(top_node));
    let dw = node!("DW", "页面");
    extensions.push(Extensions::N(dw));
    extensions.push(Extensions::E(get_extension()));
    extensions
}

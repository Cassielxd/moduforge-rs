use mf_derive::Node;

/// 单价构成节点
#[derive(Node)]
#[node_type = "djgc"]
#[desc = "单价构成"]
#[content = "djgcRowNode+"]
pub struct DjgcNode {
    #[attr]
    value: String,
}

/// 单价构成行节点
#[derive(Node)]
#[node_type = "djgcRowNode"]
#[desc = "单价构成行节点"]
pub struct DjgcRowNode {
    #[attr]
    qf_code: String,
    #[attr]
    dtype: String,
    #[attr]
    code: String,
    #[attr]
    caculate_base: String,
    #[attr]
    desc: String,
    #[attr]
    rate: String,
    #[attr]
    price: i32,
}

lazy_static! {
    pub static ref DJGC: mf_core::node::Node = DjgcNode::node_definition();
    pub static ref DJGC_NODE: mf_core::node::Node = DjgcRowNode::node_definition();
}

///构建单价构成节点 节点定义
///
/// 节点树结构:
/// djgc (单价构成)
/// └── djgcNode+ (单价构成行节点)
///     ├── djgcqfCode (模版编码)
///     ├── djgcstandard (标准)
///     ├── djgctype (单价构成类型)
///     ├── djgccode (费用代号)
///     ├── djgccaculateBase (计算基数)
///     ├── djgcdesc (描述)
///     ├── djgcrate (费率)
///     └── djgcprice (单价)
///
pub fn init_nodes() -> Vec<mf_core::node::Node> {
    let mut nodes = vec![DJGC_NODE.clone()];
    let mut djgc = DJGC.clone();
    djgc.set_content("djgcRowNode+");
    nodes.push(djgc);
    nodes
}

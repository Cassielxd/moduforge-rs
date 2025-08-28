use mf_core::node::Node;
use mf_macro::node;
lazy_static! {
    pub static ref DJGC: Node = node!("djgc", "单价构成","","value"=>"".into());
    pub static ref DJGC_NODE: Node = node!("djgcRowNode", "单价构成行节点","","qfCode"=>"".into(),"type"=>"".into(),"code"=>"".into(),"caculateBase"=>"".into(),"desc"=>"".into(),"rate"=>"".into(),"price"=>0.into());
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
pub fn init_nodes() -> Vec<Node> {
    let mut nodes = vec![DJGC_NODE.clone()];
    let mut djgc = DJGC.clone();
    djgc.set_content("djgcRowNode+");
    nodes.push(djgc);
    nodes
}

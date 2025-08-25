use mf_derive::Node;

pub const FB_STR: &str = "fb";
pub const QD_STR: &str = "qd";
pub const DE_STR: &str = "de";
pub const DE_RCJ_STR: &str = "dercj";
pub const FBFX_STR: &str = "fbfx";
pub const CSXM_STR: &str = "csxm";

/// 分部节点
#[derive(Node)]
#[node_type = "fb"]
#[desc = "分部"]
#[content = "(fb|qd)*"]
pub struct FbNode {
    #[attr]
    project_code: String,
    #[attr]
    unit: String,
    #[attr]
    project_name: String,
    #[attr]
    type_name: String,
    #[attr]
    project_attr: String,
    #[attr]
    quantity: String,
    #[attr]
    quantity_expression: String,
    #[attr]
    sbf_price: i32,
    #[attr]
    sbf_total: i32,
    #[attr]
    zgf_price: i32,
    #[attr]
    zgf_total: i32,
    #[attr]
    zjf_price: i32,
    #[attr]
    zjf_total: i32,
}

/// 清单节点
#[derive(Node)]
#[node_type = "qd"]
#[desc = "清单"]
#[content = "(de|dercj)*"]
pub struct QdNode {
    #[attr]
    project_code: String,
    #[attr]
    unit: String,
    #[attr]
    project_name: String,
    #[attr]
    type_name: String,
    #[attr]
    project_attr: String,
    #[attr]
    quantity: String,
    #[attr]
    quantity_expression: String,
    #[attr]
    sbf_price: i32,
    #[attr]
    sbf_total: i32,
    #[attr]
    zgf_price: i32,
    #[attr]
    zgf_total: i32,
    #[attr]
    zjf_price: i32,
    #[attr]
    zjf_total: i32,
}

/// 定额节点
#[derive(Node)]
#[node_type = "de"]
#[desc = "定额"]
pub struct DeNode {
    #[attr]
    project_code: String,
    #[attr]
    unit: String,
    #[attr]
    project_name: String,
    #[attr]
    type_name: String,
    #[attr]
    project_attr: String,
    #[attr]
    quantity: String,
    #[attr]
    quantity_expression: String,
    #[attr]
    sbf_price: i32,
    #[attr]
    sbf_total: i32,
    #[attr]
    zgf_price: i32,
    #[attr]
    zgf_total: i32,
    #[attr]
    zjf_price: i32,
    #[attr]
    zjf_total: i32,
}

/// 定额人材机节点
#[derive(Node)]
#[node_type = "dercj"]
#[desc = "定额_人材机"]
pub struct DeRcjNode {
    #[attr]
    project_code: String,
    #[attr]
    unit: String,
    #[attr]
    project_name: String,
    #[attr]
    type_name: String,
    #[attr]
    project_attr: String,
    #[attr]
    quantity: String,
    #[attr]
    quantity_expression: String,
    #[attr]
    sbf_price: i32,
    #[attr]
    sbf_total: i32,
    #[attr]
    zgf_price: i32,
    #[attr]
    zgf_total: i32,
    #[attr]
    zjf_price: i32,
    #[attr]
    zjf_total: i32,
}

/// 分部分项节点
#[derive(Node)]
#[node_type = "fbfx"]
#[desc = "分部分项"]
#[content = "(fb|qd)+"]
pub struct FbfxNode {
    #[attr]
    name: String,
}

/// 措施项目节点
#[derive(Node)]
#[node_type = "csxm"]
#[desc = "措施项目"]
#[content = "(fb|qd)+"]
pub struct CsxmNode {
    #[attr]
    name: String,
}

lazy_static! {
    pub static ref FB: mf_core::node::Node = FbNode::node_definition();
    pub static ref QD: mf_core::node::Node = QdNode::node_definition();
    pub static ref DE: mf_core::node::Node = DeNode::node_definition();
    pub static ref RCJ: mf_core::node::Node = DeRcjNode::node_definition();
    pub static ref FBFX: mf_core::node::Node = FbfxNode::node_definition();
    pub static ref CSXM: mf_core::node::Node = CsxmNode::node_definition();
}
pub fn init_fbfx_csxm_fields() -> Vec<mf_core::node::Node> {
    vec![FB.clone(), QD.clone(), DE.clone(), RCJ.clone(), FBFX.clone(), CSXM.clone()]
}

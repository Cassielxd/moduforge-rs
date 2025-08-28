use mf_derive::Node;

pub const RCJ_STR: &str = "rcj";

/// 人材机明细节点
#[derive(Node)]
#[node_type = "rcj"]
#[desc = "人材机明细"]
pub struct RcjNode {
    #[attr]
    material_code: String,
    #[attr]
    standard_id: String,
    #[attr]
    material_name: String,
    #[attr]
    specification: String,
    #[attr]
    rtype: String,
    #[attr]
    construct_id: String,
    #[attr]
    if_donor_material: i32,
    #[attr]
    kind_back_up: i32,
    #[attr]
    de_id: String,
    #[attr]
    res_qty: i32,
    #[attr]
    price_market: i32,
    #[attr]
    price_market_tax: i32,
    #[attr]
    price_market_formula: i32,
    #[attr]
    price_market_tax_formula: i32,
}

lazy_static! {
    pub static ref RCJ: mf_core::node::Node = RcjNode::node_definition();
}

pub fn init_rcj_fields() -> mf_core::node::Node {
    RCJ.clone()
}

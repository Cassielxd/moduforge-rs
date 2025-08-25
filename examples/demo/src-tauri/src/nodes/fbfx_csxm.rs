use std::collections::HashMap;

use mf_core::node::Node;
use mf_macro::node;
use mf_model::schema::AttributeSpec;

pub const FB_STR: &str = "fb";
pub const QD_STR: &str = "qd";
pub const DE_STR: &str = "de";
pub const DE_RCJ_STR: &str = "dercj";
pub const FBFX_STR: &str = "fbfx";
pub const CSXM_STR: &str = "csxm";

lazy_static! {
    pub static ref FB: Node =
        node!(FB_STR, "分部", &format!("({}|{})*", FB_STR, QD_STR));
    pub static ref QD: Node =
        node!(QD_STR, "清单", &format!("({}|{})*", DE_STR, DE_RCJ_STR));
    pub static ref DE: Node = node!(DE_STR, "定额", "");
    pub static ref RCJ: Node = node!(DE_RCJ_STR, "定额_人材机", "");
    pub static ref FBFX: Node =
        node!(FBFX_STR, "分部分项", &format!("({}|{})+", FB_STR, QD_STR));
    pub static ref CSXM: Node =
        node!(CSXM_STR, "措施项目", &format!("({}|{})+", FB_STR, QD_STR));
}
pub fn init_fbfx_csxm_fields() -> Vec<Node> {
    let mut fb = FB.clone();
    fb.set_attrs(get_attr_spec());
    let mut qd = QD.clone();
    qd.set_attrs(get_attr_spec());
    let mut de = DE.clone();
    de.set_attrs(get_attr_spec());
    let mut rcj = RCJ.clone();
    rcj.set_attrs(get_attr_spec());
    let mut fbfx = FBFX.clone();
    fbfx.set_attrs(get_attr_name("分部分项"));
    let mut csxm = CSXM.clone();
    csxm.set_attrs(get_attr_name("措施项目"));
    vec![fb, qd, de, rcj, fbfx, csxm]
}
fn get_attr_name(name: &str) -> HashMap<String, AttributeSpec> {
    let mut att = HashMap::new();
    att.insert(
        "name".to_string(),
        AttributeSpec { default: Some(name.into()) },
    );
    att
}

fn get_attr_spec() -> HashMap<String, AttributeSpec> {
    let mut att = HashMap::new();
    att.insert(
        "projectCode".to_string(),
        AttributeSpec { default: Some("".into()) },
    ); //项目编码 默认空字符串
    att.insert("unit".to_string(), AttributeSpec { default: Some("".into()) }); //单位 默认空字符串
    att.insert(
        "projectName".to_string(),
        AttributeSpec { default: Some("".into()) },
    ); //项目名称 默认空字符串
    att.insert(
        "typeName".to_string(),
        AttributeSpec { default: Some("".into()) },
    ); //类型 默认空字符串
    att.insert(
        "projectAttr".to_string(),
        AttributeSpec { default: Some("".into()) },
    ); //项目特征 默认空字符串
    att.insert(
        "quantity".to_string(),
        AttributeSpec { default: Some("".into()) },
    ); //工程量 默认空字符串
    att.insert(
        "quantityExpression".to_string(),
        AttributeSpec { default: Some("".into()) },
    ); //工程量表达式 默认空字符串
    att.insert(
        "sbfPrice".to_string(),
        AttributeSpec { default: Some(0.into()) },
    ); //设备费单价 默认0
    att.insert(
        "sbfTotal".to_string(),
        AttributeSpec { default: Some(0.into()) },
    ); //设备费合价 默认0
    att.insert(
        "zgfPrice".to_string(),
        AttributeSpec { default: Some(0.into()) },
    ); //暂估单价 默认0
    att.insert(
        "zgfTotal".to_string(),
        AttributeSpec { default: Some(0.into()) },
    ); //暂估合价 默认0
    att.insert(
        "zjfPrice".to_string(),
        AttributeSpec { default: Some(0.into()) },
    ); //直接费单价 默认0
    att.insert(
        "zjfTotal".to_string(),
        AttributeSpec { default: Some(0.into()) },
    ); //直接费合价 默认0
    att
}

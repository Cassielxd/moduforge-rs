use std::collections::HashMap;

use mf_core::node::Node;
use mf_macro::node;
use mf_model::schema::AttributeSpec;

pub const RCJ_STR: &str = "rcj";

lazy_static! {
    pub static ref RCJ: Node = node!(RCJ_STR, "人材机明细", "");
}

pub fn init_rcj_fields() -> Node {
    let mut rcj: Node = RCJ.clone();
    rcj.set_attrs(get_attr_spec());
    rcj
}

fn get_attr_spec() -> HashMap<String, AttributeSpec> {
    let mut att = HashMap::new();
    att.insert(
        "materialCode".to_string(),
        AttributeSpec { default: Some("".into()) },
    );
    att.insert(
        "standardId".to_string(),
        AttributeSpec { default: Some("".into()) },
    );
    att.insert(
        "materialName".to_string(),
        AttributeSpec { default: Some("".into()) },
    );
    att.insert(
        "specification".to_string(),
        AttributeSpec { default: Some("".into()) },
    );
    att.insert("type".to_string(), AttributeSpec { default: Some("".into()) });
    att.insert(
        "constructId".to_string(),
        AttributeSpec { default: Some("".into()) },
    );
    att.insert(
        "ifDonorMaterial".to_string(),
        AttributeSpec { default: Some(0.into()) },
    );
    att.insert(
        "kindBackUp".to_string(),
        AttributeSpec { default: Some(0.into()) },
    );
    att.insert("deId".to_string(), AttributeSpec { default: Some("".into()) });
    att.insert("resQty".to_string(), AttributeSpec { default: Some(0.into()) });
    att.insert(
        "priceMarket".to_string(),
        AttributeSpec { default: Some(0.into()) },
    );
    att.insert(
        "priceMarketTax".to_string(),
        AttributeSpec { default: Some(0.into()) },
    );
    att.insert(
        "priceMarketFormula".to_string(),
        AttributeSpec { default: Some(0.into()) },
    );
    att.insert(
        "priceMarketTaxFormula".to_string(),
        AttributeSpec { default: Some(0.into()) },
    );
    att
}

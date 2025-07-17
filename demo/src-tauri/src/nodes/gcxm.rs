use std::collections::HashMap;

use mf_core::node::Node;
use mf_macro::node;
use mf_model::schema::AttributeSpec;

pub const DWGC_STR: &str = "DWGC";
pub const DXGC_STR: &str = "DXGC";
pub const GCXM_STR: &str = "GCXM";

lazy_static! {
    pub static ref GCXM: Node = node!(GCXM_STR, "工程项目", &format!("{}+", DXGC_STR));
    pub static ref DXGC: Node = node!(
        DXGC_STR,
        "单项工程",
        &format!("({}|{})+", DWGC_STR, DXGC_STR)
    );
    pub static ref DWGC: Node = node!(DWGC_STR, "单位工程");
}

///构建 工程项目结构          节点定义
///
/// 节点树结构:
/// GCXM (工程项目)
/// └── DXGC* (多个单项)
///     └── DWGC* (多个单位)
///
pub fn init_project_structure() -> Vec<Node> {
    let mut gcxm: Node = GCXM.clone();
    gcxm.set_top_node();
    // 设置工程项目字段
    gcxm.set_attrs(init_project_structure_field("工程项目"));
    // 设置单项工程字段
    let mut dxgc = DXGC.clone();
    dxgc.set_attrs(init_project_structure_field("单项工程"));
    // 设置单位工程字段
    let mut dwgc = DWGC.clone();
    dwgc.set_attrs(init_unit_structure_field());

    vec![gcxm, dxgc, dwgc]
}

pub fn init_project_structure_field(name: &str) -> HashMap<String, AttributeSpec> {
    HashMap::from_iter(vec![
        // 工程名称
        (
            "name".to_string(),
            AttributeSpec {
                default: Some(name.into()),
            },
        ),
        // 项目编码
        ("code".to_string(), AttributeSpec { default: None }),
    ])
}

pub fn init_unit_structure_field() -> HashMap<String, AttributeSpec> {
    HashMap::from_iter(vec![
        // 单位工程编号
        ("code".to_string(), AttributeSpec { default: None }),
        // 单位工程名称
        (
            "name".to_string(),
            AttributeSpec {
                default: Some("单位工程".into()),
            },
        ),
        // 合计金额（元）
        ("total".to_string(), AttributeSpec { default: None }),
    ])
}

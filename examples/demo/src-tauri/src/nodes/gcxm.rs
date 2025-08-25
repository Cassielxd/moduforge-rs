use mf_derive::Node;

pub const DWGC_STR: &str = "DWGC";
pub const DXGC_STR: &str = "DXGC";
pub const GCXM_STR: &str = "GCXM";

/// 工程项目节点
#[derive(Node)]
#[node_type = "GCXM"]
#[desc = "工程项目"]
#[content = "DXGC+"]
pub struct GcxmNode {
    #[attr]
    name: String,
    #[attr]
    code: Option<String>,
}

/// 单项工程节点
#[derive(Node)]
#[node_type = "DXGC"]
#[desc = "单项工程"]
#[content = "(DWGC|DXGC)+"]
pub struct DxgcNode {
    #[attr]
    name: String,
    #[attr]
    code: Option<String>,
}

/// 单位工程节点
#[derive(Node)]
#[node_type = "DWGC"]
#[desc = "单位工程"]
pub struct DwgcNode {
    #[attr]
    code: Option<String>,
    #[attr]
    name: String,
    #[attr]
    total: Option<String>,
}

lazy_static! {
    pub static ref GCXM: mf_core::node::Node = {
        let mut node = GcxmNode::node_definition();
        node.set_top_node();
        node
    };
    pub static ref DXGC: mf_core::node::Node = DxgcNode::node_definition();
    pub static ref DWGC: mf_core::node::Node = DwgcNode::node_definition();
}

///构建 工程项目结构          节点定义
///
/// 节点树结构:
/// GCXM (工程项目)
/// └── DXGC* (多个单项)
///     └── DWGC* (多个单位)
///
pub fn init_project_structure() -> Vec<mf_core::node::Node> {
    vec![GCXM.clone(), DXGC.clone(), DWGC.clone()]
}

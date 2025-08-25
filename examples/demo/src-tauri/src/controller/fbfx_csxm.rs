use std::sync::Arc;

use mf_model::{id_generator::IdGenerator, node::Node, NodeId};

use crate::{
    commands::{
        fbfx_csxm::{DeleteFbfxCsxmCommand, InsertFbfxCsxmCommand},
        AddRequest, DeleteNodeRequest,
    },
    controller::GcxmTreeItem,
    error::AppError,
    nodes::fbfx_csxm::{DE_RCJ_STR, DE_STR, FB_STR, QD_STR},
    res,
    response::Res,
    ContextHelper, ResponseResult,
};
use axum::{routing::post, Json, Router};
use serde::Deserialize;

/// 添加分部分项 措施项目
pub async fn add_fbfx_csxm(
    Json(mut param): Json<AddRequest>
) -> ResponseResult<String> {
    let id = IdGenerator::get_id();
    param.id = Some(id.clone());
    let mut demo_editor = ContextHelper::get_editor(&id).unwrap();
    let meta = serde_json::to_value(param.clone()).unwrap();
    demo_editor
        .command_with_meta(
            Arc::new(InsertFbfxCsxmCommand { data: param.clone() }),
            "插入 分部分项 节点".to_string(),
            meta,
        )
        .await?;

    res!("success".to_string())
}
/// 删除分部分项 措施项目 节点
pub async fn delete_fbfx_csxm(
    Json(param): Json<DeleteNodeRequest>
) -> ResponseResult<String> {
    let mut editor = ContextHelper::get_editor(&param.editor_name).unwrap();
    let node = editor.doc().await.get_node(&param.id).unwrap();
    let meta = serde_json::to_value(node)?;
    editor
        .command_with_meta(
            Arc::new(DeleteFbfxCsxmCommand { data: param.clone() }),
            "删除 分部分项 节点".to_string(),
            meta,
        )
        .await?;

    res!("success".to_string())
}

#[derive(Debug, Deserialize)]
pub struct FbfxCsxmPost {
    pub editor_name: String,
    pub id: NodeId,
}

/// 获取分部分项 措施项目树
pub async fn get_fbfx_csxm_tree(
    Json(param): Json<FbfxCsxmPost>
) -> ResponseResult<GcxmTreeItem> {
    let editor = ContextHelper::get_editor(&param.editor_name);
    if editor.is_none() {
        return Err(AppError(anyhow::anyhow!("工程项目不存在".to_string())));
    }
    let editor = editor.unwrap();
    let doc = editor.doc().await;
    let node: Option<Arc<Node>> = doc.get_node(&param.id);
    if node.is_none() {
        return Err(AppError(anyhow::anyhow!(
            "分部分项 措施项目 跟节点不存在".to_string()
        )));
    }
    let node = node.unwrap();
    let mut nodes: Vec<Arc<Node>> = doc
        .descendants(&param.id)
        .iter()
        .filter(|n| {
            n.r#type == FB_STR
                || n.r#type == QD_STR
                || n.r#type == DE_STR
                || n.r#type == DE_RCJ_STR
        })
        .cloned()
        .collect();
    nodes.push(node);
    let parent_map = &doc.get_inner().parent_map;
    if let Some(root_item) =
        GcxmTreeItem::from_nodes(param.id, nodes, parent_map)
    {
        res!(root_item)
    } else {
        Err(AppError(anyhow::anyhow!(
            "无法构建工程树,未找到分部分项 措施项目 跟节点"
        )))
    }
}

pub fn build_app() -> Router {
    Router::new()
        //添加分部分项 措施项目 节点
        .route("/", post(add_fbfx_csxm))
        //删除分部分项 措施项目 节点
        .route("/delete_fbfx_csxm", post(delete_fbfx_csxm))
        //获取分部分项 措施项目树
        .route("/get_fbfx_csxm_tree", post(get_fbfx_csxm_tree))
}

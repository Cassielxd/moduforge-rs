use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use mf_core::{types::NodePoolFnTrait, ForgeResult};
use mf_model::{id_generator::IdGenerator, node::Node, node_pool::NodePool, NodeId};
use mf_state::StateConfig;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    commands::{
        gcxm::{AddFootNoteCammand, DeleteGcxmCammand, InsertChildCammand},
        AddRequest, DeleteNodeRequest,
    },
    controller::{get_data_tree, get_history, get_inc_data, GcxmTreeItem},
    error::AppError,
    initialize::editor::{
        init_collab_editor, init_collab_options, init_editor, init_options,
    },
    nodes::gcxm::{DWGC_STR, DXGC_STR, GCXM_STR},
    res,
    response::Res,
    ContextHelper, ResponseResult,
};

#[derive(Debug, Deserialize, Clone)]
pub struct GcxmPost {
    pub name: String,
    pub id: Option<NodeId>,
    pub collab: bool,
}

impl GcxmPost {
    pub fn to_attr_map(&self) -> HashMap<String, Value> {
        let mut attr = HashMap::new();
        Self::insert_str(&mut attr, "name", &Some(self.name.clone()));
        attr
    }
    fn insert_str(
        map: &mut HashMap<String, Value>,
        key: &str,
        value: &Option<String>,
    ) {
        if let Some(v) = value {
            map.insert(key.to_string(), Value::String(v.clone()));
        }
    }
}

#[async_trait]
impl NodePoolFnTrait for GcxmPost {
    async fn create(
        &self,
        config: &StateConfig,
    ) -> ForgeResult<NodePool> {
        let schema = config.schema.clone().unwrap();

        let res = schema.top_node_type.clone().unwrap().create_and_fill(
            self.id.clone(),
            Some(&self.to_attr_map()),
            vec![],
            None,
            &schema,
        );
        Ok(NodePool::from(res).as_ref().clone())
    }
}

pub async fn create_editor(
    create_callback: Arc<GcxmPost>
) -> anyhow::Result<()> {
    let option = init_options(create_callback.clone()).await;
    let editor = init_editor(option).await;
    ContextHelper::set_editor(
        &create_callback.id.clone().unwrap(),
        Box::new(editor),
    );
    Ok(())
}

pub async fn create_collab_editor(
    create_callback: Arc<GcxmPost>
) -> anyhow::Result<()> {
    let option = init_collab_options(
        create_callback.clone(),
        create_callback.id.clone().unwrap().to_string(),
    )
    .await;
    let editor = init_collab_editor(option).await;
    ContextHelper::set_editor(
        &create_callback.id.clone().unwrap(),
        Box::new(editor),
    );
    Ok(())
}
///创建工程项目
pub async fn new_project(
    Json(mut param): Json<GcxmPost>
) -> ResponseResult<GcxmTreeItem> {
    let id = IdGenerator::get_id();
    param.id = Some(id.clone());
    if param.collab {
        create_collab_editor(Arc::new(param.clone())).await?;
    } else {
        create_editor(Arc::new(param.clone())).await?;
    }

    let editor = ContextHelper::get_editor(&id).unwrap();
    let doc = editor.doc().await;
    let nodes: Vec<Arc<Node>> = doc.parallel_query(Box::new(|node: &Node| {
        node.r#type == DWGC_STR
            || node.r#type == DXGC_STR
            || node.r#type == GCXM_STR
    }));
    let parent_map = &doc.get_inner().parent_map;
    if let Some(root_item) =
        GcxmTreeItem::from_nodes(id.clone(), nodes, parent_map)
    {
        res!(root_item)
    } else {
        Err(AppError(anyhow::anyhow!("无法构建工程树,未找到根节点")))
    }
}
///插入子节点
pub async fn insert_child(
    Json(mut param): Json<AddRequest>
) -> ResponseResult<()> {
    let mut editor = ContextHelper::get_editor(&param.editor_name).unwrap();
    param.id = Some(IdGenerator::get_id());
    let meta = serde_json::to_value(param.clone())?;
    editor
        .command_with_meta(
            Arc::new(InsertChildCammand { data: param.clone() }),
            "插入 {{attrs.name}} 子节点".to_string(),
            meta,
        )
        .await?;
    res!(())
}

/// 获取工程项目树节点
pub async fn get_gcxm_tree(
    Path(editor_name): Path<String>
) -> ResponseResult<GcxmTreeItem> {
    let editor = ContextHelper::get_editor(&editor_name);
    if editor.is_none() {
        return Err(AppError(anyhow::anyhow!("工程项目不存在".to_string())));
    }
    let editor = editor.unwrap();
    let doc = editor.doc().await;
    let nodes: Vec<Arc<Node>> = doc.parallel_query(Box::new(|node: &Node| {
        node.r#type == DWGC_STR
            || node.r#type == DXGC_STR
            || node.r#type == GCXM_STR
    }));

    let parent_map = &doc.get_inner().parent_map;
    if let Some(root_item) =
        GcxmTreeItem::from_nodes(editor_name.into(), nodes, parent_map)
    {
        res!(root_item)
    } else {
        Err(AppError(anyhow::anyhow!("无法构建工程树,未找到根节点")))
    }
}

///删除工程项目节点
pub async fn delete_gcxm(
    Json(param): Json<DeleteNodeRequest>
) -> ResponseResult<String> {
    if *param.id == param.editor_name {
        return Err(AppError(anyhow::anyhow!("不能删除工程项目".to_string())));
    }
    let editor = ContextHelper::get_editor(&param.editor_name);
    if editor.is_none() {
        return Err(AppError(anyhow::anyhow!("工程项目不存在".to_string())));
    }
    let mut editor = editor.unwrap();
    let node = editor.doc().await.get_node(&param.id).unwrap();
    let meta = serde_json::to_value(node)?;
    editor
        .command_with_meta(
            Arc::new(DeleteGcxmCammand { data: param.clone() }),
            "删除  {{a.name}}".to_string(),
            meta,
        )
        .await?;

    res!("删除成功".to_string())
}

///添加脚注
pub async fn add_footnote(
    Json(param): Json<AddFootNoteCammand>
) -> ResponseResult<()> {
    let editor = ContextHelper::get_editor(&param.editor_name);
    if editor.is_none() {
        return Err(AppError(anyhow::anyhow!("工程项目不存在".to_string())));
    }
    let mut editor = editor.unwrap();
    let meta = serde_json::to_value(param.clone())?;
    editor
        .command_with_meta(
            Arc::new(param.clone()),
            "添加id：{{id}}脚注".to_string(),
            meta,
        )
        .await?;

    res!(())
}

pub fn build_app() -> Router {
    Router::new()
        //创建新工程项目
        .route("/", post(new_project))
        //插入子节点 单项、单位
        .route("/insert_child", post(insert_child))
        //获取工程项目树
        .route("/get_gcxm_tree/{editor_name}", get(get_gcxm_tree))
        //添加脚注
        .route("/add_footnote", post(add_footnote))
        //删除工程项目
        .route("/delete_gcxm", post(delete_gcxm))
        // 历史记录
        .route("/get_history", post(get_history))
        //获取数据树
        .route("/get_data_tree", post(get_data_tree))
        //获取增量数据
        .route("/get_inc_data/{editor_name}", get(get_inc_data))
}

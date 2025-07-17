use std::collections::HashMap;

use async_trait::async_trait;
use mf_model::{mark::Mark, types::NodeId};
use mf_state::{transaction::Command, Transaction};
use mf_transform::TransformResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod djgc;
pub mod fbfx_csxm;
pub mod gcxm;
pub mod rcj;

/// 添加节点 请求
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddRequest {
    pub editor_name: String,
    pub parent_id: String,
    pub id: Option<NodeId>,
    pub r#type: String,
    pub attrs: Option<HashMap<String, Value>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteNodeRequest {
    pub editor_name: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateAttrsRequest {
    pub editor_name: String,
    pub id: String,
    pub attrs: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddMarkRequest {
    pub editor_name: String,
    pub id: NodeId,
    pub marks: Vec<Mark>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveMarkRequest {
    pub editor_name: String,
    pub id: String,
    pub marks: Vec<String>,
}

#[async_trait]
pub trait ShareCommand: Command {
    /// 添加节点
    async fn add_node(&self, tr: &mut Transaction, data: &AddRequest) -> TransformResult<()> {
        if tr.doc().get_node(&data.parent_id.to_string()).is_none() {
            return Err(anyhow::anyhow!("目标节点不存在".to_string()));
        }
        if let Some(node_type) = tr.schema.nodes.get(&data.r#type) {
            let nodes = node_type.create_and_fill(
                data.id.clone(),
                Some(&data.attrs.clone().unwrap_or_default()),
                vec![],
                None,
                &tr.schema,
            );
            tr.add_node(data.parent_id.to_string(), vec![nodes])?;
        } else {
            return Err(anyhow::anyhow!("节点类型不存在".to_string()));
        }
        Ok(())
    }
    /// 删除节点
    async fn delete_node(
        &self,
        tr: &mut Transaction,
        data: &DeleteNodeRequest,
    ) -> TransformResult<()> {
        //组装参数 前置必要操作
        //获取目标节点
        if tr.doc().get_node(&data.id.to_string()).is_none() {
            return Err(anyhow::anyhow!("目标节点不存在".to_string()));
        }
        let parent_id = tr.doc().get_parent_node(&data.id).unwrap().id.clone();
        tr.remove_node(parent_id, vec![data.id.clone()])?;
        Ok(())
    }
    /// 更新节点属性
    async fn update_attrs(
        &self,
        tr: &mut Transaction,
        data: &UpdateAttrsRequest,
    ) -> TransformResult<()> {
        if tr.doc().get_node(&data.id.to_string()).is_none() {
            return Err(anyhow::anyhow!("目标节点不存在".to_string()));
        }
        tr.set_node_attribute(data.id.to_string(), data.attrs.clone().into())?;
        Ok(())
    }
    /// 添加标记
    async fn add_mark(&self, tr: &mut Transaction, data: &AddMarkRequest) -> TransformResult<()> {
        if tr.doc().get_node(&data.id.to_string()).is_none() {
            return Err(anyhow::anyhow!("目标节点不存在".to_string()));
        }
        tr.add_mark(data.id.to_string(), data.marks.clone())?;
        Ok(())
    }
    /// 删除标记
    async fn remove_mark(
        &self,
        tr: &mut Transaction,
        data: &RemoveMarkRequest,
    ) -> TransformResult<()> {
        if tr.doc().get_node(&data.id.to_string()).is_none() {
            return Err(anyhow::anyhow!("目标节点不存在".to_string()));
        }
        tr.remove_mark(data.id.to_string(), data.marks.clone())?;
        Ok(())
    }
}

use std::sync::Arc;
use im::HashMap;
use serde_json::Value;
use moduforge_model::{
    error::PoolError,
    mark::Mark,
    node::Node,
    node_pool::NodePool,
    tree::Tree,
    types::NodeId,
};

use crate::step::StepResult;


/// 草稿修改上下文，用于安全地修改节点池
///
/// 跟踪以下信息：
///
/// * 基础版本节点池
/// * 当前修改的中间状态
/// * 生成的修改补丁
/// * 当前操作路径（用于嵌套数据结构）
#[derive(Debug, Clone)]
pub struct Draft {
    pub base: Arc<NodePool>,
    pub inner: Tree,
}

impl Draft {
    /// 创建基于现有节点池的草稿
    ///
    /// # 参数
    ///
    /// * `base` - 基础版本节点池的引用
    pub fn new(base: Arc<NodePool>) -> Self {
        Draft {
            inner: base.inner.as_ref().clone(),
            base
        }
    }

    /// 提交属性修改并记录补丁
    ///
    /// # 参数
    ///
    /// * `id` - 目标节点ID
    /// * `new_values` - 新属性集合
    ///
    /// # 错误
    ///
    /// 当节点不存在时返回 [`PoolError::NodeNotFound`]
    pub fn update_attr(
        &mut self,
        id: &NodeId,
        new_values: HashMap<String, Value>,
    ) -> Result<(), PoolError> {
        let _ =
            self.get_node(id).ok_or(PoolError::NodeNotFound(id.clone()))?;
        self.inner.attrs(id)+new_values;

        Ok(())
    }
    /// 从节点中移除指定标记
    ///
    /// # 参数
    ///
    /// * `id` - 目标节点ID
    /// * `mark` - 要移除的标记
    ///
    /// # 错误
    ///
    /// 当节点不存在时返回 [`PoolError::NodeNotFound`]
    pub fn remove_mark(
        &mut self,
        id: &NodeId,
        mark: Mark,
    ) -> Result<(), PoolError> {
        self.inner.mark(id)-mark.clone();
      
        Ok(())
    }
    /// 为节点添加标记
    ///
    /// # 参数
    ///
    /// * `id` - 目标节点ID
    /// * `marks` - 要添加的标记列表
    ///
    /// # 错误
    ///
    /// 当节点不存在时返回 [`PoolError::NodeNotFound`]
    pub fn add_mark(
        &mut self,
        id: &NodeId,
        marks: &Vec<Mark>,
    ) -> Result<(), PoolError> {
        self.inner.mark(id)+marks.clone();
        Ok(())
    }
    /// 对节点的子节点进行排序
    ///
    /// # 参数
    ///
    /// * `parent_id` - 父节点ID
    /// * `compare` - 排序比较函数
    ///
    /// # 错误
    ///
    /// 当父节点不存在时返回 [`PoolError::ParentNotFound`]
    pub fn sort_children<
        F: FnMut(
            &(NodeId, &Arc<Node>),
            &(NodeId, &Arc<Node>),
        ) -> std::cmp::Ordering,
    >(
        &mut self,
        parent_id: &NodeId,
        compare: F,
    ) -> Result<(), PoolError> {
        // 检查父节点是否存在
        let parent = self
            .get_node(parent_id)
            .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;

        // 获取所有子节点
        let children_ids = parent.content.clone();
        if children_ids.is_empty() {
            return Ok(()); // 没有子节点，无需排序
        }
        let mut children: Vec<(NodeId, &Arc<Node>)> = Vec::new();
        for child_id in &children_ids {
            if let Some(node) = self.get_node(child_id) {
                children.push((child_id.clone(), node));
            }
        }
        children.sort_by(compare);
        // 创建排序后的子节点ID列表
        let sorted_children: im::Vector<NodeId> =
            children.into_iter().map(|(id, _)| id).collect();
        // 更新父节点
        let mut new_parent = parent.as_ref().clone();
        new_parent.content = sorted_children.clone();

    
        self.inner.nodes.insert(parent_id.clone(), Arc::new(new_parent));
        Ok(())
    }

    /// 添加子节点
    ///
    /// # 参数
    ///
    /// * `parent_id` - 父节点ID
    /// * `nodes` - 要添加的子节点列表
    pub fn add_node(
        &mut self,
        parent_id: &NodeId,
        nodes: &Vec<Node>,
    ) -> Result<(), PoolError> {
        self.inner.node(parent_id)+nodes.clone();
        Ok(())
    }

    pub fn replace_node(
        &mut self,
        node_id: NodeId,
        nodes: &Vec<Node>,
    ) -> Result<(), PoolError> {
        // 检查节点是否存在
        let old_node = self
            .get_node(&node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
        // 确保新节点ID与原节点ID一致
        if nodes[0].id != node_id {
            return Err(PoolError::InvalidNodeId {
                nodeid: node_id,
                new_node_id: nodes[0].id.clone(),
            });
        }
        let old_content:Vec<NodeId> = old_node.content.iter().map(|id| id.clone()).collect();
        self.inner.node(&node_id)-old_content;
       
        self.inner.node(&node_id)+nodes.clone();
        Ok(())
    }
    /// 移动节点
    pub fn move_node(
        &mut self,
        source_parent_id: &NodeId,
        target_parent_id: &NodeId,
        node_id: &NodeId,
        position: Option<usize>,
    ) -> Result<(), PoolError> {
        // 检查源父节点是否存在
        let source_parent = self
            .get_node(source_parent_id)
            .ok_or(PoolError::ParentNotFound(source_parent_id.clone()))?;
        // 检查目标父节点是否存在
        let target_parent = self
            .get_node(target_parent_id)
            .ok_or(PoolError::ParentNotFound(target_parent_id.clone()))?;
        // 检查要移动的节点是否存在
        let _node = self
            .get_node(node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
        // 检查节点是否是源父节点的子节点
        if !source_parent.content.contains(node_id) {
            return Err(PoolError::InvalidParenting {
                child: node_id.clone(),
                alleged_parent: source_parent_id.clone(),
            });
        }
        // 从源父节点中移除该节点
        let mut new_source_parent = source_parent.as_ref().clone();
        new_source_parent.content = new_source_parent
            .content
            .iter()
            .filter(|&id| id != node_id)
            .cloned()
            .collect();

        // 准备将节点添加到目标父节点
        let mut new_target_parent = target_parent.as_ref().clone();
        // 根据指定位置插入节点
        if let Some(pos) = position {
            if pos <= new_target_parent.content.len() {
                // 在指定位置插入
                let mut new_content = im::Vector::new();
                for (i, child_id) in
                    new_target_parent.content.iter().enumerate()
                {
                    if i == pos {
                        new_content.push_back(node_id.clone());
                    }
                    new_content.push_back(child_id.clone());
                }
                // 如果位置是在最后，需要额外处理
                if pos == new_target_parent.content.len() {
                    new_content.push_back(node_id.clone());
                }
                new_target_parent.content = new_content;
            } else {
                // 如果位置超出范围，添加到末尾
                new_target_parent.content.push_back(node_id.clone());
            }
        } else {
            // 默认添加到末尾
            new_target_parent.content.push_back(node_id.clone());
        }

        self.inner
            .nodes
            .insert(source_parent_id.clone(), Arc::new(new_source_parent));
        self.inner
            .nodes
            .insert(target_parent_id.clone(), Arc::new(new_target_parent));
        // 更新父子关系映射
        self.inner.parent_map.insert(node_id.clone(), target_parent_id.clone());
        // 记录移动节点的补丁
        Ok(())
    }
    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<&Arc<Node>> {
        self.inner.get_node(id)
    }
   
    /// 移除子节点    
    ///
    /// # 参数
    ///
    /// * `parent_id` - 父节点ID
    /// * `nodes` - 要移除的子节点ID列表
    ///
    /// # 错误
    ///
    /// 当父节点不存在时返回 [`PoolError::ParentNotFound`]
    /// 当尝试删除根节点时返回 [`PoolError::CannotRemoveRoot`]
    /// 当要删除的节点不是父节点的直接子节点时返回 [`PoolError::InvalidParenting`]
    pub fn remove_node(
        &mut self,
        parent_id: &NodeId,
        nodes: Vec<NodeId>,
    ) -> Result<(), PoolError> {
        self.inner.node(parent_id)-nodes.clone();
        Ok(())
    }

    /// 提交修改，生成新 NodePool 和补丁列表
    pub fn commit(&self) -> StepResult {let new_pool = NodePool { inner: Arc::new(self.inner.clone()) };
    StepResult::ok(
        Arc::new(new_pool),
    )
    }
}

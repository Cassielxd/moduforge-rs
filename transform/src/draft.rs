use std::sync::Arc;
use im::HashMap;
use serde_json::Value;
use moduforge_model::{
    error::PoolError,
    mark::Mark,
    node::Node,
    node_pool::{NodePool, NodePoolInner},
    types::NodeId,
};

use crate::step::StepResult;

use super::patch::Patch;

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
    pub inner: NodePoolInner,
    pub patches: im::Vector<Patch>,
    pub current_path: im::Vector<String>,
    pub skip_record: bool,
    pub begin: bool,
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
            base,
            patches: im::Vector::new(),
            current_path: im::Vector::new(),
            skip_record: false,
            begin: false,
        }
    }

    /// 进入嵌套路径（用于记录结构化修改）
    ///
    /// # 参数
    ///
    /// * `key` - Map 类型的字段名称
    ///
    /// # 示例
    ///
    /// ```
    /// draft.enter_map("content").enter_list(0);
    /// ```
    pub fn enter_map(
        &mut self,
        key: &str,
    ) -> &mut Self {
        self.current_path.push_back(key.to_string());
        self
    }

    /// 进入嵌套路径（List类型索引）
    pub fn enter_list(
        &mut self,
        index: usize,
    ) -> &mut Self {
        self.current_path.push_back(index.to_string());
        self
    }

    /// 退出当前路径层级
    pub fn exit(&mut self) -> &mut Self {
        if !self.current_path.is_empty() {
            self.current_path =
                self.current_path.take(self.current_path.len() - 1);
        }
        self
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
        let node =
            self.get_node(id).ok_or(PoolError::NodeNotFound(id.clone()))?;
        let old_values = node.attrs.clone();

        // 更新节点属性
        let mut new_node = node.as_ref().clone();
        let new_attrs = old_values.update(new_values);
        new_node.attrs = new_attrs.clone();
        self.inner.nodes =
            self.inner.nodes.update(id.clone(), Arc::new(new_node));
        // 记录补丁
        self.record_patch(Patch::UpdateAttr {
            path: self.current_path.iter().cloned().collect(),
            id: id.clone(),
            old: old_values.clone(),
            new: new_attrs,
        });
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
        let mut node = self
            .get_node(id)
            .ok_or(PoolError::NodeNotFound(id.clone()))?
            .as_ref()
            .clone();
        node.marks =
            node.marks.iter().filter(|&m| !m.eq(&mark)).cloned().collect();
        self.inner.nodes.insert(id.clone(), Arc::new(node));
        // 记录补丁
        self.record_patch(Patch::RemoveMark {
            path: self.current_path.iter().cloned().collect(),
            parent_id: id.clone(),
            marks: vec![mark],
        });
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
        let mut node = self
            .get_node(id)
            .ok_or(PoolError::NodeNotFound(id.clone()))?
            .as_ref()
            .clone();
        
        node.marks.extend(marks.clone());
        self.inner.nodes.insert(id.clone(), Arc::new(node));
        // 记录补丁
        self.record_patch(Patch::AddMark {
            path: self.current_path.iter().cloned().collect(),
            node_id: id.clone(),
            marks: marks.clone(),
        });
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

        // 记录补丁
        self.record_patch(Patch::SortChildren {
            path: self.current_path.iter().cloned().collect(),
            parent_id: parent_id.clone(),
            old_children: children_ids.iter().cloned().collect(),
            new_children: sorted_children.iter().cloned().collect(),
        });

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
        let parent = self
            .get_node(parent_id)
            .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;
        let mut new_parent = parent.as_ref().clone();
        new_parent.content.push_back(nodes[0].id.clone());
        self.inner.nodes.insert(parent_id.clone(), Arc::new(new_parent));
        self.inner.parent_map.insert(nodes[0].id.clone(), parent_id.clone());
        let mut new_nodes = vec![];
        for node in nodes.into_iter() {
            new_nodes.push(node.clone());
            // 更新父节点映射
            for child_id in &node.content {
                self.inner.parent_map.insert(child_id.clone(), node.id.clone());
            }
            // 更新节点池
            self.inner.nodes.insert(node.id.clone(), Arc::new(node.clone()));
        }
        // 记录补丁
        self.record_patch(Patch::AddNode {
            path: self.current_path.iter().cloned().collect(),
            parent_id: parent_id.clone(),
            nodes: new_nodes,
        });
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
        let _ = self.remove_node(
            &node_id,
            old_node.content.iter().map(|id| id.clone()).collect(),
        )?;
        let _ = self.add_node(&node_id, nodes)?;
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
        self.record_patch(Patch::MoveNode {
            path: self.current_path.iter().cloned().collect(),
            node_id: node_id.clone(),
            source_parent_id: source_parent_id.clone(),
            target_parent_id: target_parent_id.clone(),
            position,
        });
        Ok(())
    }
    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<&Arc<Node>> {
        self.inner.nodes.get(id)
    }
    pub fn children(
        &self,
        parent_id: &NodeId,
    ) -> Option<&im::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| &n.content)
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
        // 检查父节点是否存在
        let parent = self
            .get_node(parent_id)
            .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;

        // 检查是否尝试删除根节点
        if nodes.contains(&self.inner.root_id) {
            return Err(PoolError::CannotRemoveRoot);
        }

        // 验证所有要删除的节点都是父节点的直接子节点
        for node_id in &nodes {
            if !parent.content.contains(node_id) {
                return Err(PoolError::InvalidParenting {
                    child: node_id.clone(),
                    alleged_parent: parent_id.clone(),
                });
            }
        }

        // 使用 HashSet 优化查找性能
        let nodes_to_remove: std::collections::HashSet<_> =
            nodes.iter().collect();

        // 过滤保留的子节点
        let filtered_children: im::Vector<NodeId> = parent
            .as_ref()
            .content
            .iter()
            .filter(|&id| !nodes_to_remove.contains(id))
            .cloned()
            .collect();

        // 更新父节点
        let mut parent_node = parent.as_ref().clone();
        parent_node.content = filtered_children;
        self.inner.nodes.insert(parent_id.clone(), Arc::new(parent_node));
        let mut remove_nodes = Vec::new();
        // 递归删除所有子节点
        for node_id in nodes {
            self.remove_subtree(&node_id, &mut remove_nodes)?;
        }
        self.record_patch(Patch::RemoveNode {
            path: self.current_path.iter().cloned().collect(),
            parent_id: parent_id.clone(),
            nodes: remove_nodes,
        });
        Ok(())
    }

    /// 递归删除子树
    ///
    /// # 参数
    ///
    /// * `parent_id` - 父节点ID
    /// * `node_id` - 要删除的节点ID
    ///
    /// # 错误
    ///
    /// 当节点不存在时返回 [`PoolError::NodeNotFound`]
    /// 当尝试删除根节点时返回 [`PoolError::CannotRemoveRoot`]
    fn remove_subtree(
        &mut self,
        node_id: &NodeId,
        remove_nodes: &mut Vec<Node>,
    ) -> Result<(), PoolError> {
        // 检查是否是根节点
        if node_id == &self.inner.root_id {
            return Err(PoolError::CannotRemoveRoot);
        }

        // 获取要删除的节点
        let _ = self
            .get_node(node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;

        // 递归删除所有子节点
        if let Some(children) = self.children(node_id).cloned() {
            for child_id in children {
                self.remove_subtree(&child_id, remove_nodes)?;
            }
        }

        // 从父节点映射中移除
        self.inner.parent_map.remove(node_id);

        // 从节点池中移除并记录补丁
        if let Some(remove_node) = self.inner.nodes.remove(node_id) {
            remove_nodes.push(remove_node.as_ref().clone());
        }
        Ok(())
    }

    /// 应用补丁集合并更新节点池
    ///
    /// # 参数
    ///
    /// * `patches` - 要应用的补丁集合
    ///
    /// # 注意
    ///
    /// 应用过程中会临时禁用补丁记录
    pub fn apply_patches(
        &mut self,
        patches: &Vec<Patch>,
    ) -> Result<(), PoolError> {
        //跳过记录
        self.skip_record = true;
        for patch in patches {
            match patch {
                Patch::UpdateAttr { path: _, id, old: _, new } => {
                    self.update_attr(id, new.attrs.clone().into())?;
                },
                Patch::AddNode { path: _, parent_id, nodes } => {
                    self.add_node(parent_id, nodes)?;
                },
                Patch::AddMark { path: _, node_id, marks } => {
                    self.add_mark(node_id, marks)?;
                },
                Patch::RemoveNode { path: _, parent_id, nodes } => {
                    self.remove_node(
                        parent_id,
                        nodes.iter().map(|n| n.id.clone()).collect(),
                    )?;
                },
                Patch::RemoveMark { path: _, parent_id, marks } => {
                    for mark in marks {
                        self.remove_mark(&parent_id, mark.clone())?;
                    }
                },
                Patch::MoveNode {
                    path: _,
                    node_id,
                    source_parent_id,
                    target_parent_id,
                    position,
                } => {
                    self.move_node(
                        source_parent_id,
                        target_parent_id,
                        node_id,
                        position.clone(),
                    )?;
                },
                Patch::SortChildren {
                    path: _,
                    parent_id,
                    old_children: _,
                    new_children,
                } => {
                    let parent = self
                        .get_node(parent_id)
                        .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;
                    let mut new_parent = parent.as_ref().clone();
                    new_parent.content = new_children.iter().cloned().collect();
                    self.inner
                        .nodes
                        .insert(parent_id.clone(), Arc::new(new_parent));
                },
            }
        }
        self.skip_record = false;
        Ok(())
    }
    /// 翻转补丁集合并应用到节点池
    pub fn reverse_patches(
        &mut self,
        patches: Vec<Patch>,
    ) -> Result<(), PoolError> {
        //跳过记录
        self.skip_record = true;
        for patch in patches {
            match patch {
                Patch::UpdateAttr { path: _, id, old, new: _ } => {
                    self.update_attr(&id, old.attrs.clone().into())?;
                },
                Patch::AddNode { path: _, parent_id, nodes } => {
                    self.remove_node(
                        &parent_id,
                        nodes.iter().map(|f| f.id.clone()).collect(),
                    )?;
                },
                Patch::AddMark { path: _, node_id, marks } => {
                    self.remove_mark(&node_id, marks[0].clone())?;
                },
                Patch::RemoveNode { path: _, parent_id, nodes } => {
                    self.add_node(&parent_id, &nodes)?;
                },
                Patch::RemoveMark { path: _, parent_id, marks } => {
                    self.add_mark(&parent_id, &marks)?;
                },
                Patch::MoveNode {
                    path: _,
                    node_id,
                    source_parent_id,
                    target_parent_id,
                    position,
                } => {
                    self.move_node(
                        &target_parent_id,
                        &source_parent_id,
                        &node_id,
                        position.clone(),
                    )?;
                },
                Patch::SortChildren {
                    path: _,
                    parent_id,
                    old_children,
                    new_children: _,
                } => {
                    let parent = self
                        .get_node(&parent_id)
                        .ok_or(PoolError::ParentNotFound(parent_id.clone()))?;
                    let mut new_parent = parent.as_ref().clone();
                    new_parent.content = old_children.iter().cloned().collect();
                    self.inner
                        .nodes
                        .insert(parent_id.clone(), Arc::new(new_parent));
                },
            }
        }
        self.skip_record = false;
        Ok(())
    }

    fn record_patch(
        &mut self,
        patch: Patch,
    ) {
        if !self.skip_record {
            self.patches.push_back(patch);
        }
    }
    /// 提交修改，生成新 NodePool 和补丁列表
    pub fn commit(&self) -> StepResult {
        match self.begin {
            true => StepResult {
                doc: None,
                failed: Some("事务操作".to_string()),
                patches: Vec::new(),
            },
            false => {
                let new_pool = NodePool { inner: Arc::new(self.inner.clone()) };
                StepResult::ok(
                    Arc::new(new_pool),
                    self.patches.iter().cloned().collect(),
                )
            },
        }
    }
}

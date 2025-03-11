use super::{error::PoolError, mark::Mark, node::Node, patch::Patch, types::NodeId};
use bincode::{Decode, Encode};
use im::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
/// 节点池内部数据结构，实现结构共享和高效克隆
///
/// # 字段
///
/// * `root_id` - 根节点标识符
/// * `nodes` - 节点存储的不可变哈希表（使用结构共享）
/// * `parent_map` - 父子关系映射表
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Decode, Encode)]
pub struct NodePoolInner {
  pub root_id: NodeId,
  #[bincode(with_serde)]
  pub nodes: im::HashMap<NodeId, Arc<Node>>, // 节点数据共享
  #[bincode(with_serde)]
  pub parent_map: im::HashMap<NodeId, NodeId>,
}
impl NodePoolInner {
  /// 更新节点属性（创建新版本的数据结构）
  ///
  /// # 参数
  ///
  /// * `id` - 目标节点ID
  /// * `values` - 要更新的属性键值对
  ///
  /// # 返回值
  ///
  /// 返回包含新节点属性的新版本 `NodePoolInner`
  ///
  /// # 错误
  ///
  /// 当节点不存在时返回 [`PoolError::NodeNotFound`]
  pub fn update_attr(
    &self,
    id: &NodeId,
    values: &HashMap<String, String>,
  ) -> Result<Self, PoolError> {
    if !self.nodes.contains_key(id) {
      return Err(PoolError::NodeNotFound(id.clone()));
    }
    let node = self.nodes.get(id).unwrap();

    let mut cope_node = node.clone().as_ref().clone();
    cope_node.attrs.extend(values.clone());
    let nodes = self.nodes.update(id.clone(), Arc::new(cope_node));
    Ok(NodePoolInner {
      nodes,
      parent_map: self.parent_map.clone(),
      root_id: self.root_id.clone(),
    })
  }
}
/// 线程安全的节点池封装
///
/// 使用 [`Arc`] 实现快速克隆，内部使用不可变数据结构保证线程安全
#[derive(Clone, PartialEq, Debug, Decode, Encode, Serialize, Deserialize)]
pub struct NodePool {
  // 使用 Arc 包裹内部结构，实现快速克隆
  pub inner: Arc<NodePoolInner>,
}
unsafe impl Send for NodePool {}
unsafe impl Sync for NodePool {}

impl NodePool {
  /// 获取节点池中节点总数
  pub fn size(&self) -> usize {
    self.inner.nodes.len()
  }

  /// 从节点列表构建节点池
  ///
  /// # 参数
  ///
  /// * `nodes` - 初始节点列表
  /// * `root_id` - 指定根节点ID
  ///
  /// # 注意
  ///
  /// 会自动构建父子关系映射表
  pub fn from(
    nodes: Vec<Node>,
    root_id: NodeId,
  ) -> Self {
    let mut nodes_ref = HashMap::new();
    let mut parent_map_ref = HashMap::new();
    for node in nodes.into_iter() {
      for child_id in &node.content {
        parent_map_ref.insert(child_id.clone(), node.id.clone());
      }
      nodes_ref.insert(node.id.clone(), Arc::new(node));
    }

    NodePool {
      inner: Arc::new(NodePoolInner {
        nodes: nodes_ref,
        parent_map: parent_map_ref,
        root_id,
      }),
    }
  }

  // -- 核心查询方法 --

  /// 根据ID获取节点(immutable)
  pub fn get_node(
    &self,
    id: &NodeId,
  ) -> Option<&Arc<Node>> {
    self.inner.nodes.get(id)
  }

  /// 检查节点是否存在
  pub fn contains_node(
    &self,
    id: &NodeId,
  ) -> bool {
    self.inner.nodes.contains_key(id)
  }

  // -- 层级关系操作 --

  /// 获取直接子节点列表
  pub fn children(
    &self,
    parent_id: &NodeId,
  ) -> Option<&im::Vector<NodeId>> {
    self.get_node(parent_id).map(|n| &n.content)
  }

  /// 递归获取所有子节点（深度优先）
  pub fn descendants(
    &self,
    parent_id: &NodeId,
  ) -> Vec<&Node> {
    let mut result: Vec<&Node> = Vec::new();
    self._collect_descendants(parent_id, &mut result);
    result
  }

  fn _collect_descendants<'a>(
    &'a self,
    parent_id: &NodeId,
    result: &mut Vec<&'a Node>,
  ) {
    if let Some(children) = self.children(parent_id) {
      for child_id in children {
        if let Some(child) = self.get_node(child_id) {
          result.push(child);
          self._collect_descendants(child_id, result);
        }
      }
    }
  }

  /// 获取父节点ID
  pub fn parent_id(
    &self,
    child_id: &NodeId,
  ) -> Option<&NodeId> {
    self.inner.parent_map.get(child_id)
  }

  /// 获取完整祖先链
  pub fn ancestors(
    &self,
    child_id: &NodeId,
  ) -> Vec<&Arc<Node>> {
    let mut chain = Vec::new();
    let mut current_id = child_id;
    while let Some(parent_id) = self.parent_id(current_id) {
      if let Some(parent) = self.get_node(parent_id) {
        chain.push(parent);
        current_id = parent_id;
      } else {
        break;
      }
    }
    chain
  }

  /// 验证父子关系一致性
  pub fn validate_hierarchy(&self) -> Result<(), PoolError> {
    for (child_id, parent_id) in &self.inner.parent_map {
      // 验证父节点存在
      if !self.contains_node(parent_id) {
        return Err(PoolError::OrphanNode(child_id.clone()));
      }

      // 验证父节点确实包含该子节点
      if let Some(children) = self.children(parent_id) {
        if !children.contains(child_id) {
          return Err(PoolError::InvalidParenting {
            child: child_id.clone(),
            alleged_parent: parent_id.clone(),
          });
        }
      }
    }
    Ok(())
  }

  // -- 高级查询 --
  /// 根据类型筛选节点
  pub fn filter_nodes<P>(
    &self,
    predicate: P,
  ) -> Vec<&Arc<Node>>
  where
    P: Fn(&Node) -> bool,
  {
    self.inner.nodes.values().filter(|n| predicate(n)).collect()
  }
  /// 查找第一个匹配节点
  pub fn find_node<P>(
    &self,
    predicate: P,
  ) -> Option<&Arc<Node>>
  where
    P: Fn(&Node) -> bool,
  {
    self.inner.nodes.values().find(|n| predicate(n))
  }
}
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
  pub patches: Vec<Patch>,
  pub current_path: Vec<String>,
  pub skip_record: bool,
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
      patches: Vec::new(),
      current_path: Vec::new(),
      skip_record: false,
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
    self.current_path.push(key.to_string());
    self
  }

  /// 进入嵌套路径（List类型索引）
  pub fn enter_list(
    &mut self,
    index: usize,
  ) -> &mut Self {
    self.current_path.push(index.to_string());
    self
  }

  /// 退出当前路径层级
  pub fn exit(&mut self) -> &mut Self {
    self.current_path.pop();
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
    new_values: HashMap<String, String>,
  ) -> Result<(), PoolError> {
    let node = self.get_node(id).ok_or(PoolError::NodeNotFound(id.clone()))?;
    let old_values = node.attrs.clone();

    // 更新节点属性
    let mut new_node = node.as_ref().clone();
    new_node.attrs = new_values.clone();
    self.inner.nodes.insert(id.clone(), Arc::new(new_node));
    // 记录补丁
    self.record_patch(Patch::UpdateAttr {
      path: self.current_path.clone(),
      id: id.clone(),
      old: old_values.into_iter().collect(),
      new: new_values.into_iter().collect(),
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
    let mut node = self.get_node(id).ok_or(PoolError::NodeNotFound(id.clone()))?.as_ref().clone();
    node.marks = node.marks.iter().filter(|&m| !m.eq(&mark)).cloned().collect();
    self.inner.nodes.insert(id.clone(), Arc::new(node));
    // 记录补丁
    self.record_patch(Patch::RemoveMark {
      path: self.current_path.clone(),
      parent_id: id.clone(),
      marks: vec![Arc::new(mark)],
    });
    Ok(())
  }
  /// 为节点添加标记
  ///
  /// # 参数
  ///
  /// * `id` - 目标节点ID
  /// * `mark` - 要添加的标记
  ///
  /// # 错误
  ///
  /// 当节点不存在时返回 [`PoolError::NodeNotFound`]
  pub fn add_mark(
    &mut self,
    id: &NodeId,
    mark: Mark,
  ) -> Result<(), PoolError> {
    let mut node = self.get_node(id).ok_or(PoolError::NodeNotFound(id.clone()))?.as_ref().clone();
    node.marks.push_back(mark.clone());
    self.inner.nodes.insert(id.clone(), Arc::new(node));
    // 记录补丁
    self.record_patch(Patch::AddMark {
      path: self.current_path.clone(),
      node_id: id.clone(),
      mark,
    });

    Ok(())
  }
  /// 添加子节点
  pub fn add_node(
    &mut self,
    parent_id: &NodeId,
    node: Node,
  ) -> Result<(), PoolError> {
    let node = Arc::new(node);
    let parent = self.get_node(parent_id).ok_or(PoolError::ParentNotFound(parent_id.clone()))?;
    let mut new_parent = parent.as_ref().clone();
    new_parent.content.push_back(node.id.clone());

    let id = node.id.clone();
    self.inner.nodes.insert(parent_id.clone(), Arc::new(new_parent));
    self.inner.nodes.insert(node.id.clone(), node.clone());
    self.inner.parent_map.insert(id.clone(), parent_id.clone());
    // 记录补丁
    self.record_patch(Patch::AddNode {
      path: self.current_path.clone(),
      parent_id: parent_id.clone(),
      node: node.clone(),
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
  fn remove_subtree(
    &mut self,
    parent_id: &NodeId,
    node_id: &NodeId,
  ) {
    if let Some(children) = self.children(node_id).cloned() {
      for child_id in children {
        self.remove_subtree(node_id, &child_id);
      }
    }
    self.inner.parent_map.remove(node_id);
    if let Some(romove_node) = self.inner.nodes.remove(node_id) {
      self.record_patch(Patch::RemoveNode {
        path: self.current_path.clone(),
        parent_id: parent_id.clone(),
        nodes: vec![romove_node],
      });
    }
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
  pub fn remove_node(
    &mut self,
    parent_id: &NodeId,
    nodes: Vec<NodeId>,
  ) -> Result<(), PoolError> {
    let parent = self.get_node(parent_id).ok_or(PoolError::ParentNotFound(parent_id.clone()))?;

    // 过滤掉不在节点池中的子节点
    let filtered_children: im::Vector<NodeId> = parent.as_ref().content.iter().filter(|&id| !nodes.contains(id)).cloned().collect();

    // 这里的逻辑需要进一步完善，例如如何处理新节点的添加
    // 以下是示例代码，实际逻辑可能需要根据需求调整
    let mut parent_node = parent.as_ref().clone();
    parent_node.content = filtered_children;
    // 更新节点池和父节点映射
    self.inner.nodes.insert(parent_id.clone(), Arc::new(parent_node));
    // 移除指定的节点
    let mut renoved_nodes = vec![];
    for node_id in nodes {
      if self.inner.nodes.contains_key(&node_id) {
        self.remove_subtree(parent_id, &node_id);
        self.inner.parent_map.remove(&node_id);
        if let Some(romove_node) = self.inner.nodes.remove(&node_id) {
          renoved_nodes.push(romove_node);
        }
      }
    }
    // 记录补丁
    self.record_patch(Patch::RemoveNode {
      path: self.current_path.clone(),
      parent_id: parent_id.clone(),
      nodes: renoved_nodes,
    });

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
          self.update_attr(id, new.clone().into())?;
        }
        Patch::AddNode { path: _, parent_id, node } => {
          self.add_node(parent_id, node.as_ref().clone())?;
        }
        Patch::AddMark { path: _, node_id, mark } => {
          self.add_mark(node_id, mark.clone())?;
        }
        Patch::RemoveNode { path: _, parent_id, nodes } => {
          self.remove_node(parent_id, nodes.iter().map(|n: &Arc<Node>| n.id.clone()).collect())?;
        }
        Patch::RemoveMark { path: _, parent_id, marks } => {
          for mark in marks {
            self.remove_mark(parent_id, mark.as_ref().clone())?;
          }
        }
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
          self.update_attr(&id, old.clone().into())?;
        }
        Patch::AddNode { path: _, parent_id, node } => {
          self.remove_node(&parent_id, vec![node.id.clone()])?;
        }
        Patch::AddMark { path: _, node_id, mark } => {
          self.remove_mark(&node_id, mark)?;
        }
        Patch::RemoveNode { path: _, parent_id, nodes } => {
          for node in nodes {
            self.add_node(&parent_id, node.as_ref().clone())?;
          }
        }
        Patch::RemoveMark { path: _, parent_id, marks } => {
          for mark in marks {
            self.add_mark(&parent_id, mark.as_ref().clone())?;
          }
        }
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
      self.patches.push(patch);
    }
  }
  /// 提交修改，生成新 NodePool 和补丁列表
  pub fn commit(&self) -> (Arc<NodePool>, Vec<Patch>) {
    let new_pool = NodePool {
      inner: Arc::new(self.inner.clone()),
    };
    (Arc::new(new_pool), self.patches.clone())
  }
}

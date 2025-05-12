use std::{ ops::{Add,  Index, Sub}, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{attrs::Attrs, error::PoolError, mark::Mark, node::Node, types::NodeId};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tree {
    pub root_id: NodeId,
    pub nodes: im::HashMap<NodeId, Arc<Node>>, // 节点数据共享
    pub parent_map: im::HashMap<NodeId, NodeId>,
}
impl Index<&NodeId> for Tree {
    type Output = Arc<Node>;

    fn index(&self, index: &NodeId) -> &Self::Output {
        self.nodes.get(index).unwrap()
    }
}
// 用于处理节点赋值的包装器
pub struct NodeRef<'a> {
    tree: &'a mut Tree,
    key: String,    
}

impl<'a> NodeRef<'a> {
    fn new(tree: &'a mut Tree, key: String) -> Self {
        Self { tree, key }
    }
}

impl<'a> std::ops::Deref for NodeRef<'a> {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}

impl<'a> std::ops::DerefMut for NodeRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree
    }
}

// 用于处理节点赋值的包装器
pub struct MarkRef<'a> {
    tree: &'a mut Tree,
    key: String,
}

impl<'a> MarkRef<'a> {
    fn new(tree: &'a mut Tree, key: String) -> Self {
        Self { tree, key }
    }
}

impl<'a> std::ops::Deref for MarkRef<'a> {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}

impl<'a> std::ops::DerefMut for MarkRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree
    }
}


pub struct AttrsRef<'a> {
    tree: &'a mut Tree,
    key: String,
}

impl<'a> AttrsRef<'a> {
    fn new(tree: &'a mut Tree, key: String) -> Self {
        Self { tree, key }
    }
}

impl<'a> std::ops::Deref for AttrsRef<'a> {
    type Target = Tree;

    fn deref(&self) -> &Self::Target {
        self.tree
    }
}

impl<'a> std::ops::DerefMut for AttrsRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tree
    }
}

impl Tree {
    // 获取节点的引用
    pub fn node(&mut self, key: &str) -> NodeRef<'_> {
        NodeRef::new(self, key.to_string())
    }
    pub fn mark(&mut self, key: &str) -> MarkRef<'_> {
        MarkRef::new(self, key.to_string())
    }
    pub fn attrs(&mut self, key: &str) -> AttrsRef<'_> {
        AttrsRef::new(self, key.to_string())
    }
    
    
    pub fn get_node(
        &self,
        id: &NodeId,
    ) -> Option<&Arc<Node>> {
        self.nodes.get(id)
    }
    pub fn get_parent_node(
        &self,
        id: &NodeId,
    ) -> Option<&Arc<Node>> {
        self.parent_map.get(id).and_then(|id| self.nodes.get(id))
    }

    pub fn children(
        &self,
        parent_id: &NodeId,
    ) -> Option<&im::Vector<NodeId>> {
        self.get_node(parent_id).map(|n| &n.content)
    }
    pub fn children_node(
        &self,
        parent_id: &NodeId,
    ) -> Option<im::Vector<&Arc<Node>>> {
        self.children(parent_id).map(|ids| ids.iter().filter_map(|id| self.get_node(id)).collect())
    }
    pub fn children_count(
        &self,
        parent_id: &NodeId,
    ) -> usize {
        self.get_node(parent_id).map(|n| n.content.len()).unwrap_or(0)
    }
    


    pub fn new(root: Node) -> Self {
        Self { root_id: root.id.clone(), nodes: im::HashMap::from(vec![(root.id.clone(), Arc::new(root))]), parent_map: im::HashMap::new() }
    }

    
    pub fn update_attr(
        &mut self,
        id: &NodeId,
        new_values: im::HashMap<String, Value>,
    ) -> Result<(), PoolError> {
        let node =
            self.get_node(id).ok_or(PoolError::NodeNotFound(id.clone()))?;
        let old_values = node.attrs.clone();

        // 更新节点属性
        let mut new_node = node.as_ref().clone();
        let new_attrs = old_values.update(new_values);
        new_node.attrs = new_attrs.clone();
        self.nodes.insert(id.clone(), Arc::new(new_node));
        Ok(())
    }
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
        self.nodes.insert(id.clone(), Arc::new(node));
     
        Ok(())
    }

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
        self.nodes.insert(id.clone(), Arc::new(node));
      
        Ok(())
    }
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
        self.nodes.insert(parent_id.clone(), Arc::new(new_parent));
        self.parent_map.insert(nodes[0].id.clone(), parent_id.clone());
        let mut new_nodes = vec![];
        for node in nodes.into_iter() {
            new_nodes.push(node.clone());
            // 更新父节点映射
            for child_id in &node.content {
                self.parent_map.insert(child_id.clone(), node.id.clone());
            }
            // 更新节点池
            self.nodes.insert(node.id.clone(), Arc::new(node.clone()));
        }
        Ok(())
    }

    pub fn replace_node(
        &mut self,
        node_id: NodeId,
        nodes: &Vec<Node>,
    ) -> Result<(), PoolError> {
        // 检查节点是否存在
        let _ = self
            .get_node(&node_id)
            .ok_or(PoolError::NodeNotFound(node_id.clone()))?;
        // 确保新节点ID与原节点ID一致
        if nodes[0].id != node_id {
            return Err(PoolError::InvalidNodeId {
                nodeid: node_id,
                new_node_id: nodes[0].id.clone(),
            });
        }
        let _ = self.add_node(&node_id, nodes)?;
        Ok(())
    }

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

        self
            .nodes
            .insert(source_parent_id.clone(), Arc::new(new_source_parent));
        self
            .nodes
            .insert(target_parent_id.clone(), Arc::new(new_target_parent));
        // 更新父子关系映射
        self.parent_map.insert(node_id.clone(), target_parent_id.clone());
        
        Ok(())
    }

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
        if nodes.contains(&self.root_id) {
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
        self.nodes.insert(parent_id.clone(), Arc::new(parent_node));
        let mut remove_nodes = Vec::new();
        // 递归删除所有子节点
        for node_id in nodes {
            self.remove_subtree(&node_id, &mut remove_nodes)?;
        }
        
        Ok(())
    }

    fn remove_subtree(
        &mut self,
        node_id: &NodeId,
        remove_nodes: &mut Vec<Node>,
    ) -> Result<(), PoolError> {
        // 检查是否是根节点
        if node_id == &self.root_id {
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
        self.parent_map.remove(node_id);

        // 从节点池中移除并记录补丁
        if let Some(remove_node) = self.nodes.remove(node_id) {
            remove_nodes.push(remove_node.as_ref().clone());
        }
        Ok(())
    }
}

impl Index<&str> for Tree {
    type Output = Arc<Node>;

    fn index(&self, index: &str) -> &Self::Output {
        self.nodes.get(index).expect("Node not found")
    }
}

// 实现自定义的 + 运算符
impl<'a> Add<Node> for NodeRef<'a> {
    type Output = ();
    fn add(self, node: Node) -> Self::Output {
        let _ = self.tree.add_node(&self.key.into(), &vec![node]);
    }
}
impl<'a> Add<Vec<Node>> for NodeRef<'a> {
    type Output = ();
    fn add(self, nodes: Vec<Node>) -> Self::Output {
        let _ = self.tree.add_node(&self.key.into(), &nodes);
    }
}

impl<'a> Add<Mark> for MarkRef<'a> {
    type Output = ();
    fn add(self, mark: Mark) -> Self::Output {
        let _ = self.tree.add_mark(&self.key.into(), &vec![mark]);
    }
}
impl<'a> Add<Vec<Mark>> for MarkRef<'a> {
    type Output = ();
    fn add(self, marks: Vec<Mark>) -> Self::Output {
        let _ = self.tree.add_mark(&self.key.into(), &marks);
    }
}

impl<'a> Add<Attrs> for AttrsRef<'a> {
    type Output = ();
    fn add(self, attrs: Attrs) -> Self::Output {
        let _ = self.tree.update_attr(&self.key.into(), attrs.attrs);
    }
}
impl<'a> Add<im::HashMap<String, Value>> for AttrsRef<'a> {
    type Output = ();
    fn add(self, attrs: im::HashMap<String, Value>) -> Self::Output {
        let _ = self.tree.update_attr(&self.key.into(), attrs);
    }
}


// 实现自定义的 - 运算符
impl<'a> Sub<NodeId> for NodeRef<'a> {
    type Output = ();
    fn sub(self, node_id: NodeId) -> Self::Output {
        let _ = self.tree.remove_node(&self.key.into(), vec![node_id]);
    }
    
}
impl<'a> Sub<Vec<NodeId>> for NodeRef<'a> {
    type Output = ();
    fn sub(self, node_ids: Vec<NodeId>) -> Self::Output {
        let _ = self.tree.remove_node(&self.key.into(), node_ids);
    }
}
impl<'a> Sub<Mark> for MarkRef<'a> {
    type Output = ();
    fn sub(self, mark: Mark) -> Self::Output {
        let _ = self.tree.remove_mark(&self.key.into(), mark);
    }
    
}
impl<'a> Sub<Vec<Mark>> for MarkRef<'a> {
    type Output = ();
    fn sub(self, marks: Vec<Mark>) -> Self::Output {
        for mark in marks {
            let _ = self.tree.remove_mark(&self.key, mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::mark::Mark;
    use crate::attrs::Attrs;
    use crate::types::NodeId;
    use serde_json::json;

    fn create_test_node(id: &str) -> Node {
        Node {
            id: id.into(),
            r#type: "text".to_string(),
            content: im::Vector::new(),
            marks: im::Vector::new(),
            attrs: Attrs::default(),
        }
    }

    #[test]
    fn test_tree_creation() {
        let root = create_test_node("root");
        let tree = Tree::new(root);
        assert!(tree.nodes.is_empty());
        assert!(tree.parent_map.is_empty());
    }

    #[test]
    fn test_add_node() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root);
        let node = create_test_node("node1");
        
        // Add node to root
        let result = tree.add_node(&"root".into(), &vec![node.clone()]);
        assert!(result.is_ok());
        
        // Verify node was added
        let node1_id: NodeId = "node1".into();
        let root_id: NodeId = "root".into();
        assert!(tree.get_node(&node1_id).is_some());
        assert_eq!(tree.parent_map.get(&node1_id), Some(&root_id));
    }

    #[test]
    fn test_remove_node() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root);
        let node = create_test_node("node1");
        
        // Add and then remove node
        tree.add_node(&"root".into(), &vec![node.clone()]).unwrap();
        let node1_id: NodeId = "node1".into();
        let result = tree.remove_node(&"root".into(), vec![node1_id.clone()]);
        assert!(result.is_ok());
        
        // Verify node was removed
        assert!(tree.get_node(&node1_id).is_none());
        assert!(tree.parent_map.get(&node1_id).is_none());
    }

    #[test]
    fn test_add_mark() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root);
        let mut node = create_test_node("node1");
        tree.add_node(&"root".into(), &vec![node.clone()]).unwrap();

        let mark = Mark {
            r#type: "bold".to_string(),
            attrs: Attrs::default(),
        };

        let node1_id: NodeId = "node1".into();
        let result = tree.add_mark(&node1_id, &vec![mark.clone()]);
        assert!(result.is_ok());

        let updated_node = tree.get_node(&node1_id).unwrap();
        assert!(updated_node.marks.contains(&mark));
    }

    #[test]
    fn test_update_attrs() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root);
        let node = create_test_node("node1");
        tree.add_node(&"root".into(), &vec![node.clone()]).unwrap();

        let mut attrs = im::HashMap::new();
        attrs.insert("color".to_string(), json!("red"));

        let node1_id: NodeId = "node1".into();
        let result = tree.update_attr(&node1_id, attrs.clone());
        assert!(result.is_ok());

        let updated_node = tree.get_node(&node1_id).unwrap();
        assert_eq!(updated_node.attrs.attrs, attrs);
    }

    #[test]
    fn test_move_node() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root);
        let node1 = create_test_node("node1");
        let node2 = create_test_node("node2");
        
        // Add nodes
        tree.add_node(&"root".into(), &vec![node1.clone()]).unwrap();
        tree.add_node(&"root".into(), &vec![node2.clone()]).unwrap();

        let node1_id: NodeId = "node1".into();
        let node2_id: NodeId = "node2".into();
        let root_id: NodeId = "root".into();

        // Move node1 to be a child of node2
        let result = tree.move_node(&root_id, &node2_id, &node1_id, None);
        assert!(result.is_ok());

        // Verify new parent relationship
        assert_eq!(tree.parent_map.get(&node1_id), Some(&node2_id));
        assert!(tree.children(&node2_id).unwrap().contains(&node1_id));
    }

    #[test]
    fn test_children_operations() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root);
        let node1 = create_test_node("node1");
        let node2 = create_test_node("node2");
        
        // Add nodes
        tree.node("root")+node1.clone();
        tree.node("root")+node2.clone();
        dbg!("{:?}", &tree);
        tree.node("root")-"node1".to_string();
        dbg!("{:?}", &tree);
    }

    #[test]
    fn test_operator_overloading() {
        let root = create_test_node("root");
        let mut tree = Tree::new(root);
        let node = create_test_node("node1");
        
        // Test + operator for adding node
        tree.node("root") + node.clone();

        // Test + operator for adding mark
        let mark = Mark {
            r#type: "bold".to_string(),
            attrs: Attrs::default(),
        };
        tree.mark("node1") + mark.clone();
        dbg!("{:?}", &tree);
    }

}

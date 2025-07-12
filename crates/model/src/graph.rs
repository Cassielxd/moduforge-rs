use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use serde::{Deserialize, Serialize};
use petgraph::prelude::{StableDiGraph, NodeIndex, EdgeIndex};
use petgraph::visit::{EdgeRef, IntoNodeIdentifiers, VisitMap, Visitable};
use petgraph::{Incoming, Outgoing};
use im::HashMap as ImHashMap;
use im::Vector as ImVector;

use crate::{
    node::Node,
    types::NodeId,
    error::PoolResult,
};

/// 关系类型枚举，定义节点间的不同关系类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    /// 父子关系（树形结构）
    ParentChild,
    /// 引用关系（节点引用）
    Reference,
    /// 依赖关系（依赖其他节点）
    Dependency,
    /// 关联关系（一般关联）
    Association,
    /// 包含关系（包含其他节点）
    Contains,
    /// 继承关系（继承自其他节点）
    Inherits,
    /// 实现关系（实现接口）
    Implements,
    /// 组合关系（组合其他节点）
    Composition,
    /// 聚合关系（聚合其他节点）
    Aggregation,
    /// 自定义关系
    Custom(String),
}

impl RelationType {
    /// 获取关系的可读名称
    pub fn name(&self) -> &str {
        match self {
            RelationType::ParentChild => "parent_child",
            RelationType::Reference => "reference",
            RelationType::Dependency => "dependency",
            RelationType::Association => "association",
            RelationType::Contains => "contains",
            RelationType::Inherits => "inherits",
            RelationType::Implements => "implements",
            RelationType::Composition => "composition",
            RelationType::Aggregation => "aggregation",
            RelationType::Custom(name) => name,
        }
    }

    /// 检查是否为树形关系
    pub fn is_tree_relation(&self) -> bool {
        matches!(self, RelationType::ParentChild | RelationType::Contains)
    }

    /// 检查是否为有向关系
    pub fn is_directed(&self) -> bool {
        !matches!(self, RelationType::Association)
    }
}

/// 关系边定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// 关系类型
    pub relation_type: RelationType,
    /// 关系属性
    pub attrs: ImHashMap<String, serde_json::Value>,
    /// 关系权重（用于算法计算）
    pub weight: f64,
    /// 关系描述
    pub description: Option<String>,
    /// 是否可见
    pub visible: bool,
}

impl Relation {
    /// 创建新的关系
    pub fn new(relation_type: RelationType) -> Self {
        Self {
            relation_type,
            attrs: ImHashMap::new(),
            weight: 1.0,
            description: None,
            visible: true,
        }
    }

    /// 设置关系属性
    pub fn with_attrs(mut self, attrs: ImHashMap<String, serde_json::Value>) -> Self {
        self.attrs = attrs;
        self
    }

    /// 设置关系权重
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// 设置关系描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置可见性
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// 图节点包装器，包含节点数据和元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// 原始节点数据
    pub node: Arc<Node>,
    /// 节点在图中的索引
    #[serde(skip)]
    pub index: Option<NodeIndex>,
    /// 节点标签
    pub labels: ImVector<String>,
    /// 节点属性
    pub properties: ImHashMap<String, serde_json::Value>,
    /// 节点状态
    pub state: NodeState,
}

/// 节点状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// 活跃状态
    Active,
    /// 非活跃状态
    Inactive,
    /// 删除状态
    Deleted,
    /// 锁定状态
    Locked,
    /// 草稿状态
    Draft,
}

impl GraphNode {
    /// 创建新的图节点
    pub fn new(node: Node) -> Self {
        Self {
            node: Arc::new(node),
            index: None,
            labels: ImVector::new(),
            properties: ImHashMap::new(),
            state: NodeState::Active,
        }
    }

    /// 获取节点ID
    pub fn id(&self) -> &NodeId {
        &self.node.id
    }

    /// 获取节点类型
    pub fn node_type(&self) -> &str {
        &self.node.r#type
    }

    /// 添加标签
    pub fn add_label(&mut self, label: String) {
        self.labels = self.labels.push_back(label);
    }

    /// 移除标签
    pub fn remove_label(&mut self, label: &str) {
        self.labels = self.labels
            .iter()
            .filter(|l| l != label)
            .cloned()
            .collect();
    }

    /// 设置属性
    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.properties = self.properties.update(key, value);
    }

    /// 获取属性
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// 设置状态
    pub fn set_state(&mut self, state: NodeState) {
        self.state = state;
    }
}

/// 混合图结构，结合 Petgraph 和 Im 的优势
#[derive(Clone)]
pub struct HybridGraph {
    /// Petgraph 图结构，用于算法和遍历
    graph: StableDiGraph<GraphNode, Relation>,
    /// Im 映射，用于快速查找和不可变操作
    node_map: ImHashMap<NodeId, NodeIndex>,
    /// 关系索引，按类型分组
    relation_index: ImHashMap<RelationType, ImVector<EdgeIndex>>,
    /// 根节点ID
    root_id: Option<NodeId>,
    /// 图元数据
    metadata: ImHashMap<String, serde_json::Value>,
}

impl Debug for HybridGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridGraph")
            .field("node_count", &self.graph.node_count())
            .field("edge_count", &self.graph.edge_count())
            .field("root_id", &self.root_id)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl HybridGraph {
    /// 创建新的混合图
    pub fn new() -> Self {
        Self {
            graph: StableDiGraph::new(),
            node_map: ImHashMap::new(),
            relation_index: ImHashMap::new(),
            root_id: None,
            metadata: ImHashMap::new(),
        }
    }

    /// 添加节点到图中
    pub fn add_node(&mut self, node: Node) -> PoolResult<NodeIndex> {
        let node_id = node.id.clone();
        let graph_node = GraphNode::new(node);
        let index = self.graph.add_node(graph_node);
        
        // 更新节点映射
        self.node_map = self.node_map.update(node_id.clone(), index);
        
        // 如果是第一个节点，设为根节点
        if self.root_id.is_none() {
            self.root_id = Some(node_id);
        }
        
        Ok(index)
    }

    /// 添加关系边
    pub fn add_relation(
        &mut self,
        source_id: &NodeId,
        target_id: &NodeId,
        relation: Relation,
    ) -> PoolResult<EdgeIndex> {
        let source_index = self.node_map.get(source_id)
            .ok_or_else(|| anyhow::anyhow!("Source node not found: {}", source_id))?;
        let target_index = self.node_map.get(target_id)
            .ok_or_else(|| anyhow::anyhow!("Target node not found: {}", target_id))?;

        let edge_index = self.graph.add_edge(*source_index, *target_index, relation.clone());
        
        // 更新关系索引
        let relation_type = relation.relation_type.clone();
        let current_edges = self.relation_index.get(&relation_type)
            .cloned()
            .unwrap_or_else(ImVector::new);
        self.relation_index = self.relation_index.update(
            relation_type,
            current_edges.push_back(edge_index)
        );
        
        Ok(edge_index)
    }

    /// 获取节点
    pub fn get_node(&self, node_id: &NodeId) -> Option<&GraphNode> {
        self.node_map.get(node_id)
            .and_then(|&index| self.graph.node_weight(index))
    }

    /// 获取节点索引
    pub fn get_node_index(&self, node_id: &NodeId) -> Option<NodeIndex> {
        self.node_map.get(node_id).copied()
    }

    /// 获取节点的所有邻居
    pub fn get_neighbors(&self, node_id: &NodeId) -> Vec<&GraphNode> {
        self.get_node_index(node_id)
            .map(|index| {
                self.graph.neighbors(index)
                    .filter_map(|neighbor_index| self.graph.node_weight(neighbor_index))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取节点的所有关系
    pub fn get_relations(&self, node_id: &NodeId) -> Vec<(&GraphNode, &Relation)> {
        self.get_node_index(node_id)
            .map(|index| {
                self.graph.edges(index)
                    .filter_map(|edge_ref| {
                        let target_index = edge_ref.target();
                        let target_node = self.graph.node_weight(target_index)?;
                        Some((target_node, edge_ref.weight()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取特定类型的关系
    pub fn get_relations_by_type(
        &self,
        node_id: &NodeId,
        relation_type: &RelationType,
    ) -> Vec<(&GraphNode, &Relation)> {
        self.get_relations(node_id)
            .into_iter()
            .filter(|(_, relation)| &relation.relation_type == relation_type)
            .collect()
    }

    /// 获取父子关系（树形结构）
    pub fn get_children(&self, node_id: &NodeId) -> Vec<&GraphNode> {
        self.get_relations_by_type(node_id, &RelationType::ParentChild)
            .into_iter()
            .map(|(node, _)| node)
            .collect()
    }

    /// 获取父节点
    pub fn get_parent(&self, node_id: &NodeId) -> Option<&GraphNode> {
        self.get_node_index(node_id)
            .and_then(|index| {
                self.graph.edges_directed(index, Incoming)
                    .find(|edge_ref| edge_ref.weight().relation_type == RelationType::ParentChild)
                    .and_then(|edge_ref| {
                        let source_index = edge_ref.source();
                        self.graph.node_weight(source_index)
                    })
            })
    }

    /// 获取所有祖先节点
    pub fn get_ancestors(&self, node_id: &NodeId) -> Vec<&GraphNode> {
        let mut ancestors = Vec::new();
        let mut current = self.get_parent(node_id);
        
        while let Some(parent) = current {
            ancestors.push(parent);
            current = self.get_parent(&parent.node.id);
        }
        
        ancestors
    }

    /// 获取所有后代节点
    pub fn get_descendants(&self, node_id: &NodeId) -> Vec<&GraphNode> {
        let mut descendants = Vec::new();
        let mut to_visit = vec![node_id.clone()];
        
        while let Some(current_id) = to_visit.pop() {
            let children = self.get_children(&current_id);
            for child in children {
                descendants.push(child);
                to_visit.push(child.node.id.clone());
            }
        }
        
        descendants
    }

    /// 检查是否存在循环依赖
    pub fn has_cycles(&self) -> bool {
        use petgraph::algo::is_cyclic_directed;
        is_cyclic_directed(&self.graph)
    }

    /// 获取拓扑排序
    pub fn topological_sort(&self) -> Result<Vec<&GraphNode>, String> {
        if self.has_cycles() {
            return Err("Graph contains cycles, cannot perform topological sort".to_string());
        }
        
        use petgraph::algo::toposort;
        let indices = toposort(&self.graph, None)
            .map_err(|_| "Failed to perform topological sort")?;
        
        Ok(indices
            .into_iter()
            .filter_map(|index| self.graph.node_weight(index))
            .collect())
    }

    /// 获取最短路径
    pub fn shortest_path(
        &self,
        source_id: &NodeId,
        target_id: &NodeId,
    ) -> Option<Vec<&GraphNode>> {
        use petgraph::algo::dijkstra;
        
        let source_index = self.get_node_index(source_id)?;
        let target_index = self.get_node_index(target_id)?;
        
        let paths = dijkstra(&self.graph, source_index, Some(target_index), |edge| {
            edge.weight().weight as u32
        });
        
        paths.get(&target_index)
            .map(|&_cost| {
                // 这里需要实现路径重建逻辑
                // 简化实现，实际应用中需要更复杂的路径重建
                vec![self.get_node(source_id).unwrap()]
            })
    }

    /// 移除节点
    pub fn remove_node(&mut self, node_id: &NodeId) -> PoolResult<()> {
        if let Some(index) = self.get_node_index(node_id) {
            // 移除所有相关的边
            let edges_to_remove: Vec<_> = self.graph.edges(index).collect();
            for edge_ref in edges_to_remove {
                self.graph.remove_edge(edge_ref.id());
            }
            
            // 移除节点
            self.graph.remove_node(index);
            
            // 更新节点映射
            self.node_map = self.node_map.without(node_id);
            
            // 如果移除的是根节点，重新选择根节点
            if self.root_id.as_ref() == Some(node_id) {
                self.root_id = self.node_map.keys().next().cloned();
            }
        }
        
        Ok(())
    }

    /// 移除关系
    pub fn remove_relation(
        &mut self,
        source_id: &NodeId,
        target_id: &NodeId,
        relation_type: &RelationType,
    ) -> PoolResult<()> {
        if let (Some(source_index), Some(target_index)) = (
            self.get_node_index(source_id),
            self.get_node_index(target_id),
        ) {
            let edge_to_remove = self.graph.edges_connecting(source_index, target_index)
                .find(|edge_ref| edge_ref.weight().relation_type == *relation_type);
            
            if let Some(edge_ref) = edge_to_remove {
                self.graph.remove_edge(edge_ref.id());
            }
        }
        
        Ok(())
    }

    /// 设置图元数据
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata = self.metadata.update(key, value);
    }

    /// 获取图元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// 获取边数量
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// 获取根节点
    pub fn get_root(&self) -> Option<&GraphNode> {
        self.root_id.as_ref().and_then(|id| self.get_node(id))
    }

    /// 检查节点是否存在
    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.node_map.contains_key(node_id)
    }

    /// 获取所有节点
    pub fn get_all_nodes(&self) -> Vec<&GraphNode> {
        self.graph.node_weights().collect()
    }

    /// 获取所有关系
    pub fn get_all_relations(&self) -> Vec<(&GraphNode, &GraphNode, &Relation)> {
        self.graph.edge_references()
            .filter_map(|edge_ref| {
                let source = self.graph.node_weight(edge_ref.source())?;
                let target = self.graph.node_weight(edge_ref.target())?;
                Some((source, target, edge_ref.weight()))
            })
            .collect()
    }
}

impl Default for HybridGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::attrs::Attrs;

    fn create_test_node(id: &str, node_type: &str) -> Node {
        Node::new(
            id,
            node_type.to_string(),
            Attrs::default(),
            vec![],
            vec![],
        )
    }

    #[test]
    fn test_hybrid_graph_creation() {
        let mut graph = HybridGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = HybridGraph::new();
        let node = create_test_node("node1", "paragraph");
        
        let index = graph.add_node(node).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(&"node1".into()));
        assert_eq!(graph.get_root().unwrap().id(), &"node1".into());
    }

    #[test]
    fn test_add_relation() {
        let mut graph = HybridGraph::new();
        let node1 = create_test_node("node1", "document");
        let node2 = create_test_node("node2", "paragraph");
        
        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();
        
        let relation = Relation::new(RelationType::ParentChild);
        graph.add_relation(&"node1".into(), &"node2".into(), relation).unwrap();
        
        assert_eq!(graph.edge_count(), 1);
        let children = graph.get_children(&"node1".into());
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id(), &"node2".into());
    }

    #[test]
    fn test_get_parent() {
        let mut graph = HybridGraph::new();
        let node1 = create_test_node("node1", "document");
        let node2 = create_test_node("node2", "paragraph");
        
        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();
        
        let relation = Relation::new(RelationType::ParentChild);
        graph.add_relation(&"node1".into(), &"node2".into(), relation).unwrap();
        
        let parent = graph.get_parent(&"node2".into());
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().id(), &"node1".into());
    }

    #[test]
    fn test_remove_node() {
        let mut graph = HybridGraph::new();
        let node1 = create_test_node("node1", "document");
        let node2 = create_test_node("node2", "paragraph");
        
        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();
        
        let relation = Relation::new(RelationType::ParentChild);
        graph.add_relation(&"node1".into(), &"node2".into(), relation).unwrap();
        
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        
        graph.remove_node(&"node2".into()).unwrap();
        
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
        assert!(!graph.contains_node(&"node2".into()));
    }
}
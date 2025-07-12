use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use serde::{Deserialize, Serialize};
use petgraph::prelude::{StableDiGraph, NodeIndex, EdgeIndex};
use petgraph::visit::{EdgeRef, IntoNodeIdentifiers, VisitMap, Visitable};
use petgraph::{Incoming, Outgoing};
use im::HashMap as ImHashMap;
use im::Vector as ImVector;
use uuid::Uuid;

use crate::{
    node::Node,
    types::NodeId,
    error::PoolResult,
    graph::{GraphNode, Relation, RelationType},
};

/// 版本化图快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    /// 快照ID
    pub id: Uuid,
    /// 快照版本号
    pub version: u64,
    /// 快照时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// 快照描述
    pub description: Option<String>,
    /// 节点数据（序列化形式）
    pub nodes: ImHashMap<NodeId, GraphNode>,
    /// 边数据（序列化形式）
    pub edges: ImVector<(NodeId, NodeId, Relation)>,
    /// 节点索引映射
    pub node_indices: ImHashMap<NodeId, usize>,
    /// 根节点ID
    pub root_id: Option<NodeId>,
    /// 元数据
    pub metadata: ImHashMap<String, serde_json::Value>,
}

impl GraphSnapshot {
    /// 创建新的快照
    pub fn new(
        version: u64,
        description: Option<String>,
        nodes: ImHashMap<NodeId, GraphNode>,
        edges: ImVector<(NodeId, NodeId, Relation)>,
        node_indices: ImHashMap<NodeId, usize>,
        root_id: Option<NodeId>,
        metadata: ImHashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            version,
            timestamp: chrono::Utc::now(),
            description,
            nodes,
            edges,
            node_indices,
            root_id,
            metadata,
        }
    }

    /// 获取快照ID
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// 获取版本号
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 获取时间戳
    pub fn timestamp(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.timestamp
    }

    /// 获取描述
    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

/// 版本化图结构，结合 StableDiGraph 和 im 的不可变特性
#[derive(Clone)]
pub struct VersionedGraph {
    /// 当前图实例
    current_graph: StableDiGraph<GraphNode, Relation>,
    /// 节点映射
    node_map: ImHashMap<NodeId, NodeIndex>,
    /// 关系索引
    relation_index: ImHashMap<RelationType, ImVector<EdgeIndex>>,
    /// 根节点ID
    root_id: Option<NodeId>,
    /// 元数据
    metadata: ImHashMap<String, serde_json::Value>,
    /// 版本历史
    snapshots: ImVector<GraphSnapshot>,
    /// 当前版本号
    current_version: u64,
    /// 最大快照数量
    max_snapshots: usize,
}

impl Debug for VersionedGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VersionedGraph")
            .field("node_count", &self.current_graph.node_count())
            .field("edge_count", &self.current_graph.edge_count())
            .field("current_version", &self.current_version)
            .field("snapshot_count", &self.snapshots.len())
            .field("root_id", &self.root_id)
            .finish()
    }
}

impl VersionedGraph {
    /// 创建新的版本化图
    pub fn new() -> Self {
        Self {
            current_graph: StableDiGraph::new(),
            node_map: ImHashMap::new(),
            relation_index: ImHashMap::new(),
            root_id: None,
            metadata: ImHashMap::new(),
            snapshots: ImVector::new(),
            current_version: 0,
            max_snapshots: 100, // 默认最大快照数量
        }
    }

    /// 设置最大快照数量
    pub fn with_max_snapshots(mut self, max_snapshots: usize) -> Self {
        self.max_snapshots = max_snapshots;
        self
    }

    /// 创建快照
    pub fn create_snapshot(&mut self, description: Option<String>) -> PoolResult<GraphSnapshot> {
        // 收集当前图的所有数据
        let mut nodes = ImHashMap::new();
        let mut edges = ImVector::new();
        let mut node_indices = ImHashMap::new();

        // 收集节点数据
        for (node_id, &node_index) in self.node_map.iter() {
            if let Some(node) = self.current_graph.node_weight(node_index) {
                nodes = nodes.update(node_id.clone(), node.clone());
                node_indices = node_indices.update(node_id.clone(), node_index.index());
            }
        }

        // 收集边数据
        for edge_ref in self.current_graph.edge_references() {
            let source_id = self.get_node_id_by_index(edge_ref.source())?;
            let target_id = self.get_node_id_by_index(edge_ref.target())?;
            let relation = edge_ref.weight().clone();
            edges = edges.push_back((source_id, target_id, relation));
        }

        // 创建快照
        let snapshot = GraphSnapshot::new(
            self.current_version,
            description,
            nodes,
            edges,
            node_indices,
            self.root_id.clone(),
            self.metadata.clone(),
        );

        // 添加到快照历史
        self.snapshots = self.snapshots.push_back(snapshot.clone());

        // 限制快照数量
        if self.snapshots.len() > self.max_snapshots {
            self.snapshots = self.snapshots.skip(1);
        }

        Ok(snapshot)
    }

    /// 恢复到指定版本
    pub fn restore_snapshot(&mut self, version: u64) -> PoolResult<()> {
        let snapshot = self.snapshots
            .iter()
            .find(|s| s.version == version)
            .ok_or_else(|| crate::error::PoolError::NotFound(format!("Snapshot version {} not found", version)))?;

        // 清空当前图
        self.current_graph = StableDiGraph::new();
        self.node_map = ImHashMap::new();
        self.relation_index = ImHashMap::new();

        // 恢复节点
        for (node_id, graph_node) in snapshot.nodes.iter() {
            let mut restored_node = graph_node.clone();
            let index = self.current_graph.add_node(restored_node);
            restored_node.index = Some(index);
            self.current_graph[node_index] = restored_node;
            self.node_map = self.node_map.update(node_id.clone(), index);
        }

        // 恢复边
        for (source_id, target_id, relation) in snapshot.edges.iter() {
            if let (Some(&source_index), Some(&target_index)) = (
                self.node_map.get(source_id),
                self.node_map.get(target_id)
            ) {
                let edge_index = self.current_graph.add_edge(source_index, target_index, relation.clone());
                
                // 更新关系索引
                let relation_type = relation.relation_type.clone();
                let current_edges = self.relation_index.get(&relation_type).cloned().unwrap_or_else(ImVector::new);
                let updated_edges = current_edges.push_back(edge_index);
                self.relation_index = self.relation_index.update(relation_type, updated_edges);
            }
        }

        // 恢复其他状态
        self.root_id = snapshot.root_id.clone();
        self.metadata = snapshot.metadata.clone();
        self.current_version = version;

        Ok(())
    }

    /// 获取版本历史
    pub fn get_snapshots(&self) -> &ImVector<GraphSnapshot> {
        &self.snapshots
    }

    /// 获取当前版本
    pub fn current_version(&self) -> u64 {
        self.current_version
    }

    /// 添加节点（带版本控制）
    pub fn add_node(&mut self, node: Node) -> PoolResult<NodeIndex> {
        // 创建快照（如果需要）
        if self.should_create_snapshot() {
            self.create_snapshot(Some("Node addition".to_string()))?;
        }

        let node_id = node.id.clone();
        let graph_node = GraphNode::new(node);
        let index = self.current_graph.add_node(graph_node);
        
        // 更新节点映射
        self.node_map = self.node_map.update(node_id.clone(), index);
        
        // 如果是第一个节点，设为根节点
        if self.root_id.is_none() {
            self.root_id = Some(node_id);
        }

        self.current_version += 1;
        Ok(index)
    }

    /// 添加关系（带版本控制）
    pub fn add_relation(
        &mut self,
        source_id: &NodeId,
        target_id: &NodeId,
        relation: Relation,
    ) -> PoolResult<EdgeIndex> {
        // 创建快照（如果需要）
        if self.should_create_snapshot() {
            self.create_snapshot(Some("Relation addition".to_string()))?;
        }

        let source_index = self.node_map.get(source_id)
            .ok_or_else(|| crate::error::PoolError::NotFound(format!("Source node {} not found", source_id)))?;
        let target_index = self.node_map.get(target_id)
            .ok_or_else(|| crate::error::PoolError::NotFound(format!("Target node {} not found", target_id)))?;

        let edge_index = self.current_graph.add_edge(*source_index, *target_index, relation.clone());
        
        // 更新关系索引
        let relation_type = relation.relation_type.clone();
        let current_edges = self.relation_index.get(&relation_type).cloned().unwrap_or_else(ImVector::new);
        let updated_edges = current_edges.push_back(edge_index);
        self.relation_index = self.relation_index.update(relation_type, updated_edges);

        self.current_version += 1;
        Ok(edge_index)
    }

    /// 移除节点（带版本控制）
    pub fn remove_node(&mut self, node_id: &NodeId) -> PoolResult<()> {
        // 创建快照
        self.create_snapshot(Some("Node removal".to_string()))?;

        let node_index = self.node_map.get(node_id)
            .ok_or_else(|| crate::error::PoolError::NotFound(format!("Node {} not found", node_id)))?;

        // 移除所有相关边
        let edges_to_remove: Vec<EdgeIndex> = self.current_graph
            .edges(*node_index)
            .map(|edge_ref| edge_ref.id())
            .collect();

        for edge_index in edges_to_remove {
            self.current_graph.remove_edge(edge_index);
        }

        // 移除节点
        self.current_graph.remove_node(*node_index);
        self.node_map = self.node_map.without(node_id);

        // 更新根节点
        if self.root_id.as_ref() == Some(node_id) {
            self.root_id = None;
        }

        self.current_version += 1;
        Ok(())
    }

    /// 获取节点
    pub fn get_node(&self, node_id: &NodeId) -> Option<&GraphNode> {
        self.node_map.get(node_id)
            .and_then(|&index| self.current_graph.node_weight(index))
    }

    /// 获取节点索引
    pub fn get_node_index(&self, node_id: &NodeId) -> Option<NodeIndex> {
        self.node_map.get(node_id).copied()
    }

    /// 获取邻居节点
    pub fn get_neighbors(&self, node_id: &NodeId) -> Vec<&GraphNode> {
        self.node_map.get(node_id)
            .and_then(|&index| {
                let neighbors: Vec<&GraphNode> = self.current_graph
                    .neighbors(index)
                    .filter_map(|neighbor_index| self.current_graph.node_weight(neighbor_index))
                    .collect();
                Some(neighbors)
            })
            .unwrap_or_default()
    }

    /// 获取关系
    pub fn get_relations(&self, node_id: &NodeId) -> Vec<(&GraphNode, &Relation)> {
        self.node_map.get(node_id)
            .and_then(|&index| {
                let relations: Vec<(&GraphNode, &Relation)> = self.current_graph
                    .edges(index)
                    .filter_map(|edge_ref| {
                        let target_index = edge_ref.target();
                        let target_node = self.current_graph.node_weight(target_index)?;
                        Some((target_node, edge_ref.weight()))
                    })
                    .collect();
                Some(relations)
            })
            .unwrap_or_default()
    }

    /// 获取子节点
    pub fn get_children(&self, node_id: &NodeId) -> Vec<&GraphNode> {
        self.get_relations_by_type(node_id, &RelationType::ParentChild)
            .into_iter()
            .map(|(node, _)| node)
            .collect()
    }

    /// 获取父节点
    pub fn get_parent(&self, node_id: &NodeId) -> Option<&GraphNode> {
        self.node_map.get(node_id)
            .and_then(|&index| {
                self.current_graph
                    .edges_directed(index, Incoming)
                    .find(|edge_ref| edge_ref.weight().relation_type == RelationType::ParentChild)
                    .and_then(|edge_ref| {
                        let parent_index = edge_ref.source();
                        self.current_graph.node_weight(parent_index)
                    })
            })
    }

    /// 按类型获取关系
    pub fn get_relations_by_type(
        &self,
        node_id: &NodeId,
        relation_type: &RelationType,
    ) -> Vec<(&GraphNode, &Relation)> {
        self.node_map.get(node_id)
            .and_then(|&index| {
                let relations: Vec<(&GraphNode, &Relation)> = self.current_graph
                    .edges(index)
                    .filter(|edge_ref| edge_ref.weight().relation_type == *relation_type)
                    .filter_map(|edge_ref| {
                        let target_index = edge_ref.target();
                        let target_node = self.current_graph.node_weight(target_index)?;
                        Some((target_node, edge_ref.weight()))
                    })
                    .collect();
                Some(relations)
            })
            .unwrap_or_default()
    }

    /// 检查是否有循环
    pub fn has_cycles(&self) -> bool {
        use petgraph::algo::is_cyclic_directed;
        is_cyclic_directed(&self.current_graph)
    }

    /// 拓扑排序
    pub fn topological_sort(&self) -> Result<Vec<&GraphNode>, String> {
        use petgraph::algo::toposort;
        let node_indices: Vec<NodeIndex> = toposort(&self.current_graph, None)
            .map_err(|_| "Graph contains cycles".to_string())?;
        
        let sorted_nodes: Vec<&GraphNode> = node_indices
            .into_iter()
            .filter_map(|index| self.current_graph.node_weight(index))
            .collect();
        
        Ok(sorted_nodes)
    }

    /// 最短路径
    pub fn shortest_path(
        &self,
        source_id: &NodeId,
        target_id: &NodeId,
    ) -> Option<Vec<&GraphNode>> {
        use petgraph::algo::dijkstra;
        
        let source_index = self.node_map.get(source_id)?;
        let target_index = self.node_map.get(target_id)?;
        
        let path = dijkstra(&self.current_graph, *source_index, Some(*target_index), |edge| {
            edge.weight().weight as u32
        });
        
        path.1.get(target_index).map(|path_indices| {
            path_indices
                .iter()
                .filter_map(|&index| self.current_graph.node_weight(index))
                .collect()
        })
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata = self.metadata.update(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.current_graph.node_count()
    }

    /// 获取边数量
    pub fn edge_count(&self) -> usize {
        self.current_graph.edge_count()
    }

    /// 获取根节点
    pub fn get_root(&self) -> Option<&GraphNode> {
        self.root_id.as_ref()
            .and_then(|id| self.get_node(id))
    }

    /// 检查是否包含节点
    pub fn contains_node(&self, node_id: &NodeId) -> bool {
        self.node_map.contains_key(node_id)
    }

    /// 获取所有节点
    pub fn get_all_nodes(&self) -> Vec<&GraphNode> {
        self.current_graph
            .node_weights()
            .collect()
    }

    /// 获取所有关系
    pub fn get_all_relations(&self) -> Vec<(&GraphNode, &GraphNode, &Relation)> {
        self.current_graph
            .edge_references()
            .filter_map(|edge_ref| {
                let source_node = self.current_graph.node_weight(edge_ref.source())?;
                let target_node = self.current_graph.node_weight(edge_ref.target())?;
                Some((source_node, target_node, edge_ref.weight()))
            })
            .collect()
    }

    /// 根据索引获取节点ID
    fn get_node_id_by_index(&self, index: NodeIndex) -> PoolResult<NodeId> {
        self.node_map
            .iter()
            .find(|(_, &node_index)| node_index == index)
            .map(|(node_id, _)| node_id.clone())
            .ok_or_else(|| crate::error::PoolError::NotFound("Node index not found".to_string()))
    }

    /// 判断是否应该创建快照
    fn should_create_snapshot(&self) -> bool {
        // 可以根据策略决定何时创建快照
        // 例如：每N个操作创建一次，或者根据时间间隔
        self.current_version % 10 == 0 // 每10个操作创建一次快照
    }
}

impl Default for VersionedGraph {
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
            NodeId::from(id),
            node_type.to_string(),
            Attrs::default(),
            vec![],
            vec![],
        )
    }

    #[test]
    fn test_versioned_graph_creation() {
        let graph = VersionedGraph::new();
        assert_eq!(graph.current_version(), 0);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_node_with_versioning() {
        let mut graph = VersionedGraph::new();
        let node = create_test_node("test1", "paragraph");
        
        let index = graph.add_node(node).unwrap();
        assert_eq!(graph.current_version(), 1);
        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(&NodeId::from("test1")));
    }

    #[test]
    fn test_snapshot_creation() {
        let mut graph = VersionedGraph::new();
        let node = create_test_node("test1", "paragraph");
        
        graph.add_node(node).unwrap();
        let snapshot = graph.create_snapshot(Some("Test snapshot".to_string())).unwrap();
        
        assert_eq!(snapshot.version(), 1);
        assert_eq!(graph.get_snapshots().len(), 1);
    }

    #[test]
    fn test_restore_snapshot() {
        let mut graph = VersionedGraph::new();
        let node1 = create_test_node("test1", "paragraph");
        let node2 = create_test_node("test2", "paragraph");
        
        graph.add_node(node1).unwrap();
        let snapshot = graph.create_snapshot(Some("First snapshot".to_string())).unwrap();
        graph.add_node(node2).unwrap();
        
        // 恢复到第一个快照
        graph.restore_snapshot(snapshot.version()).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(&NodeId::from("test1")));
        assert!(!graph.contains_node(&NodeId::from("test2")));
    }

    #[test]
    fn test_add_relation_with_versioning() {
        let mut graph = VersionedGraph::new();
        let node1 = create_test_node("test1", "paragraph");
        let node2 = create_test_node("test2", "paragraph");
        
        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();
        
        let relation = Relation::new(RelationType::ParentChild);
        graph.add_relation(&NodeId::from("test1"), &NodeId::from("test2"), relation).unwrap();
        
        assert_eq!(graph.current_version(), 3); // 2个节点 + 1个关系
        assert_eq!(graph.edge_count(), 1);
        
        let children = graph.get_children(&NodeId::from("test1"));
        assert_eq!(children.len(), 1);
    }
}
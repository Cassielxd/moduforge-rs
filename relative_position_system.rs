use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{NodeId, Tree, Node};
use crate::collaboration::types::{YrsOperation, YrsOperationType, ConflictError};

/// 相对位置系统 - y-prosemirror 风格的位置处理
/// 
/// 相对位置允许操作在文档结构发生变化时仍能找到正确的目标位置
/// 这是协作编辑中处理并发操作的关键机制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelativePosition {
    /// 相对于的锚点节点ID
    pub anchor: NodeId,
    /// 相对位置类型
    pub position_type: RelativePositionType,
    /// 在锚点基础上的偏移量
    pub offset: i32,
    /// 附加的路径信息，用于复杂导航
    pub path_hint: Vec<PathSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelativePositionType {
    /// 在锚点节点之前
    Before,
    /// 在锚点节点之后
    After,
    /// 作为锚点节点的第N个子节点
    ChildAt(u32),
    /// 在锚点节点内容的指定偏移位置
    WithinAt(u32),
    /// 替换锚点节点
    Replace,
    /// 在锚点的父节点中，相对于锚点的位置
    SiblingBefore,
    /// 在锚点的父节点中，相对于锚点的位置  
    SiblingAfter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    /// 段类型
    pub segment_type: PathSegmentType,
    /// 段的标识符
    pub identifier: String,
    /// 段内的偏移
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathSegmentType {
    /// 节点ID段
    NodeId,
    /// 节点类型段
    NodeType,
    /// 属性名段
    AttributeName,
    /// 数组索引段
    ArrayIndex,
}

/// 绝对位置 - 传统的基于索引的位置表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsolutePosition {
    /// 目标节点ID
    pub node_id: NodeId,
    /// 在父节点中的索引
    pub parent_index: Option<u32>,
    /// 节点内的偏移位置
    pub content_offset: Option<u32>,
    /// 完整的路径（从根到目标节点）
    pub full_path: Vec<NodeId>,
}

/// 位置映射器 - 处理操作序列中的位置转换
pub struct PositionMapper {
    /// 操作历史缓存
    operation_cache: VecDeque<YrsOperation>,
    /// 位置转换缓存
    position_cache: HashMap<String, RelativePosition>,
    /// 缓存大小限制
    cache_limit: usize,
    /// 稳定锚点缓存（不容易被删除的节点）
    stable_anchors: HashMap<NodeId, AnchorInfo>,
}

#[derive(Debug, Clone)]
struct AnchorInfo {
    node_id: NodeId,
    stability_score: f32, // 0.0 - 1.0，越高越稳定
    last_accessed: std::time::Instant,
    reference_count: u32,
}

impl PositionMapper {
    pub fn new() -> Self {
        Self {
            operation_cache: VecDeque::new(),
            position_cache: HashMap::new(),
            cache_limit: 1000,
            stable_anchors: HashMap::new(),
        }
    }
    
    /// 将绝对位置转换为相对位置
    /// 这是 y-prosemirror 中的核心算法，确保位置的持久性
    pub fn absolute_to_relative(
        &mut self,
        absolute_pos: AbsolutePosition,
        tree: &Tree,
    ) -> Result<RelativePosition, PositionError> {
        // 1. 寻找最佳锚点
        let anchor = self.find_best_anchor(&absolute_pos, tree)?;
        
        // 2. 计算相对位置类型
        let position_type = self.calculate_position_type(&absolute_pos, &anchor, tree)?;
        
        // 3. 计算偏移量
        let offset = self.calculate_offset(&absolute_pos, &anchor, tree)?;
        
        // 4. 生成路径提示
        let path_hint = self.generate_path_hint(&absolute_pos, &anchor, tree)?;
        
        let relative_pos = RelativePosition {
            anchor: anchor.id.clone(),
            position_type,
            offset,
            path_hint,
        };
        
        // 5. 缓存结果
        let cache_key = format!("{:?}", absolute_pos);
        self.position_cache.insert(cache_key, relative_pos.clone());
        
        // 6. 更新锚点信息
        self.update_anchor_info(&anchor);
        
        Ok(relative_pos)
    }
    
    /// 将相对位置转换为当前的绝对位置
    /// 考虑文档结构的变化，重新计算绝对位置
    pub fn relative_to_absolute(
        &self,
        relative_pos: &RelativePosition,
        tree: &Tree,
    ) -> Result<AbsolutePosition, PositionError> {
        // 1. 找到锚点节点
        let anchor_node = tree.get_node(&relative_pos.anchor)
            .ok_or(PositionError::AnchorNotFound(relative_pos.anchor.clone()))?;
            
        // 2. 根据位置类型计算绝对位置
        match relative_pos.position_type {
            RelativePositionType::Before => {
                self.calculate_before_position(anchor_node, tree)
            }
            RelativePositionType::After => {
                self.calculate_after_position(anchor_node, tree)
            }
            RelativePositionType::ChildAt(index) => {
                self.calculate_child_position(anchor_node, index, relative_pos.offset, tree)
            }
            RelativePositionType::WithinAt(offset) => {
                self.calculate_within_position(anchor_node, offset, tree)
            }
            RelativePositionType::Replace => {
                self.calculate_replace_position(anchor_node, tree)
            }
            RelativePositionType::SiblingBefore => {
                self.calculate_sibling_before_position(anchor_node, relative_pos.offset, tree)
            }
            RelativePositionType::SiblingAfter => {
                self.calculate_sibling_after_position(anchor_node, relative_pos.offset, tree)
            }
        }
    }
    
    /// 通过操作序列映射位置
    /// 这是处理并发操作的关键方法
    pub fn map_position_through_operations(
        &mut self,
        position: RelativePosition,
        operations: &[YrsOperation],
        tree: &Tree,
    ) -> Result<RelativePosition, PositionError> {
        let mut current_position = position;
        
        for operation in operations {
            current_position = self.map_position_through_single_operation(
                current_position, 
                operation,
                tree
            )?;
        }
        
        // 缓存操作序列
        self.cache_operations(operations);
        
        Ok(current_position)
    }
    
    /// 通过单个操作映射位置
    fn map_position_through_single_operation(
        &self,
        position: RelativePosition,
        operation: &YrsOperation,
        tree: &Tree,
    ) -> Result<RelativePosition, PositionError> {
        match &operation.operation_type {
            YrsOperationType::ArrayInsert { index, values } => {
                self.map_through_insert(position, *index, values.len() as u32, operation, tree)
            }
            YrsOperationType::ArrayDelete { index, length } => {
                self.map_through_delete(position, *index, *length, operation, tree)
            }
            YrsOperationType::MapSet { key, .. } => {
                self.map_through_set(position, key, operation, tree)
            }
            YrsOperationType::MapDelete { key } => {
                self.map_through_map_delete(position, key, operation, tree)
            }
            YrsOperationType::TextInsert { index, text } => {
                self.map_through_text_insert(position, *index, text.len() as u32, operation, tree)
            }
            YrsOperationType::TextDelete { index, length } => {
                self.map_through_text_delete(position, *index, *length, operation, tree)
            }
            YrsOperationType::Custom { .. } => {
                // 自定义操作需要特殊处理
                self.map_through_custom_operation(position, operation, tree)
            }
        }
    }
    
    /// 寻找最佳锚点 - y-prosemirror 的关键算法
    fn find_best_anchor(
        &mut self,
        absolute_pos: &AbsolutePosition,
        tree: &Tree,
    ) -> Result<&Node, PositionError> {
        // 策略1: 如果目标节点本身很稳定，使用它作为锚点
        if let Some(target_node) = tree.get_node(&absolute_pos.node_id) {
            if self.is_stable_node(target_node, tree) {
                return Ok(target_node);
            }
        }
        
        // 策略2: 寻找路径中的稳定节点
        for node_id in absolute_pos.full_path.iter().rev() {
            if let Some(node) = tree.get_node(node_id) {
                if self.is_stable_node(node, tree) {
                    return Ok(node);
                }
            }
        }
        
        // 策略3: 使用父节点
        if let Some(parent_id) = absolute_pos.full_path.get(absolute_pos.full_path.len().saturating_sub(2)) {
            if let Some(parent_node) = tree.get_node(parent_id) {
                return Ok(parent_node);
            }
        }
        
        // 策略4: 使用根节点作为最后选择
        tree.get_root().ok_or(PositionError::NoValidAnchor)
    }
    
    /// 判断节点是否稳定（不容易被删除）
    fn is_stable_node(&self, node: &Node, tree: &Tree) -> bool {
        // 稳定性评估标准：
        // 1. 是否为根节点或接近根节点
        let depth = tree.get_node_depth(&node.id).unwrap_or(0);
        let depth_score = if depth <= 2 { 1.0 } else { 1.0 / (depth as f32) };
        
        // 2. 子节点数量（子节点多的节点通常更稳定）
        let children_count = tree.get_children(&node.id).map(|children| children.len()).unwrap_or(0);
        let children_score = (children_count as f32).min(10.0) / 10.0;
        
        // 3. 节点类型（某些类型的节点更稳定）
        let type_score = match node.node_type.as_str() {
            "document" | "section" | "chapter" => 1.0,
            "paragraph" | "heading" => 0.8,
            "text" | "inline" => 0.3,
            _ => 0.5,
        };
        
        // 4. 历史访问频率
        let access_score = self.stable_anchors.get(&node.id)
            .map(|info| info.reference_count as f32 / 100.0)
            .unwrap_or(0.0)
            .min(1.0);
        
        let total_score = (depth_score + children_score + type_score + access_score) / 4.0;
        total_score > 0.6 // 阈值可以调整
    }
    
    /// 通过插入操作映射位置
    fn map_through_insert(
        &self,
        mut position: RelativePosition,
        insert_index: u32,
        insert_count: u32,
        operation: &YrsOperation,
        tree: &Tree,
    ) -> Result<RelativePosition, PositionError> {
        // 检查插入是否影响当前位置
        if self.operation_affects_position(&position, operation, tree)? {
            match position.position_type {
                RelativePositionType::ChildAt(ref mut index) => {
                    if *index >= insert_index {
                        *index += insert_count;
                    }
                }
                RelativePositionType::WithinAt(ref mut offset) => {
                    if *offset >= insert_index {
                        *offset += insert_count;
                    }
                }
                RelativePositionType::SiblingAfter => {
                    // 如果在兄弟节点之后插入，可能需要调整偏移
                    if position.offset >= 0 && insert_index <= position.offset as u32 {
                        position.offset += insert_count as i32;
                    }
                }
                _ => {
                    // 其他位置类型可能不受影响，或需要特殊处理
                }
            }
        }
        
        Ok(position)
    }
    
    /// 通过删除操作映射位置
    fn map_through_delete(
        &self,
        mut position: RelativePosition,
        delete_index: u32,
        delete_length: u32,
        operation: &YrsOperation,
        tree: &Tree,
    ) -> Result<RelativePosition, PositionError> {
        if self.operation_affects_position(&position, operation, tree)? {
            let delete_end = delete_index + delete_length;
            
            match position.position_type {
                RelativePositionType::ChildAt(ref mut index) => {
                    if *index >= delete_end {
                        *index -= delete_length;
                    } else if *index >= delete_index {
                        // 位置在删除范围内，需要找到新的锚点
                        return self.find_alternative_position_after_delete(position, operation, tree);
                    }
                }
                RelativePositionType::WithinAt(ref mut offset) => {
                    if *offset >= delete_end {
                        *offset -= delete_length;
                    } else if *offset >= delete_index {
                        // 内容被删除，移到删除位置的开始
                        *offset = delete_index;
                    }
                }
                _ => {
                    // 其他类型的处理
                }
            }
        }
        
        Ok(position)
    }
    
    /// 在删除后寻找替代位置
    fn find_alternative_position_after_delete(
        &self,
        original_position: RelativePosition,
        _operation: &YrsOperation,
        tree: &Tree,
    ) -> Result<RelativePosition, PositionError> {
        // 尝试使用路径提示寻找替代锚点
        for hint in &original_position.path_hint {
            if let PathSegmentType::NodeId = hint.segment_type {
                if let Some(node) = tree.get_node(&NodeId::from(hint.identifier.clone())) {
                    if self.is_stable_node(node, tree) {
                        return Ok(RelativePosition {
                            anchor: node.id.clone(),
                            position_type: RelativePositionType::Before,
                            offset: 0,
                            path_hint: vec![],
                        });
                    }
                }
            }
        }
        
        // 如果找不到替代位置，使用原锚点的父节点
        if let Some(anchor_node) = tree.get_node(&original_position.anchor) {
            if let Some(parent_id) = tree.get_parent(&anchor_node.id) {
                return Ok(RelativePosition {
                    anchor: parent_id,
                    position_type: RelativePositionType::ChildAt(0),
                    offset: 0,
                    path_hint: vec![],
                });
            }
        }
        
        Err(PositionError::CannotFindAlternativePosition)
    }
    
    /// 检查操作是否影响位置
    fn operation_affects_position(
        &self,
        position: &RelativePosition,
        operation: &YrsOperation,
        tree: &Tree,
    ) -> Result<bool, PositionError> {
        // 检查操作路径是否与位置的锚点相关
        let anchor_path = tree.get_node_path(&position.anchor)
            .ok_or(PositionError::AnchorNotFound(position.anchor.clone()))?;
            
        // 将 Yrs 路径转换为节点路径进行比较
        let operation_path = self.yrs_path_to_node_path(&operation.target_path, tree)?;
        
        // 检查路径是否重叠或相关
        Ok(self.paths_intersect(&anchor_path, &operation_path))
    }
    
    /// 检查两个路径是否相交
    fn paths_intersect(&self, path1: &[NodeId], path2: &[NodeId]) -> bool {
        // 简单的路径相交检查
        for id1 in path1 {
            for id2 in path2 {
                if id1 == id2 {
                    return true;
                }
            }
        }
        false
    }
    
    /// 将 Yrs 路径转换为节点路径
    fn yrs_path_to_node_path(
        &self,
        yrs_path: &[String],
        tree: &Tree,
    ) -> Result<Vec<NodeId>, PositionError> {
        // 这里需要根据具体的 Yrs 到节点的映射逻辑来实现
        // 暂时返回空路径
        Ok(vec![])
    }
    
    /// 生成路径提示
    fn generate_path_hint(
        &self,
        absolute_pos: &AbsolutePosition,
        anchor: &Node,
        tree: &Tree,
    ) -> Result<Vec<PathSegment>, PositionError> {
        let mut hints = Vec::new();
        
        // 添加锚点到目标的路径信息
        for node_id in &absolute_pos.full_path {
            if let Some(node) = tree.get_node(node_id) {
                hints.push(PathSegment {
                    segment_type: PathSegmentType::NodeId,
                    identifier: node.id.to_string(),
                    offset: None,
                });
                
                hints.push(PathSegment {
                    segment_type: PathSegmentType::NodeType,
                    identifier: node.node_type.clone(),
                    offset: None,
                });
            }
        }
        
        Ok(hints)
    }
    
    /// 更新锚点信息
    fn update_anchor_info(&mut self, anchor: &Node) {
        let info = self.stable_anchors.entry(anchor.id.clone()).or_insert_with(|| {
            AnchorInfo {
                node_id: anchor.id.clone(),
                stability_score: 0.5,
                last_accessed: std::time::Instant::now(),
                reference_count: 0,
            }
        });
        
        info.reference_count += 1;
        info.last_accessed = std::time::Instant::now();
        
        // 动态调整稳定性分数
        if info.reference_count > 10 {
            info.stability_score = (info.stability_score + 0.1).min(1.0);
        }
    }
    
    /// 缓存操作序列
    fn cache_operations(&mut self, operations: &[YrsOperation]) {
        for op in operations {
            self.operation_cache.push_back(op.clone());
            
            // 限制缓存大小
            if self.operation_cache.len() > self.cache_limit {
                self.operation_cache.pop_front();
            }
        }
    }
    
    /// 计算各种位置类型的绝对位置
    fn calculate_before_position(
        &self,
        anchor_node: &Node,
        tree: &Tree,
    ) -> Result<AbsolutePosition, PositionError> {
        let parent_id = tree.get_parent(&anchor_node.id)
            .ok_or(PositionError::NoParent(anchor_node.id.clone()))?;
            
        let siblings = tree.get_children(&parent_id)
            .ok_or(PositionError::NoSiblings)?;
            
        let index = siblings.iter().position(|id| id == &anchor_node.id)
            .ok_or(PositionError::NodeNotFoundInParent)?;
            
        let path = tree.get_node_path(&anchor_node.id)
            .ok_or(PositionError::CannotCalculatePath)?;
            
        Ok(AbsolutePosition {
            node_id: parent_id,
            parent_index: Some(index as u32),
            content_offset: None,
            full_path: path,
        })
    }
    
    fn calculate_after_position(
        &self,
        anchor_node: &Node,
        tree: &Tree,
    ) -> Result<AbsolutePosition, PositionError> {
        let parent_id = tree.get_parent(&anchor_node.id)
            .ok_or(PositionError::NoParent(anchor_node.id.clone()))?;
            
        let siblings = tree.get_children(&parent_id)
            .ok_or(PositionError::NoSiblings)?;
            
        let index = siblings.iter().position(|id| id == &anchor_node.id)
            .ok_or(PositionError::NodeNotFoundInParent)?;
            
        let path = tree.get_node_path(&anchor_node.id)
            .ok_or(PositionError::CannotCalculatePath)?;
            
        Ok(AbsolutePosition {
            node_id: parent_id,
            parent_index: Some((index + 1) as u32),
            content_offset: None,
            full_path: path,
        })
    }
    
    fn calculate_child_position(
        &self,
        anchor_node: &Node,
        index: u32,
        offset: i32,
        tree: &Tree,
    ) -> Result<AbsolutePosition, PositionError> {
        let children = tree.get_children(&anchor_node.id)
            .ok_or(PositionError::NoChildren)?;
            
        let adjusted_index = if offset >= 0 {
            (index as i32 + offset) as u32
        } else {
            index.saturating_sub((-offset) as u32)
        };
        
        let target_index = adjusted_index.min(children.len() as u32);
        
        let path = tree.get_node_path(&anchor_node.id)
            .ok_or(PositionError::CannotCalculatePath)?;
            
        Ok(AbsolutePosition {
            node_id: anchor_node.id.clone(),
            parent_index: Some(target_index),
            content_offset: None,
            full_path: path,
        })
    }
    
    fn calculate_within_position(
        &self,
        anchor_node: &Node,
        offset: u32,
        tree: &Tree,
    ) -> Result<AbsolutePosition, PositionError> {
        let path = tree.get_node_path(&anchor_node.id)
            .ok_or(PositionError::CannotCalculatePath)?;
            
        Ok(AbsolutePosition {
            node_id: anchor_node.id.clone(),
            parent_index: None,
            content_offset: Some(offset),
            full_path: path,
        })
    }
    
    // 其他计算方法的占位符实现
    fn calculate_replace_position(&self, anchor_node: &Node, tree: &Tree) -> Result<AbsolutePosition, PositionError> {
        self.calculate_before_position(anchor_node, tree)
    }
    
    fn calculate_sibling_before_position(&self, anchor_node: &Node, offset: i32, tree: &Tree) -> Result<AbsolutePosition, PositionError> {
        self.calculate_before_position(anchor_node, tree)
    }
    
    fn calculate_sibling_after_position(&self, anchor_node: &Node, offset: i32, tree: &Tree) -> Result<AbsolutePosition, PositionError> {
        self.calculate_after_position(anchor_node, tree)
    }
    
    // 其他映射方法的占位符实现
    fn map_through_set(&self, position: RelativePosition, key: &str, operation: &YrsOperation, tree: &Tree) -> Result<RelativePosition, PositionError> {
        Ok(position)
    }
    
    fn map_through_map_delete(&self, position: RelativePosition, key: &str, operation: &YrsOperation, tree: &Tree) -> Result<RelativePosition, PositionError> {
        Ok(position)
    }
    
    fn map_through_text_insert(&self, position: RelativePosition, index: u32, length: u32, operation: &YrsOperation, tree: &Tree) -> Result<RelativePosition, PositionError> {
        self.map_through_insert(position, index, length, operation, tree)
    }
    
    fn map_through_text_delete(&self, position: RelativePosition, index: u32, length: u32, operation: &YrsOperation, tree: &Tree) -> Result<RelativePosition, PositionError> {
        self.map_through_delete(position, index, length, operation, tree)
    }
    
    fn map_through_custom_operation(&self, position: RelativePosition, operation: &YrsOperation, tree: &Tree) -> Result<RelativePosition, PositionError> {
        Ok(position)
    }
    
    fn calculate_position_type(&self, absolute_pos: &AbsolutePosition, anchor: &Node, tree: &Tree) -> Result<RelativePositionType, PositionError> {
        Ok(RelativePositionType::Before)
    }
    
    fn calculate_offset(&self, absolute_pos: &AbsolutePosition, anchor: &Node, tree: &Tree) -> Result<i32, PositionError> {
        Ok(0)
    }
}

/// 位置映射错误
#[derive(Debug, thiserror::Error)]
pub enum PositionError {
    #[error("Anchor node not found: {0}")]
    AnchorNotFound(NodeId),
    
    #[error("No valid anchor found")]
    NoValidAnchor,
    
    #[error("Node has no parent: {0}")]
    NoParent(NodeId),
    
    #[error("Node has no children")]
    NoChildren,
    
    #[error("Node has no siblings")]
    NoSiblings,
    
    #[error("Node not found in parent")]
    NodeNotFoundInParent,
    
    #[error("Cannot calculate node path")]
    CannotCalculatePath,
    
    #[error("Cannot find alternative position")]
    CannotFindAlternativePosition,
    
    #[error("Invalid position mapping")]
    InvalidMapping,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Node, Attrs};
    
    #[test]
    fn test_relative_position_creation() {
        let rel_pos = RelativePosition {
            anchor: NodeId::from("anchor_1"),
            position_type: RelativePositionType::ChildAt(2),
            offset: 1,
            path_hint: vec![PathSegment {
                segment_type: PathSegmentType::NodeId,
                identifier: "parent_1".to_string(),
                offset: None,
            }],
        };
        
        assert_eq!(rel_pos.anchor, NodeId::from("anchor_1"));
        assert!(matches!(rel_pos.position_type, RelativePositionType::ChildAt(2)));
        assert_eq!(rel_pos.offset, 1);
        assert_eq!(rel_pos.path_hint.len(), 1);
    }
    
    #[test]
    fn test_position_mapper_creation() {
        let mapper = PositionMapper::new();
        assert_eq!(mapper.operation_cache.len(), 0);
        assert_eq!(mapper.position_cache.len(), 0);
        assert_eq!(mapper.cache_limit, 1000);
    }
}
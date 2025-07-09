use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yrs::{Doc, TransactionMut};
use chrono::{DateTime, Utc};

use crate::model::{NodeId, Tree};
use crate::collaboration::types::{YrsOperation, YrsOperationType, ConflictError};
use crate::collaboration::relative_position::{RelativePosition, PositionMapper, PositionError};
use crate::collaboration::conflict_resolver::{ModuForgeConflictResolver, ConflictContext, ConflictType};

/// 协作环境下的撤销/重做管理器
/// 
/// 基于 y-prosemirror 的设计，支持在多用户协作环境中的撤销/重做操作
/// 关键特性：
/// - 只撤销用户自己的操作
/// - 自动处理远程操作对本地撤销栈的影响
/// - 使用相对位置确保撤销操作的准确性
/// - 支持撤销操作的冲突解决
pub struct CollaborativeUndoManager {
    /// 用户标识
    user_id: String,
    /// 本地撤销栈 - 只包含本用户的操作
    undo_stack: VecDeque<UndoItem>,
    /// 本地重做栈
    redo_stack: VecDeque<UndoItem>,
    /// 撤销栈最大大小
    max_stack_size: usize,
    /// Yrs 文档引用
    yrs_doc: Arc<yrs::Doc>,
    /// 位置映射器
    position_mapper: PositionMapper,
    /// 冲突解决器
    conflict_resolver: Arc<ModuForgeConflictResolver>,
    /// 撤销统计信息
    undo_stats: UndoStatistics,
    /// 撤销项ID映射（用于快速查找）
    item_id_map: HashMap<Uuid, usize>,
}

/// 撤销项 - 表示一个可撤销的操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoItem {
    /// 唯一标识符
    pub id: Uuid,
    /// 原始操作
    pub original_operation: YrsOperation,
    /// 逆操作（用于撤销）
    pub inverse_operation: YrsOperation,
    /// 操作时的相对位置信息
    pub relative_positions: Vec<RelativePosition>,
    /// 操作时间戳
    pub timestamp: u64,
    /// 操作时的文档版本（Yrs 状态向量）
    pub document_version: Vec<u8>,
    /// 撤销的复杂度评分（影响撤销的难度）
    pub complexity_score: f32,
    /// 是否已经被远程操作影响
    pub affected_by_remote: bool,
    /// 依赖的其他操作ID（用于操作链）
    pub dependencies: Vec<Uuid>,
}

/// 撤销结果
#[derive(Debug, Clone)]
pub struct UndoResult {
    /// 被撤销的原始操作
    pub undone_operation: YrsOperation,
    /// 实际应用的逆操作（可能经过位置调整）
    pub applied_inverse: YrsOperation,
    /// 受影响的位置信息
    pub affected_positions: Vec<RelativePosition>,
    /// 撤销的置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 是否需要用户确认
    pub requires_confirmation: bool,
    /// 警告信息
    pub warnings: Vec<String>,
}

/// 重做结果
#[derive(Debug, Clone)]
pub struct RedoResult {
    /// 重新应用的操作
    pub reapplied_operation: YrsOperation,
    /// 受影响的位置信息
    pub affected_positions: Vec<RelativePosition>,
    /// 重做的置信度
    pub confidence: f32,
}

/// 撤销统计信息
#[derive(Debug, Default)]
pub struct UndoStatistics {
    /// 总撤销次数
    pub total_undos: u64,
    /// 总重做次数
    pub total_redos: u64,
    /// 撤销失败次数
    pub failed_undos: u64,
    /// 平均撤销延迟
    pub average_undo_latency: std::time::Duration,
    /// 需要位置调整的撤销次数
    pub position_adjusted_undos: u64,
}

impl CollaborativeUndoManager {
    pub fn new(
        user_id: String,
        yrs_doc: Arc<yrs::Doc>,
        conflict_resolver: Arc<ModuForgeConflictResolver>,
    ) -> Self {
        Self {
            user_id,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_stack_size: 100, // 可配置
            yrs_doc,
            position_mapper: PositionMapper::new(),
            conflict_resolver,
            undo_stats: UndoStatistics::default(),
            item_id_map: HashMap::new(),
        }
    }
    
    /// 添加可撤销操作 - y-prosemirror 风格
    /// 
    /// 只记录当前用户的操作，并生成相应的逆操作
    pub fn add_undoable_operation(
        &mut self,
        operation: YrsOperation,
        tree_before: &Tree,
        tree_after: &Tree,
    ) -> Result<(), UndoError> {
        // 只记录本用户的操作
        if operation.user_id != self.user_id {
            return Ok(()); // 不是本用户操作，跳过
        }
        
        // 生成逆操作
        let inverse_operation = self.generate_inverse_operation(&operation, tree_before)?;
        
        // 提取相对位置信息
        let relative_positions = self.extract_relative_positions(&operation, tree_before)?;
        
        // 计算复杂度评分
        let complexity_score = self.calculate_complexity_score(&operation, tree_before);
        
        // 获取当前文档版本
        let document_version = self.yrs_doc.transact().state_vector().encode_v1();
        
        let undo_item = UndoItem {
            id: Uuid::new_v4(),
            original_operation: operation,
            inverse_operation,
            relative_positions,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            document_version,
            complexity_score,
            affected_by_remote: false,
            dependencies: vec![],
        };
        
        // 添加到撤销栈
        self.undo_stack.push_back(undo_item.clone());
        self.item_id_map.insert(undo_item.id, self.undo_stack.len() - 1);
        
        // 清空重做栈（新操作使重做无效）
        self.redo_stack.clear();
        
        // 限制栈大小
        if self.undo_stack.len() > self.max_stack_size {
            if let Some(removed) = self.undo_stack.pop_front() {
                self.item_id_map.remove(&removed.id);
                // 更新索引
                for (_, index) in self.item_id_map.iter_mut() {
                    if *index > 0 {
                        *index -= 1;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 执行撤销操作 - 考虑并发修改
    pub async fn undo(&mut self, current_tree: &Tree) -> Result<UndoResult, UndoError> {
        let start_time = std::time::Instant::now();
        
        let undo_item = self.undo_stack.pop_back()
            .ok_or(UndoError::NothingToUndo)?;
            
        // 更新索引映射
        self.item_id_map.remove(&undo_item.id);
        
        // 检查撤销的可行性
        let feasibility = self.check_undo_feasibility(&undo_item, current_tree).await?;
        
        let result = match feasibility {
            UndoFeasibility::Safe => {
                // 直接应用逆操作
                self.apply_inverse_operation_directly(&undo_item, current_tree).await?
            }
            UndoFeasibility::RequiresPositionMapping => {
                // 需要位置映射
                self.apply_inverse_operation_with_mapping(&undo_item, current_tree).await?
            }
            UndoFeasibility::RequiresConflictResolution => {
                // 需要冲突解决
                self.apply_inverse_operation_with_conflict_resolution(&undo_item, current_tree).await?
            }
            UndoFeasibility::Unsafe(reason) => {
                // 撤销不安全，放回栈中并返回错误
                self.undo_stack.push_back(undo_item);
                return Err(UndoError::UnsafeUndo(reason));
            }
        };
        
        // 将撤销的项移到重做栈
        self.redo_stack.push_back(undo_item);
        
        // 更新统计
        self.undo_stats.total_undos += 1;
        let duration = start_time.elapsed();
        self.undo_stats.average_undo_latency = 
            (self.undo_stats.average_undo_latency + duration) / 2;
            
        if result.affected_positions.len() > 0 {
            self.undo_stats.position_adjusted_undos += 1;
        }
        
        Ok(result)
    }
    
    /// 执行重做操作
    pub async fn redo(&mut self, current_tree: &Tree) -> Result<RedoResult, UndoError> {
        let redo_item = self.redo_stack.pop_back()
            .ok_or(UndoError::NothingToRedo)?;
            
        // 将相对位置映射到当前状态
        let current_positions = self.position_mapper.map_positions_to_current(
            &redo_item.relative_positions,
            current_tree,
        )?;
        
        // 调整原始操作以适应当前状态
        let adjusted_operation = self.adjust_operation_to_current_state(
            redo_item.original_operation.clone(),
            &current_positions,
            current_tree,
        )?;
        
        // 在 Yrs 文档中应用调整后的操作
        let mut txn = self.yrs_doc.transact_mut();
        self.apply_yrs_operation(&mut txn, &adjusted_operation)?;
        txn.commit();
        
        // 将重做的项移回撤销栈
        self.undo_stack.push_back(redo_item);
        
        // 更新统计
        self.undo_stats.total_redos += 1;
        
        Ok(RedoResult {
            reapplied_operation: adjusted_operation,
            affected_positions: current_positions,
            confidence: 0.9, // 重做通常比撤销更可靠
        })
    }
    
    /// 处理远程操作对撤销栈的影响
    /// 
    /// 当接收到远程操作时，需要更新本地撤销栈中的位置信息
    pub async fn handle_remote_operation(
        &mut self,
        remote_operation: &YrsOperation,
        tree: &Tree,
    ) -> Result<(), UndoError> {
        // 遍历撤销栈，更新受影响的项
        for undo_item in &mut self.undo_stack {
            if self.operation_affects_undo_item(remote_operation, undo_item, tree)? {
                // 标记为受远程操作影响
                undo_item.affected_by_remote = true;
                
                // 更新相对位置
                undo_item.relative_positions = self.position_mapper
                    .map_positions_through_operations(
                        undo_item.relative_positions.clone(),
                        &[remote_operation.clone()],
                        tree,
                    )?;
                    
                // 重新计算复杂度（可能变得更复杂）
                undo_item.complexity_score *= 1.2;
            }
        }
        
        // 同样处理重做栈
        for redo_item in &mut self.redo_stack {
            if self.operation_affects_undo_item(remote_operation, redo_item, tree)? {
                redo_item.affected_by_remote = true;
                redo_item.relative_positions = self.position_mapper
                    .map_positions_through_operations(
                        redo_item.relative_positions.clone(),
                        &[remote_operation.clone()],
                        tree,
                    )?;
            }
        }
        
        Ok(())
    }
    
    /// 检查撤销操作的可行性
    async fn check_undo_feasibility(
        &self,
        undo_item: &UndoItem,
        current_tree: &Tree,
    ) -> Result<UndoFeasibility, UndoError> {
        // 检查1: 时间间隔（太久远的操作可能不安全）
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let time_diff = now - undo_item.timestamp;
        if time_diff > 3600000 { // 1小时
            return Ok(UndoFeasibility::Unsafe(
                "Operation is too old to safely undo".to_string()
            ));
        }
        
        // 检查2: 复杂度评分
        if undo_item.complexity_score > 0.8 {
            return Ok(UndoFeasibility::RequiresConflictResolution);
        }
        
        // 检查3: 是否被远程操作影响
        if undo_item.affected_by_remote {
            return Ok(UndoFeasibility::RequiresPositionMapping);
        }
        
        // 检查4: 依赖关系
        if !undo_item.dependencies.is_empty() {
            // 检查依赖的操作是否还在栈中
            for dep_id in &undo_item.dependencies {
                if !self.item_id_map.contains_key(dep_id) {
                    return Ok(UndoFeasibility::Unsafe(
                        "Dependent operation has been removed".to_string()
                    ));
                }
            }
        }
        
        // 检查5: 目标节点是否仍然存在
        for pos in &undo_item.relative_positions {
            if current_tree.get_node(&pos.anchor).is_none() {
                return Ok(UndoFeasibility::RequiresPositionMapping);
            }
        }
        
        Ok(UndoFeasibility::Safe)
    }
    
    /// 直接应用逆操作（最简单的情况）
    async fn apply_inverse_operation_directly(
        &self,
        undo_item: &UndoItem,
        tree: &Tree,
    ) -> Result<UndoResult, UndoError> {
        let mut txn = self.yrs_doc.transact_mut();
        self.apply_yrs_operation(&mut txn, &undo_item.inverse_operation)?;
        txn.commit();
        
        Ok(UndoResult {
            undone_operation: undo_item.original_operation.clone(),
            applied_inverse: undo_item.inverse_operation.clone(),
            affected_positions: undo_item.relative_positions.clone(),
            confidence: 0.95,
            requires_confirmation: false,
            warnings: vec![],
        })
    }
    
    /// 应用需要位置映射的逆操作
    async fn apply_inverse_operation_with_mapping(
        &mut self,
        undo_item: &UndoItem,
        tree: &Tree,
    ) -> Result<UndoResult, UndoError> {
        // 将相对位置映射到当前状态
        let current_positions = self.position_mapper.map_positions_to_current(
            &undo_item.relative_positions,
            tree,
        )?;
        
        // 调整逆操作以适应当前状态
        let adjusted_inverse = self.adjust_operation_to_current_state(
            undo_item.inverse_operation.clone(),
            &current_positions,
            tree,
        )?;
        
        // 应用调整后的逆操作
        let mut txn = self.yrs_doc.transact_mut();
        self.apply_yrs_operation(&mut txn, &adjusted_inverse)?;
        txn.commit();
        
        let mut warnings = vec![];
        if current_positions != undo_item.relative_positions {
            warnings.push("Operation positions were adjusted due to concurrent changes".to_string());
        }
        
        Ok(UndoResult {
            undone_operation: undo_item.original_operation.clone(),
            applied_inverse: adjusted_inverse,
            affected_positions: current_positions,
            confidence: 0.8, // 较低置信度
            requires_confirmation: false,
            warnings,
        })
    }
    
    /// 应用需要冲突解决的逆操作
    async fn apply_inverse_operation_with_conflict_resolution(
        &mut self,
        undo_item: &UndoItem,
        tree: &Tree,
    ) -> Result<UndoResult, UndoError> {
        // 检测可能的冲突
        let conflicts = self.detect_undo_conflicts(undo_item, tree).await?;
        
        if conflicts.is_empty() {
            // 实际上没有冲突，使用位置映射方法
            return self.apply_inverse_operation_with_mapping(undo_item, tree).await;
        }
        
        // 解决每个冲突
        let mut resolved_operations = vec![];
        for conflict in conflicts {
            let resolution = self.conflict_resolver.resolve_conflict(conflict).await
                .map_err(|e| UndoError::ConflictResolutionFailed(e.to_string()))?;
            resolved_operations.extend(resolution.operations);
        }
        
        // 应用解决后的操作
        let mut txn = self.yrs_doc.transact_mut();
        for operation in &resolved_operations {
            self.apply_yrs_operation(&mut txn, operation)?;
        }
        txn.commit();
        
        Ok(UndoResult {
            undone_operation: undo_item.original_operation.clone(),
            applied_inverse: resolved_operations.into_iter().next()
                .unwrap_or(undo_item.inverse_operation.clone()),
            affected_positions: undo_item.relative_positions.clone(),
            confidence: 0.6, // 低置信度，因为涉及冲突解决
            requires_confirmation: true, // 可能需要用户确认
            warnings: vec!["Undo operation required conflict resolution".to_string()],
        })
    }
    
    /// 生成逆操作
    fn generate_inverse_operation(
        &self,
        operation: &YrsOperation,
        tree_before: &Tree,
    ) -> Result<YrsOperation, UndoError> {
        let inverse_op_type = match &operation.operation_type {
            YrsOperationType::ArrayInsert { index, values } => {
                // 插入的逆操作是删除
                YrsOperationType::ArrayDelete { 
                    index: *index, 
                    length: values.len() as u32 
                }
            }
            YrsOperationType::ArrayDelete { index, length } => {
                // 删除的逆操作是插入（需要保存被删除的内容）
                let deleted_values = self.get_deleted_values_from_tree(
                    &operation.target_path, 
                    *index, 
                    *length, 
                    tree_before
                )?;
                YrsOperationType::ArrayInsert { 
                    index: *index, 
                    values: deleted_values 
                }
            }
            YrsOperationType::MapSet { key, value } => {
                // 设置的逆操作是恢复原值或删除
                if let Some(original_value) = self.get_original_map_value(
                    &operation.target_path,
                    key,
                    tree_before
                )? {
                    YrsOperationType::MapSet { 
                        key: key.clone(), 
                        value: original_value 
                    }
                } else {
                    YrsOperationType::MapDelete { key: key.clone() }
                }
            }
            YrsOperationType::MapDelete { key } => {
                // 删除的逆操作是恢复原值
                let original_value = self.get_original_map_value(
                    &operation.target_path,
                    key,
                    tree_before
                )?.ok_or_else(|| UndoError::CannotGenerateInverse(
                    "Original value not found for map delete".to_string()
                ))?;
                YrsOperationType::MapSet { 
                    key: key.clone(), 
                    value: original_value 
                }
            }
            YrsOperationType::TextInsert { index, text } => {
                YrsOperationType::TextDelete { 
                    index: *index, 
                    length: text.len() as u32 
                }
            }
            YrsOperationType::TextDelete { index, length } => {
                let deleted_text = self.get_deleted_text_from_tree(
                    &operation.target_path,
                    *index,
                    *length,
                    tree_before
                )?;
                YrsOperationType::TextInsert { 
                    index: *index, 
                    text: deleted_text 
                }
            }
            YrsOperationType::Custom { operation: op_name, data } => {
                // 自定义操作需要特殊处理
                return self.generate_custom_inverse_operation(op_name, data, tree_before);
            }
        };
        
        Ok(YrsOperation {
            id: Uuid::new_v4(),
            operation_type: inverse_op_type,
            target_path: operation.target_path.clone(),
            user_id: operation.user_id.clone(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            data: serde_json::json!({"inverse_of": operation.id}),
        })
    }
    
    /// 提取相对位置信息
    fn extract_relative_positions(
        &mut self,
        operation: &YrsOperation,
        tree: &Tree,
    ) -> Result<Vec<RelativePosition>, UndoError> {
        let mut positions = Vec::new();
        
        match &operation.operation_type {
            YrsOperationType::ArrayInsert { index, .. } |
            YrsOperationType::ArrayDelete { index, .. } => {
                // 为数组操作创建相对位置
                if let Some(target_node_id) = self.resolve_target_node(&operation.target_path, tree) {
                    let abs_pos = crate::collaboration::relative_position::AbsolutePosition {
                        node_id: target_node_id.clone(),
                        parent_index: Some(*index),
                        content_offset: None,
                        full_path: tree.get_node_path(&target_node_id).unwrap_or_default(),
                    };
                    
                    let rel_pos = self.position_mapper.absolute_to_relative(abs_pos, tree)
                        .map_err(|e| UndoError::PositionMappingFailed(e.to_string()))?;
                    positions.push(rel_pos);
                }
            }
            YrsOperationType::MapSet { key, .. } |
            YrsOperationType::MapDelete { key } => {
                // 为映射操作创建相对位置
                if let Some(target_node_id) = self.resolve_target_node(&operation.target_path, tree) {
                    let abs_pos = crate::collaboration::relative_position::AbsolutePosition {
                        node_id: target_node_id.clone(),
                        parent_index: None,
                        content_offset: None,
                        full_path: tree.get_node_path(&target_node_id).unwrap_or_default(),
                    };
                    
                    let rel_pos = self.position_mapper.absolute_to_relative(abs_pos, tree)
                        .map_err(|e| UndoError::PositionMappingFailed(e.to_string()))?;
                    positions.push(rel_pos);
                }
            }
            _ => {
                // 其他操作类型的处理
            }
        }
        
        Ok(positions)
    }
    
    /// 计算操作的复杂度评分
    fn calculate_complexity_score(&self, operation: &YrsOperation, tree: &Tree) -> f32 {
        let mut score = 0.0;
        
        // 基础操作类型的复杂度
        match &operation.operation_type {
            YrsOperationType::ArrayInsert { values, .. } => {
                score += 0.3 + (values.len() as f32 * 0.1);
            }
            YrsOperationType::ArrayDelete { length, .. } => {
                score += 0.4 + (*length as f32 * 0.1);
            }
            YrsOperationType::MapSet { .. } => score += 0.2,
            YrsOperationType::MapDelete { .. } => score += 0.3,
            YrsOperationType::TextInsert { text, .. } => {
                score += 0.2 + (text.len() as f32 * 0.01);
            }
            YrsOperationType::TextDelete { length, .. } => {
                score += 0.3 + (*length as f32 * 0.01);
            }
            YrsOperationType::Custom { .. } => score += 0.5,
        }
        
        // 路径深度的影响
        score += operation.target_path.len() as f32 * 0.05;
        
        // 限制在 [0.0, 1.0] 范围内
        score.min(1.0)
    }
    
    // 辅助方法的占位符实现
    fn get_deleted_values_from_tree(
        &self, 
        path: &[String], 
        index: u32, 
        length: u32, 
        tree: &Tree
    ) -> Result<Vec<yrs::Any>, UndoError> {
        // 这里需要根据具体的树结构和 Yrs 映射来实现
        Ok(vec![])
    }
    
    fn get_original_map_value(
        &self,
        path: &[String],
        key: &str,
        tree: &Tree,
    ) -> Result<Option<yrs::Any>, UndoError> {
        // 这里需要根据具体的映射逻辑来实现
        Ok(None)
    }
    
    fn get_deleted_text_from_tree(
        &self,
        path: &[String],
        index: u32,
        length: u32,
        tree: &Tree,
    ) -> Result<String, UndoError> {
        Ok(String::new())
    }
    
    fn generate_custom_inverse_operation(
        &self,
        operation: &str,
        data: &serde_json::Value,
        tree: &Tree,
    ) -> Result<YrsOperation, UndoError> {
        Err(UndoError::CustomOperationNotSupported)
    }
    
    fn resolve_target_node(&self, path: &[String], tree: &Tree) -> Option<NodeId> {
        // 根据路径解析目标节点
        None
    }
    
    fn operation_affects_undo_item(
        &self,
        operation: &YrsOperation,
        undo_item: &UndoItem,
        tree: &Tree,
    ) -> Result<bool, UndoError> {
        // 检查远程操作是否影响撤销项
        Ok(false)
    }
    
    fn detect_undo_conflicts(
        &self,
        undo_item: &UndoItem,
        tree: &Tree,
    ) -> Result<Vec<ConflictContext>, UndoError> {
        Ok(vec![])
    }
    
    fn adjust_operation_to_current_state(
        &self,
        operation: YrsOperation,
        positions: &[RelativePosition],
        tree: &Tree,
    ) -> Result<YrsOperation, UndoError> {
        Ok(operation)
    }
    
    fn apply_yrs_operation(
        &self,
        txn: &mut TransactionMut,
        operation: &YrsOperation,
    ) -> Result<(), UndoError> {
        // 实际的 Yrs 操作应用
        Ok(())
    }
}

// 扩展 PositionMapper 以支持批量位置映射
impl PositionMapper {
    pub fn map_positions_to_current(
        &mut self,
        positions: &[RelativePosition],
        tree: &Tree,
    ) -> Result<Vec<RelativePosition>, PositionError> {
        let mut current_positions = Vec::new();
        for pos in positions {
            let current_pos = self.relative_to_absolute(pos, tree)
                .and_then(|abs_pos| self.absolute_to_relative(abs_pos, tree))?;
            current_positions.push(current_pos);
        }
        Ok(current_positions)
    }
}

/// 撤销可行性评估结果
#[derive(Debug, Clone)]
enum UndoFeasibility {
    /// 安全，可以直接撤销
    Safe,
    /// 需要位置映射
    RequiresPositionMapping,
    /// 需要冲突解决
    RequiresConflictResolution,
    /// 不安全，不能撤销
    Unsafe(String),
}

/// 撤销错误类型
#[derive(Debug, thiserror::Error)]
pub enum UndoError {
    #[error("Nothing to undo")]
    NothingToUndo,
    
    #[error("Nothing to redo")]
    NothingToRedo,
    
    #[error("Cannot generate inverse operation: {0}")]
    CannotGenerateInverse(String),
    
    #[error("Position mapping failed: {0}")]
    PositionMappingFailed(String),
    
    #[error("Conflict resolution failed: {0}")]
    ConflictResolutionFailed(String),
    
    #[error("Unsafe undo operation: {0}")]
    UnsafeUndo(String),
    
    #[error("Custom operation not supported")]
    CustomOperationNotSupported,
    
    #[error("Yrs operation failed: {0}")]
    YrsOperationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_undo_manager_creation() {
        let yrs_doc = Arc::new(yrs::Doc::new());
        let conflict_resolver = Arc::new(ModuForgeConflictResolver::new());
        
        let manager = CollaborativeUndoManager::new(
            "user1".to_string(),
            yrs_doc,
            conflict_resolver,
        );
        
        assert_eq!(manager.user_id, "user1");
        assert_eq!(manager.undo_stack.len(), 0);
        assert_eq!(manager.redo_stack.len(), 0);
    }
    
    #[test]
    fn test_undo_item_creation() {
        let operation = YrsOperation {
            id: Uuid::new_v4(),
            operation_type: YrsOperationType::ArrayInsert {
                index: 0,
                values: vec![yrs::Any::String("test".into())],
            },
            target_path: vec!["nodes".to_string()],
            user_id: "user1".to_string(),
            timestamp: 1000,
            data: serde_json::json!({}),
        };
        
        let undo_item = UndoItem {
            id: Uuid::new_v4(),
            original_operation: operation.clone(),
            inverse_operation: operation.clone(),
            relative_positions: vec![],
            timestamp: 1000,
            document_version: vec![],
            complexity_score: 0.5,
            affected_by_remote: false,
            dependencies: vec![],
        };
        
        assert_eq!(undo_item.timestamp, 1000);
        assert_eq!(undo_item.complexity_score, 0.5);
        assert!(!undo_item.affected_by_remote);
    }
}
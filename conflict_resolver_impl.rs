use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use yrs::{Any as YrsAny, Doc, TransactionMut};
use chrono::{DateTime, Utc};

use crate::collaboration::types::*;
use crate::state::{State, Resource};
use crate::model::{NodeId, Tree};
use crate::transform::Transaction;

/// ModuForge-RS 冲突解决器 - 基于 y-prosemirror 设计
pub struct ModuForgeConflictResolver {
    /// 冲突解决策略映射
    strategies: HashMap<ConflictType, ResolutionStrategy>,
    /// 自定义解决器注册表
    custom_resolvers: HashMap<String, Box<dyn CustomConflictResolver + Send + Sync>>,
    /// 用户优先级映射
    user_priorities: HashMap<UserId, u32>,
    /// 冲突统计
    conflict_stats: ConflictStatistics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConflictType {
    NodeStructure,
    NodeAttributes, 
    NodeMarks,
    PluginState,
    ConcurrentTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    LastWriterWins,
    Merge,
    UserPriority(UserId),
    TimestampPriority,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ConflictContext {
    pub conflict_type: ConflictType,
    pub local_operation: YrsOperation,
    pub remote_operation: YrsOperation,
    pub local_user: UserId,
    pub remote_user: UserId,
    pub local_timestamp: u64,
    pub remote_timestamp: u64,
    pub node_path: Vec<NodeId>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct YrsOperation {
    pub id: Uuid,
    pub operation_type: YrsOperationType,
    pub target_path: Vec<String>,
    pub user_id: UserId,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum YrsOperationType {
    MapSet { key: String, value: YrsAny },
    MapDelete { key: String },
    ArrayInsert { index: u32, values: Vec<YrsAny> },
    ArrayDelete { index: u32, length: u32 },
    TextInsert { index: u32, text: String },
    TextDelete { index: u32, length: u32 },
    Custom { operation: String, data: serde_json::Value },
}

#[derive(Debug, Clone)]
pub struct ConflictResolution {
    pub resolution_type: ResolutionType,
    pub operations: Vec<YrsOperation>,
    pub explanation: String,
    pub confidence: f32, // 0.0 - 1.0
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionType {
    LastWriterWins,
    Merge,
    DeleteWins,
    TimestampWins,
    UserPriorityWins,
    Custom,
}

pub type UserId = String;

impl ModuForgeConflictResolver {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // 基于 y-prosemirror 的默认策略
        strategies.insert(ConflictType::NodeStructure, ResolutionStrategy::Merge);
        strategies.insert(ConflictType::NodeAttributes, ResolutionStrategy::Merge);
        strategies.insert(ConflictType::NodeMarks, ResolutionStrategy::Merge);
        strategies.insert(ConflictType::PluginState, ResolutionStrategy::LastWriterWins);
        strategies.insert(ConflictType::ConcurrentTransaction, ResolutionStrategy::TimestampPriority);
        
        Self {
            strategies,
            custom_resolvers: HashMap::new(),
            user_priorities: HashMap::new(),
            conflict_stats: ConflictStatistics::new(),
        }
    }
    
    /// 主冲突解决入口点
    pub async fn resolve_conflict(
        &mut self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        self.conflict_stats.record_conflict(&context.conflict_type);
        
        let start_time = std::time::Instant::now();
        
        let strategy = self.strategies.get(&context.conflict_type)
            .unwrap_or(&ResolutionStrategy::LastWriterWins);
            
        let resolution = match strategy {
            ResolutionStrategy::LastWriterWins => {
                self.resolve_last_writer_wins(context).await
            }
            ResolutionStrategy::Merge => {
                self.resolve_merge(context).await
            }
            ResolutionStrategy::UserPriority(user_id) => {
                self.resolve_user_priority(context, user_id).await
            }
            ResolutionStrategy::TimestampPriority => {
                self.resolve_timestamp_priority(context).await
            }
            ResolutionStrategy::Custom(resolver_name) => {
                self.resolve_custom(context, resolver_name).await
            }
        }?;
        
        let duration = start_time.elapsed();
        self.conflict_stats.record_resolution_time(duration);
        
        Ok(resolution)
    }
    
    /// 最后写入获胜策略
    async fn resolve_last_writer_wins(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        let winning_operation = if context.local_timestamp >= context.remote_timestamp {
            context.local_operation
        } else {
            context.remote_operation
        };
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::LastWriterWins,
            operations: vec![winning_operation],
            explanation: "Resolved using last writer wins strategy".to_string(),
            confidence: 0.8,
            metadata: HashMap::new(),
        })
    }
    
    /// 合并策略 - y-prosemirror 风格
    async fn resolve_merge(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        match context.conflict_type {
            ConflictType::NodeStructure => {
                self.resolve_node_structure_conflict(context).await
            }
            ConflictType::NodeAttributes => {
                self.resolve_attribute_conflict(context).await
            }
            ConflictType::NodeMarks => {
                self.resolve_marks_conflict(context).await
            }
            _ => {
                // 对于其他类型，回退到时间戳策略
                self.resolve_timestamp_priority(context).await
            }
        }
    }
    
    /// 节点结构冲突解决 - 核心算法
    async fn resolve_node_structure_conflict(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        match (&context.local_operation.operation_type, &context.remote_operation.operation_type) {
            // 并发插入冲突
            (YrsOperationType::ArrayInsert { index: local_idx, values: local_vals }, 
             YrsOperationType::ArrayInsert { index: remote_idx, values: remote_vals }) => {
                self.resolve_concurrent_inserts(*local_idx, local_vals, *remote_idx, remote_vals, &context).await
            }
            
            // 删除-修改冲突
            (YrsOperationType::ArrayDelete { index: del_idx, length: del_len },
             YrsOperationType::MapSet { key, value }) => {
                self.resolve_delete_modify_conflict(*del_idx, *del_len, key, value, &context).await
            }
            
            // 移动冲突 (删除 + 插入)
            (YrsOperationType::ArrayDelete { .. }, YrsOperationType::ArrayInsert { .. }) |
            (YrsOperationType::ArrayInsert { .. }, YrsOperationType::ArrayDelete { .. }) => {
                self.resolve_move_conflict(&context).await
            }
            
            // 并发删除 - 合并删除范围
            (YrsOperationType::ArrayDelete { index: idx1, length: len1 },
             YrsOperationType::ArrayDelete { index: idx2, length: len2 }) => {
                self.resolve_concurrent_deletes(*idx1, *len1, *idx2, *len2, &context).await
            }
            
            _ => {
                // 其他情况使用时间戳优先
                self.resolve_timestamp_priority(context).await
            }
        }
    }
    
    /// 并发插入冲突解决 - y-prosemirror 算法
    async fn resolve_concurrent_inserts(
        &self,
        local_index: u32,
        local_values: &[YrsAny],
        remote_index: u32,
        remote_values: &[YrsAny],
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // 策略：保持两个插入，但智能调整位置
        // 参考 y-prosemirror 的位置映射算法
        
        let (first_op, second_op, adjusted_index) = if local_index <= remote_index {
            // 本地插入在前，远程插入位置需要向后调整
            let adjusted_remote_index = remote_index + local_values.len() as u32;
            (
                context.local_operation.clone(),
                YrsOperation {
                    id: Uuid::new_v4(),
                    operation_type: YrsOperationType::ArrayInsert {
                        index: adjusted_remote_index,
                        values: remote_values.to_vec(),
                    },
                    target_path: context.remote_operation.target_path.clone(),
                    user_id: context.remote_user.clone(),
                    timestamp: context.remote_timestamp,
                    data: context.remote_operation.data.clone(),
                },
                adjusted_remote_index
            )
        } else {
            // 远程插入在前，本地插入位置需要向后调整
            let adjusted_local_index = local_index + remote_values.len() as u32;
            (
                context.remote_operation.clone(),
                YrsOperation {
                    id: Uuid::new_v4(),
                    operation_type: YrsOperationType::ArrayInsert {
                        index: adjusted_local_index,
                        values: local_values.to_vec(),
                    },
                    target_path: context.local_operation.target_path.clone(),
                    user_id: context.local_user.clone(),
                    timestamp: context.local_timestamp,
                    data: context.local_operation.data.clone(),
                },
                adjusted_local_index
            )
        };
        
        let mut metadata = HashMap::new();
        metadata.insert(
            "original_positions".to_string(), 
            serde_json::json!({
                "local": local_index,
                "remote": remote_index,
                "adjusted": adjusted_index
            })
        );
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::Merge,
            operations: vec![first_op, second_op],
            explanation: format!(
                "Concurrent inserts merged: positions {} and {} -> {} and {}",
                local_index, remote_index, 
                std::cmp::min(local_index, remote_index),
                adjusted_index
            ),
            confidence: 0.95,
            metadata,
        })
    }
    
    /// 删除-修改冲突解决
    async fn resolve_delete_modify_conflict(
        &self,
        delete_index: u32,
        delete_length: u32,
        modify_key: &str,
        modify_value: &YrsAny,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // y-prosemirror 策略：删除优先，但记录意图
        // 在实际应用中，可能需要通知用户修改的内容已被删除
        
        let mut metadata = HashMap::new();
        metadata.insert(
            "lost_modification".to_string(),
            serde_json::json!({
                "key": modify_key,
                "value": self.yrs_any_to_json(modify_value)?,
                "user": context.remote_user,
                "timestamp": context.remote_timestamp
            })
        );
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::DeleteWins,
            operations: vec![context.local_operation.clone()],
            explanation: format!(
                "Delete operation wins over modification. Modified '{}' by {} was lost.",
                modify_key, context.remote_user
            ),
            confidence: 0.7, // 较低置信度，因为丢失了用户意图
            metadata,
        })
    }
    
    /// 属性冲突解决 - 智能合并
    async fn resolve_attribute_conflict(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        match (&context.local_operation.operation_type, &context.remote_operation.operation_type) {
            (YrsOperationType::MapSet { key: local_key, value: local_value },
             YrsOperationType::MapSet { key: remote_key, value: remote_value }) => {
                
                if local_key == remote_key {
                    // 同一属性的值冲突 - 基于属性类型智能合并
                    self.resolve_same_attribute_conflict(
                        local_key, local_value, remote_value, &context
                    ).await
                } else {
                    // 不同属性，可以安全合并
                    Ok(ConflictResolution {
                        resolution_type: ResolutionType::Merge,
                        operations: vec![
                            context.local_operation.clone(),
                            context.remote_operation.clone(),
                        ],
                        explanation: format!(
                            "Different attributes '{}' and '{}' merged successfully",
                            local_key, remote_key
                        ),
                        confidence: 0.98,
                        metadata: HashMap::new(),
                    })
                }
            }
            _ => self.resolve_timestamp_priority(context).await
        }
    }
    
    /// 同一属性的智能合并 - y-prosemirror 风格
    async fn resolve_same_attribute_conflict(
        &self,
        key: &str,
        local_value: &YrsAny,
        remote_value: &YrsAny,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        match key {
            // 文本内容 - 尝试合并
            "text" | "content" | "title" => {
                self.merge_text_attributes(local_value, remote_value, context).await
            }
            
            // 样式属性 - 合并样式对象
            "style" | "styles" | "class" | "className" => {
                self.merge_style_attributes(local_value, remote_value, context).await
            }
            
            // 位置和尺寸 - 使用最新值
            "x" | "y" | "width" | "height" | "left" | "top" => {
                self.resolve_timestamp_priority(context.clone()).await
            }
            
            // 配置属性 - 合并配置对象
            "config" | "options" | "settings" => {
                self.merge_object_attributes(local_value, remote_value, context).await
            }
            
            // 数值属性 - 使用平均值或最大值
            "count" | "size" | "level" | "priority" => {
                self.merge_numeric_attributes(local_value, remote_value, context).await
            }
            
            // 其他属性 - 用户优先级或时间戳
            _ => {
                if let Some(priority_user) = self.get_higher_priority_user(&context.local_user, &context.remote_user) {
                    self.resolve_user_priority(context.clone(), &priority_user).await
                } else {
                    self.resolve_timestamp_priority(context.clone()).await
                }
            }
        }
    }
    
    /// 文本属性智能合并
    async fn merge_text_attributes(
        &self,
        local_value: &YrsAny,
        remote_value: &YrsAny,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        let local_text = self.extract_text_from_yrs_any(local_value)?;
        let remote_text = self.extract_text_from_yrs_any(remote_value)?;
        
        // 智能文本合并策略
        let merged_text = if local_text.is_empty() {
            remote_text
        } else if remote_text.is_empty() {
            local_text
        } else if local_text.contains(&remote_text) {
            local_text // 本地包含远程内容
        } else if remote_text.contains(&local_text) {
            remote_text // 远程包含本地内容
        } else {
            // 使用简单的文本合并策略
            // 实际应用中可以使用更复杂的 diff 算法
            self.merge_text_with_diff(&local_text, &remote_text)?
        };
        
        let merged_operation = YrsOperation {
            id: Uuid::new_v4(),
            operation_type: YrsOperationType::MapSet {
                key: "text".to_string(),
                value: YrsAny::String(merged_text.clone().into()),
            },
            target_path: context.local_operation.target_path.clone(),
            user_id: context.local_user.clone(),
            timestamp: std::cmp::max(context.local_timestamp, context.remote_timestamp),
            data: serde_json::json!({"merged": true, "strategy": "text_merge"}),
        };
        
        let mut metadata = HashMap::new();
        metadata.insert(
            "merge_details".to_string(),
            serde_json::json!({
                "local_text": local_text,
                "remote_text": remote_text,
                "merged_text": merged_text,
                "strategy": "intelligent_text_merge"
            })
        );
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::Merge,
            operations: vec![merged_operation],
            explanation: format!(
                "Text attributes merged intelligently: '{}' + '{}' = '{}'",
                self.truncate_text(&local_text, 30),
                self.truncate_text(&remote_text, 30),
                self.truncate_text(&merged_text, 50)
            ),
            confidence: 0.85,
            metadata,
        })
    }
    
    /// 对象属性合并
    async fn merge_object_attributes(
        &self,
        local_value: &YrsAny,
        remote_value: &YrsAny,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        let local_obj = self.yrs_any_to_json(local_value)?;
        let remote_obj = self.yrs_any_to_json(remote_value)?;
        
        let merged_obj = match (local_obj, remote_obj) {
            (serde_json::Value::Object(mut local_map), serde_json::Value::Object(remote_map)) => {
                // 合并对象，远程值覆盖本地值
                for (key, value) in remote_map {
                    local_map.insert(key, value);
                }
                serde_json::Value::Object(local_map)
            }
            (_, remote_val) => remote_val, // 如果类型不匹配，使用远程值
        };
        
        let merged_operation = YrsOperation {
            id: Uuid::new_v4(),
            operation_type: YrsOperationType::MapSet {
                key: "config".to_string(),
                value: self.json_to_yrs_any(&merged_obj)?,
            },
            target_path: context.local_operation.target_path.clone(),
            user_id: context.local_user.clone(),
            timestamp: std::cmp::max(context.local_timestamp, context.remote_timestamp),
            data: serde_json::json!({"merged": true, "strategy": "object_merge"}),
        };
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::Merge,
            operations: vec![merged_operation],
            explanation: "Object attributes merged successfully".to_string(),
            confidence: 0.9,
            metadata: HashMap::new(),
        })
    }
    
    /// 时间戳优先策略
    async fn resolve_timestamp_priority(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        let (winning_operation, winning_user) = if context.local_timestamp > context.remote_timestamp {
            (context.local_operation, context.local_user)
        } else {
            (context.remote_operation, context.remote_user)
        };
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::TimestampWins,
            operations: vec![winning_operation],
            explanation: format!("Resolved by timestamp priority. Winner: {}", winning_user),
            confidence: 0.75,
            metadata: HashMap::new(),
        })
    }
    
    /// 获取更高优先级的用户
    fn get_higher_priority_user(&self, user1: &UserId, user2: &UserId) -> Option<UserId> {
        let priority1 = self.user_priorities.get(user1).unwrap_or(&0);
        let priority2 = self.user_priorities.get(user2).unwrap_or(&0);
        
        if priority1 > priority2 {
            Some(user1.clone())
        } else if priority2 > priority1 {
            Some(user2.clone())
        } else {
            None // 优先级相同
        }
    }
    
    /// 工具方法：从 YrsAny 提取文本
    fn extract_text_from_yrs_any(&self, value: &YrsAny) -> Result<String, ConflictError> {
        match value {
            YrsAny::String(s) => Ok(s.to_string()),
            YrsAny::BigInt(n) => Ok(n.to_string()),
            YrsAny::Number(n) => Ok(n.to_string()),
            YrsAny::Bool(b) => Ok(b.to_string()),
            _ => Err(ConflictError::InvalidDataType("Expected text-like value".to_string())),
        }
    }
    
    /// 智能文本合并使用 diff 算法
    fn merge_text_with_diff(&self, text1: &str, text2: &str) -> Result<String, ConflictError> {
        // 简单的文本合并策略
        // 实际应用中可以使用 patience diff 或其他高级算法
        
        if text1.len() > text2.len() {
            // 如果一个文本明显更长，可能包含了更多信息
            Ok(format!("{}\n{}", text1, text2))
        } else {
            Ok(format!("{}\n{}", text2, text1))
        }
    }
    
    /// 截断文本用于显示
    fn truncate_text(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len])
        }
    }
    
    /// 辅助方法：YrsAny 转 JSON
    fn yrs_any_to_json(&self, value: &YrsAny) -> Result<serde_json::Value, ConflictError> {
        match value {
            YrsAny::String(s) => Ok(serde_json::Value::String(s.to_string())),
            YrsAny::Number(n) => Ok(serde_json::Value::Number(
                serde_json::Number::from_f64(*n)
                    .ok_or_else(|| ConflictError::InvalidDataType("Invalid number".to_string()))?
            )),
            YrsAny::BigInt(n) => Ok(serde_json::Value::Number(
                serde_json::Number::from(*n)
            )),
            YrsAny::Bool(b) => Ok(serde_json::Value::Bool(*b)),
            YrsAny::Null => Ok(serde_json::Value::Null),
            YrsAny::Array(arr) => {
                let mut json_arr = Vec::new();
                for item in arr {
                    json_arr.push(self.yrs_any_to_json(item)?);
                }
                Ok(serde_json::Value::Array(json_arr))
            }
            YrsAny::Map(map) => {
                let mut json_obj = serde_json::Map::new();
                for (key, value) in map {
                    json_obj.insert(key.clone(), self.yrs_any_to_json(value)?);
                }
                Ok(serde_json::Value::Object(json_obj))
            }
            _ => Err(ConflictError::InvalidDataType("Unsupported YrsAny type".to_string())),
        }
    }
    
    /// 辅助方法：JSON 转 YrsAny
    fn json_to_yrs_any(&self, value: &serde_json::Value) -> Result<YrsAny, ConflictError> {
        match value {
            serde_json::Value::String(s) => Ok(YrsAny::String(s.clone().into())),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(YrsAny::BigInt(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(YrsAny::Number(f))
                } else {
                    Err(ConflictError::InvalidDataType("Invalid number format".to_string()))
                }
            }
            serde_json::Value::Bool(b) => Ok(YrsAny::Bool(*b)),
            serde_json::Value::Null => Ok(YrsAny::Null),
            serde_json::Value::Array(arr) => {
                let mut yrs_arr = Vec::new();
                for item in arr {
                    yrs_arr.push(self.json_to_yrs_any(item)?);
                }
                Ok(YrsAny::Array(yrs_arr.into()))
            }
            serde_json::Value::Object(obj) => {
                let mut yrs_map = std::collections::HashMap::new();
                for (key, value) in obj {
                    yrs_map.insert(key.clone(), self.json_to_yrs_any(value)?);
                }
                Ok(YrsAny::Map(yrs_map.into()))
            }
        }
    }
}

/// 冲突统计
#[derive(Debug, Default)]
pub struct ConflictStatistics {
    pub total_conflicts: u64,
    pub conflicts_by_type: HashMap<ConflictType, u64>,
    pub total_resolution_time: std::time::Duration,
    pub average_resolution_time: std::time::Duration,
}

impl ConflictStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_conflict(&mut self, conflict_type: &ConflictType) {
        self.total_conflicts += 1;
        *self.conflicts_by_type.entry(conflict_type.clone()).or_insert(0) += 1;
    }
    
    pub fn record_resolution_time(&mut self, duration: std::time::Duration) {
        self.total_resolution_time += duration;
        if self.total_conflicts > 0 {
            self.average_resolution_time = self.total_resolution_time / self.total_conflicts as u32;
        }
    }
}

/// 冲突错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConflictError {
    #[error("Invalid data type: {0}")]
    InvalidDataType(String),
    
    #[error("Resolution strategy not found: {0}")]
    StrategyNotFound(String),
    
    #[error("Custom resolver error: {0}")]
    CustomResolverError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Position mapping error: {0}")]
    PositionMappingError(String),
}

/// 自定义冲突解决器特征
#[async_trait]
pub trait CustomConflictResolver {
    async fn resolve(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError>;
    
    fn name(&self) -> &str;
    fn priority(&self) -> u32 { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_concurrent_inserts() {
        let mut resolver = ModuForgeConflictResolver::new();
        
        let context = ConflictContext {
            conflict_type: ConflictType::NodeStructure,
            local_operation: YrsOperation {
                id: Uuid::new_v4(),
                operation_type: YrsOperationType::ArrayInsert {
                    index: 2,
                    values: vec![YrsAny::String("local".into())],
                },
                target_path: vec!["nodes".to_string()],
                user_id: "user1".to_string(),
                timestamp: 1000,
                data: serde_json::json!({}),
            },
            remote_operation: YrsOperation {
                id: Uuid::new_v4(),
                operation_type: YrsOperationType::ArrayInsert {
                    index: 2,
                    values: vec![YrsAny::String("remote".into())],
                },
                target_path: vec!["nodes".to_string()],
                user_id: "user2".to_string(),
                timestamp: 1001,
                data: serde_json::json!({}),
            },
            local_user: "user1".to_string(),
            remote_user: "user2".to_string(),
            local_timestamp: 1000,
            remote_timestamp: 1001,
            node_path: vec![],
            metadata: HashMap::new(),
        };
        
        let resolution = resolver.resolve_conflict(context).await.unwrap();
        assert_eq!(resolution.resolution_type, ResolutionType::Merge);
        assert_eq!(resolution.operations.len(), 2);
    }
    
    #[tokio::test]
    async fn test_text_attribute_merge() {
        let mut resolver = ModuForgeConflictResolver::new();
        
        let context = ConflictContext {
            conflict_type: ConflictType::NodeAttributes,
            local_operation: YrsOperation {
                id: Uuid::new_v4(),
                operation_type: YrsOperationType::MapSet {
                    key: "text".to_string(),
                    value: YrsAny::String("Hello".into()),
                },
                target_path: vec!["attrs".to_string()],
                user_id: "user1".to_string(),
                timestamp: 1000,
                data: serde_json::json!({}),
            },
            remote_operation: YrsOperation {
                id: Uuid::new_v4(),
                operation_type: YrsOperationType::MapSet {
                    key: "text".to_string(),
                    value: YrsAny::String("World".into()),
                },
                target_path: vec!["attrs".to_string()],
                user_id: "user2".to_string(),
                timestamp: 1001,
                data: serde_json::json!({}),
            },
            local_user: "user1".to_string(),
            remote_user: "user2".to_string(),
            local_timestamp: 1000,
            remote_timestamp: 1001,
            node_path: vec![],
            metadata: HashMap::new(),
        };
        
        let resolution = resolver.resolve_conflict(context).await.unwrap();
        assert_eq!(resolution.resolution_type, ResolutionType::Merge);
        assert_eq!(resolution.operations.len(), 1);
    }
}
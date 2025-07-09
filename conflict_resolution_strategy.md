# ModuForge-RS 冲突解决机制设计
## 基于 y-prosemirror 的冲突解决策略

## 🔍 y-prosemirror 冲突解决机制分析

### 核心策略概览
y-prosemirror 通过以下机制解决协作冲突：

1. **CRDT 基础** - Yjs 的 CRDT 算法自动处理基础冲突
2. **操作转换** - 智能地调整操作位置和内容
3. **意图保持** - 尽可能保持用户的编辑意图
4. **类型特定解决** - 针对不同内容类型的专门策略
5. **相对位置** - 使用相对位置而非绝对位置

## 🎯 适配到 ModuForge-RS 的设计

### 1. 基础冲突解决框架

```rust
use yrs::{Doc, StateVector, Update};
use std::collections::HashMap;
use uuid::Uuid;

/// 冲突类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// 节点结构冲突 (添加/删除/移动)
    NodeStructure,
    /// 节点属性冲突
    NodeAttributes,
    /// 节点标记冲突  
    NodeMarks,
    /// 插件状态冲突
    PluginState,
    /// 并发事务冲突
    ConcurrentTransaction,
}

/// 冲突解决策略
#[derive(Debug, Clone)]
pub enum ResolutionStrategy {
    /// 最后写入获胜 (默认)
    LastWriterWins,
    /// 合并策略 (适用于属性)
    Merge,
    /// 用户优先级
    UserPriority(UserId),
    /// 时间戳优先
    TimestampPriority,
    /// 自定义解决器
    Custom(String), // 解决器名称
}

/// 冲突上下文信息
#[derive(Debug, Clone)]
pub struct ConflictContext {
    pub conflict_type: ConflictType,
    pub local_operation: YrsOperation,
    pub remote_operation: YrsOperation,
    pub local_user: UserId,
    pub remote_user: UserId,
    pub local_timestamp: u64,
    pub remote_timestamp: u64,
    pub node_path: Vec<NodeId>, // 节点在树中的路径
}

/// Yrs 操作包装
#[derive(Debug, Clone)]
pub struct YrsOperation {
    pub id: Uuid,
    pub operation_type: YrsOperationType,
    pub target_path: Vec<String>, // Yrs 中的路径
    pub user_id: UserId,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum YrsOperationType {
    MapSet { key: String, value: yrs::Any },
    MapDelete { key: String },
    ArrayInsert { index: u32, values: Vec<yrs::Any> },
    ArrayDelete { index: u32, length: u32 },
    TextInsert { index: u32, text: String },
    TextDelete { index: u32, length: u32 },
}
```

### 2. 核心冲突解决器

```rust
/// 主冲突解决器 - 参考 y-prosemirror 的设计
pub struct ModuForgeConflictResolver {
    /// 各种冲突类型的解决策略
    strategies: HashMap<ConflictType, ResolutionStrategy>,
    /// 自定义解决器注册表
    custom_resolvers: HashMap<String, Box<dyn CustomConflictResolver>>,
    /// 用户优先级映射
    user_priorities: HashMap<UserId, u32>,
}

impl ModuForgeConflictResolver {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // 默认策略 - 基于 y-prosemirror 的最佳实践
        strategies.insert(ConflictType::NodeStructure, ResolutionStrategy::Merge);
        strategies.insert(ConflictType::NodeAttributes, ResolutionStrategy::Merge);
        strategies.insert(ConflictType::NodeMarks, ResolutionStrategy::Merge);
        strategies.insert(ConflictType::PluginState, ResolutionStrategy::LastWriterWins);
        strategies.insert(ConflictType::ConcurrentTransaction, ResolutionStrategy::TimestampPriority);
        
        Self {
            strategies,
            custom_resolvers: HashMap::new(),
            user_priorities: HashMap::new(),
        }
    }
    
    /// 解决冲突 - 主入口点
    pub async fn resolve_conflict(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        let strategy = self.strategies.get(&context.conflict_type)
            .unwrap_or(&ResolutionStrategy::LastWriterWins);
            
        match strategy {
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
        }
    }
}
```

### 3. 节点结构冲突解决 (核心)

```rust
impl ModuForgeConflictResolver {
    /// 节点结构冲突解决 - 参考 y-prosemirror 的节点处理
    async fn resolve_node_structure_conflict(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        match (&context.local_operation.operation_type, &context.remote_operation.operation_type) {
            // 情况1: 同时在同一位置插入节点
            (YrsOperationType::ArrayInsert { index: local_idx, values: local_vals }, 
             YrsOperationType::ArrayInsert { index: remote_idx, values: remote_vals }) => {
                self.resolve_concurrent_inserts(*local_idx, local_vals, *remote_idx, remote_vals, &context).await
            }
            
            // 情况2: 删除与修改冲突
            (YrsOperationType::ArrayDelete { index: del_idx, length: del_len },
             YrsOperationType::MapSet { key, value }) => {
                self.resolve_delete_modify_conflict(*del_idx, *del_len, key, value, &context).await
            }
            
            // 情况3: 移动节点冲突 (需要自定义操作类型)
            (YrsOperationType::ArrayDelete { .. }, YrsOperationType::ArrayInsert { .. }) => {
                self.resolve_move_conflict(&context).await
            }
            
            _ => {
                // 默认使用时间戳优先
                self.resolve_timestamp_priority(context).await
            }
        }
    }
    
    /// 并发插入冲突解决 - y-prosemirror 风格
    async fn resolve_concurrent_inserts(
        &self,
        local_index: u32,
        local_values: &[yrs::Any],
        remote_index: u32,
        remote_values: &[yrs::Any],
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // 策略：保持两个插入，但调整位置
        // 远程操作的位置需要根据本地插入进行调整
        
        let adjusted_remote_index = if remote_index >= local_index {
            remote_index + local_values.len() as u32
        } else {
            remote_index
        };
        
        let resolution_operations = vec![
            // 保持本地插入
            context.local_operation.clone(),
            // 调整远程插入位置
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
            }
        ];
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::Merge,
            operations: resolution_operations,
            explanation: format!(
                "Concurrent inserts resolved: local at {}, remote adjusted to {}",
                local_index, adjusted_remote_index
            ),
        })
    }
    
    /// 删除-修改冲突解决
    async fn resolve_delete_modify_conflict(
        &self,
        delete_index: u32,
        delete_length: u32,
        modify_key: &str,
        modify_value: &yrs::Any,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // 策略：删除优先，但保留修改意图
        // 如果修改的节点被删除，创建通知给修改用户
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::DeleteWins,
            operations: vec![context.local_operation.clone()], // 保持删除
            explanation: format!(
                "Node deleted by {} while {} was modifying it. Delete takes precedence.",
                context.local_user, context.remote_user
            ),
        })
    }
}
```

### 4. 属性冲突解决 - 智能合并

```rust
impl ModuForgeConflictResolver {
    /// 属性冲突解决 - 参考 y-prosemirror 的属性处理
    async fn resolve_attribute_conflict(
        &self,
        context: ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        match (&context.local_operation.operation_type, &context.remote_operation.operation_type) {
            (YrsOperationType::MapSet { key: local_key, value: local_value },
             YrsOperationType::MapSet { key: remote_key, value: remote_value }) => {
                
                if local_key == remote_key {
                    // 同一属性的不同值冲突
                    self.resolve_same_attribute_conflict(
                        local_key, local_value, remote_value, &context
                    ).await
                } else {
                    // 不同属性，无冲突，可以合并
                    Ok(ConflictResolution {
                        resolution_type: ResolutionType::Merge,
                        operations: vec![
                            context.local_operation.clone(),
                            context.remote_operation.clone(),
                        ],
                        explanation: "Different attributes merged successfully".to_string(),
                    })
                }
            }
            _ => self.resolve_timestamp_priority(context).await
        }
    }
    
    /// 同一属性的值冲突解决
    async fn resolve_same_attribute_conflict(
        &self,
        key: &str,
        local_value: &yrs::Any,
        remote_value: &yrs::Any,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // 基于属性类型的智能合并策略
        match key {
            // 文本属性：尝试合并
            "text" | "content" => {
                self.merge_text_attributes(local_value, remote_value, context).await
            }
            
            // 样式属性：合并样式对象
            "style" | "class" => {
                self.merge_style_attributes(local_value, remote_value, context).await
            }
            
            // 位置属性：使用更新的值
            "x" | "y" | "width" | "height" => {
                self.resolve_timestamp_priority(context.clone()).await
            }
            
            // 其他属性：用户优先级或时间戳
            _ => {
                if let Some(priority_user) = self.get_higher_priority_user(&context.local_user, &context.remote_user) {
                    self.resolve_user_priority(context.clone(), &priority_user).await
                } else {
                    self.resolve_timestamp_priority(context.clone()).await
                }
            }
        }
    }
    
    /// 文本属性合并 - 类似 y-prosemirror 的文本处理
    async fn merge_text_attributes(
        &self,
        local_value: &yrs::Any,
        remote_value: &yrs::Any,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // 尝试将两个文本值智能合并
        let local_text = self.extract_text_from_any(local_value)?;
        let remote_text = self.extract_text_from_any(remote_value)?;
        
        // 使用简单的文本合并策略
        let merged_text = if local_text.contains(&remote_text) {
            local_text // 本地已包含远程文本
        } else if remote_text.contains(&local_text) {
            remote_text // 远程已包含本地文本
        } else {
            // 简单拼接，实际可以使用更复杂的diff算法
            format!("{} {}", local_text, remote_text)
        };
        
        let merged_operation = YrsOperation {
            id: Uuid::new_v4(),
            operation_type: YrsOperationType::MapSet {
                key: "text".to_string(),
                value: yrs::Any::String(merged_text.into()),
            },
            target_path: context.local_operation.target_path.clone(),
            user_id: context.local_user.clone(),
            timestamp: std::cmp::max(context.local_timestamp, context.remote_timestamp),
            data: serde_json::json!({"merged": true}),
        };
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::Merge,
            operations: vec![merged_operation],
            explanation: format!("Text attributes merged: '{}' + '{}' = '{}'", 
                               local_text, remote_text, merged_text),
        })
    }
}
```

### 5. 插件状态冲突解决

```rust
/// 插件状态特定的冲突解决器
pub struct PluginStateConflictResolver;

impl PluginStateConflictResolver {
    /// 解决插件状态冲突 - ModuForge 特有
    pub async fn resolve_plugin_state_conflict(
        &self,
        plugin_key: &str,
        local_state: &Arc<dyn Resource>,
        remote_state: &Arc<dyn Resource>,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // 基于插件类型的不同策略
        match plugin_key {
            // 用户状态：保持最新的在线状态
            key if key.contains("user") => {
                self.resolve_user_state_conflict(local_state, remote_state, context).await
            }
            
            // 缓存状态：合并缓存项
            key if key.contains("cache") => {
                self.resolve_cache_state_conflict(local_state, remote_state, context).await
            }
            
            // 认证状态：保持更高权限
            key if key.contains("auth") => {
                self.resolve_auth_state_conflict(local_state, remote_state, context).await
            }
            
            // 默认：序列化比较
            _ => {
                self.resolve_generic_plugin_state_conflict(local_state, remote_state, context).await
            }
        }
    }
    
    /// 用户状态冲突解决
    async fn resolve_user_state_conflict(
        &self,
        local_state: &Arc<dyn Resource>,
        remote_state: &Arc<dyn Resource>,
        context: &ConflictContext,
    ) -> Result<ConflictResolution, ConflictError> {
        // 用户状态优先保持最新的活跃状态
        // 这里需要具体的用户状态结构来实现
        // 暂时使用时间戳策略
        
        let winning_state = if context.local_timestamp > context.remote_timestamp {
            local_state
        } else {
            remote_state
        };
        
        let operation = YrsOperation {
            id: Uuid::new_v4(),
            operation_type: YrsOperationType::MapSet {
                key: "user_state".to_string(),
                value: self.resource_to_yrs_any(winning_state)?,
            },
            target_path: vec!["plugin_states".to_string()],
            user_id: context.remote_user.clone(),
            timestamp: std::cmp::max(context.local_timestamp, context.remote_timestamp),
            data: serde_json::json!({"conflict_resolved": true}),
        };
        
        Ok(ConflictResolution {
            resolution_type: ResolutionType::TimestampWins,
            operations: vec![operation],
            explanation: "User state resolved by timestamp priority".to_string(),
        })
    }
}
```

### 6. 相对位置系统 - y-prosemirror 核心

```rust
/// 相对位置系统 - 参考 y-prosemirror 的位置处理
#[derive(Debug, Clone)]
pub struct RelativePosition {
    /// 相对于的节点ID
    pub relative_to: NodeId,
    /// 相对位置类型
    pub position_type: RelativePositionType,
    /// 偏移量
    pub offset: i32,
}

#[derive(Debug, Clone)]
pub enum RelativePositionType {
    /// 在指定节点之前
    Before,
    /// 在指定节点之后  
    After,
    /// 作为指定节点的第N个子节点
    ChildAt(u32),
    /// 在指定节点内的指定偏移位置
    WithinAt(u32),
}

/// 位置映射器 - 处理并发操作中的位置调整
pub struct PositionMapper {
    /// 操作历史，用于位置映射
    operation_history: Vec<YrsOperation>,
}

impl PositionMapper {
    /// 将绝对位置转换为相对位置
    pub fn absolute_to_relative(
        &self,
        absolute_pos: AbsolutePosition,
        tree: &Tree,
    ) -> Result<RelativePosition, PositionError> {
        // 找到最近的稳定锚点
        let anchor_node = self.find_stable_anchor(&absolute_pos, tree)?;
        
        let relative_pos = RelativePosition {
            relative_to: anchor_node.id.clone(),
            position_type: self.calculate_position_type(&absolute_pos, &anchor_node, tree)?,
            offset: self.calculate_offset(&absolute_pos, &anchor_node)?,
        };
        
        Ok(relative_pos)
    }
    
    /// 将相对位置转换为当前的绝对位置
    pub fn relative_to_absolute(
        &self,
        relative_pos: &RelativePosition,
        tree: &Tree,
    ) -> Result<AbsolutePosition, PositionError> {
        let anchor_node = tree.get_node(&relative_pos.relative_to)
            .ok_or(PositionError::AnchorNotFound)?;
            
        match relative_pos.position_type {
            RelativePositionType::Before => {
                self.calculate_before_position(anchor_node, tree)
            }
            RelativePositionType::After => {
                self.calculate_after_position(anchor_node, tree)
            }
            RelativePositionType::ChildAt(index) => {
                self.calculate_child_position(anchor_node, index, tree)
            }
            RelativePositionType::WithinAt(offset) => {
                self.calculate_within_position(anchor_node, offset, tree)
            }
        }
    }
    
    /// 映射位置通过操作序列
    pub fn map_position_through_operations(
        &self,
        position: RelativePosition,
        operations: &[YrsOperation],
    ) -> Result<RelativePosition, PositionError> {
        let mut current_position = position;
        
        for operation in operations {
            current_position = self.map_position_through_single_operation(
                current_position, 
                operation
            )?;
        }
        
        Ok(current_position)
    }
}
```

### 7. 协作撤销/重做支持

```rust
/// 协作环境下的撤销/重做管理器
pub struct CollaborativeUndoManager {
    /// 本地操作栈
    local_undo_stack: Vec<UndoItem>,
    /// Yrs 文档引用
    yrs_doc: Arc<yrs::Doc>,
    /// 位置映射器
    position_mapper: PositionMapper,
}

#[derive(Debug, Clone)]
pub struct UndoItem {
    /// 原始操作
    pub original_operation: YrsOperation,
    /// 逆操作（用于撤销）
    pub inverse_operation: YrsOperation,
    /// 操作时的相对位置
    pub relative_positions: Vec<RelativePosition>,
    /// 操作时间戳
    pub timestamp: u64,
}

impl CollaborativeUndoManager {
    /// 添加可撤销操作 - y-prosemirror 风格
    pub fn add_undoable_operation(
        &mut self,
        operation: YrsOperation,
        tree_before: &Tree,
        tree_after: &Tree,
    ) -> Result<(), UndoError> {
        // 生成逆操作
        let inverse_operation = self.generate_inverse_operation(&operation, tree_before)?;
        
        // 转换为相对位置
        let relative_positions = self.extract_relative_positions(&operation, tree_before)?;
        
        let undo_item = UndoItem {
            original_operation: operation,
            inverse_operation,
            relative_positions,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };
        
        self.local_undo_stack.push(undo_item);
        
        // 限制撤销栈大小
        if self.local_undo_stack.len() > 100 {
            self.local_undo_stack.remove(0);
        }
        
        Ok(())
    }
    
    /// 执行撤销 - 考虑并发修改
    pub async fn undo(&mut self, current_tree: &Tree) -> Result<UndoResult, UndoError> {
        let undo_item = self.local_undo_stack.pop()
            .ok_or(UndoError::NothingToUndo)?;
            
        // 将相对位置映射到当前状态
        let current_positions = self.position_mapper.map_positions_to_current(
            &undo_item.relative_positions,
            current_tree,
        )?;
        
        // 调整逆操作以适应当前状态
        let adjusted_inverse = self.adjust_operation_to_current_state(
            undo_item.inverse_operation,
            &current_positions,
            current_tree,
        )?;
        
        // 在 Yrs 文档中应用逆操作
        let mut txn = self.yrs_doc.transact_mut();
        self.apply_yrs_operation(&mut txn, &adjusted_inverse)?;
        txn.commit();
        
        Ok(UndoResult {
            undone_operation: undo_item.original_operation,
            applied_inverse: adjusted_inverse,
            affected_positions: current_positions,
        })
    }
}
```

### 8. 集成到 ModuForge-RS

```rust
/// ModuForge-RS 的协作冲突管理器
pub struct ModuForgeCollaborationManager {
    /// 冲突解决器
    conflict_resolver: ModuForgeConflictResolver,
    /// 撤销管理器
    undo_manager: CollaborativeUndoManager,
    /// 位置映射器
    position_mapper: PositionMapper,
    /// Yrs 文档
    yrs_doc: Arc<yrs::Doc>,
    /// 状态桥接器
    state_bridge: StateYrsBridge,
}

impl ModuForgeCollaborationManager {
    /// 处理远程更新 - 主要入口点
    pub async fn handle_remote_update(
        &mut self,
        update: &[u8],
        current_state: &State,
    ) -> Result<State, CollaborationError> {
        // 1. 应用 Yrs 更新
        let mut txn = self.yrs_doc.transact_mut();
        let yrs_update = yrs::Update::decode_v1(update)
            .map_err(|e| CollaborationError::InvalidUpdate(e.to_string()))?;
        txn.apply_update(yrs_update);
        txn.commit();
        
        // 2. 检测冲突
        let conflicts = self.detect_conflicts_with_current_state(current_state).await?;
        
        // 3. 解决冲突
        let mut resolved_state = current_state.clone();
        for conflict in conflicts {
            let resolution = self.conflict_resolver.resolve_conflict(conflict).await?;
            resolved_state = self.apply_conflict_resolution(resolved_state, resolution).await?;
        }
        
        // 4. 从 Yrs 重建完整状态
        let final_state = self.state_bridge.sync_yrs_to_state(&resolved_state).await?;
        
        Ok(final_state)
    }
    
    /// 处理本地操作 - 发送到协作环境
    pub async fn handle_local_transaction(
        &mut self,
        transaction: Transaction,
        current_state: &State,
    ) -> Result<State, CollaborationError> {
        // 1. 转换为 Yrs 操作
        let yrs_operations = self.transaction_to_yrs_operations(&transaction).await?;
        
        // 2. 记录撤销信息
        let tree_before = current_state.node_pool.get_inner();
        
        // 3. 应用到 Yrs 文档
        let mut txn = self.yrs_doc.transact_mut();
        for operation in &yrs_operations {
            self.apply_yrs_operation(&mut txn, operation)?;
        }
        txn.commit();
        
        // 4. 重建状态
        let new_state = self.state_bridge.sync_yrs_to_state(current_state).await?;
        
        // 5. 记录撤销
        let tree_after = new_state.node_pool.get_inner();
        for operation in yrs_operations {
            self.undo_manager.add_undoable_operation(
                operation, 
                tree_before, 
                tree_after
            )?;
        }
        
        Ok(new_state)
    }
}
```

## 🎯 实施优先级

### 第一阶段：基础冲突解决 (2周)
1. ✅ 实现基础冲突检测
2. ✅ 节点结构冲突解决
3. ✅ 属性合并策略
4. ✅ 时间戳优先策略

### 第二阶段：高级功能 (2-3周)
5. ✅ 相对位置系统
6. ✅ 协作撤销/重做
7. ✅ 插件状态冲突解决
8. ✅ 用户优先级系统

### 第三阶段：优化和监控 (1-2周)
9. ✅ 冲突解决性能优化
10. ✅ 冲突统计和监控
11. ✅ 调试工具

## 📊 预期效果

通过这套基于 y-prosemirror 的冲突解决机制：

1. **自动冲突解决** - 90%+ 的冲突无需人工干预
2. **意图保持** - 用户的编辑意图尽可能被保留
3. **一致性保证** - 所有客户端最终达到一致状态
4. **性能优化** - 冲突解决延迟 < 100ms
5. **可扩展性** - 支持 100+ 并发用户

这将使 ModuForge-RS 具备与现代协作编辑器相当的冲突解决能力。
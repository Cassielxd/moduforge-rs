# 简化的历史记录增强方案

## 🎯 **简单需求**

在现有的 `HistoryManager<T>` 基础上，只需要增加一些基本的元信息：
- 命令描述（如"新增商品"）
- 执行时间戳
- 执行结果（成功/失败）
- 可选的用户ID

## 🏗️ **简化设计**

### 1. **历史记录项包装器**

```rust
use std::sync::Arc;
use std::time::SystemTime;
use moduforge_state::state::State;

/// 带元信息的历史记录项
#[derive(Debug, Clone)]
pub struct HistoryEntryWithMeta {
    /// 状态快照
    pub state: Arc<State>,
    
    /// 操作描述
    pub description: String,
    
    /// 时间戳
    pub timestamp: SystemTime,
    
    /// 是否执行成功
    pub success: bool,
    
    /// 错误信息（如果有）
    pub error_message: Option<String>,
    
    /// 用户ID（可选）
    pub user_id: Option<String>,
    
    /// 版本号
    pub version: u64,
}

impl HistoryEntryWithMeta {
    pub fn new(
        state: Arc<State>,
        description: String,
        success: bool,
        error_message: Option<String>,
        user_id: Option<String>,
        version: u64,
    ) -> Self {
        Self {
            state,
            description,
            timestamp: SystemTime::now(),
            success,
            error_message,
            user_id,
            version,
        }
    }
    
    /// 创建成功的记录
    pub fn success(
        state: Arc<State>,
        description: String,
        user_id: Option<String>,
        version: u64,
    ) -> Self {
        Self::new(state, description, true, None, user_id, version)
    }
    
    /// 创建失败的记录
    pub fn failed(
        state: Arc<State>,
        description: String,
        error: String,
        user_id: Option<String>,
        version: u64,
    ) -> Self {
        Self::new(state, description, false, Some(error), user_id, version)
    }
}
```

### 2. **增强的历史管理器**

```rust
use crate::history_manager::HistoryManager;

/// 带元信息的历史管理器
pub struct SimpleEnhancedHistoryManager {
    /// 使用现有的历史管理器
    history: HistoryManager<HistoryEntryWithMeta>,
    
    /// 版本计数器
    version_counter: u64,
}

impl SimpleEnhancedHistoryManager {
    /// 创建新实例
    pub fn new(
        initial_state: Arc<State>,
        history_limit: Option<usize>,
    ) -> Self {
        let initial_entry = HistoryEntryWithMeta::success(
            initial_state,
            "初始状态".to_string(),
            None,
            0,
        );
        
        Self {
            history: HistoryManager::new(initial_entry, history_limit),
            version_counter: 0,
        }
    }
    
    /// 记录成功的操作
    pub fn record_success(
        &mut self,
        state: Arc<State>,
        description: String,
        user_id: Option<String>,
    ) {
        self.version_counter += 1;
        let entry = HistoryEntryWithMeta::success(
            state,
            description,
            user_id,
            self.version_counter,
        );
        self.history.insert(entry);
    }
    
    /// 记录失败的操作
    pub fn record_failure(
        &mut self,
        state: Arc<State>,
        description: String,
        error: String,
        user_id: Option<String>,
    ) {
        self.version_counter += 1;
        let entry = HistoryEntryWithMeta::failed(
            state,
            description,
            error,
            user_id,
            self.version_counter,
        );
        self.history.insert(entry);
    }
    
    /// 获取当前状态
    pub fn get_current_state(&self) -> Arc<State> {
        self.history.get_present().state.clone()
    }
    
    /// 获取当前记录
    pub fn get_current_entry(&self) -> &HistoryEntryWithMeta {
        self.history.get_present()
    }
    
    /// 撤销操作
    pub fn undo(&mut self) -> Option<Arc<State>> {
        self.history.jump(-1);
        Some(self.get_current_state())
    }
    
    /// 重做操作
    pub fn redo(&mut self) -> Option<Arc<State>> {
        self.history.jump(1);
        Some(self.get_current_state())
    }
    
    /// 获取最近的操作记录
    pub fn get_recent_operations(&self, count: usize) -> Vec<String> {
        // 这里需要访问历史记录，可能需要扩展HistoryManager的接口
        // 暂时返回当前操作的描述
        vec![self.get_current_entry().description.clone()]
    }
    
    /// 获取简单统计
    pub fn get_stats(&self) -> (usize, usize) {
        // 返回 (成功次数, 失败次数)
        // 这里需要遍历历史记录来统计，暂时返回简单值
        (self.version_counter as usize, 0)
    }
}
```

### 3. **集成到AsyncEditor**

```rust
// 在 AsyncEditor 中添加字段
impl AsyncEditor {
    /// 带描述的命令执行
    pub async fn command_with_description(
        &mut self,
        command: Arc<dyn Command>,
        description: String,
        user_id: Option<String>,
    ) -> EditorResult<()> {
        let cmd_name = command.name();
        debug!("正在执行命令: {} ({})", cmd_name, description);

        // 创建事务并应用命令
        let mut tr = self.get_tr();
        
        match command.execute(&mut tr).await {
            Ok(()) => {
                tr.commit();
                
                // 使用高性能处理引擎处理事务
                match self.dispatch_flow(tr).await {
                    Ok(_) => {
                        // 记录成功的操作
                        if let Some(ref mut enhanced_history) = self.enhanced_history {
                            enhanced_history.record_success(
                                self.get_state(),
                                description,
                                user_id,
                            );
                        }
                        
                        debug!("命令 '{}' 执行成功", cmd_name);
                        Ok(())
                    }
                    Err(e) => {
                        // 记录失败的操作
                        if let Some(ref mut enhanced_history) = self.enhanced_history {
                            enhanced_history.record_failure(
                                self.get_state(),
                                description,
                                e.to_string(),
                                user_id,
                            );
                        }
                        
                        debug!("命令 '{}' 执行失败: {}", cmd_name, e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // 记录失败的操作
                if let Some(ref mut enhanced_history) = self.enhanced_history {
                    enhanced_history.record_failure(
                        self.get_state(),
                        description,
                        e.to_string(),
                        user_id,
                    );
                }
                
                debug!("命令执行失败: {}", e);
                Err(e.into())
            }
        }
    }
    
    /// 获取操作历史摘要
    pub fn get_operation_history(&self) -> Vec<String> {
        if let Some(ref enhanced_history) = self.enhanced_history {
            enhanced_history.get_recent_operations(10)
        } else {
            vec![]
        }
    }
    
    /// 带描述的撤销
    pub fn undo_with_info(&mut self) -> Option<(Arc<State>, String)> {
        if let Some(ref mut enhanced_history) = self.enhanced_history {
            let current_desc = enhanced_history.get_current_entry().description.clone();
            if let Some(state) = enhanced_history.undo() {
                return Some((state, format!("撤销: {}", current_desc)));
            }
        }
        None
    }
    
    /// 带描述的重做
    pub fn redo_with_info(&mut self) -> Option<(Arc<State>, String)> {
        if let Some(ref mut enhanced_history) = self.enhanced_history {
            if let Some(state) = enhanced_history.redo() {
                let current_desc = enhanced_history.get_current_entry().description.clone();
                return Some((state, format!("重做: {}", current_desc)));
            }
        }
        None
    }
}
```

## 🚀 **使用示例**

### 1. **新增商品示例**

```rust
// 简单的命令执行带描述
async fn add_product_example(editor: &mut AsyncEditor) -> EditorResult<()> {
    let command = Arc::new(AddProductCommand {
        product_name: "iPhone 15".to_string(),
        price: 999.99,
        category: "电子产品".to_string(),
    });

    // 执行命令并记录描述
    editor.command_with_description(
        command,
        "新增商品：iPhone 15".to_string(),
        Some("user_123".to_string()),
    ).await?;

    Ok(())
}
```

### 2. **查看操作历史**

```rust
// 查看最近的操作
fn show_recent_operations(editor: &AsyncEditor) {
    let history = editor.get_operation_history();
    println!("最近操作:");
    for (i, op) in history.iter().enumerate() {
        println!("{}. {}", i + 1, op);
    }
}
```

### 3. **撤销/重做带提示**

```rust
// 撤销操作
fn undo_operation(editor: &mut AsyncEditor) {
    if let Some((state, description)) = editor.undo_with_info() {
        println!("已执行: {}", description);
        // 更新UI状态...
    } else {
        println!("没有可撤销的操作");
    }
}

// 重做操作
fn redo_operation(editor: &mut AsyncEditor) {
    if let Some((state, description)) = editor.redo_with_info() {
        println!("已执行: {}", description);
        // 更新UI状态...
    } else {
        println!("没有可重做的操作");
    }
}
```

## 📋 **总结**

这个简化方案只需要：

1. **一个包装结构** `HistoryEntryWithMeta` - 包含状态和基本元信息
2. **一个简单的管理器** `SimpleEnhancedHistoryManager` - 复用现有的HistoryManager
3. **几个辅助方法** - 在AsyncEditor中添加带描述的命令执行方法

**优点**：
- ✅ 最小改动：复用现有的HistoryManager
- ✅ 简单易用：只增加必要的元信息
- ✅ 轻量级：没有复杂的搜索和索引功能
- ✅ 向后兼容：不影响现有代码

这样你就能在执行每个命令时记录"新增商品"这样的描述信息，同时保持系统的简洁性。 
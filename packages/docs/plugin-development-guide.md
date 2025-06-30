# ModuForge-RS 插件开发完整指南

## 概述

ModuForge-RS 的插件系统基于三个核心组件：
- **Resource**: 插件状态数据
- **StateField**: 状态管理器
- **PluginTrait**: 插件行为定义

## 1. 基础插件结构

### 第一步：定义插件状态资源

```rust
use moduforge_state::resource::Resource;

#[derive(Debug, Clone)]
pub struct MyPluginState {
    pub counter: u64,
    pub settings: im::HashMap<String, String>,
    pub active: bool,
}

impl Resource for MyPluginState {}

impl MyPluginState {
    pub fn new() -> Self {
        Self {
            counter: 0,
            settings: im::HashMap::new(),
            active: true,
        }
    }
    
    pub fn increment(&mut self) {
        self.counter += 1;
    }
    
    pub fn set_setting(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }
}
```

### 第二步：实现状态字段管理器

```rust
use moduforge_state::{
    plugin::StateField,
    resource::Resource,
    state::{State, StateConfig},
    transaction::Transaction,
};
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Debug)]
pub struct MyStateField;

#[async_trait]
impl StateField for MyStateField {
    // 初始化插件状态
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("🔧 初始化我的插件状态");
        Arc::new(MyPluginState::new())
    }

    // 处理状态变更
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        // 尝试向下转型为具体的状态类型
        if let Ok(state) = value.clone().downcast::<MyPluginState>() {
            let mut new_state = (*state).clone();
            
            // 根据事务元数据更新状态
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "increment_counter" => {
                        new_state.increment();
                        println!("📈 计数器更新: {}", new_state.counter);
                    }
                    "set_plugin_setting" => {
                        if let Some(key) = tr.get_meta::<String>("setting_key") {
                            if let Some(val) = tr.get_meta::<String>("setting_value") {
                                new_state.set_setting(
                                    key.as_str().to_string(), 
                                    val.as_str().to_string()
                                );
                                println!("⚙️ 设置更新: {} = {}", key.as_str(), val.as_str());
                            }
                        }
                    }
                    "toggle_plugin" => {
                        new_state.active = !new_state.active;
                        println!("🔄 插件状态: {}", if new_state.active { "激活" } else { "停用" });
                    }
                    _ => {}
                }
            }
            
            Arc::new(new_state)
        } else {
            // 如果类型转换失败，返回原状态
            value
        }
    }

    // 可选：序列化状态
    async fn serialize(&self, value: Arc<dyn Resource>) -> Option<Vec<u8>> {
        if let Ok(state) = value.downcast::<MyPluginState>() {
            serde_json::to_vec(&*state).ok()
        } else {
            None
        }
    }
    
    // 可选：反序列化状态
    async fn deserialize(&self, data: &[u8]) -> Option<Arc<dyn Resource>> {
        serde_json::from_slice::<MyPluginState>(data)
            .ok()
            .map(|state| Arc::new(state) as Arc<dyn Resource>)
    }
}
```

### 第三步：实现插件行为

```rust
use moduforge_state::{
    plugin::PluginTrait,
    transaction::Transaction,
    state::State,
    error::StateResult,
};
use async_trait::async_trait;

#[derive(Debug)]
pub struct MyPlugin;

#[async_trait]
impl PluginTrait for MyPlugin {
    // 事务后处理：生成额外的事务
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 检查传入的事务
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "document_created" => {
                        // 当创建文档时，自动增加计数器
                        let mut counter_tr = Transaction::new();
                        counter_tr.set_meta("generated_by", "my_plugin");
                        counter_tr.set_meta("action", "increment_counter");
                        counter_tr.set_meta("reason", "document_created");
                        
                        println!("📄 检测到文档创建，自动增加计数器");
                        return Ok(Some(counter_tr));
                    }
                    "user_login" => {
                        // 用户登录时，记录设置
                        if let Some(username) = tr.get_meta::<String>("username") {
                            let mut setting_tr = Transaction::new();
                            setting_tr.set_meta("generated_by", "my_plugin");
                            setting_tr.set_meta("action", "set_plugin_setting");
                            setting_tr.set_meta("setting_key", "last_user");
                            setting_tr.set_meta("setting_value", username.as_str());
                            
                            println!("👤 用户登录，记录最后用户: {}", username.as_str());
                            return Ok(Some(setting_tr));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(None)
    }

    // 事务过滤：决定是否允许事务执行
    async fn filter_transaction(
        &self,
        transaction: &Transaction,
        state: &State,
    ) -> bool {
        // 获取插件状态
        if let Some(plugin_state) = state.get_resource::<MyPluginState>("my_plugin") {
            // 如果插件未激活，拒绝某些操作
            if !plugin_state.active {
                if let Some(action) = transaction.get_meta::<String>("action") {
                    if action.as_str() == "sensitive_operation" {
                        println!("❌ 插件未激活，拒绝敏感操作");
                        return false;
                    }
                }
            }
        }
        
        // 拒绝危险操作
        if let Some(action) = transaction.get_meta::<String>("action") {
            if action.as_str() == "dangerous_operation" {
                println!("⚠️ 拒绝危险操作");
                return false;
            }
        }
        
        true
    }
}
```

### 第四步：创建完整的插件

```rust
use moduforge_core::extension::Extension;
use moduforge_state::plugin::{Plugin, PluginSpec};
use std::sync::Arc;

/// 创建我的插件扩展
pub fn create_my_plugin_extension() -> Extension {
    let mut extension = Extension::new();
    
    // 创建插件
    let plugin = Plugin::new(PluginSpec {
        key: ("my_plugin".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(MyStateField)),
        tr: Some(Arc::new(MyPlugin)),
        priority: 10,
    });
    
    extension.add_plugin(Arc::new(plugin));
    extension
}
```

## 2. 高级插件模式

### 缓存插件示例

```rust
use std::collections::HashMap;
use lru::LruCache;
use std::num::NonZeroUsize;

#[derive(Debug, Clone)]
pub struct CachePluginState {
    pub cache: LruCache<String, serde_json::Value>,
    pub hit_count: u64,
    pub miss_count: u64,
}

impl Resource for CachePluginState {}

impl CachePluginState {
    pub fn new() -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
            hit_count: 0,
            miss_count: 0,
        }
    }
    
    pub fn get(&mut self, key: &str) -> Option<&serde_json::Value> {
        if let Some(value) = self.cache.get(key) {
            self.hit_count += 1;
            Some(value)
        } else {
            self.miss_count += 1;
            None
        }
    }
    
    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.cache.put(key, value);
    }
    
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

#[derive(Debug)]
pub struct CacheStateField;

#[async_trait]
impl StateField for CacheStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        Arc::new(CachePluginState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Ok(state) = value.clone().downcast::<CachePluginState>() {
            let mut new_state = (*state).clone();
            
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "cache_set" => {
                        if let Some(key) = tr.get_meta::<String>("cache_key") {
                            if let Some(value) = tr.get_meta::<serde_json::Value>("cache_value") {
                                new_state.set(key.as_str().to_string(), value.clone());
                                println!("💾 缓存设置: {}", key.as_str());
                            }
                        }
                    }
                    "cache_clear" => {
                        new_state.cache.clear();
                        println!("🗑️ 缓存已清空");
                    }
                    _ => {}
                }
            }
            
            Arc::new(new_state)
        } else {
            value
        }
    }
}

#[derive(Debug)]
pub struct CachePlugin;

#[async_trait]
impl PluginTrait for CachePlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 检查是否需要缓存数据
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                if action.as_str() == "expensive_computation" {
                    if let Some(result) = tr.get_meta::<serde_json::Value>("result") {
                        if let Some(cache_key) = tr.get_meta::<String>("cache_key") {
                            let mut cache_tr = Transaction::new();
                            cache_tr.set_meta("action", "cache_set");
                            cache_tr.set_meta("cache_key", cache_key.as_str());
                            cache_tr.set_meta("cache_value", result);
                            cache_tr.set_meta("generated_by", "cache_plugin");
                            
                            return Ok(Some(cache_tr));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        _transaction: &Transaction,
        _state: &State,
    ) -> bool {
        true
    }
}
```

### 权限控制插件示例

```rust
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PermissionState {
    pub user_permissions: im::HashMap<String, HashSet<String>>,
    pub role_permissions: im::HashMap<String, HashSet<String>>,
    pub user_roles: im::HashMap<String, HashSet<String>>,
}

impl Resource for PermissionState {}

impl PermissionState {
    pub fn new() -> Self {
        Self {
            user_permissions: im::HashMap::new(),
            role_permissions: im::HashMap::new(),
            user_roles: im::HashMap::new(),
        }
    }
    
    pub fn has_permission(&self, user_id: &str, permission: &str) -> bool {
        // 检查直接权限
        if let Some(permissions) = self.user_permissions.get(user_id) {
            if permissions.contains(permission) {
                return true;
            }
        }
        
        // 检查角色权限
        if let Some(roles) = self.user_roles.get(user_id) {
            for role in roles {
                if let Some(role_perms) = self.role_permissions.get(role) {
                    if role_perms.contains(permission) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
}

#[derive(Debug)]
pub struct PermissionPlugin;

#[async_trait]
impl PluginTrait for PermissionPlugin {
    async fn append_transaction(
        &self,
        _transactions: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        transaction: &Transaction,
        state: &State,
    ) -> bool {
        // 检查权限
        if let Some(user_id) = transaction.get_meta::<String>("user_id") {
            if let Some(required_permission) = transaction.get_meta::<String>("required_permission") {
                if let Some(perm_state) = state.get_resource::<PermissionState>("permissions") {
                    if !perm_state.has_permission(user_id.as_str(), required_permission.as_str()) {
                        println!("🚫 用户 {} 没有权限 {}", user_id.as_str(), required_permission.as_str());
                        return false;
                    }
                }
            }
        }
        
        true
    }
}
```

## 3. 插件测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use moduforge_state::{State, StateConfig};
    use tokio_test;

    #[tokio::test]
    async fn test_plugin_state_initialization() {
        let state_field = MyStateField;
        let config = StateConfig::default();
        
        let resource = state_field.init(&config, None).await;
        
        // 验证状态正确初始化
        let plugin_state = resource.downcast::<MyPluginState>().unwrap();
        assert_eq!(plugin_state.counter, 0);
        assert!(plugin_state.active);
    }

    #[tokio::test]
    async fn test_plugin_state_update() {
        let state_field = MyStateField;
        let config = StateConfig::default();
        
        let initial_resource = state_field.init(&config, None).await;
        
        // 创建事务
        let mut transaction = Transaction::new();
        transaction.set_meta("action", "increment_counter");
        
        // 应用事务
        let state = State::new(StateConfig::default()).await.unwrap();
        let updated_resource = state_field.apply(
            &transaction,
            initial_resource,
            &state,
            &state
        ).await;
        
        // 验证状态更新
        let plugin_state = updated_resource.downcast::<MyPluginState>().unwrap();
        assert_eq!(plugin_state.counter, 1);
    }

    #[tokio::test]
    async fn test_plugin_transaction_filter() {
        let plugin = MyPlugin;
        let state = State::new(StateConfig::default()).await.unwrap();
        
        // 测试允许的事务
        let mut allowed_tr = Transaction::new();
        allowed_tr.set_meta("action", "safe_operation");
        assert!(plugin.filter_transaction(&allowed_tr, &state).await);
        
        // 测试拒绝的事务
        let mut denied_tr = Transaction::new();
        denied_tr.set_meta("action", "dangerous_operation");
        assert!(!plugin.filter_transaction(&denied_tr, &state).await);
    }
}
```

### 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use moduforge_core::{async_runtime::AsyncRuntime, types::RuntimeOptions};
    use moduforge_state::StateConfig;

    #[tokio::test]
    async fn test_plugin_integration() {
        // 创建带插件的运行时
        let mut options = RuntimeOptions::default();
        options.add_extension(moduforge_core::types::Extensions::E(create_my_plugin_extension()));
        
        let state_config = StateConfig::default();
        let mut runtime = AsyncRuntime::new(options, state_config).await.unwrap();
        
        // 测试插件功能
        let mut transaction = Transaction::new();
        transaction.set_meta("action", "document_created");
        transaction.set_meta("document_id", "test_doc");
        
        // 应用事务
        runtime.apply_transaction(transaction).await.unwrap();
        
        // 验证插件生成的附加事务被执行
        // 这里需要根据具体的状态检查逻辑
    }
}
```

## 4. 插件最佳实践

### 性能优化

```rust
// 使用不可变数据结构避免不必要的克隆
impl MyStateField {
    async fn apply(&self, tr: &Transaction, value: Arc<dyn Resource>, _old_state: &State, _new_state: &State) -> Arc<dyn Resource> {
        if let Ok(state) = value.clone().downcast::<MyPluginState>() {
            // 只在需要修改时才克隆
            if self.needs_update(tr) {
                let mut new_state = (*state).clone();
                self.update_state(&mut new_state, tr);
                Arc::new(new_state)
            } else {
                // 返回原始状态，避免克隆
                value
            }
        } else {
            value
        }
    }
    
    fn needs_update(&self, tr: &Transaction) -> bool {
        tr.get_meta::<String>("action")
            .map(|action| matches!(action.as_str(), "increment_counter" | "set_plugin_setting" | "toggle_plugin"))
            .unwrap_or(false)
    }
    
    fn update_state(&self, state: &mut MyPluginState, tr: &Transaction) {
        // 具体的更新逻辑
    }
}
```

### 错误处理

```rust
use moduforge_state::error::{StateError, StateResult};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("插件未初始化")]
    NotInitialized,
    #[error("无效的配置: {0}")]
    InvalidConfig(String),
    #[error("权限不足: {0}")]
    PermissionDenied(String),
}

impl PluginTrait for MyPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            // 验证事务
            if let Err(e) = self.validate_transaction(tr) {
                return Err(StateError::Plugin(format!("事务验证失败: {}", e)));
            }
            
            // 处理事务...
        }
        
        Ok(None)
    }
    
    fn validate_transaction(&self, tr: &Transaction) -> Result<(), PluginError> {
        if let Some(action) = tr.get_meta::<String>("action") {
            if action.as_str().is_empty() {
                return Err(PluginError::InvalidConfig("动作不能为空".to_string()));
            }
        }
        Ok(())
    }
}
```

### 日志和监控

```rust
use tracing::{info, warn, error, debug};

#[async_trait]
impl StateField for MyStateField {
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        debug!("插件状态更新开始: {:?}", tr.id);
        
        if let Ok(state) = value.clone().downcast::<MyPluginState>() {
            let mut new_state = (*state).clone();
            
            if let Some(action) = tr.get_meta::<String>("action") {
                info!("执行插件动作: {}", action.as_str());
                
                match action.as_str() {
                    "increment_counter" => {
                        let old_count = new_state.counter;
                        new_state.increment();
                        info!("计数器更新: {} -> {}", old_count, new_state.counter);
                    }
                    unknown_action => {
                        warn!("未知的插件动作: {}", unknown_action);
                    }
                }
            }
            
            debug!("插件状态更新完成");
            Arc::new(new_state)
        } else {
            error!("插件状态类型转换失败");
            value
        }
    }
}
```

## 5. 插件配置和管理

### 配置文件支持

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub priority: i32,
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority: 10,
            settings: std::collections::HashMap::new(),
        }
    }
}

impl MyStateField {
    pub fn from_config(config: &PluginConfig) -> Self {
        // 根据配置创建状态字段
        Self
    }
}
```

### 动态插件管理

```rust
pub struct PluginManager {
    plugins: im::HashMap<String, Arc<dyn PluginTrait>>,
    configs: im::HashMap<String, PluginConfig>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: im::HashMap::new(),
            configs: im::HashMap::new(),
        }
    }
    
    pub fn register_plugin(&mut self, name: String, plugin: Arc<dyn PluginTrait>, config: PluginConfig) {
        if config.enabled {
            self.plugins.insert(name.clone(), plugin);
            self.configs.insert(name, config);
        }
    }
    
    pub fn unregister_plugin(&mut self, name: &str) {
        self.plugins.remove(name);
        self.configs.remove(name);
    }
    
    pub fn get_plugin(&self, name: &str) -> Option<&Arc<dyn PluginTrait>> {
        self.plugins.get(name)
    }
}
```

这个完整的插件开发指南提供了：

1. **基础插件结构**：从状态定义到完整插件创建的完整流程
2. **高级模式**：缓存、权限控制等实用插件示例
3. **测试策略**：单元测试和集成测试的完整覆盖
4. **最佳实践**：性能优化、错误处理、日志监控
5. **配置管理**：插件配置和动态管理机制

通过这个指南，开发者可以创建功能完整、性能良好、易于维护的 ModuForge-RS 插件。 
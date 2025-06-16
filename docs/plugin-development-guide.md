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
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct MyPluginState {
    pub counter: u64,
    pub settings: im::HashMap<String, String>,
    pub active: bool,
}

impl Resource for MyPluginState {
    fn name(&self) -> Cow<str> {
        "MyPluginState".into()
    }
}

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
        if let Some(state) = value.downcast_arc::<MyPluginState>() {
            let mut new_state = (**state).clone();
            
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
    fn serialize(&self, value: Arc<dyn Resource>) -> Option<Vec<u8>> {
        if let Some(state) = value.downcast_arc::<MyPluginState>() {
            serde_json::to_vec(&**state).ok()
        } else {
            None
        }
    }
    
    // 可选：反序列化状态
    fn deserialize(&self, data: &Vec<u8>) -> Option<Arc<dyn Resource>> {
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
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 检查传入的事务
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "document_created" => {
                        // 当创建文档时，自动增加计数器
                        let mut counter_tr = Transaction::new(new_state);
                        counter_tr.set_meta("generated_by", "my_plugin");
                        counter_tr.set_meta("action", "increment_counter");
                        counter_tr.set_meta("reason", "document_created");
                        
                        println!("📄 检测到文档创建，自动增加计数器");
                        return Ok(Some(counter_tr));
                    }
                    "user_login" => {
                        // 用户登录时，记录设置
                        if let Some(username) = tr.get_meta::<String>("username") {
                            let mut setting_tr = Transaction::new(new_state);
                            setting_tr.set_meta("generated_by", "my_plugin");
                            setting_tr.set_meta("action", "set_plugin_setting");
                            setting_tr.set_meta("setting_key", "last_user");
                            setting_tr.set_meta("setting_value", username.as_ptr().clone());
                            
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
        if let Some(plugin_state) = self.get_plugin_state(state) {
            // 如果插件被停用，拒绝某些操作
            if !plugin_state.active {
                if let Some(action) = transaction.get_meta::<String>("action") {
                    match action.as_str() {
                        "sensitive_operation" => {
                            println!("🚫 插件已停用，拒绝敏感操作");
                            return false;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // 默认允许所有事务
        true
    }
}

impl MyPlugin {
    // 辅助方法：获取插件状态
    fn get_plugin_state(&self, state: &State) -> Option<Arc<MyPluginState>> {
        state.get_field("my_plugin.v1")
            .and_then(|resource| resource.downcast_arc::<MyPluginState>())
    }
}
```

### 第四步：组装插件

```rust
use moduforge_core::{extension::Extension, types::Extensions};
use moduforge_state::plugin::{Plugin, PluginSpec};
use std::sync::Arc;

pub fn create_my_plugin_extension() -> Extension {
    let mut extension = Extension::new();
    
    // 创建插件规格
    let plugin_spec = PluginSpec {
        key: ("my_plugin".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(MyStateField)),
        tr: Some(Arc::new(MyPlugin)),
        priority: 10, // 优先级：数字越小优先级越高
    };
    
    // 创建插件实例
    let plugin = Plugin::new(plugin_spec);
    
    // 添加到扩展
    extension.add_plugin(Arc::new(plugin));
    
    extension
}
```

## 2. 在编辑器中使用插件

```rust
use moduforge_core::{
    RuntimeResult,
    runtime::Runtime,
    types::{RuntimeOptionsBuilder, Extensions},
};

async fn create_runtime_with_my_plugin() -> RuntimeResult<Runtime> {
    let options = RuntimeOptionsBuilder::new()
        .add_extension(Extensions::E(create_my_plugin_extension()))
        .history_limit(100)
        .build();
        
    let runtime = Runtime::create(options).await?;
    Ok(runtime)
}

// 使用示例
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    moduforge_state::init_logging("info", None)?;
    
    // 创建编辑器
    let mut runtime = create_runtime_with_my_plugin().await?;
    
    // 触发插件行为的事务
    let mut tr = runtime.get_tr();
    tr.set_meta("action", "document_created");
    tr.set_meta("document_title", "测试文档");
    runtime.dispatch(tr).await?;
    
    // 手动触发计数器增加
    let mut tr2 = runtime.get_tr();
    tr2.set_meta("action", "increment_counter");
    runtime.dispatch(tr2).await?;
    
    // 设置插件配置
    let mut tr3 = runtime.get_tr();
    tr3.set_meta("action", "set_plugin_setting");
    tr3.set_meta("setting_key", "theme");
    tr3.set_meta("setting_value", "dark");
    runtime.dispatch(tr3).await?;
    
    println!("✅ 插件演示完成");
    Ok(())
}
```

## 3. 高级插件模式

### 插件间通信

```rust
#[derive(Debug)]
pub struct CommunicationPlugin;

#[async_trait]
impl PluginTrait for CommunicationPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(sender) = tr.get_meta::<String>("sender_plugin") {
                if sender.as_str() == "my_plugin" {
                    // 响应来自其他插件的消息
                    let mut response_tr = Transaction::new(new_state);
                    response_tr.set_meta("generated_by", "communication_plugin");
                    response_tr.set_meta("action", "plugin_message_received");
                    response_tr.set_meta("from", sender.as_ptr().clone());
                    
                    return Ok(Some(response_tr));
                }
            }
        }
        Ok(None)
    }
}
```

### 资源共享插件

```rust
#[derive(Debug)]
pub struct ResourceSharingPlugin;

impl ResourceSharingPlugin {
    // 向全局资源管理器添加共享资源
    pub fn setup_shared_resources(
        resource_manager: &moduforge_state::ops::GlobalResourceManager
    ) -> moduforge_core::RuntimeResult<()> {
        // 添加共享缓存
        let shared_cache = MySharedCache::new();
        resource_manager.resource_table.add(shared_cache);
        
        // 添加配置管理器
        let config_manager = ConfigManager::new();
        resource_manager.resource_table.add(config_manager);
        
        Ok(())
    }
}

// 在扩展中添加资源设置函数
pub fn create_resource_sharing_extension() -> Extension {
    let mut extension = Extension::new();
    
    // 添加资源设置操作
    extension.add_op_fn(Arc::new(|resource_manager| {
        ResourceSharingPlugin::setup_shared_resources(resource_manager)
    }));
    
    // 添加插件...
    
    extension
}
```

## 4. 测试插件

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use moduforge_core::{runtime::Runtime, types::RuntimeOptionsBuilder};
    
    #[tokio::test]
    async fn test_plugin_counter() {
        let mut runtime = create_runtime_with_my_plugin().await.unwrap();
        
        // 触发计数器增加
        let mut tr = runtime.get_tr();
        tr.set_meta("action", "increment_counter");
        runtime.dispatch(tr).await.unwrap();
        
        // 验证状态
        let state = runtime.get_state();
        let plugin_state = state.get_field("my_plugin.v1")
            .unwrap()
            .downcast_arc::<MyPluginState>()
            .unwrap();
            
        assert_eq!(plugin_state.counter, 1);
    }
    
    #[tokio::test]
    async fn test_plugin_settings() {
        let mut runtime = create_runtime_with_my_plugin().await.unwrap();
        
        // 设置配置
        let mut tr = runtime.get_tr();
        tr.set_meta("action", "set_plugin_setting");
        tr.set_meta("setting_key", "test_key");
        tr.set_meta("setting_value", "test_value");
        runtime.dispatch(tr).await.unwrap();
        
        // 验证设置
        let state = runtime.get_state();
        let plugin_state = state.get_field("my_plugin.v1")
            .unwrap()
            .downcast_arc::<MyPluginState>()
            .unwrap();
            
        assert_eq!(
            plugin_state.settings.get("test_key").unwrap(),
            "test_value"
        );
    }
}
```

## 5. 最佳实践

### 错误处理
```rust
#[async_trait]
impl StateField for MyStateField {
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(state) = value.downcast_arc::<MyPluginState>() {
            let mut new_state = (**state).clone();
            
            // 安全的元数据访问
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "risky_operation" => {
                        // 添加错误检查
                        if let Some(param) = tr.get_meta::<i32>("param") {
                            if **param > 0 {
                                new_state.counter += **param as u64;
                            } else {
                                eprintln!("⚠️ 无效参数: {}", **param);
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            Arc::new(new_state)
        } else {
            // 始终返回有效状态
            eprintln!("❌ 状态类型转换失败");
            value
        }
    }
}
```

### 性能优化
```rust
// 使用 lazy_static 缓存重复计算
lazy_static::lazy_static! {
    static ref PLUGIN_CONFIG: im::HashMap<String, String> = {
        let mut config = im::HashMap::new();
        config.insert("cache_size".to_string(), "1000".to_string());
        config.insert("timeout".to_string(), "5000".to_string());
        config
    };
}

// 减少克隆操作
#[async_trait]
impl StateField for OptimizedStateField {
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(state) = value.downcast_arc::<MyPluginState>() {
            // 只在需要时才克隆
            if tr.get_meta::<String>("action").is_some() {
                let mut new_state = (**state).clone();
                // 处理变更...
                Arc::new(new_state)
            } else {
                // 无变更时直接返回原状态
                value
            }
        } else {
            value
        }
    }
}
```

这个完整的插件开发指南提供了从基础到高级的所有必要信息，让开发者能够成功创建和使用 ModuForge-RS 插件。 
# A业务依赖B业务的可插拔架构设计

## 1. 架构设计原则

### 1.1 解耦设计
- A业务插件不直接依赖B业务插件
- 通过事件系统和资源管理器进行间接通信
- B业务插件完全可插拔，不影响A业务基本功能

### 1.2 依赖管理
- 使用依赖管理器统一处理业务依赖关系
- 支持动态依赖注册和解除
- 支持依赖优先级管理

## 2. 核心组件设计

### 2.1 业务依赖管理器 (BusinessDependencyManager)

```rust
use std::{collections::HashMap, sync::Arc};
use async_trait::async_trait;
use mf_state::{resource::Resource, plugin::{Plugin, PluginSpec}};
use mf_core::{event::Event, async_processor::AsyncProcessor};

/// 业务依赖描述
#[derive(Debug, Clone)]
pub struct BusinessDependency {
    pub dependent: String,        // 依赖方 (A业务)
    pub dependency: String,       // 被依赖方 (B业务)
    pub dependency_type: DependencyType,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub enum DependencyType {
    Compute,      // 计算依赖
    Data,         // 数据依赖
    Event,        // 事件依赖
}

/// 业务依赖管理器
#[derive(Debug)]
pub struct BusinessDependencyManager {
    dependencies: HashMap<String, Vec<BusinessDependency>>,
    compute_results: HashMap<String, Arc<dyn Resource>>,
    event_subscribers: HashMap<String, Vec<String>>,
}

impl Resource for BusinessDependencyManager {
    fn name(&self) -> std::borrow::Cow<str> {
        "BusinessDependencyManager".into()
    }
}

impl BusinessDependencyManager {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            compute_results: HashMap::new(),
            event_subscribers: HashMap::new(),
        }
    }

    /// 注册业务依赖关系
    pub fn register_dependency(&mut self, dependency: BusinessDependency) {
        self.dependencies
            .entry(dependency.dependent.clone())
            .or_insert_with(Vec::new)
            .push(dependency);
    }

    /// 检查依赖是否满足
    pub fn check_dependencies(&self, business: &str) -> Vec<String> {
        self.dependencies
            .get(business)
            .map(|deps| {
                deps.iter()
                    .filter(|dep| !self.compute_results.contains_key(&dep.dependency))
                    .map(|dep| dep.dependency.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 设置计算结果
    pub fn set_compute_result(&mut self, business: &str, result: Arc<dyn Resource>) {
        self.compute_results.insert(business.to_string(), result);
    }

    /// 获取计算结果
    pub fn get_compute_result(&self, business: &str) -> Option<Arc<dyn Resource>> {
        self.compute_results.get(business).cloned()
    }

    /// 获取依赖顺序
    pub fn get_execution_order(&self) -> Vec<String> {
        // 简单的拓扑排序实现
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        for business in self.dependencies.keys() {
            self.dfs_visit(business, &mut visited, &mut result);
        }
        
        result
    }

    fn dfs_visit(&self, business: &str, visited: &mut std::collections::HashSet<String>, result: &mut Vec<String>) {
        if visited.contains(business) {
            return;
        }
        
        visited.insert(business.to_string());
        
        if let Some(deps) = self.dependencies.get(business) {
            for dep in deps {
                self.dfs_visit(&dep.dependency, visited, result);
            }
        }
        
        result.push(business.to_string());
    }
}
```

### 2.2 A业务插件实现

```rust
use std::{collections::HashMap, sync::Arc};
use async_trait::async_trait;
use mf_state::{
    resource::Resource,
    plugin::{Plugin, PluginSpec, PluginTrait, StateField},
    state::{State, StateConfig},
    transaction::Transaction,
    error::StateResult,
};

/// A业务状态
#[derive(Debug)]
pub struct ABusinessState {
    pub data: HashMap<String, String>,
    pub compute_result: Option<String>,
    pub dependencies_ready: bool,
}

impl Resource for ABusinessState {
    fn name(&self) -> std::borrow::Cow<str> {
        "ABusinessState".into()
    }
}

/// A业务插件
#[derive(Debug)]
pub struct ABusinessPlugin;

#[async_trait]
impl PluginTrait for ABusinessPlugin {
    async fn append_transaction(
        &self,
        trs: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 检查是否需要等待B业务完成
        let resource_manager = new_state.resource_manager();
        let resource_manager = resource_manager.read().unwrap();
        
        if let Some(dep_manager) = resource_manager.resource_table.get::<BusinessDependencyManager>(0) {
            let unmet_deps = dep_manager.check_dependencies("a_business");
            if !unmet_deps.is_empty() {
                // 依赖未满足，等待
                tracing::info!("A业务等待依赖: {:?}", unmet_deps);
                return Ok(None);
            }
        }

        // 依赖已满足，执行A业务计算
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        tr: &Transaction,
        state: &State,
    ) -> bool {
        // 检查A业务是否应该执行
        let resource_manager = state.resource_manager();
        let resource_manager = resource_manager.read().unwrap();
        
        if let Some(dep_manager) = resource_manager.resource_table.get::<BusinessDependencyManager>(0) {
            let unmet_deps = dep_manager.check_dependencies("a_business");
            return unmet_deps.is_empty();
        }
        
        true
    }
}

/// A业务状态字段
#[derive(Debug)]
pub struct ABusinessStateField;

#[async_trait]
impl StateField for ABusinessStateField {
    async fn init(
        &self,
        config: &StateConfig,
        instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        // 注册依赖关系
        if let Some(state) = instance {
            let resource_manager = state.resource_manager();
            let mut resource_manager = resource_manager.write().unwrap();
            
            if let Some(dep_manager) = resource_manager.resource_table.get_mut::<BusinessDependencyManager>(0) {
                dep_manager.register_dependency(BusinessDependency {
                    dependent: "a_business".to_string(),
                    dependency: "b_business".to_string(),
                    dependency_type: DependencyType::Compute,
                    priority: 1,
                });
            }
        }

        Arc::new(ABusinessState {
            data: HashMap::new(),
            compute_result: None,
            dependencies_ready: false,
        })
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut a_state = value.downcast_arc::<ABusinessState>().unwrap().as_ref().clone();
        
        // 检查B业务计算结果
        let resource_manager = new_state.resource_manager();
        let resource_manager = resource_manager.read().unwrap();
        
        if let Some(dep_manager) = resource_manager.resource_table.get::<BusinessDependencyManager>(0) {
            if let Some(b_result) = dep_manager.get_compute_result("b_business") {
                // 使用B业务结果进行A业务计算
                a_state.compute_result = Some(format!("A computed with B result: {:?}", b_result.name()));
                a_state.dependencies_ready = true;
                
                tracing::info!("A业务计算完成: {:?}", a_state.compute_result);
            }
        }

        Arc::new(a_state)
    }
}
```

### 2.3 B业务插件实现 (可插拔)

```rust
/// B业务状态
#[derive(Debug)]
pub struct BBusinessState {
    pub data: HashMap<String, String>,
    pub compute_result: Option<String>,
}

impl Resource for BBusinessState {
    fn name(&self) -> std::borrow::Cow<str> {
        "BBusinessState".into()
    }
}

/// B业务插件 (可插拔)
#[derive(Debug)]
pub struct BBusinessPlugin;

#[async_trait]
impl PluginTrait for BBusinessPlugin {
    async fn append_transaction(
        &self,
        _trs: &[Transaction],
        _old_state: &State,
        _new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        Ok(None)
    }
}

/// B业务状态字段
#[derive(Debug)]
pub struct BBusinessStateField;

#[async_trait]
impl StateField for BBusinessStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        Arc::new(BBusinessState {
            data: HashMap::new(),
            compute_result: None,
        })
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut b_state = value.downcast_arc::<BBusinessState>().unwrap().as_ref().clone();
        
        // B业务先执行计算
        b_state.compute_result = Some("B business computed result".to_string());
        
        // 将结果保存到依赖管理器
        let resource_manager = new_state.resource_manager();
        let mut resource_manager = resource_manager.write().unwrap();
        
        if let Some(dep_manager) = resource_manager.resource_table.get_mut::<BusinessDependencyManager>(0) {
            dep_manager.set_compute_result("b_business", Arc::new(b_state.clone()));
        }
        
        tracing::info!("B业务计算完成: {:?}", b_state.compute_result);
        
        Arc::new(b_state)
    }
}
```

## 3. 使用示例

### 3.1 插件注册和配置

```rust
use std::sync::Arc;
use mf_state::{
    plugin::{Plugin, PluginSpec},
    state::{State, StateConfig},
    ops::GlobalResourceManager,
};

async fn setup_business_plugins() -> Vec<Arc<Plugin>> {
    let mut plugins = Vec::new();
    
    // 注册依赖管理器到全局资源
    let resource_manager = Arc::new(GlobalResourceManager::default());
    {
        let mut rm = resource_manager.write().unwrap();
        rm.resource_table.add(BusinessDependencyManager::new());
    }
    
    // B业务插件 (优先级高，先执行)
    let b_plugin = Plugin::new(PluginSpec {
        state_field: Some(Arc::new(BBusinessStateField)),
        key: ("b_business".to_string(), "v1".to_string()),
        tr: Some(Arc::new(BBusinessPlugin)),
        priority: 1, // 高优先级
    });
    plugins.push(Arc::new(b_plugin));
    
    // A业务插件 (优先级低，后执行)
    let a_plugin = Plugin::new(PluginSpec {
        state_field: Some(Arc::new(ABusinessStateField)),
        key: ("a_business".to_string(), "v1".to_string()),
        tr: Some(Arc::new(ABusinessPlugin)),
        priority: 2, // 低优先级
    });
    plugins.push(Arc::new(a_plugin));
    
    plugins
}
```

### 3.2 业务执行流程

```rust
async fn execute_business_flow() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 设置插件
    let plugins = setup_business_plugins().await;
    
    // 2. 创建状态配置
    let state_config = StateConfig {
        schema: None, // 需要提供具体的Schema
        doc: None,
        stored_marks: None,
        plugins: Some(plugins),
        resource_manager: None,
    };
    
    // 3. 创建状态
    let state = State::create(state_config).await?;
    
    // 4. 执行事务 (同时触发A和B业务)
    let tr = state.tr();
    // 添加业务逻辑到事务...
    
    let result = state.apply(tr).await?;
    
    // 5. B业务先执行，A业务等待B完成后执行
    tracing::info!("业务流程执行完成");
    
    Ok(())
}
```

## 4. 可插拔性保证

### 4.1 B业务插件的可插拔性

```rust
/// 无B业务时的降级处理
impl ABusinessStateField {
    async fn handle_missing_dependency(&self, new_state: &State) -> Arc<dyn Resource> {
        // 当B业务插件不存在时的降级逻辑
        let a_state = ABusinessState {
            data: HashMap::new(),
            compute_result: Some("A business computed without B".to_string()),
            dependencies_ready: false,
        };
        
        tracing::warn!("B业务插件未找到，A业务使用默认计算");
        Arc::new(a_state)
    }
}
```

### 4.2 动态依赖管理

```rust
/// 支持运行时动态添加/移除B业务插件
impl BusinessDependencyManager {
    pub fn add_dependency_provider(&mut self, provider: String) {
        // 动态添加依赖提供者
    }
    
    pub fn remove_dependency_provider(&mut self, provider: String) {
        // 动态移除依赖提供者
        self.compute_results.remove(&provider);
    }
}
```

## 5. 优势总结

1. **完全解耦**: A和B业务插件之间没有直接依赖
2. **可插拔**: B业务插件可以随时添加或移除
3. **事务性**: 整个流程在事务中执行，保证一致性
4. **异步支持**: 支持异步计算和等待
5. **优先级管理**: 通过插件优先级控制执行顺序
6. **资源共享**: 通过全局资源管理器共享计算结果
7. **降级支持**: A业务在B业务不存在时可以降级处理

这个设计充分利用了Moduforge-RS框架的插件系统、状态管理、事务系统和资源管理能力，实现了您需要的业务依赖场景。 
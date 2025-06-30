# 基于Transaction Meta的业务依赖解耦设计

## 🎯 **设计理念**

使用`Transaction`的`meta`字段来携带业务依赖信息，实现A业务依赖B业务的完全解耦。这种方式更加轻量级，不需要额外的依赖管理器，利用现有的事务系统即可实现。

## 🏗️ **核心设计**

### 1. **Meta字段结构设计**

```rust
/// 业务类型标识
#[derive(Debug, Clone, PartialEq)]
pub enum BusinessType {
    A,
    B,
    Other(String),
}

/// 业务状态
#[derive(Debug, Clone, PartialEq)]
pub enum BusinessStatus {
    Pending,      // 等待执行
    Computing,    // 计算中
    Completed,    // 完成
    Failed,       // 失败
}

/// 业务依赖信息
#[derive(Debug, Clone)]
pub struct BusinessDependencyInfo {
    pub business_type: BusinessType,
    pub status: BusinessStatus,
    pub dependencies: Vec<BusinessType>,  // 依赖的业务类型
    pub result: Option<String>,           // 计算结果
    pub timestamp: std::time::SystemTime,
}

/// 业务执行上下文
#[derive(Debug, Clone)]
pub struct BusinessExecutionContext {
    pub all_businesses: Vec<BusinessDependencyInfo>,
    pub execution_order: Vec<BusinessType>,
}
```

### 2. **Meta键定义**

```rust
pub const META_BUSINESS_TYPE: &str = "business_type";
pub const META_BUSINESS_STATUS: &str = "business_status";
pub const META_BUSINESS_DEPENDENCIES: &str = "business_dependencies";
pub const META_BUSINESS_RESULT: &str = "business_result";
pub const META_EXECUTION_CONTEXT: &str = "execution_context";
pub const META_DEPENDENCY_SATISFIED: &str = "dependency_satisfied";
```

## 🔄 **实现方案**

### 1. **A业务插件实现**

```rust
use std::{collections::HashMap, sync::Arc};
use async_trait::async_trait;
use moduforge_state::{
    resource::Resource,
    plugin::{Plugin, PluginSpec, PluginTrait, StateField},
    state::{State, StateConfig},
    transaction::Transaction,
    error::StateResult,
};

/// A业务状态
#[derive(Debug, Clone)]
pub struct ABusinessState {
    pub data: HashMap<String, String>,
    pub compute_result: Option<String>,
    pub b_result_used: Option<String>,
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
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 检查最新事务的meta信息
        if let Some(latest_tr) = trs.last() {
            // 检查是否有B业务完成的信息
            if let Some(b_result) = latest_tr.get_meta::<String>("b_business_result") {
                // B业务已完成，创建A业务事务
                let mut new_tr = new_state.tr();
                new_tr.set_meta(META_BUSINESS_TYPE, BusinessType::A);
                new_tr.set_meta(META_BUSINESS_STATUS, BusinessStatus::Computing);
                new_tr.set_meta("a_business_uses_b_result", b_result.clone());
                
                tracing::info!("A业务开始执行，使用B业务结果: {:?}", b_result);
                return Ok(Some(new_tr));
            }
            
            // 检查是否B业务还未完成
            if let Some(business_type) = latest_tr.get_meta::<BusinessType>(META_BUSINESS_TYPE) {
                if **business_type == BusinessType::B {
                    if let Some(status) = latest_tr.get_meta::<BusinessStatus>(META_BUSINESS_STATUS) {
                        if **status != BusinessStatus::Completed {
                            tracing::info!("A业务等待B业务完成");
                            return Ok(None); // 等待B业务完成
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        tr: &Transaction,
        _state: &State,
    ) -> bool {
        // 检查当前事务是否为A业务，以及依赖是否满足
        if let Some(business_type) = tr.get_meta::<BusinessType>(META_BUSINESS_TYPE) {
            if **business_type == BusinessType::A {
                // 检查依赖是否满足
                if let Some(satisfied) = tr.get_meta::<bool>(META_DEPENDENCY_SATISFIED) {
                    return **satisfied;
                }
                
                // 如果没有依赖满足标记，检查是否有B业务结果
                return tr.get_meta::<String>("b_business_result").is_some();
            }
        }
        
        true // 非A业务事务通过
    }
}

/// A业务状态字段
#[derive(Debug)]
pub struct ABusinessStateField;

#[async_trait]
impl StateField for ABusinessStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        Arc::new(ABusinessState {
            data: HashMap::new(),
            compute_result: None,
            b_result_used: None,
        })
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut a_state = value.downcast_arc::<ABusinessState>().unwrap().as_ref().clone();
        
        // 检查是否为A业务事务
        if let Some(business_type) = tr.get_meta::<BusinessType>(META_BUSINESS_TYPE) {
            if **business_type == BusinessType::A {
                // 获取B业务结果并进行A业务计算
                if let Some(b_result) = tr.get_meta::<String>("a_business_uses_b_result") {
                    a_state.b_result_used = Some(b_result.to_string());
                    a_state.compute_result = Some(format!("A计算结果 (基于B: {})", b_result));
                    
                    tracing::info!("A业务计算完成: {:?}", a_state.compute_result);
                } else {
                    // 降级处理
                    a_state.compute_result = Some("A计算结果 (降级模式)".to_string());
                    tracing::warn!("A业务执行降级计算");
                }
            }
        }

        Arc::new(a_state)
    }
}
```

### 2. **B业务插件实现**

```rust
/// B业务状态
#[derive(Debug, Clone)]
pub struct BBusinessState {
    pub data: HashMap<String, String>,
    pub compute_result: Option<String>,
}

impl Resource for BBusinessState {
    fn name(&self) -> std::borrow::Cow<str> {
        "BBusinessState".into()
    }
}

/// B业务插件
#[derive(Debug)]
pub struct BBusinessPlugin;

#[async_trait]
impl PluginTrait for BBusinessPlugin {
    async fn append_transaction(
        &self,
        _trs: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // B业务不依赖其他业务，可以直接执行
        let mut new_tr = new_state.tr();
        new_tr.set_meta(META_BUSINESS_TYPE, BusinessType::B);
        new_tr.set_meta(META_BUSINESS_STATUS, BusinessStatus::Computing);
        
        tracing::info!("B业务开始执行");
        Ok(Some(new_tr))
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
        _old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut b_state = value.downcast_arc::<BBusinessState>().unwrap().as_ref().clone();
        
        // 检查是否为B业务事务
        if let Some(business_type) = tr.get_meta::<BusinessType>(META_BUSINESS_TYPE) {
            if **business_type == BusinessType::B {
                // 执行B业务计算
                b_state.compute_result = Some("B业务计算结果".to_string());
                
                // 将结果保存到事务meta中，供其他业务使用
                // 注意：这里需要修改事务，但apply方法接收的是不可变引用
                // 实际实现中需要通过其他方式传递结果，比如通过状态或者在append_transaction中处理
                
                tracing::info!("B业务计算完成: {:?}", b_state.compute_result);
            }
        }

        Arc::new(b_state)
    }
}
```

### 3. **事务协调器**

```rust
/// 事务协调器，负责管理业务执行顺序和依赖关系
#[derive(Debug)]
pub struct TransactionCoordinator;

impl TransactionCoordinator {
    /// 检查事务的业务依赖关系
    pub fn check_dependencies(tr: &Transaction) -> bool {
        if let Some(business_type) = tr.get_meta::<BusinessType>(META_BUSINESS_TYPE) {
            match **business_type {
                BusinessType::A => {
                    // A业务依赖B业务，检查B业务是否完成
                    tr.get_meta::<String>("b_business_result").is_some()
                }
                BusinessType::B => {
                    // B业务不依赖其他业务
                    true
                }
                BusinessType::Other(_) => true,
            }
        } else {
            true // 非业务事务直接通过
        }
    }

    /// 为事务标记依赖满足状态
    pub fn mark_dependency_status(tr: &mut Transaction) {
        let satisfied = Self::check_dependencies(tr);
        tr.set_meta(META_DEPENDENCY_SATISFIED, satisfied);
    }

    /// 从已完成的事务中提取业务结果
    pub fn extract_business_results(transactions: &[Transaction]) -> HashMap<BusinessType, String> {
        let mut results = HashMap::new();
        
        for tr in transactions {
            if let Some(business_type) = tr.get_meta::<BusinessType>(META_BUSINESS_TYPE) {
                if let Some(status) = tr.get_meta::<BusinessStatus>(META_BUSINESS_STATUS) {
                    if **status == BusinessStatus::Completed {
                        if let Some(result) = tr.get_meta::<String>(META_BUSINESS_RESULT) {
                            results.insert((**business_type).clone(), result.to_string());
                        }
                    }
                }
            }
        }
        
        results
    }
}
```

## ✨ **执行流程**

### 1. **同时触发A和B业务**
```rust
// 创建初始事务，标记为混合业务触发
let mut tr = state.tr();
tr.set_meta("trigger_type", "mixed_business");
tr.set_meta("triggered_businesses", vec![BusinessType::A, BusinessType::B]);
```

### 2. **B业务先执行**
```rust
// B业务插件的append_transaction被调用
let mut b_tr = state.tr();
b_tr.set_meta(META_BUSINESS_TYPE, BusinessType::B);
b_tr.set_meta(META_BUSINESS_STATUS, BusinessStatus::Computing);

// B业务完成后
b_tr.set_meta(META_BUSINESS_STATUS, BusinessStatus::Completed);
b_tr.set_meta("b_business_result", "B业务计算结果");
```

### 3. **A业务检查依赖并执行**
```rust
// A业务插件的append_transaction检查B业务结果
if let Some(b_result) = latest_tr.get_meta::<String>("b_business_result") {
    let mut a_tr = state.tr();
    a_tr.set_meta(META_BUSINESS_TYPE, BusinessType::A);
    a_tr.set_meta("a_business_uses_b_result", b_result.clone());
    TransactionCoordinator::mark_dependency_status(&mut a_tr);
}
```

## 🎯 **优势对比**

### **相比BusinessDependencyManager方案：**

1. **更轻量级**: 不需要额外的依赖管理器组件
2. **更直接**: 依赖信息直接携带在事务中
3. **更透明**: 事务meta中包含完整的业务上下文
4. **更灵活**: 可以轻松添加更多meta信息
5. **更符合架构**: 充分利用现有的事务系统

### **缺点:**

1. **事务meta膨胀**: 大量依赖信息可能使事务meta变得复杂
2. **状态传递限制**: apply方法中无法直接修改事务meta
3. **调试复杂**: 依赖关系需要从事务meta中追踪

## 🚀 **使用示例**

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置插件
    let plugins = vec![
        Arc::new(Plugin::new(PluginSpec {
            state_field: Some(Arc::new(BBusinessStateField)),
            key: ("b_business".to_string(), "v1".to_string()),
            tr: Some(Arc::new(BBusinessPlugin)),
            priority: 1, // B业务优先级高
        })),
        Arc::new(Plugin::new(PluginSpec {
            state_field: Some(Arc::new(ABusinessStateField)),
            key: ("a_business".to_string(), "v1".to_string()),
            tr: Some(Arc::new(ABusinessPlugin)),
            priority: 2, // A业务优先级低
        })),
    ];

    // 创建状态
    let state = State::create(StateConfig {
        schema: None,
        doc: None,
        stored_marks: None,
        plugins: Some(plugins),
        resource_manager: None,
    }).await?;

    // 触发混合业务事务
    let mut tr = state.tr();
    tr.set_meta("trigger_type", "mixed_business");
    tr.set_meta("triggered_businesses", vec![BusinessType::A, BusinessType::B]);

    // 应用事务
    let result = state.apply(tr).await?;

    println!("业务执行完成!");
    Ok(())
}
```

## 📋 **总结**

使用`Transaction meta`进行业务依赖解耦是一个**更优雅的解决方案**：

- ✅ **轻量级**: 不需要额外组件
- ✅ **自包含**: 依赖信息在事务中自包含
- ✅ **可追踪**: 通过事务meta可以完整追踪业务执行过程
- ✅ **可扩展**: 容易添加更多业务类型和依赖关系
- ✅ **符合架构**: 充分利用现有事务系统的设计

这种方式特别适合您的场景：**A业务依赖B业务，A和B同时新增，B先计算，A再基于B的结果计算，且B完全可插拔**。 
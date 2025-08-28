# ModuForge-RS 插件宏使用指南

本文档介绍 ModuForge-RS 中新的插件宏系统，它参考了 Extension 宏的设计模式，为插件开发提供了声明式、类型安全的开发体验。

## 概述

插件宏系统包含以下主要宏：

- `mf_plugin!` - 主要的声明式插件定义宏
- `mf_plugin_with_config!` - 带配置支持的插件宏
- `mf_plugin_metadata!` - 插件元数据创建辅助宏
- `mf_plugin_config!` - 插件配置创建辅助宏
- `impl_plugin!` - 传统插件实现宏（兼容性）
- `impl_state_field!` - 状态字段实现宏
- `derive_plugin_state!` - 插件状态资源 trait 派生宏

## 核心宏详解

### 1. mf_plugin! - 声明式插件宏

这是新的主要插件定义宏，类似于 extension 宏的设计：

```rust
use mf_macro::{mf_plugin, mf_plugin_metadata, mf_plugin_config};

mf_plugin!(
    my_plugin,
    metadata = mf_plugin_metadata!(
        "my_plugin",
        version = "1.0.0",
        description = "我的插件",
        author = "开发者",
        dependencies = ["other_plugin"],
        tags = ["category1", "category2"]
    ),
    config = mf_plugin_config!(
        enabled = true,
        priority = 10,
        settings = { "debug" => true, "timeout" => 5000 }
    ),
    append_transaction = my_append_fn,
    filter_transaction = my_filter_fn,
    state_field = MyStateField,
    docs = "插件描述文档"
);
```

### 2. mf_plugin_metadata! - 元数据创建宏

```rust
// 简单版本
let metadata = mf_plugin_metadata!("plugin_name");

// 完整版本
let metadata = mf_plugin_metadata!(
    "plugin_name",
    version = "2.0.0",
    description = "插件描述",
    author = "作者名",
    dependencies = ["dep1", "dep2"],
    conflicts = ["conflict1"],
    state_fields = ["field1", "field2"],
    tags = ["tag1", "tag2"]
);
```

### 3. mf_plugin_config! - 配置创建宏

```rust
// 默认配置
let config = mf_plugin_config!();

// 简单配置
let config = mf_plugin_config!(enabled = true, priority = 5);

// 完整配置
let config = mf_plugin_config!(
    enabled = true,
    priority = 20,
    settings = {
        "strict_mode" => true,
        "batch_size" => 100,
        "timeout" => 30
    }
);
```

### 4. mf_plugin_with_config! - 可配置插件宏

```rust
mf_plugin_with_config!(
    configurable_plugin,
    config = {
        name: String,
        enabled: bool,
        log_level: u32
    },
    init_fn = |name: String, enabled: bool, log_level: u32| {
        // 动态创建插件规范
        let metadata = mf_plugin_metadata!(&name, version = "1.0.0");
        let config = mf_plugin_config!(enabled = enabled, priority = log_level as i32);
        
        // 返回 PluginSpec
        // ... 实现逻辑
    },
    docs = "可配置插件描述"
);
```

## 使用示例

### 基础插件

```rust
use mf_macro::mf_plugin;

// 最简单的插件
mf_plugin!(
    simple_plugin,
    docs = "简单插件示例"
);

fn main() {
    let plugin = simple_plugin::new();
    println!("插件名称: {}", plugin.get_name());
}
```

### 完整功能插件

```rust
use mf_macro::{mf_plugin, mf_plugin_metadata, mf_plugin_config};
use mf_state::{Transaction, State, error::StateResult};

// 事务处理函数
async fn validate_transactions(
    trs: &[Transaction],
    _old_state: &State,
    _new_state: &State,
) -> StateResult<Option<Transaction>> {
    println!("验证 {} 个事务", trs.len());
    Ok(None)
}

async fn security_filter(
    tr: &Transaction,
    _state: &State,
) -> bool {
    // 实际的安全过滤逻辑
    
    // 检查事务步骤数量限制
    if tr.steps().len() > 100 {
        println!("🚫 事务被拒绝: 操作步骤过多");
        return false;
    }
    
    // 检查危险操作
    let steps = tr.steps();
    for step in steps {
        if step.name().contains("delete") || step.name().contains("Drop") {
            // 删除操作需要管理员权限
            if !tr.meta().contains_key("admin_approved") {
                println!("🚫 危险操作缺少管理员批准");
                return false;
            }
        }
    }
    
    // 检查事务来源
    if let Some(source) = tr.meta().get("source") {
        if source == "untrusted" {
            println!("🚫 不可信来源的事务被拒绝");
            return false;
        }
    }
    
    println!("✅ 安全检查通过");
    true
}

// 定义完整插件
mf_plugin!(
    validation_plugin,
    metadata = mf_plugin_metadata!(
        "validation_plugin",
        version = "1.0.0",
        description = "事务验证插件",
        author = "ModuForge Team",
        tags = ["validation", "security"]
    ),
    config = mf_plugin_config!(
        enabled = true,
        priority = 100,
        settings = { "strict_mode" => true }
    ),
    append_transaction = validate_transactions,
    filter_transaction = security_filter,
    docs = "提供事务验证和安全检查功能"
);

fn main() {
    let plugin = validation_plugin::new();
    let metadata = plugin.get_metadata();
    let config = plugin.get_config();
    
    println!("插件: {} v{}", metadata.name, metadata.version);
    println!("启用: {}, 优先级: {}", config.enabled, config.priority);
}
```

### 动态配置插件

```rust
use mf_macro::{mf_plugin_with_config, mf_plugin_metadata, mf_plugin_config};

mf_plugin_with_config!(
    dynamic_logger,
    config = {
        service_name: String,
        log_level: u32,
        output_file: Option<String>
    },
    init_fn = |service_name: String, log_level: u32, output_file: Option<String>| {
        // 根据配置动态创建插件
        let metadata = mf_plugin_metadata!(
            &service_name,
            version = "1.0.0",
            description = "动态日志插件"
        );
        
        // 实现动态插件逻辑...
        // 返回 PluginSpec
    },
    docs = "可动态配置的日志插件"
);

fn main() {
    let plugin = dynamic_logger::new(
        "MyService".to_string(),
        2,
        Some("/var/log/service.log".to_string())
    );
    
    println!("动态插件: {}", plugin.get_name());
}
```

## 与 Extension 宏的对比

| 特性 | Extension 宏 | Plugin 宏 |
|------|--------------|-----------|
| 声明式语法 | ✅ | ✅ |
| 类型安全 | ✅ | ✅ |
| 元数据支持 | 简单 | 完整（版本、依赖、冲突等） |
| 配置支持 | 基础 | 高级（设置字典、优先级） |
| 条件逻辑 | 支持 | 支持 |
| 状态管理 | 操作函数 | StateField + 事务处理 |
| 文档生成 | 自动 | 自动 |

## 架构优势

### 1. 类型安全
- 编译时验证所有插件接口
- 自动类型推断和转换
- Arc 包装确保线程安全

### 2. 零成本抽象
- 宏在编译时完全展开
- 无运行时开销
- 优化友好的代码生成

### 3. 灵活配置
- 支持静态配置和动态配置
- 条件功能启用/禁用
- 运行时配置修改

### 4. 完整生态
- 与 ModuForge 架构深度集成
- 状态管理系统兼容
- 事务系统无缝对接

## 迁移指南

### 从 impl_plugin! 迁移

**旧版本：**
```rust
impl_plugin!(
    MyPlugin,
    |trs, old_state, new_state| async {
        // 处理逻辑
        Ok(None)
    }
);
```

**新版本：**
```rust
async fn handle_transaction(
    trs: &[Transaction],
    old_state: &State,
    new_state: &State,
) -> StateResult<Option<Transaction>> {
    // 处理逻辑
    Ok(None)
}

mf_plugin!(
    MyPlugin,
    metadata = mf_plugin_metadata!(
        "MyPlugin",
        version = "1.0.0",
        description = "我的插件"
    ),
    append_transaction = handle_transaction,
    docs = "插件描述"
);
```

## 过滤逻辑最佳实践

### 实际的过滤场景

`filter_transaction` 是插件系统的核心安全机制，不应该总是返回 `true`。以下是一些实际的过滤场景：

#### 1. 权限控制过滤
```rust
async fn permission_filter(
    tr: &Transaction,
    state: &State,
) -> bool {
    // 获取当前用户权限
    if let Some(user_role) = tr.meta().get("user_role") {
        match user_role.as_str() {
            "admin" => true,  // 管理员允许所有操作
            "user" => {
                // 普通用户不能执行删除操作
                let steps = tr.steps();
                !steps.iter().any(|step| 
                    step.name().contains("delete") || 
                    step.name().contains("Drop")
                )
            },
            "guest" => {
                // 访客只能执行读操作
                let steps = tr.steps();
                steps.iter().all(|step|
                    step.name().contains("read") ||
                    step.name().contains("query") ||
                    step.name().contains("get")
                )
            },
            _ => false  // 未知角色拒绝
        }
    } else {
        false  // 无权限信息拒绝
    }
}
```

#### 2. 频率限制过滤
```rust
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

async fn rate_limit_filter(
    tr: &Transaction,
    state: &State,
) -> bool {
    if let Some(user_id) = tr.meta().get("user_id") {
        // 从状态中获取用户操作历史
        if let Some(rate_limiter) = state.get_field("rate_limiter") {
            // 检查用户在过去1分钟内的操作次数
            let current_time = SystemTime::now();
            // 简化逻辑 - 实际实现需要更复杂的时间窗口检查
            
            // 普通用户每分钟最多10个事务
            let max_per_minute = if tr.meta().get("user_role") == Some("admin") {
                100  // 管理员限制较松
            } else {
                10   // 普通用户严格限制
            };
            
            // 这里需要实际的计数逻辑
            // return check_user_rate(user_id, max_per_minute);
        }
    }
    
    // 默认允许（在实际实现中应该有更严格的默认策略）
    true
}
```

#### 3. 业务规则过滤
```rust
async fn business_rule_filter(
    tr: &Transaction,
    state: &State,
) -> bool {
    // 检查业务时间限制
    let now = chrono::Utc::now();
    let hour = now.hour();
    
    // 营业时间外禁止某些操作
    if hour < 9 || hour > 18 {
        let steps = tr.steps();
        for step in steps {
            if step.name().contains("payment") || step.name().contains("transfer") {
                println!("🚫 营业时间外禁止金融操作");
                return false;
            }
        }
    }
    
    // 检查金额限制
    if let Some(amount_str) = tr.meta().get("amount") {
        if let Ok(amount) = amount_str.parse::<f64>() {
            // 大额交易需要特殊批准
            if amount > 10000.0 && !tr.meta().contains_key("manager_approved") {
                println!("🚫 大额交易需要经理批准");
                return false;
            }
        }
    }
    
    // 检查账户状态
    if let Some(account_id) = tr.meta().get("account_id") {
        if let Some(account_status) = state.get_field(&format!("account_status_{}", account_id)) {
            // 这里需要检查账户状态的具体逻辑
            // if account_is_frozen(account_status) { return false; }
        }
    }
    
    true
}
```

#### 4. 数据完整性过滤
```rust
async fn data_integrity_filter(
    tr: &Transaction,
    state: &State,
) -> bool {
    let steps = tr.steps();
    
    for step in steps {
        match step.name() {
            "UpdateNode" => {
                // 检查更新操作的数据完整性
                if let Some(node_id) = step.params().get("node_id") {
                    if let Some(existing_node) = state.get_node(node_id) {
                        // 检查必填字段
                        if !step.params().contains_key("required_field") {
                            println!("🚫 缺少必填字段");
                            return false;
                        }
                        
                        // 检查数据格式
                        if let Some(email) = step.params().get("email") {
                            if !email.contains('@') {
                                println!("🚫 邮箱格式无效");
                                return false;
                            }
                        }
                    }
                }
            },
            "DeleteNode" => {
                // 检查删除操作的依赖关系
                if let Some(node_id) = step.params().get("node_id") {
                    if has_dependent_nodes(state, node_id) {
                        println!("🚫 无法删除: 存在依赖节点");
                        return false;
                    }
                }
            },
            _ => {}
        }
    }
    
    true
}

fn has_dependent_nodes(state: &State, node_id: &str) -> bool {
    // 实际实现需要检查依赖关系
    // 这里简化为总是false
    false
}
```

#### 5. 组合过滤策略
```rust
mf_plugin!(
    comprehensive_security_plugin,
    metadata = mf_plugin_metadata!(
        "comprehensive_security_plugin",
        version = "1.0.0",
        description = "综合安全过滤插件"
    ),
    filter_transaction = comprehensive_filter,
    docs = "提供多层安全检查的综合过滤插件"
);

async fn comprehensive_filter(
    tr: &Transaction,
    state: &State,
) -> bool {
    // 第一层: 基础权限检查
    if !permission_filter(tr, state).await {
        return false;
    }
    
    // 第二层: 频率限制检查
    if !rate_limit_filter(tr, state).await {
        return false;
    }
    
    // 第三层: 业务规则检查
    if !business_rule_filter(tr, state).await {
        return false;
    }
    
    // 第四层: 数据完整性检查
    if !data_integrity_filter(tr, state).await {
        return false;
    }
    
    // 所有检查都通过
    println!("✅ 综合安全检查通过");
    true
}
```

### 过滤逻辑原则

1. **安全优先**: 默认拒绝，明确允许
2. **分层检查**: 从简单到复杂的多层验证
3. **详细日志**: 记录拒绝原因便于调试
4. **性能考虑**: 先执行快速检查，再做复杂验证
5. **可配置性**: 允许运行时调整过滤策略

## 最佳实践

### 1. 命名约定
- 插件名使用 snake_case
- 元数据名称与插件名保持一致
- 函数名清晰表达功能

### 2. 文档编写
- 为每个插件添加 docs 参数
- 在函数上添加详细注释
- 提供使用示例

### 3. 错误处理
- 使用 StateResult 进行错误传播
- 记录详细的错误信息
- 提供适当的回滚机制

### 4. 性能优化
- 避免在过滤函数中执行重操作
- 合理设置插件优先级
- 考虑批处理优化

## 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = my_plugin::new();
        assert_eq!(plugin.get_name(), "my_plugin");
        
        let metadata = plugin.get_metadata();
        assert_eq!(metadata.version, "1.0.0");
    }
    
    #[test]
    fn test_plugin_config() {
        let plugin = my_plugin::new();
        let config = plugin.get_config();
        
        assert!(config.enabled);
        assert_eq!(config.priority, 10);
    }
    
    #[test]
    fn test_plugin_spec() {
        let spec = my_plugin::spec();
        assert!(spec.state_field.is_none());
        
        let metadata = spec.tr.metadata();
        assert_eq!(metadata.name, "my_plugin");
    }
}
```

## 总结

新的插件宏系统提供了：

1. **现代化开发体验** - 声明式语法，简洁明了
2. **类型安全保证** - 编译时验证，减少运行时错误
3. **丰富的功能** - 完整的元数据、配置和状态管理
4. **高性能** - 零成本抽象，编译时优化
5. **易于测试** - 结构清晰，测试友好
6. **向后兼容** - 保留旧版宏，平滑迁移

这个插件宏系统为 ModuForge-RS 框架提供了强大而灵活的插件开发能力，使开发者能够轻松创建高质量、高性能的插件组件。
# ModuForge 扩展宏完整示例与宏展开分析

本文档提供了一个完整的 ModuForge 扩展系统示例，包括宏使用、宏展开分析和实际应用场景。

## 1. 完整示例代码

### 1.1 操作函数定义

使用 `mf_op!` 宏定义操作函数，展示两种语法形式：

```rust
// 简单操作（不使用 manager 参数）
mf_op!(init_connection_pool, {
    println!("数据库连接池初始化完成");
    Ok(())
});

// 使用 manager 参数的操作
mf_op!(setup_migrations, |manager| {
    println!("运行数据库迁移，manager: {:?}", std::ptr::addr_of!(manager));
    Ok(())
});
```

**宏展开结果：**
```rust
// mf_op! 宏展开为标准的 Rust 函数
fn init_connection_pool(
    _manager: &mf_state::ops::GlobalResourceManager,
) -> mf_core::ForgeResult<()> {
    {
        println!("数据库连接池初始化完成");
        Ok(())
    }
}

fn setup_migrations(
    manager: &mf_state::ops::GlobalResourceManager,
) -> mf_core::ForgeResult<()> {
    {
        println!("运行数据库迁移，manager: {:?}", std::ptr::addr_of!(manager));
        Ok(())
    }
}
```

### 1.2 插件定义

标准 Rust 结构体，实现 `PluginTrait`：

```rust
#[derive(Debug)]
pub struct DatabasePlugin {
    pub name: String,
}

impl Default for DatabasePlugin {
    fn default() -> Self {
        Self {
            name: "DatabasePlugin".to_string(),
        }
    }
}

#[async_trait]
impl PluginTrait for DatabasePlugin {
    async fn append_transaction(
        &self,
        _transactions: &[mf_state::Transaction],
        _old_state: &mf_state::State,
        _new_state: &mf_state::State,
    ) -> ForgeResult<Option<mf_state::Transaction>> {
        println!("数据库插件: 处理事务追加");
        Ok(None)
    }
}
```

### 1.3 操作块定义

使用 `mf_ops!` 宏批量管理操作函数：

```rust
// 传统函数定义
fn backup_database(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("备份数据库");
    Ok(())
}

fn restore_database(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("恢复数据库");
    Ok(())
}

fn optimize_database(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("优化数据库");
    Ok(())
}

// 使用 mf_ops! 宏创建操作块
mf_ops!(maintenance_ops, [backup_database, restore_database, optimize_database]);
```

**宏展开结果：**
```rust
// mf_ops! 宏展开为返回 OpFn 的函数
pub fn maintenance_ops() -> mf_core::extension::OpFn {
    vec![
        std::sync::Arc::new(backup_database),
        std::sync::Arc::new(restore_database),
        std::sync::Arc::new(optimize_database),
    ]
}
```

### 1.4 扩展定义

#### 1.4.1 基础扩展

```rust
mf_extension!(
    database_extension,
    ops = [init_connection_pool, setup_migrations, create_indexes],
    plugins = [DatabasePlugin::default()],
    global_attributes = [
        mf_global_attr!("database", "url", "postgresql://localhost:5432/mydb"),
        mf_global_attr!("database", "pool_size", "10"),
        mf_global_attr!("database", "timeout", "30")
    ],
    docs = "基础数据库扩展，包含连接池、迁移和索引管理"
);
```

**宏展开结果：**
```rust
/// 基础数据库扩展，包含连接池、迁移和索引管理
/// 
/// A ModuForge extension for use with the framework.
/// To use it, call the init() method to get an Extension instance:
/// 
/// ```rust,ignore
/// use mf_core::extension::Extension;
/// 
/// let extension = database_extension::init();
/// ```
#[allow(non_camel_case_types)]
pub struct database_extension;

impl database_extension {
    /// Initialize this extension for use with ModuForge runtime.
    /// 
    /// # Returns
    /// An Extension object that can be used during framework initialization
    pub fn init() -> mf_core::extension::Extension {
        let mut ext = mf_core::extension::Extension::new();
        
        // 添加操作函数
        let ops: mf_core::extension::OpFn = vec![
            std::sync::Arc::new(init_connection_pool),
            std::sync::Arc::new(setup_migrations),
            std::sync::Arc::new(create_indexes),
        ];
        for op in ops {
            ext.add_op_fn(op);
        }

        // 添加插件
        ext.add_plugin(std::sync::Arc::new(DatabasePlugin::default()));

        // 添加全局属性
        ext.add_global_attribute({
            use std::collections::HashMap;
            use mf_model::schema::AttributeSpec;
            use serde_json::Value;
            
            let mut attr_map = HashMap::new();
            attr_map.insert("url".to_string(), AttributeSpec {
                default: Some(Value::String("postgresql://localhost:5432/mydb".to_string())),
            });
            
            mf_core::types::GlobalAttributeItem {
                types: vec!["database".to_string()],
                attributes: attr_map,
            }
        });
        
        // ... 其他全局属性的类似展开
        
        ext
    }
}
```

#### 1.4.2 可配置扩展

```rust
mf_extension_with_config!(
    configurable_database_extension,
    config = {
        database_url: String,
        pool_size: usize,
        enable_cache: bool,
        cache_ttl: u64
    },
    init_fn = |ext: &mut mf_core::extension::Extension, database_url: String, pool_size: usize, enable_cache: bool, cache_ttl: u64| {
        // 添加数据库配置
        ext.add_global_attribute(mf_global_attr!("database", "url", &database_url));
        ext.add_global_attribute(mf_global_attr!("database", "pool_size", &pool_size.to_string()));
        
        // 根据配置添加缓存相关属性
        if enable_cache {
            ext.add_global_attribute(mf_global_attr!("cache", "enabled", "true"));
            ext.add_global_attribute(mf_global_attr!("cache", "ttl", &cache_ttl.to_string()));
            
            // 添加缓存插件
            ext.add_plugin(Arc::new(CachePlugin));
        }
        
        // 添加基础数据库插件
        ext.add_plugin(Arc::new(DatabasePlugin::default()));
        
        // 添加初始化操作
        ext.add_op_fn(Arc::new(init_connection_pool));
        ext.add_op_fn(Arc::new(setup_migrations));
        
        if enable_cache {
            ext.add_op_fn(Arc::new(create_indexes));
        }
    },
    docs = "完全可配置的数据库扩展，支持动态缓存启用"
);
```

**宏展开结果：**
```rust
/// 完全可配置的数据库扩展，支持动态缓存启用
/// 
/// A configurable ModuForge extension.
#[allow(non_camel_case_types)]
pub struct configurable_database_extension;

impl configurable_database_extension {
    /// Initialize this extension with configuration.
    pub fn init(
        database_url: String,
        pool_size: usize,
        enable_cache: bool,
        cache_ttl: u64,
    ) -> mf_core::extension::Extension {
        let mut ext = mf_core::extension::Extension::new();
        
        // 执行自定义初始化函数
        (|ext: &mut mf_core::extension::Extension, 
          database_url: String, 
          pool_size: usize, 
          enable_cache: bool, 
          cache_ttl: u64| {
            // 添加数据库配置
            ext.add_global_attribute(/* mf_global_attr 宏展开结果 */);
            ext.add_global_attribute(/* mf_global_attr 宏展开结果 */);
            
            // 条件逻辑
            if enable_cache {
                ext.add_global_attribute(/* 缓存配置 */);
                ext.add_plugin(Arc::new(CachePlugin));
            }
            
            ext.add_plugin(Arc::new(DatabasePlugin::default()));
            ext.add_op_fn(Arc::new(init_connection_pool));
            ext.add_op_fn(Arc::new(setup_migrations));
            
            if enable_cache {
                ext.add_op_fn(Arc::new(create_indexes));
            }
        })(&mut ext, database_url, pool_size, enable_cache, cache_ttl);
        
        ext
    }
}
```

## 2. 使用示例

### 2.1 基础用法

```rust
pub fn main() -> ForgeResult<()> {
    // 1. 基础数据库扩展
    let basic_ext = database_extension::init();
    println!("操作函数数量: {}", basic_ext.get_op_fns().len());  // 输出: 3
    println!("插件数量: {}", basic_ext.get_plugins().len());      // 输出: 1
    println!("全局属性数量: {}", basic_ext.get_global_attributes().len()); // 输出: 3
    
    // 2. 可配置数据库扩展
    let config_ext = configurable_database_extension::init(
        "postgresql://localhost:5432/production".to_string(),
        20,      // pool_size
        true,    // enable_cache
        7200     // cache_ttl
    );
    
    // 3. 使用操作块
    let maintenance = maintenance_ops();
    println!("维护操作数量: {}", maintenance.len());  // 输出: 3
    
    Ok(())
}
```

### 2.2 与运行时集成

```rust
use mf_core::runtime::Runtime;
use mf_core::types::RuntimeOptions;

async fn setup_runtime() -> ForgeResult<Runtime> {
    let mut runtime_options = RuntimeOptions::default();
    
    // 添加数据库扩展
    let db_extension = database_extension::init();
    runtime_options = runtime_options.add_extension(
        mf_core::types::Extensions::E(db_extension)
    );
    
    // 添加可配置扩展
    let config_ext = configurable_database_extension::init(
        "postgresql://production:5432/app".to_string(),
        50,
        true,
        3600
    );
    runtime_options = runtime_options.add_extension(
        mf_core::types::Extensions::E(config_ext)
    );
    
    Runtime::new(runtime_options).await
}
```

## 3. 宏展开详细分析

### 3.1 mf_op! 宏展开分析

**原始代码：**
```rust
mf_op!(init_connection_pool, {
    println!("数据库连接池初始化完成");
    Ok(())
});
```

**展开后：**
```rust
fn init_connection_pool(
    _manager: &mf_state::ops::GlobalResourceManager,
) -> mf_core::ForgeResult<()> {
    {
        println!("数据库连接池初始化完成");
        Ok(())
    }
}
```

**分析：**
- 宏自动添加了 `GlobalResourceManager` 参数
- 返回类型为 `ForgeResult<()>`
- 函数体被包装在代码块中确保作用域隔离

### 3.2 mf_global_attr! 宏展开分析

**原始代码：**
```rust
mf_global_attr!("database", "url", "postgresql://localhost:5432/mydb")
```

**展开后：**
```rust
{
    use std::collections::HashMap;
    use mf_model::schema::AttributeSpec;
    use serde_json::Value;
    
    let mut attr_map = HashMap::new();
    attr_map.insert("url".to_string(), AttributeSpec {
        default: Some(Value::String("postgresql://localhost:5432/mydb".to_string())),
    });
    
    mf_core::types::GlobalAttributeItem {
        types: vec!["database".to_string()],
        attributes: attr_map,
    }
}
```

**分析：**
- 创建了类型安全的属性结构
- 自动转换字符串值为 JSON Value
- 支持多类型节点的属性定义

### 3.3 mf_extension! 宏展开分析

**关键特性：**

1. **结构体生成**：创建以扩展名命名的空结构体
2. **init() 方法**：生成初始化方法，按顺序添加组件
3. **文档生成**：自动生成带使用示例的文档
4. **类型安全**：编译时验证所有组件类型

**执行流程：**
```rust
pub fn init() -> mf_core::extension::Extension {
    let mut ext = mf_core::extension::Extension::new();
    
    // 1. 添加操作函数
    let ops = vec![/* 操作函数Arc列表 */];
    for op in ops {
        ext.add_op_fn(op);
    }
    
    // 2. 添加插件
    ext.add_plugin(std::sync::Arc::new(/* 插件实例 */));
    
    // 3. 添加全局属性
    ext.add_global_attribute(/* 属性项 */);
    
    ext
}
```

## 4. 性能考虑

### 4.1 编译时优化

- **零成本抽象**：宏在编译时完全展开，运行时无额外开销
- **内联优化**：简单操作函数可被编译器内联
- **Arc 共享**：操作函数使用 Arc 包装，支持高效共享

### 4.2 内存使用

- **结构化共享**：插件和操作函数使用引用计数共享
- **延迟初始化**：扩展组件按需加载
- **最小化克隆**：避免不必要的数据复制

### 4.3 并发安全

- **线程安全**：所有生成的结构都是 Send + Sync
- **无锁设计**：避免运行时锁竞争
- **不可变性**：扩展配置在初始化后不可变

## 5. 错误处理

### 5.1 编译时错误

```rust
// 错误示例：缺少必需参数
mf_extension!(
    invalid_extension,
    // 编译错误：必须指定至少一个组件
);

// 错误示例：类型不匹配
mf_extension!(
    invalid_extension,
    ops = [non_existent_function], // 编译错误：函数不存在
);
```

### 5.2 运行时错误处理

```rust
// 操作函数中的错误处理
mf_op!(risky_operation, |manager| {
    database_operation()
        .map_err(|e| ForgeError::DatabaseError(e))?;
    Ok(())
});
```

## 6. 最佳实践

### 6.1 扩展设计

1. **单一职责**：每个扩展专注单一功能领域
2. **组合优于继承**：使用多个小扩展而非一个大扩展
3. **配置外部化**：使用配置宏处理变化需求

### 6.2 操作函数设计

1. **幂等性**：操作函数应可安全重复执行
2. **错误透明**：使用 ForgeResult 传播错误
3. **资源清理**：确保操作函数不泄漏资源

### 6.3 插件集成

1. **生命周期管理**：正确实现插件生命周期
2. **状态隔离**：避免插件间状态耦合
3. **错误恢复**：插件错误不应影响整体系统

## 7. 扩展生态

### 7.1 内置扩展

- **数据库扩展**：连接池、迁移、备份
- **缓存扩展**：Redis、内存缓存
- **日志扩展**：结构化日志、指标收集

### 7.2 社区扩展

- **认证扩展**：JWT、OAuth、RBAC
- **存储扩展**：文件系统、对象存储
- **通信扩展**：WebSocket、gRPC、消息队列

### 7.3 自定义扩展

开发者可使用提供的宏系统创建特定业务需求的扩展：

```rust
// 业务特定扩展示例
mf_extension!(
    ecommerce_extension,
    ops = [init_payment_gateway, setup_inventory, configure_shipping],
    plugins = [PaymentPlugin, InventoryPlugin, ShippingPlugin],
    global_attributes = [
        mf_global_attr!("payment", "gateway", "stripe"),
        mf_global_attr!("inventory", "threshold", "10"),
        mf_global_attr!("shipping", "provider", "fedex")
    ],
    docs = "电商业务扩展，集成支付、库存和物流功能"
);
```

## 8. 总结

ModuForge 扩展宏系统提供了：

1. **声明式语法**：简洁表达复杂扩展结构
2. **类型安全**：编译时验证确保正确性
3. **高性能**：零成本抽象和优化友好
4. **可扩展性**：支持复杂业务场景
5. **维护性**：清晰的代码组织和文档生成

通过这套宏系统，开发者可以快速构建功能丰富、性能优秀的 ModuForge 扩展，同时保持代码的清晰性和可维护性。
# ModuForge 扩展宏使用指南

本文档演示了新的 `mf_extension!` 宏和相关辅助宏的使用方法，这些宏提供了类似于 Deno 的 `extension!` 宏的声明式扩展定义方式。

## 概述

新的扩展宏系统提供：

- **mf_extension!**: 主要的扩展定义宏，采用声明式语法
- **mf_extension_with_config!**: 带配置参数的扩展宏
- **mf_ops!**: 操作块声明宏，用于批量定义操作函数
- **mf_op!**: 操作函数辅助宏，具备自动错误处理功能
- **mf_global_attr!**: 全局属性辅助宏，用于类型安全的属性创建

## 基本扩展使用 mf_extension!

```rust
use mf_macro::{mf_extension, mf_op, mf_global_attr};
use mf_core::ForgeResult;
use mf_state::ops::GlobalResourceManager;

// 使用 mf_op! 辅助宏定义操作函数
mf_op!(setup_logging, {
    println!("日志系统初始化完成");
    Ok(())
});

mf_op!(cleanup_resources, |manager| {
    println!("清理资源中...");
    // 使用 manager 进行资源清理
    Ok(())
});

// 创建包含操作函数的扩展
mf_extension!(
    logging_extension,
    ops = [setup_logging, cleanup_resources],
    docs = "用于日志记录和资源管理的扩展"
);

// 使用方法
let ext = logging_extension::init();
```

## 带插件和全局属性的扩展

```rust
use mf_macro::{mf_extension, mf_global_attr};
use mf_state::plugin::Plugin;
use std::sync::Arc;

// 创建插件
#[derive(Debug)]
struct ValidationPlugin;

// 为 ValidationPlugin 实现插件特征
impl mf_state::plugin::PluginTrait for ValidationPlugin {
    // 实现详情...
}

// 创建包含多个组件的扩展
mf_extension!(
    validation_extension,
    plugins = [ValidationPlugin],
    global_attributes = [
        mf_global_attr!("validation", "enabled", "true"),
        mf_global_attr!("validation", "strict_mode", "false")
    ],
    docs = "提供数据验证功能的扩展"
);

// 使用方法
let ext = validation_extension::init();
```

## 带配置的扩展

```rust
use mf_macro::mf_extension_with_config;

mf_extension_with_config!(
    configurable_extension,
    config = {
        debug_mode: bool,
        max_connections: usize
    },
    init_fn = |ext: &mut mf_core::extension::Extension, debug_mode, max_connections| {
        if debug_mode {
            ext.add_global_attribute(mf_global_attr!("debug", "enabled", "true"));
        }
        ext.add_global_attribute(mf_global_attr!("config", "max_connections", &max_connections.to_string()));
    },
    docs = "带运行时配置的扩展"
);

// 带配置的使用方法
let ext = configurable_extension::init(true, 100);
```

## 高级示例：完整扩展

```rust
use mf_macro::{mf_extension, mf_ops, mf_op, mf_global_attr};
use mf_core::ForgeResult;
use mf_state::ops::GlobalResourceManager;
use std::sync::Arc;

// 使用 mf_op! 宏定义操作函数
mf_op!(init_database, |manager| {
    println!("数据库初始化完成");
    Ok(())
});

mf_op!(setup_cache, {
    println!("缓存系统就绪");
    Ok(())
});

mf_op!(register_handlers, |manager| {
    println!("事件处理器注册完成");
    Ok(())
});

// 创建插件
#[derive(Debug)]
struct DatabasePlugin;
impl mf_state::plugin::PluginTrait for DatabasePlugin {
    // 实现详情...
}

// 创建综合性扩展
mf_extension!(
    database_extension,
    ops = [init_database, setup_cache, register_handlers],
    plugins = [DatabasePlugin],
    global_attributes = [
        mf_global_attr!("database", "url", "postgresql://localhost/mydb"),
        mf_global_attr!("cache", "size", "1000"),
        mf_global_attr!("cache", "ttl", "3600")
    ],
    docs = "完整的数据库扩展，包含连接池和缓存功能"
);

// 使用方法
let ext = database_extension::init();
```

## 使用 mf_ops! 进行操作块管理

```rust
use mf_macro::{mf_ops, mf_extension};
use mf_core::ForgeResult;
use mf_state::ops::GlobalResourceManager;

// 定义操作函数
fn op_create(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("创建资源");
    Ok(())
}

fn op_update(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("更新资源");
    Ok(())
}

fn op_delete(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("删除资源");
    Ok(())
}

// 创建操作块
mf_ops!(crud_ops, [op_create, op_update, op_delete]);

// 在扩展中使用（替代方法）
fn setup_extension() -> mf_core::extension::Extension {
    let mut ext = mf_core::extension::Extension::new();
    let ops = crud_ops();
    for op in ops {
        ext.add_op_fn(op);
    }
    ext
}
```

## 宏功能特性

### 1. **mf_extension!** - 主要扩展定义宏
- **声明式语法**，类似于 Deno 的 `extension!` 宏
- **类型安全组件**，支持编译时验证
- **自动文档生成**，包含使用示例
- **模块化结构**，关注点清晰分离

#### 可用选项：
- `ops`: 操作函数列表，函数签名为 `fn(&GlobalResourceManager) -> ForgeResult<()>`
- `plugins`: 要包含的插件实例列表
- `global_attributes`: 全局属性项列表
- `docs`: 扩展的文档字符串

### 2. **mf_extension_with_config!** - 可配置扩展宏
- **运行时配置**，支持类型化参数
- **自定义初始化**函数，支持参数注入
- **类型安全配置**处理

#### 可用选项：
- `config`: 带类型字段的配置结构
- `init_fn`: 接收扩展和配置参数的自定义初始化函数
- `docs`: 扩展的文档字符串

### 3. **mf_ops!** - 操作块声明宏
- **批量操作定义**，代码组织更清晰
- **类型安全操作函数**，签名一致
- **可重用操作块**，跨多个扩展使用

### 4. **mf_op!** - 操作函数辅助宏
- **简化操作创建**，自动错误处理
- **管理器参数处理**（可选或必需）
- **一致函数签名**，所有操作统一

#### 语法变体：
```rust
// 不带管理器参数的简单操作
mf_op!(operation_name, {
    println!("操作执行");
    Ok(())
});

// 带管理器参数的操作
mf_op!(operation_name, |manager| {
    // 使用 manager 进行操作
    Ok(())
});
```

### 5. **mf_global_attr!** - 全局属性辅助宏
- **类型安全属性创建**，自动 JSON 值转换
- **简洁语法**，全局属性定义更清晰
- **扩展系统集成**

#### 语法变体：
```rust
// 简单键值对
mf_global_attr!("node_type", "attribute_key", "value");

// 复杂属性规范
mf_global_attr!(
    vec!["node_type1", "node_type2"], 
    vec![("key1", AttributeSpec { default: Some(Value::String("value1".into())) })]
);
```

## 迁移指南

### 从 impl_extension!（旧版）到 mf_extension!（新版）

**旧语法：**
```rust
let ext = impl_extension!(
    attr: mf_global_attr!("key", "value");
    plugin: MyPlugin;
    op: my_operation
);
```

**新语法：**
```rust
mf_extension!(
    my_extension,
    ops = [my_operation],
    plugins = [MyPlugin],
    global_attributes = [mf_global_attr!("type", "key", "value")]
);

let ext = my_extension::init();
```

## 新宏系统的优势

1. **更好的组织性**: 扩展定义为模块，结构清晰
2. **类型安全**: 所有组件在编译时验证
3. **文档生成**: 自动生成使用示例
4. **可维护性**: 关注点清晰分离，组件可重用
5. **灵活性**: 支持复杂初始化逻辑和配置
6. **一致性**: 遵循既定的 Rust 宏模式（受 Deno 启发）

## 与 ModuForge 架构的集成

新的扩展宏与现有的 ModuForge 框架完全兼容：

- **扩展**与运行时系统无缝集成
- **操作**遵循标准 `GlobalResourceManager` 模式
- **插件**使用现有插件特征系统
- **全局属性**与既定属性系统协作

## 最佳实践

1. **使用描述性名称**命名扩展和操作
2. **将相关操作分组**在同一扩展中
3. **提供清晰文档**，使用 `docs` 参数
4. **为配置函数使用类型注解**
5. **保持操作专注**于单一职责
6. **充分测试扩展**，编写单元测试

## 示例：实际应用扩展

```rust
use mf_macro::{mf_extension, mf_op, mf_global_attr};

// 文件系统操作
mf_op!(init_file_system, |_manager| {
    std::fs::create_dir_all("./data")?;
    Ok(())
});

mf_op!(cleanup_temp_files, |_manager| {
    // 清理逻辑在此处
    Ok(())
});

// 创建文件系统扩展
mf_extension!(
    file_system_extension,
    ops = [init_file_system, cleanup_temp_files],
    global_attributes = [
        mf_global_attr!("file", "base_path", "./data"),
        mf_global_attr!("file", "temp_dir", "./tmp")
    ],
    docs = "文件系统管理扩展，自动清理功能"
);

// 应用程序中的使用
fn main() -> mf_core::ForgeResult<()> {
    let fs_extension = file_system_extension::init();
    
    // 在运行时中使用扩展...
    Ok(())
}
```

新的扩展宏提供了一种现代化、可维护的 ModuForge 扩展定义方法，同时保持与现有框架架构的完全兼容性。
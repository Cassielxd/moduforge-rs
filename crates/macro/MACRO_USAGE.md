# ModuForge 宏使用指?

ModuForge提供了两种类型的宏，现在分别在不同的crate中：

## 🔧 过程?(Proc Macros) - `crates/derive`

过程宏位?`moduforge-macros-derive` crate中，用于依赖注入?

```toml
[dependencies]
mf-derive = { path = "../derive" }
# 或者通过 contex crate间接使用
mf-contex = { path = "../contex" }
```

```rust
use mf_derive::{Component, Injectable, service, bean};
// 或?
use mf_contex::{Component, Injectable, service, bean};

#[derive(Component)]
#[component(name = "user_service", lifecycle = "singleton")]
pub struct UserService {
    #[inject]
    database: DatabaseService,
}

#[service(name = "api_service")]
pub struct ApiService;

#[bean]
pub fn create_config() -> Config {
    Config::default()
}
```

## 📝 声明式宏 (Declarative Macros) - `crates/macro`

声明式宏位于 `moduforge-macros` crate中，现在可以直接使用?

```toml
[dependencies]
mf-macro = { path = "../macro" }
```

```rust
use mf_macro::{impl_extension, impl_plugin, mark, node};
use mf_derive::impl_command;
```

## 🚀 宏功能说?

### 1. `#[impl_command]` - ʵֺ

Ϊ첽 `Command` ʵ֣

```rust
use mf_derive::impl_command;

#[impl_command(CreateUserCommand)]
async fn create_user(tr: &mut Transaction) -> TransformResult<()> {
    // ʵ߼
    println!("Creating user...");
    Ok(())
}

// ʹ
let command = CreateUserCommand;
command.execute(&mut transaction).await?;
```

> ʾҲʹ `#[impl_command(CreateUserCommand, "create-user")]` Զƣдݺ `CreateUserCommand` ṹ塣

### 2. impl_extension! - 扩展创建?

创建Extension实例?

```rust
use mf_macro_utils::impl_extension;

// 创建空扩?
let ext = impl_extension!();

// 创建带属性的扩展
let ext = impl_extension!(
    attr: "key1=value1",
    attr: "key2=value2"
);

// 创建带插件的扩展
let ext = impl_extension!(
    plugin: MyPlugin::new(),
    plugin: AnotherPlugin::new()
);
```

### 3. mark! - 标记创建?

创建Mark实例?

```rust
use mf_macro_utils::mark;

// 简单标?
let mark = mark!("my_mark");

// 带描述的标记
let mark = mark!("my_mark", "This is a description");

// 带属性的标记
let mark = mark!("my_mark", "Description", 
    "key1" => "value1",
    "key2" => "value2"
);
```

### 4. node! - 节点创建?

创建Node实例?

```rust
use mf_macro_utils::node;

// 简单节?
let node = node!("my_node");

// 带描述的节点
let node = node!("my_node", "Node description");

// 带内容的节点
let node = node!("my_node", "Description", "content");

// 带属性的节点
let node = node!("my_node", "Description", "content",
    "attr1" => "value1",
    "attr2" => "value2"
);
```

### 5. impl_plugin! - 插件实现?

快速实现Plugin trait?

```rust
use mf_macro_utils::impl_plugin;

impl_plugin!(MyPlugin, |trs, old_state, new_state| async move {
    // 插件逻辑
    println!("Processing plugin...");
    Ok(None)
});

// 使用
let plugin = MyPlugin {};
```

### 6. impl_state_field! - 状态字段宏

```rust
use mf_macro_utils::impl_state_field;

impl_state_field!(balance, i64, 0);
```

### 7. derive_plugin_state! - 插件状态派生宏

```rust
use mf_macro_utils::derive_plugin_state;

derive_plugin_state!(MyPluginState, {
    balance: i64 = 0,
    name: String = "default".to_string(),
    active: bool = true
});
```

## 📋 完整示例

```rust
// Cargo.toml
[dependencies]
mf-macro = { path = "../macro" }  # 过程?
mf-macro-utils = { path = "../macro-utils" }  # 声明式宏

// main.rs
use mf_macro::{Component, service};             // 过程宏
use mf_derive::impl_command;             // 属性宏
use mf_macro_utils::{mark, node};                  // 声明式宏  // 声明式宏

#[derive(Component)]
#[component(name = "my_service")]
pub struct MyService;

#[impl_command(ProcessDataCommand)]
async fn process_data(tr: &mut Transaction) -> TransformResult<()> {
    println!("Processing data...");
    Ok(())
}

fn main() {
    let mark = mark!("process", "Data processing mark");
    let node = node!("data_node", "Processing node", "content");
    
    println!("Mark: {:?}", mark);
    println!("Node: {:?}", node);
}
```

## 🔍 问题排查

### 编译错误: "cannot export macro_rules! macros from a proc-macro crate"

这是正常的，因为Rust不允许从 `proc-macro` crate导出声明式宏。请使用上述的解决方案之一?

### 编译错误: "unresolved import"

确保?
1. 添加了正确的依赖
2. 使用了正确的导入路径
3. 宏所需的依赖crate已经添加

### 宏展开错误

使用 `cargo expand` 查看宏展开结果?

```bash
cargo install cargo-expand
cargo expand --bin your_binary
```

## 💡 最佳实?

1. **分离关注?*: 过程宏用于derive和属性，声明式宏用于代码生成
2. **文档?*: 为自定义宏添加文档注?
3. **测试**: 为宏编写单元测试
4. **版本控制**: 宏API变更时注意向后兼容?

---

如需更多帮助，请查看项目文档或提交Issue

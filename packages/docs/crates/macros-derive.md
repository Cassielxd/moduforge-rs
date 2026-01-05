# ModuForge 宏系统

ModuForge 提供了强大的宏系统来简化扩展开发，包括过程宏（derive）和声明式宏两部分。

## 宏系统架构

ModuForge 的宏系统分为两个 crate：

- **`moduforge-macros-derive`** (mf_derive)：过程宏，用于代码生成
- **`moduforge-macros`** (mf_macro)：声明式宏，用于简化 API 使用

## 过程宏 (Procedural Macros)

### #[derive(Node)]

用于自动生成节点类型定义的样板代码。

```rust
use mf_derive::Node;

#[derive(Node)]
#[node(name = "price_item", desc = "价格项节点")]
pub struct PriceItem {
    #[node(attr, default = "")]
    pub name: String,

    #[node(attr, default = 0.0)]
    pub unit_price: f64,

    #[node(attr, default = 1)]
    pub quantity: i32,

    #[node(content = "text*")]
    pub description: String,
}
```

**属性配置**：
- `name`: 节点名称
- `desc`: 节点描述
- `content`: 内容模型（如 `"text*"`、`"block+"`）
- `group`: 节点分组（如 `"block"`、`"inline"`）
- `inline`: 是否为内联节点
- `atom`: 是否为原子节点
- `defining`: 是否为定义节点

### #[derive(Mark)]

用于自动生成标记类型定义的样板代码。

```rust
use mf_derive::Mark;

#[derive(Mark)]
#[mark(name = "discount", desc = "折扣标记")]
pub struct Discount {
    #[mark(attr, default = 0.0)]
    pub rate: f64,

    #[mark(attr, default = "percentage")]
    pub discount_type: String,
}
```

**属性配置**：
- `name`: 标记名称
- `desc`: 标记描述
- `inclusive`: 是否包含边界
- `excludes`: 排除的其他标记
- `group`: 标记分组
- `spanning`: 是否跨节点

### #[impl_command]

用于生成异步命令实现的样板代码。

```rust
use mf_derive::impl_command;

#[impl_command]
async fn calculate_total_price(
    state: &State,
    items: Vec<PriceItem>
) -> Result<f64, Error> {
    let mut total = 0.0;

    for item in items {
        let subtotal = item.unit_price * item.quantity as f64;
        total += subtotal;
    }

    // 应用折扣
    if let Some(discount) = state.get_discount() {
        total *= (1.0 - discount);
    }

    Ok(total)
}
```

**自动生成**：
- 命令注册函数
- 错误处理包装
- 状态管理集成
- 异步运行时支持

## 声明式宏 (Declarative Macros)

### mf_extension!

创建完整的扩展定义，包含操作、节点和标记。

```rust
use mf_macro::{mf_extension, mf_op, node, mark};

mf_extension!(
    price_calculation,
    ops = [init_calculator, setup_currency],
    global_attributes = [
        mf_global_attr!("currency", "default", "CNY"),
        mf_global_attr!("precision", "decimal_places", "2")
    ],
    node_transform = transform_price_nodes,
    nodes = [
        node!("price_table", "价格表", "", "currency" => "CNY"),
        node!("price_item", "价格项", "",
            "name" => "",
            "unit_price" => "0",
            "quantity" => "1"
        ),
        node!("subtotal", "小计", "", "include_tax" => "false"),
        node!("total", "总计", "", "display_currency" => "true")
    ],
    marks = [
        mark!("discount", "折扣", "rate" => "0", "type" => "percentage"),
        mark!("tax", "税费", "rate" => "0.13", "type" => "vat"),
        mark!("currency", "货币", "code" => "CNY", "symbol" => "¥")
    ],
    docs = "价格计算扩展，支持多币种、折扣和税费计算"
);
```

### mf_plugin!

创建插件定义，支持状态管理和生命周期。

```rust
use mf_macro::{mf_plugin, mf_plugin_metadata};

mf_plugin!(
    price_validator,
    metadata = mf_plugin_metadata!(
        name = "价格验证插件",
        version = "1.0.0",
        author = "Price-RS Team",
        description = "验证价格数据的完整性和准确性"
    ),
    init = |state| {
        state.register_validator("price", validate_price);
        state.register_validator("discount", validate_discount);
        Ok(())
    },
    destroy = |state| {
        state.unregister_validator("price");
        state.unregister_validator("discount");
        Ok(())
    }
);
```

### node! 和 mark!

快速创建节点和标记实例。

```rust
// 创建节点
let price_node = node!(
    "price_item",           // 节点名称
    "价格项",               // 描述
    "",                    // 内容模型
    "name" => "材料费",     // 属性及默认值
    "unit_price" => "100",
    "quantity" => "10"
);

// 创建标记
let discount_mark = mark!(
    "discount",            // 标记名称
    "折扣",                // 描述
    "rate" => "0.1",       // 属性及默认值
    "type" => "percentage"
);
```

### mf_op!

创建操作函数。

```rust
// 使用闭包语法
mf_op!(init_price_engine, |manager| {
    println!("初始化价格计算引擎");
    manager.set_resource("price_engine", PriceEngine::new());
    Ok(())
});

// 使用块语法（不需要参数）
mf_op!(cleanup_cache, {
    println!("清理价格缓存");
    PriceCache::clear();
    Ok(())
});
```

### impl_state_field!

实现状态字段 trait。

```rust
use mf_macro::impl_state_field;

struct PriceHistory {
    records: Vec<PriceRecord>,
    max_size: usize,
}

impl_state_field!(PriceHistory, {
    fn initialize() -> Self {
        Self {
            records: Vec::new(),
            max_size: 1000,
        }
    }

    fn add_record(&mut self, record: PriceRecord) {
        if self.records.len() >= self.max_size {
            self.records.remove(0);
        }
        self.records.push(record);
    }
});
```

## 实际应用示例

### Price-RS 价格计算系统集成

```rust
// 使用过程宏定义价格节点
#[derive(Node)]
#[node(name = "engineering_item", desc = "工程项目")]
pub struct EngineeringItem {
    #[node(attr, default = "")]
    pub code: String,

    #[node(attr, default = "")]
    pub name: String,

    #[node(attr, default = "")]
    pub unit: String,

    #[node(attr, default = 0.0)]
    pub quantity: f64,

    #[node(attr, default = 0.0)]
    pub unit_price: f64,
}

// 使用声明式宏创建扩展
mf_extension!(
    engineering_calculation,
    ops = [
        mf_op!(init_calculation, |manager| {
            let calc_engine = EngineeringCalculator::new();
            manager.set_resource("calc_engine", calc_engine);
            Ok(())
        })
    ],
    nodes = [
        node!("project", "工程项目"),
        node!("section", "分部分项"),
        node!("measure", "措施项目"),
        node!("material", "材料设备"),
        node!("labor", "人工费"),
        node!("machinery", "机械费")
    ],
    marks = [
        mark!("adjustment", "调整", "factor" => "1.0"),
        mark!("tax_rate", "税率", "rate" => "0.09"),
        mark!("management_fee", "管理费", "rate" => "0.05")
    ],
    docs = "工程造价计算扩展"
);
```

### 宏组合使用

```rust
// 1. 定义数据结构
#[derive(Node)]
#[node(name = "cost_summary")]
pub struct CostSummary {
    #[node(attr)]
    pub direct_cost: f64,

    #[node(attr)]
    pub indirect_cost: f64,

    #[node(attr)]
    pub profit: f64,

    #[node(attr)]
    pub tax: f64,
}

// 2. 实现命令
#[impl_command]
async fn calculate_summary(state: &State) -> Result<CostSummary> {
    let items = state.get_all_items().await?;
    let summary = CostCalculator::calculate(items)?;
    Ok(summary)
}

// 3. 创建插件
mf_plugin!(
    cost_analyzer,
    init = |state| {
        state.register_command("calculate_summary", calculate_summary);
        Ok(())
    }
);

// 4. 组装成扩展
mf_extension!(
    cost_management,
    ops = [/* ... */],
    nodes = [/* 使用前面定义的节点 */],
    plugins = [cost_analyzer]
);
```

## 最佳实践

### 1. 选择合适的宏

- **数据结构定义**：使用 `#[derive(Node)]` 或 `#[derive(Mark)]`
- **命令实现**：使用 `#[impl_command]`
- **快速原型**：使用 `node!` 和 `mark!`
- **完整扩展**：使用 `mf_extension!`

### 2. 属性设计原则

```rust
// ✅ 好的设计：属性有明确的类型和默认值
#[node(attr, default = 0.0)]
pub price: f64,

// ✅ 使用枚举保证类型安全
#[node(attr, default = DiscountType::Percentage)]
pub discount_type: DiscountType,

// ❌ 避免：使用字符串表示所有类型
#[node(attr, default = "0")]  // 应该用数字类型
pub price: String,
```

### 3. 扩展组织

```rust
// 按功能模块组织扩展
mod pricing {
    // 价格相关的节点和标记
    mf_extension!(pricing_ext, /* ... */);
}

mod calculation {
    // 计算相关的节点和标记
    mf_extension!(calculation_ext, /* ... */);
}

mod reporting {
    // 报表相关的节点和标记
    mf_extension!(reporting_ext, /* ... */);
}

// 组合成完整功能
pub fn init_all_extensions() -> Vec<Extensions> {
    let mut extensions = vec![];
    extensions.extend(pricing::pricing_ext::init());
    extensions.extend(calculation::calculation_ext::init());
    extensions.extend(reporting::reporting_ext::init());
    extensions
}
```

### 4. 错误处理

```rust
#[impl_command]
async fn risky_calculation(state: &State) -> Result<f64> {
    // 宏会自动处理错误传播
    let data = state.load_data().await?;  // ? 操作符正常工作

    // 自定义错误处理
    let result = match calculate(data) {
        Ok(val) => val,
        Err(e) => {
            log::error!("计算失败: {}", e);
            return Err(e.into());
        }
    };

    Ok(result)
}
```

## 性能考虑

### 编译时优化

过程宏在编译时生成代码，不会影响运行时性能：

```rust
// 编译时生成完整的实现
#[derive(Node)]
pub struct LargeNode {
    // 50+ 个字段也不会影响运行时性能
    #[node(attr)] field1: String,
    #[node(attr)] field2: String,
    // ...
}
```

### 运行时优化

声明式宏生成的代码经过优化：

```rust
// mf_extension! 宏生成的初始化代码是懒加载的
let extensions = price_extension::init();  // 快速返回
// 实际初始化在首次使用时进行
```

## 调试技巧

### 1. 查看生成的代码

```bash
# 使用 cargo expand 查看宏展开后的代码
cargo expand --package your-package
```

### 2. 编译错误定位

```rust
// 如果宏生成的代码有错误，编译器会指向原始位置
#[derive(Node)]
#[node(name = 123)]  // 错误：name 应该是字符串
                      // 编译器会准确指出这一行
pub struct BadNode {}
```

### 3. 运行时调试

```rust
// 宏生成的代码支持标准调试工具
#[impl_command]
async fn debug_command(state: &State) -> Result<()> {
    dbg!(&state);  // 正常工作
    println!("State: {:?}", state);  // 如果实现了 Debug
    Ok(())
}
```

## 版本兼容性

- ModuForge 宏系统从 v0.1.0 开始稳定
- 过程宏需要 Rust 1.70+
- 声明式宏兼容 Rust 1.56+
- 向后兼容：新版本保持与旧版本 API 兼容
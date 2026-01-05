# ModuForge 声明式宏

声明式宏（`moduforge-macros`）提供了简洁的 API 来创建扩展、插件、节点和标记，是 ModuForge 开发的主要接口。

## 宏 API 概览

ModuForge 的声明式宏系统提供两类宏：
- **构建宏**：用于创建扩展、插件等复杂结构
- **实例宏**：用于快速创建节点、标记等简单实例

## 核心宏

### mf_extension!

创建完整的 ModuForge 扩展，包含操作函数、节点、标记和配置。

```rust
use mf_macro::{mf_extension, node, mark};
use mf_core::{ForgeResult, types::Extensions};
use mf_state::ops::GlobalResourceManager;

// 定义操作函数
fn init_price_engine(manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("初始化价格计算引擎");
    // 设置价格引擎资源
    manager.set_resource("price_engine", PriceEngine::new());
    Ok(())
}

fn setup_currency(manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("配置货币系统: CNY");
    manager.set_resource("default_currency", "CNY");
    Ok(())
}

// 节点转换函数
fn transform_price_nodes(node: &mut Node) -> ForgeResult<()> {
    match node.name.as_str() {
        "price_item" => {
            // 确保价格项有默认值
            if node.get_attr("unit_price").is_none() {
                node.set_attr("unit_price", Some(Value::Number(0.0.into())));
            }
            if node.get_attr("quantity").is_none() {
                node.set_attr("quantity", Some(Value::Number(1.into())));
            }
        },
        "total" => {
            // 总计节点添加货币符号
            node.set_attr("currency_symbol", Some(Value::String("¥".to_string())));
        },
        _ => {}
    }
    Ok(())
}

// 创建价格计算扩展
mf_extension!(
    price_calculation,
    // 操作函数列表
    ops = [init_price_engine, setup_currency],
    // 全局属性配置
    global_attributes = [
        mf_global_attr!(
            vec!["pricing"],
            vec![
                ("currency", AttributeSpec {
                    default: Some(Value::String("CNY".to_string()))
                }),
                ("precision", AttributeSpec {
                    default: Some(Value::Number(2.into()))
                })
            ]
        )
    ],
    // 节点转换
    node_transform = transform_price_nodes,
    // 节点定义
    nodes = [
        node!("price_table", "价格表"),
        node!("price_item", "价格项", "",
            "name" => "",
            "unit_price" => "0",
            "quantity" => "1",
            "unit" => "个"
        ),
        node!("subtotal", "小计", "", "include_tax" => "false"),
        node!("total", "总计", "", "display_currency" => "true"),
        node!("discount_item", "折扣项", "",
            "type" => "percentage",
            "value" => "0"
        )
    ],
    // 标记定义
    marks = [
        mark!("discount", "折扣", "rate" => "0", "type" => "percentage"),
        mark!("tax", "税费", "rate" => "0.09", "type" => "vat"),
        mark!("currency", "货币", "code" => "CNY", "symbol" => "¥"),
        mark!("important", "重要价格"),
        mark!("promotional", "促销价格")
    ],
    // 文档说明
    docs = "工程造价计算扩展，支持价格项管理、折扣、税费计算"
);

// 使用扩展
fn main() -> ForgeResult<()> {
    let extensions = price_calculation::init();

    // 统计各类型组件
    for ext in &extensions {
        match ext {
            Extensions::E(_) => println!("扩展主体已加载"),
            Extensions::N(node) => println!("节点: {}", node.name),
            Extensions::M(mark) => println!("标记: {}", mark.name),
        }
    }

    Ok(())
}
```

### mf_extension_with_config!

创建可配置的扩展，支持运行时参数。

```rust
mf_extension_with_config!(
    configurable_price,
    config = {
        default_currency: &str,
        tax_rate: f64,
        precision: u8
    },
    init_fn = |ext, currency, tax, precision| {
        // 根据配置初始化扩展
        ext.add_global_attribute(mf_global_attr!(
            "config",
            "currency", currency,
            "tax_rate", &tax.to_string(),
            "precision", &precision.to_string()
        ));
    },
    nodes = [
        node!("configurable_price", "可配置价格")
    ],
    docs = "支持运行时配置的价格扩展"
);

// 使用配置
let extensions = configurable_price::init("USD", 0.07, 2);
```

### mf_plugin!

创建插件，支持生命周期管理和状态集成。

```rust
use mf_macro::{mf_plugin, mf_meta};

mf_plugin!(
    price_validator,
    metadata = mf_meta!(
        version = "1.0.0",
        description = "价格数据验证插件",
        author = "Price-RS Team",
        dependencies = ["core_validator"],
        tags = ["validation", "price"]
    ),
    init = |state| {
        // 插件初始化
        state.register_validator("price", validate_price_range);
        state.register_validator("discount", validate_discount_rate);
        println!("价格验证器已注册");
        Ok(())
    },
    destroy = |state| {
        // 插件清理
        state.unregister_validator("price");
        state.unregister_validator("discount");
        println!("价格验证器已卸载");
        Ok(())
    },
    append_transaction = |trs, old_state, new_state| {
        // 事务处理逻辑
        if should_validate_prices(trs) {
            Some(create_validation_transaction())
        } else {
            None
        }
    },
    filter_transaction = |tr, state| {
        // 事务过滤逻辑
        !is_invalid_price_transaction(tr)
    }
);
```

### mf_plugin_with_config!

创建可配置的插件。

```rust
mf_plugin_with_config!(
    configurable_validator,
    config = {
        min_price: f64,
        max_price: f64,
        allow_negative: bool
    },
    metadata = mf_meta!(
        version = "2.0.0",
        description = "可配置的价格验证器"
    ),
    init_fn = |plugin, min, max, allow_neg| {
        plugin.set_config("min_price", min);
        plugin.set_config("max_price", max);
        plugin.set_config("allow_negative", allow_neg);
    }
);

// 使用配置创建插件
let plugin = configurable_validator::init(0.01, 999999.99, false);
```

## 实例创建宏

### node!

快速创建节点实例。

```rust
// 基本用法
let text_node = node!("text");

// 带描述
let paragraph = node!("paragraph", "段落节点");

// 带内容模型
let list = node!("bullet_list", "无序列表", "list_item+");

// 带属性
let price_item = node!(
    "price_item",
    "价格项",
    "",
    "name" => "材料费",
    "unit_price" => "100.00",
    "quantity" => "10",
    "unit" => "吨"
);

// Price-RS 实际示例
let engineering_item = node!(
    "engineering_item",
    "工程项目",
    "",
    "code" => "010101001",
    "name" => "挖土方",
    "unit" => "m³",
    "quantity" => "1000",
    "unit_price" => "25.50"
);
```

### mark!

快速创建标记实例。

```rust
// 基本用法
let bold = mark!("bold");

// 带描述
let italic = mark!("italic", "斜体文本");

// 带属性
let discount = mark!(
    "discount",
    "折扣标记",
    "rate" => "0.15",
    "type" => "percentage",
    "reason" => "批量采购"
);

// Price-RS 实际示例
let tax_mark = mark!(
    "tax",
    "税费标记",
    "rate" => "0.09",
    "type" => "vat",
    "deductible" => "true"
);
```

## 操作函数宏

### mf_op!

创建操作函数，支持两种语法。

```rust
// 闭包语法（需要 manager 参数）
mf_op!(init_database, |manager| {
    println!("初始化价格数据库");
    let db = PriceDatabase::connect("price.db")?;
    manager.set_resource("database", db);
    Ok(())
});

// 块语法（不需要参数）
mf_op!(clear_cache, {
    println!("清理价格缓存");
    PriceCache::global().clear();
    println!("缓存已清理");
    Ok(())
});

// 复杂操作函数
mf_op!(setup_price_rules, |manager| {
    // 设置价格规则
    let rules = vec![
        PriceRule::new("volume_discount", 0.05),
        PriceRule::new("member_discount", 0.10),
        PriceRule::new("seasonal_adjustment", -0.02),
    ];

    for rule in rules {
        manager.register_rule(rule)?;
    }

    println!("已注册 {} 条价格规则", rules.len());
    Ok(())
});
```

### mf_ops!

批量声明操作函数。

```rust
// 声明操作函数块
mf_ops!(price_operations, [
    init_calculator,
    setup_currency,
    load_price_data,
    validate_prices
]);

// 实现各个函数
fn init_calculator(manager: &GlobalResourceManager) -> ForgeResult<()> {
    manager.set_resource("calculator", PriceCalculator::new());
    Ok(())
}

fn setup_currency(manager: &GlobalResourceManager) -> ForgeResult<()> {
    manager.set_resource("currency", "CNY");
    Ok(())
}

fn load_price_data(manager: &GlobalResourceManager) -> ForgeResult<()> {
    let data = load_from_database()?;
    manager.set_resource("price_data", data);
    Ok(())
}

fn validate_prices(manager: &GlobalResourceManager) -> ForgeResult<()> {
    let data = manager.get_resource::<PriceData>("price_data")?;
    data.validate()?;
    Ok(())
}
```

### mf_global_attr!

创建全局属性项。

```rust
// 单个属性
let attr1 = mf_global_attr!("editor", "theme", "light");

// 属性组
let attr2 = mf_global_attr!(
    vec!["pricing", "config"],
    vec![
        ("currency", AttributeSpec {
            default: Some(Value::String("CNY".to_string()))
        }),
        ("tax_rate", AttributeSpec {
            default: Some(Value::Number(0.09.into()))
        }),
        ("precision", AttributeSpec {
            default: Some(Value::Number(2.into()))
        })
    ]
);

// Price-RS 配置示例
let price_config = mf_global_attr!(
    vec!["engineering", "calculation"],
    vec![
        ("overhead_rate", AttributeSpec {
            default: Some(Value::Number(0.05.into()))
        }),
        ("profit_rate", AttributeSpec {
            default: Some(Value::Number(0.07.into()))
        }),
        ("risk_factor", AttributeSpec {
            default: Some(Value::Number(0.02.into()))
        })
    ]
);
```

## 辅助宏

### impl_state_field!

快速实现 StateField trait。

```rust
use mf_macro::impl_state_field;

// 价格历史记录
struct PriceHistory {
    records: Vec<PriceRecord>,
    max_records: usize,
}

impl_state_field!(PriceHistory, {
    fn initialize() -> Self {
        Self {
            records: Vec::new(),
            max_records: 1000,
        }
    }

    fn add_record(&mut self, record: PriceRecord) {
        if self.records.len() >= self.max_records {
            self.records.remove(0); // 移除最旧的记录
        }
        self.records.push(record);
    }

    fn get_average_price(&self) -> Option<f64> {
        if self.records.is_empty() {
            return None;
        }
        let sum: f64 = self.records.iter()
            .map(|r| r.price)
            .sum();
        Some(sum / self.records.len() as f64)
    }
});
```

### derive_plugin_state!

为类型实现 Resource trait。

```rust
use mf_macro::derive_plugin_state;

#[derive(Clone, Debug)]
struct PriceCache {
    cache: HashMap<String, f64>,
    last_update: DateTime<Utc>,
}

derive_plugin_state!(PriceCache);

// 现在 PriceCache 可以作为插件状态使用
```

## 完整示例：工程造价系统

展示如何组合使用各种宏构建完整的价格计算系统。

```rust
use mf_macro::*;
use mf_core::{ForgeResult, types::Extensions};

// 1. 定义操作函数
fn init_price_system(manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("初始化工程造价系统");

    // 加载价格数据库
    let db = EngineeringPriceDB::connect("engineering.db")?;
    manager.set_resource("price_db", db);

    // 设置计算引擎
    let engine = PriceCalculationEngine::new();
    manager.set_resource("calc_engine", engine);

    Ok(())
}

// 2. 创建价格验证插件
mf_plugin!(
    engineering_price_validator,
    metadata = mf_meta!(
        version = "1.0.0",
        description = "工程造价数据验证",
        author = "Price-RS Team"
    ),
    init = |state| {
        state.register_validator("price_range", |price: f64| {
            price >= 0.0 && price <= 999999999.99
        });
        state.register_validator("quantity", |qty: f64| {
            qty > 0.0
        });
        Ok(())
    }
);

// 3. 节点转换函数
fn transform_engineering_nodes(node: &mut Node) -> ForgeResult<()> {
    match node.name.as_str() {
        "engineering_item" => {
            // 添加项目编码前缀
            if let Some(code) = node.get_attr("code") {
                if !code.starts_with("PROJ-") {
                    let new_code = format!("PROJ-{}", code);
                    node.set_attr("code", Some(Value::String(new_code)));
                }
            }
        },
        "summary" => {
            // 汇总节点添加统计信息
            node.set_attr("generated_at",
                Some(Value::String(Utc::now().to_rfc3339())));
        },
        _ => {}
    }
    Ok(())
}

// 4. 组装成完整扩展
mf_extension!(
    engineering_price_system,
    ops = [init_price_system],
    plugins = [engineering_price_validator],
    global_attributes = [
        mf_global_attr!(
            vec!["engineering"],
            vec![
                ("project_name", AttributeSpec {
                    default: Some(Value::String("未命名项目".to_string()))
                }),
                ("location", AttributeSpec {
                    default: Some(Value::String("".to_string()))
                }),
                ("currency", AttributeSpec {
                    default: Some(Value::String("CNY".to_string()))
                })
            ]
        )
    ],
    node_transform = transform_engineering_nodes,
    nodes = [
        // 项目结构节点
        node!("project", "工程项目"),
        node!("section", "分部工程"),
        node!("subsection", "分项工程"),

        // 清单项节点
        node!("bill_item", "清单项", "",
            "code" => "",
            "name" => "",
            "unit" => "",
            "quantity" => "0",
            "unit_price" => "0",
            "total_price" => "0"
        ),

        // 费用节点
        node!("labor_cost", "人工费", "", "amount" => "0"),
        node!("material_cost", "材料费", "", "amount" => "0"),
        node!("machinery_cost", "机械费", "", "amount" => "0"),
        node!("overhead", "管理费", "", "rate" => "0.05"),
        node!("profit", "利润", "", "rate" => "0.07"),
        node!("tax", "税金", "", "rate" => "0.09"),

        // 汇总节点
        node!("subtotal", "小计"),
        node!("total", "合计"),
        node!("summary", "工程造价汇总")
    ],
    marks = [
        // 价格标记
        mark!("adjusted", "调整价格", "factor" => "1.0", "reason" => ""),
        mark!("provisional", "暂定价格"),
        mark!("fixed", "固定价格"),

        // 状态标记
        mark!("approved", "已审核", "approver" => "", "date" => ""),
        mark!("revised", "已修订", "version" => "1", "date" => ""),

        // 分类标记
        mark!("main_material", "主材"),
        mark!("auxiliary_material", "辅材"),
        mark!("consumable", "耗材")
    ],
    docs = "工程造价计算系统扩展，提供完整的造价编制和管理功能"
);

// 5. 使用扩展
fn main() -> ForgeResult<()> {
    // 初始化扩展
    let extensions = engineering_price_system::init();

    println!("工程造价系统已初始化");
    println!("加载了 {} 个扩展组件", extensions.len());

    // 分类统计
    let mut ext_count = 0;
    let mut node_count = 0;
    let mut mark_count = 0;

    for ext in &extensions {
        match ext {
            Extensions::E(_) => ext_count += 1,
            Extensions::N(_) => node_count += 1,
            Extensions::M(_) => mark_count += 1,
        }
    }

    println!("- Extension: {}", ext_count);
    println!("- Node 定义: {}", node_count);
    println!("- Mark 定义: {}", mark_count);

    Ok(())
}
```

## 最佳实践

### 1. 宏选择指南

| 场景 | 推荐宏 | 原因 |
|------|--------|------|
| 完整功能模块 | `mf_extension!` | 包含所有组件，易于管理 |
| 动态配置需求 | `mf_extension_with_config!` | 支持运行时参数 |
| 状态管理 | `mf_plugin!` | 提供生命周期钩子 |
| 快速原型 | `node!`, `mark!` | 简单直接，适合测试 |
| 批量操作 | `mf_ops!` | 组织相关函数 |

### 2. 性能优化

```rust
// ✅ 好：预编译静态内容
mf_extension!(
    optimized_extension,
    ops = [init_once],  // 初始化函数只调用一次
    nodes = [/* 静态节点定义 */],
    marks = [/* 静态标记定义 */]
);

// ❌ 避免：运行时重复创建
fn bad_pattern() {
    for _ in 0..100 {
        let node = node!("temp", "临时节点"); // 重复创建
    }
}
```

### 3. 错误处理

```rust
// 操作函数中的错误处理
mf_op!(safe_operation, |manager| {
    // 使用 ? 操作符传播错误
    let data = load_data()?;

    // 自定义错误处理
    let result = match process_data(data) {
        Ok(r) => r,
        Err(e) => {
            log::error!("处理失败: {}", e);
            return Err(e.into());
        }
    };

    manager.set_resource("result", result);
    Ok(())
});
```

### 4. 模块化组织

```rust
// 按功能模块组织
mod pricing {
    use mf_macro::*;

    mf_extension!(pricing_module, /* ... */);
}

mod calculation {
    use mf_macro::*;

    mf_extension!(calculation_module, /* ... */);
}

mod reporting {
    use mf_macro::*;

    mf_extension!(reporting_module, /* ... */);
}

// 统一导出
pub fn init_all() -> Vec<Extensions> {
    let mut all = Vec::new();
    all.extend(pricing::pricing_module::init());
    all.extend(calculation::calculation_module::init());
    all.extend(reporting::reporting_module::init());
    all
}
```

## 版本兼容性

- 声明式宏从 ModuForge v0.1.0 开始提供
- 需要 Rust 1.56+ （稳定版）
- 向后兼容：新版本保持 API 稳定性
- 与过程宏（`moduforge-macros-derive`）完全兼容
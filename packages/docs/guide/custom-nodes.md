# 自定义节点

ModuForge-RS 提供了强大的节点系统，允许开发者定义复杂的业务节点。本章将通过 price-rs 项目的实际实现展示如何创建自定义节点。

## 节点定义基础

### 使用 derive 宏

ModuForge-RS 使用 `#[derive(Node)]` 宏来简化节点定义。来自 `extension-base-schema/src/nodes/project_structure.rs` 的实际示例：

```rust
use mf_derive::Node;
use serde::{Deserialize, Serialize};

/// 工程项目节点 - 整个工程造价文档的根节点
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "GCXM"]                    // 节点类型标识符
#[desc = "工程项目"]                     // 节点描述
#[content = "(DXGC|DWGC)+"]              // 内容规则：可包含一个或多个单项/单位工程
pub struct ProjectNode {
    // 基础信息
    #[attr]
    pub construct_name: Option<String>,  // 工程名称

    #[attr]
    pub construct_code: Option<String>,  // 工程编号

    #[attr]
    pub path: Option<String>,            // 文件路径

    // 地区信息
    #[attr]
    pub ss_province: Option<String>,     // 省份代码

    #[attr]
    pub ss_province_name: Option<String>, // 省份名称

    #[attr]
    pub ss_city: Option<String>,         // 城市代码

    #[attr]
    pub ss_city_name: Option<String>,    // 城市名称

    // 业务属性
    #[attr]
    pub bidding_type: i32,               // 招标类型（默认值）

    #[attr]
    pub version: Option<String>,         // 版本号

    // 标准规范
    #[attr]
    pub qd_standard_id: Option<String>,  // 清单标准ID

    #[attr]
    pub de_standard_id: Option<String>,  // 定额标准ID
}
```

### 节点宏注解说明

| 注解 | 说明 | 示例 |
|------|------|------|
| `#[node_type]` | 定义节点的唯一类型标识 | `#[node_type = "GCXM"]` |
| `#[desc]` | 节点的描述信息 | `#[desc = "工程项目"]` |
| `#[content]` | 定义允许的子节点类型 | `#[content = "(DXGC\|DWGC)+"]` |
| `#[attr]` | 标记字段为节点属性 | `#[attr]` |
| `#[attr(default = value)]` | 设置属性默认值 | `#[attr(default = 0)]` |

## 复杂节点定义

### 单位工程节点（包含大量财务字段）

```rust
/// 单位工程节点 - 展示了处理大量属性的能力
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "DWGC"]
#[desc = "单位工程"]
#[content = "fbfx|csxm|qtxm"]  // 可包含分部分项、措施项目、其他项目
pub struct UnitProjectNode {
    // 基础信息
    #[attr]
    pub up_code: Option<String>,        // 单位工程代码

    #[attr]
    pub up_name: Option<String>,        // 单位工程名称

    #[attr]
    pub uptotal: Option<f64>,           // 单位工程总价

    // 分部分项费用明细（6个字段）
    #[attr]
    pub fbfxhj: Option<f64>,            // 分部分项合计

    #[attr]
    pub fbfxrgf: Option<f64>,           // 分部分项人工费

    #[attr]
    pub fbfxclf: Option<f64>,           // 分部分项材料费

    #[attr]
    pub fbfxjxf: Option<f64>,           // 分部分项机械费

    #[attr]
    pub fbfxlr: Option<f64>,            // 分部分项利润

    #[attr]
    pub fbfxglf: Option<f64>,           // 分部分项管理费

    // 措施项目费用明细（6个字段）
    #[attr]
    pub csxhj: Option<f64>,             // 措施项目合计

    #[attr]
    pub csxrgf: Option<f64>,            // 措施项目人工费

    #[attr]
    pub csxclf: Option<f64>,            // 措施项目材料费

    #[attr]
    pub csxjxf: Option<f64>,            // 措施项目机械费

    #[attr]
    pub csxglf: Option<f64>,            // 措施项目管理费

    #[attr]
    pub csxlr: Option<f64>,             // 措施项目利润

    // 税费相关
    #[attr]
    pub jxse: Option<f64>,              // 进项税额

    #[attr]
    pub xxse: Option<f64>,              // 销项税额

    #[attr]
    pub zzsynse: Option<f64>,           // 增值税应纳税额

    #[attr]
    pub fjse: Option<f64>,              // 附加税额

    #[attr]
    pub sj: Option<f64>,                // 税金

    // 其他费用
    #[attr]
    pub gfee: Option<f64>,              // 规费

    #[attr]
    pub safe_fee: Option<f64>,          // 安全文明施工费

    #[attr]
    pub sbf: Option<f64>,               // 设备费

    // 元数据
    #[attr]
    pub create_date: Option<String>,    // 创建时间

    #[attr]
    pub update_date: Option<String>,    // 更新时间

    #[attr]
    pub tenant_id: Option<String>,      // 租户ID

    #[attr]
    pub sort_no: Option<i32>,           // 排序号

    // ... 实际包含 70+ 个财务相关字段
}
```

## 内容规则定义

### 内容规则语法

ModuForge-RS 支持类似正则表达式的内容规则：

| 规则 | 说明 | 示例 |
|------|------|------|
| `type` | 允许特定类型 | `"paragraph"` |
| `type+` | 一个或多个 | `"DXGC+"` |
| `type*` | 零个或多个 | `"text*"` |
| `type?` | 零个或一个 | `"title?"` |
| `(a\|b)` | 选择 | `"(DXGC\|DWGC)"` |
| `(a\|b)+` | 一个或多个选择 | `"(DXGC\|DWGC)+"` |

### 实际应用示例

```rust
// 工程项目：必须包含至少一个单项或单位工程
#[content = "(DXGC|DWGC)+"]

// 单项工程：可包含单位工程或子单项工程
#[content = "(DWGC|DXGC)+"]

// 分部分项：可包含分部或清单
#[content = "(fb|qd)*"]

// 清单：可包含定额或定额人材机
#[content = "(de|dercj)*"]
```

## 节点注册与扩展

### 创建扩展模块

```rust
use mf_macro::{mf_extension, mf_op};
use mf_core::{Extension, Node, ForgeResult};

// 定义初始化操作
mf_op!(setup, |manager| {
    println!("初始化工程项目节点系统");
    // 设置相关资源
    Ok(())
});

// 节点转换函数
fn transform_nodes(node: &mut Node) -> ForgeResult<()> {
    // 标记根节点
    if node.name == "GCXM" {
        node.set_top_node();
    }

    // 其他节点转换逻辑
    match node.name.as_str() {
        "DWGC" => {
            // 为单位工程设置默认属性
            if node.attrs.get("fbfxhj").is_none() {
                node.attrs.insert("fbfxhj", json!(0.0));
            }
        }
        _ => {}
    }

    Ok(())
}

// 创建扩展
mf_extension!(
    base_schema,
    ops = [setup],
    plugins = [project_structure::new()],
    node_transform = transform_nodes,
    nodes = [
        ProjectNode::node_definition(),
        SingleProjectNode::node_definition(),
        UnitProjectNode::node_definition()
    ],
    docs = "工程项目基础树结构定义"
);
```

### 节点定义辅助函数

```rust
// 提供静态访问器以便向后兼容
pub const DWGC_STR: &str = "DWGC";
pub const DXGC_STR: &str = "DXGC";
pub const GCXM_STR: &str = "GCXM";

pub fn gcxm_node() -> Node {
    ProjectNode::node_definition()
}

pub fn dxgc_node() -> Node {
    SingleProjectNode::node_definition()
}

pub fn dwgc_node() -> Node {
    UnitProjectNode::node_definition()
}
```

## 节点属性管理

### 属性类型支持

```rust
use serde_json::Value;

// 支持的属性类型
pub enum AttrType {
    String,     // 字符串：Option<String>
    Number,     // 数字：Option<f64> 或 i32
    Boolean,    // 布尔值：Option<bool>
    Object,     // 对象：HashMap<String, Value>
    Array,      // 数组：Vec<Value>
}

// 属性定义示例
#[derive(Node)]
pub struct CustomNode {
    #[attr]
    pub name: Option<String>,           // 字符串属性

    #[attr]
    pub price: Option<f64>,             // 浮点数属性

    #[attr]
    pub quantity: Option<i32>,          // 整数属性

    #[attr]
    pub is_active: Option<bool>,        // 布尔属性

    #[attr(default = 1.0)]
    pub tax_rate: f64,                  // 带默认值的属性
}
```

### 属性验证

```rust
impl CustomNode {
    /// 验证节点属性的合法性
    pub fn validate(&self) -> Result<(), String> {
        // 验证必填字段
        if self.name.is_none() {
            return Err("名称不能为空".to_string());
        }

        // 验证数值范围
        if let Some(price) = self.price {
            if price < 0.0 {
                return Err("价格不能为负数".to_string());
            }
        }

        // 验证业务规则
        if let (Some(price), Some(quantity)) = (self.price, self.quantity) {
            let total = price * quantity as f64;
            if total > 1_000_000.0 {
                return Err("总价超出限制".to_string());
            }
        }

        Ok(())
    }
}
```

## 节点生命周期

### 创建节点

```rust
use mf_state::Transaction;

// 方式1：使用工厂方法
let factory = tr.schema.factory();
let node = factory.create_tree(
    "GCXM",                              // 节点类型
    None,                                // 可选的节点ID
    Some(&hashmap!{                     // 属性
        "construct_name" => json!("示例工程"),
        "construct_code" => json!("PROJ-001"),
    }),
    vec![],                              // 子节点
    None,                                // 标记
)?;

// 方式2：直接构造
let project = ProjectNode {
    construct_name: Some("示例工程".to_string()),
    construct_code: Some("PROJ-001".to_string()),
    bidding_type: 1,
    ..Default::default()
};

// 转换为Node并添加到文档
let node = project.to_node();
tr.add_node(parent_id, vec![node])?;
```

### 更新节点

```rust
// 更新节点属性
tr.set_node_attribute(
    node_id,
    hashmap!{
        "uptotal" => json!(1000000.0),
        "update_date" => json!(chrono::Utc::now().to_string()),
    }
)?;
```

### 删除节点

```rust
// 从父节点删除
let parent = tr.doc().get_parent_node(&node_id);
if let Some(parent_node) = parent {
    tr.remove_node(parent_node.id, vec![node_id])?;
}
```

## 高级特性

### 计算属性

某些属性可以根据其他属性自动计算：

```rust
impl UnitProjectNode {
    /// 计算单位工程总价
    pub fn calculate_total(&mut self) {
        let fbfx = self.fbfxhj.unwrap_or(0.0);   // 分部分项合计
        let csx = self.csxhj.unwrap_or(0.0);     // 措施项目合计
        let qtxm = self.qtxmhj.unwrap_or(0.0);   // 其他项目合计
        let gf = self.gfee.unwrap_or(0.0);       // 规费
        let sj = self.sj.unwrap_or(0.0);         // 税金

        // 计算总价
        self.uptotal = Some(fbfx + csx + qtxm + gf + sj);

        // 计算单位造价（如果有建筑面积）
        if let Some(area) = self.jzmj {
            if area > 0.0 {
                self.unitcost = Some(self.uptotal.unwrap_or(0.0) / area);
            }
        }
    }
}
```

### 节点关系验证

```rust
impl ProjectNode {
    /// 验证子节点的合法性
    pub fn validate_children(&self, children: &[Node]) -> Result<()> {
        // 至少需要一个子工程
        if children.is_empty() {
            return Err(anyhow!("工程项目必须包含至少一个单项或单位工程"));
        }

        // 验证子节点类型
        for child in children {
            match child.node_type.as_str() {
                "DXGC" | "DWGC" => {
                    // 合法的子节点类型
                }
                _ => {
                    return Err(anyhow!(
                        "工程项目只能包含单项工程(DXGC)或单位工程(DWGC)，不能包含 {}",
                        child.node_type
                    ));
                }
            }
        }

        Ok(())
    }
}
```

### 节点序列化

```rust
// 序列化为 JSON
let json_str = serde_json::to_string(&project_node)?;

// 反序列化
let project: ProjectNode = serde_json::from_str(&json_str)?;

// 自定义序列化（跳过空值）
#[derive(Serialize)]
#[serde(skip_serializing_if = "Option::is_none")]
pub struct CompactNode {
    pub name: Option<String>,
    pub value: Option<f64>,
}
```

## 最佳实践

### 1. 节点设计原则

- **单一职责**：每个节点类型代表一个明确的业务概念
- **属性最小化**：只包含必要的属性，避免冗余
- **类型安全**：使用 `Option<T>` 处理可选属性，使用具体类型而非 `Value`
- **默认值合理**：为常用属性提供合理的默认值

### 2. 命名规范

```rust
// 节点类型：大写字母或大写下划线
#[node_type = "GCXM"]        // ✓ 工程项目
#[node_type = "UNIT_PROJECT"] // ✓ 单位工程

// 结构体名：PascalCase
pub struct ProjectNode { }    // ✓
pub struct UnitProjectNode { } // ✓

// 属性名：snake_case
pub construct_name: Option<String>  // ✓
pub up_code: Option<String>         // ✓
```

### 3. 性能优化

- 对于大量属性的节点，考虑分组或使用嵌套结构
- 使用 Arc 共享大型只读数据
- 实现懒加载机制处理计算密集型属性

### 4. 错误处理

```rust
// 提供清晰的错误信息
if tr.doc().get_node(parent_id).is_none() {
    return Err(anyhow::anyhow!(
        "无法添加节点：父节点 {} 不存在",
        parent_id
    ));
}

// 验证输入数据
if node_type != "GCXM" && node_type != "DXGC" {
    return Err(anyhow::anyhow!(
        "不支持的节点类型: {}。支持的类型: GCXM, DXGC, DWGC",
        node_type
    ));
}
```

## 总结

ModuForge-RS 的自定义节点系统通过：

1. **简洁的宏定义**：使用 derive 宏快速定义复杂节点
2. **灵活的内容规则**：支持复杂的父子关系定义
3. **强大的属性系统**：支持多种类型和默认值
4. **完整的生命周期**：创建、更新、删除、验证一应俱全
5. **实战验证**：Price-RS 项目证明了系统能处理包含 70+ 属性的复杂业务节点

通过本章的学习，您应该能够为自己的项目创建符合业务需求的自定义节点。

下一章：[自定义标记](./custom-marks.md)
# ModuForge Node 模型到建筑预算的精确映射

## 📊 ModuForge Node 模型分析

### 核心结构

```rust
// ModuForge 的 Node 定义
pub struct Node {
    pub id: NodeId,              // 节点唯一标识
    pub r#type: String,          // 节点类型名称
    pub attrs: Attrs,            // 节点属性（键值对）
    pub content: Vec<NodeId>,    // 子节点ID列表
    pub marks: Vec<Mark>,        // 节点标记列表
}

// 节点类型规范
pub struct NodeSpec {
    pub content: Option<String>,         // 子节点约束表达式 "item*" 
    pub marks: Option<String>,           // 标记约束
    pub attrs: Option<HashMap<String, AttributeSpec>>, // 属性规范
    pub desc: Option<String>,            // 描述
}
```

## 🏗️ 建筑预算业务映射

### 1. 预算项目层级结构映射

```
预算文档 (Document)
├── 工程项目 (Project)
│   ├── 单位工程 (Unit Engineering)  
│   │   ├── 分部工程 (Division)
│   │   │   ├── 分项工程 (Subdivision)
│   │   │   │   ├── 清单项目 (Item)
│   │   │   │   └── 清单项目 (Item)
│   │   │   └── 分项工程 (Subdivision)
│   │   └── 分部工程 (Division)
│   └── 单位工程 (Unit Engineering)
└── 材料库 (Material Library)
    ├── 材料分类 (Material Category)
    └── 材料项 (Material Item)
```

### 2. Node 类型定义

```rust
// 预算业务的 NodeSpec 定义
use moduforge_model::node_type::NodeSpec;
use std::collections::HashMap;

pub fn create_budget_schema() -> HashMap<String, NodeSpec> {
    let mut nodes = HashMap::new();
    
    // 1. 预算文档 (顶级节点)
    nodes.insert("budget_doc".to_string(), NodeSpec {
        content: Some("project+ material_lib?".to_string()), // 1个或多个项目，可选材料库
        marks: None,
        attrs: Some(HashMap::from([
            ("doc_name".to_string(), AttributeSpec { default: None }),
            ("doc_code".to_string(), AttributeSpec { default: None }),
            ("region".to_string(), AttributeSpec { default: Some(json!("beijing")) }),
            ("quota_standard".to_string(), AttributeSpec { default: Some(json!("2012")) }),
            ("created_at".to_string(), AttributeSpec { default: None }),
        ])),
        desc: Some("预算文档根节点".to_string()),
    });

    // 2. 工程项目
    nodes.insert("project".to_string(), NodeSpec {
        content: Some("unit_engineering+".to_string()), // 1个或多个单位工程
        marks: Some("priority cost_type".to_string()),   // 支持优先级和费用类型标记
        attrs: Some(HashMap::from([
            ("project_name".to_string(), AttributeSpec { default: None }),
            ("project_code".to_string(), AttributeSpec { default: None }),
            ("total_amount".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("status".to_string(), AttributeSpec { default: Some(json!("draft")) }),
        ])),
        desc: Some("工程项目".to_string()),
    });

    // 3. 单位工程
    nodes.insert("unit_engineering".to_string(), NodeSpec {
        content: Some("division+".to_string()), // 1个或多个分部工程
        marks: None,
        attrs: Some(HashMap::from([
            ("unit_name".to_string(), AttributeSpec { default: None }),
            ("unit_code".to_string(), AttributeSpec { default: None }),
            ("engineering_type".to_string(), AttributeSpec { 
                default: Some(json!("construction")) // construction, installation, decoration
            }),
        ])),
        desc: Some("单位工程".to_string()),
    });

    // 4. 分部工程
    nodes.insert("division".to_string(), NodeSpec {
        content: Some("subdivision+".to_string()), // 1个或多个分项工程
        marks: None,
        attrs: Some(HashMap::from([
            ("division_name".to_string(), AttributeSpec { default: None }),
            ("division_code".to_string(), AttributeSpec { default: None }),
        ])),
        desc: Some("分部工程".to_string()),
    });

    // 5. 分项工程
    nodes.insert("subdivision".to_string(), NodeSpec {
        content: Some("budget_item+".to_string()), // 1个或多个清单项目
        marks: None,
        attrs: Some(HashMap::from([
            ("subdivision_name".to_string(), AttributeSpec { default: None }),
            ("subdivision_code".to_string(), AttributeSpec { default: None }),
        ])),
        desc: Some("分项工程".to_string()),
    });

    // 6. 清单项目 (叶子节点 - 实际的计价单元)
    nodes.insert("budget_item".to_string(), NodeSpec {
        content: None, // 叶子节点，无子节点
        marks: Some("calculated price_locked quota_applied".to_string()),
        attrs: Some(HashMap::from([
            ("item_code".to_string(), AttributeSpec { default: None }),
            ("item_name".to_string(), AttributeSpec { default: None }),
            ("unit".to_string(), AttributeSpec { default: Some(json!("m³")) }),
            ("quantity".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("unit_price".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("amount".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("quota_code".to_string(), AttributeSpec { default: None }),
            ("material_cost".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("labor_cost".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("machine_cost".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("measure_fee_rate".to_string(), AttributeSpec { default: Some(json!(0.05)) }),
        ])),
        desc: Some("清单项目".to_string()),
    });

    // 7. 材料库
    nodes.insert("material_lib".to_string(), NodeSpec {
        content: Some("material_category+ material_item*".to_string()),
        marks: None,
        attrs: Some(HashMap::from([
            ("lib_name".to_string(), AttributeSpec { default: Some(json!("默认材料库")) }),
            ("version".to_string(), AttributeSpec { default: Some(json!("1.0")) }),
        ])),
        desc: Some("材料库".to_string()),
    });

    // 8. 材料分类
    nodes.insert("material_category".to_string(), NodeSpec {
        content: Some("material_item+".to_string()),
        marks: None,
        attrs: Some(HashMap::from([
            ("category_name".to_string(), AttributeSpec { default: None }),
            ("category_code".to_string(), AttributeSpec { default: None }),
        ])),
        desc: Some("材料分类".to_string()),
    });

    // 9. 材料项目
    nodes.insert("material_item".to_string(), NodeSpec {
        content: None, // 叶子节点
        marks: Some("price_updated regional".to_string()),
        attrs: Some(HashMap::from([
            ("material_code".to_string(), AttributeSpec { default: None }),
            ("material_name".to_string(), AttributeSpec { default: None }),
            ("unit".to_string(), AttributeSpec { default: None }),
            ("market_price".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("budget_price".to_string(), AttributeSpec { default: Some(json!(0.0)) }),
            ("supplier".to_string(), AttributeSpec { default: None }),
            ("region".to_string(), AttributeSpec { default: None }),
            ("price_date".to_string(), AttributeSpec { default: None }),
        ])),
        desc: Some("材料项目".to_string()),
    });

    nodes
}
```

### 3. 标记 (Mark) 系统映射

```rust
use moduforge_model::mark_type::MarkSpec;

pub fn create_budget_marks() -> HashMap<String, MarkSpec> {
    let mut marks = HashMap::new();
    
    // 计算状态标记
    marks.insert("calculated".to_string(), MarkSpec {
        attrs: Some(HashMap::from([
            ("last_calculated".to_string(), AttributeSpec { default: None }),
            ("calculation_version".to_string(), AttributeSpec { default: Some(json!("1.0")) }),
        ])),
        excludes: None,
        inclusive: Some(false),
        spanning: Some(false),
    });
    
    // 价格锁定标记
    marks.insert("price_locked".to_string(), MarkSpec {
        attrs: Some(HashMap::from([
            ("locked_by".to_string(), AttributeSpec { default: None }),
            ("locked_at".to_string(), AttributeSpec { default: None }),
            ("reason".to_string(), AttributeSpec { default: None }),
        ])),
        excludes: None,
        inclusive: Some(false),
        spanning: Some(false),
    });
    
    // 定额应用标记
    marks.insert("quota_applied".to_string(), MarkSpec {
        attrs: Some(HashMap::from([
            ("quota_code".to_string(), AttributeSpec { default: None }),
            ("applied_at".to_string(), AttributeSpec { default: None }),
            ("conversion_factor".to_string(), AttributeSpec { default: Some(json!(1.0)) }),
        ])),
        excludes: None,
        inclusive: Some(false),
        spanning: Some(false),
    });
    
    // 优先级标记
    marks.insert("priority".to_string(), MarkSpec {
        attrs: Some(HashMap::from([
            ("level".to_string(), AttributeSpec { 
                default: Some(json!("normal")) // high, normal, low
            }),
        ])),
        excludes: None,
        inclusive: Some(false),
        spanning: Some(false),
    });

    marks
}
```

### 4. 实际使用示例

```rust
use moduforge_model::node::Node;
use moduforge_model::attrs::Attrs;
use moduforge_model::mark::Mark;

// 创建一个清单项目节点
pub fn create_budget_item_node() -> Node {
    let mut attrs = im::HashMap::new();
    attrs.insert("item_code".to_string(), json!("010101001001"));
    attrs.insert("item_name".to_string(), json!("挖基础土方"));
    attrs.insert("unit".to_string(), json!("m³"));
    attrs.insert("quantity".to_string(), json!(1000.0));
    attrs.insert("unit_price".to_string(), json!(45.50));
    attrs.insert("amount".to_string(), json!(45500.0));
    attrs.insert("quota_code".to_string(), json!("A1-1"));

    // 添加计算标记
    let calculated_mark = Mark {
        r#type: "calculated".to_string(),
        attrs: {
            let mut mark_attrs = im::HashMap::new();
            mark_attrs.insert("last_calculated".to_string(), json!("2024-01-20T10:00:00Z"));
            mark_attrs.insert("calculation_version".to_string(), json!("1.0"));
            Attrs::from(mark_attrs)
        },
    };

    Node::new(
        "item_001",
        "budget_item".to_string(),
        Attrs::from(attrs),
        vec![], // 无子节点
        vec![calculated_mark],
    )
}

// 创建项目结构
pub fn create_project_structure() -> Node {
    let mut project_attrs = im::HashMap::new();
    project_attrs.insert("project_name".to_string(), json!("住宅楼工程"));
    project_attrs.insert("project_code".to_string(), json!("BJ2024001"));
    project_attrs.insert("total_amount".to_string(), json!(5000000.0));

    // 创建项目节点，包含子节点ID
    Node::new(
        "project_001",
        "project".to_string(),
        Attrs::from(project_attrs),
        vec!["unit_001".to_string()], // 包含一个单位工程
        vec![],
    )
}
```

### 5. 业务操作映射

#### 5.1 数据变更操作

```rust
// ModuForge 的事务操作映射到预算业务
use moduforge_state::transaction::Transaction;

impl BudgetRuntime {
    // 更新工程量
    pub async fn update_quantity(&mut self, item_id: &str, quantity: f64) -> RuntimeResult<()> {
        let mut tr = self.get_tr();
        
        // 使用 ModuForge 的属性更新机制
        tr.set_node_markup(item_id, "quantity", json!(quantity))?;
        
        // 触发重新计算 (通过中间件自动处理)
        self.dispatch(tr).await
    }
    
    // 应用定额
    pub async fn apply_quota(&mut self, item_id: &str, quota_code: &str) -> RuntimeResult<()> {
        let mut tr = self.get_tr();
        
        // 更新定额代码属性
        tr.set_node_markup(item_id, "quota_code", json!(quota_code))?;
        
        // 添加定额应用标记
        let quota_mark = Mark {
            r#type: "quota_applied".to_string(),
            attrs: {
                let mut attrs = im::HashMap::new();
                attrs.insert("quota_code".to_string(), json!(quota_code));
                attrs.insert("applied_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
                Attrs::from(attrs)
            },
        };
        tr.add_mark(item_id, quota_mark)?;
        
        self.dispatch(tr).await
    }
}
```

#### 5.2 查询操作

```rust
// 利用 ModuForge 的查询能力
impl BudgetRuntime {
    // 查询所有清单项目
    pub fn get_all_budget_items(&self) -> Vec<Arc<Node>> {
        self.doc()
            .query()
            .by_type("budget_item")
            .execute()
    }
    
    // 查询特定分部工程下的项目
    pub fn get_items_by_division(&self, division_code: &str) -> Vec<Arc<Node>> {
        self.doc()
            .query()
            .by_type("budget_item")
            .by_ancestor_attr("division_code", &json!(division_code))
            .execute()
    }
    
    // 查询已应用定额的项目
    pub fn get_quota_applied_items(&self) -> Vec<Arc<Node>> {
        self.doc()
            .query()
            .by_type("budget_item")
            .by_mark("quota_applied")
            .execute()
    }
    
    // 计算项目总金额
    pub fn calculate_total_amount(&self) -> f64 {
        self.get_all_budget_items()
            .iter()
            .map(|item| {
                item.attrs.get("amount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            })
            .sum()
    }
}
```

## 🎯 核心优势

### 1. **结构化数据管理**
- Node 的层级结构完美映射预算的分级结构
- 属性系统承载业务数据（价格、数量等）
- 标记系统记录业务状态（计算状态、锁定状态等）

### 2. **强类型约束**
- NodeSpec 的 content 约束确保结构合法性
- 属性规范保证数据完整性
- 标记规范控制业务状态流转

### 3. **事务性操作**
- 所有业务变更通过 Transaction 进行
- 保证数据一致性和可回滚
- 支持批量操作的原子性

### 4. **高效查询**
- 基于类型、属性、标记的多维度查询
- 支持层级关系查询
- 内置索引优化性能

### 5. **事件驱动**
- 数据变更自动触发计算
- 支持复杂的业务逻辑响应
- 实现松耦合的模块通信

这种映射方式充分利用了 ModuForge 的技术优势，同时完美适配了建筑预算的业务特点！ 
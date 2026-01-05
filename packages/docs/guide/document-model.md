# 文档模型

ModuForge-RS 的文档模型基于不可变的树形结构，提供高性能和安全的文档操作。本章将通过 price-rs 项目的实际应用来展示文档模型的强大功能。

## 核心概念

### 节点（Node）

节点是文档的基本构建块。在 price-rs 项目中，节点表示工程造价的各个组成部分：

- **工程项目节点（GCXM）**：表示整个工程项目
- **单项工程节点（DXGC）**：表示项目中的单个工程
- **单位工程节点（DWGC）**：表示工程的具体单位
- **分部节点（fb）**：表示工程的分部信息
- **清单节点（qd）**：表示工程清单项

### Price-RS 项目结构层级

```
GCXM（工程项目）
├── DXGC（单项工程）
│   ├── DWGC（单位工程）
│   │   ├── fbfx（分部分项）
│   │   │   ├── fb（分部）
│   │   │   │   └── qd（清单）
│   │   │   │       ├── de（定额）
│   │   │   │       └── dercj（定额人材机）
│   │   ├── csxm（措施项目）
│   │   └── qtxm（其他项目）
│   └── DXGC（子单项工程）
└── DWGC（直接单位工程）
```

## 实际节点定义

### 工程项目根节点

来自 `price-rs/exts/base-schema/extension-base-schema/src/nodes/project_structure.rs`：

```rust
use mf_derive::Node;
use serde::{Deserialize, Serialize};

/// 工程项目节点 - 整个造价文档的根节点
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "GCXM"]
#[desc = "工程项目"]
#[content = "(DXGC|DWGC)+"]  // 可包含单项工程或单位工程
pub struct ProjectNode {
    // 基础信息
    #[attr]
    pub construct_name: Option<String>,     // 项目名称

    #[attr]
    pub construct_code: Option<String>,     // 项目编号

    #[attr]
    pub path: Option<String>,               // 文件路径

    // 地区信息
    #[attr]
    pub ss_province: Option<String>,        // 省份代码

    #[attr]
    pub ss_province_name: Option<String>,   // 省份名称

    #[attr]
    pub ss_city: Option<String>,           // 城市代码

    #[attr]
    pub ss_city_name: Option<String>,      // 城市名称

    // 标准规范
    #[attr]
    pub qd_standard_id: Option<String>,    // 清单标准ID

    #[attr]
    pub de_standard_id: Option<String>,    // 定额标准ID

    #[attr]
    pub de_standard_release_year: Option<String>, // 定额发布年份

    // 业务属性
    #[attr]
    pub bidding_type: i32,                 // 招标类型

    #[attr]
    pub version: Option<String>,           // 版本号

    #[attr]
    pub main_rcj_show_flag: Option<bool>,  // 主材机显示标志
}
```

### 单项工程节点

```rust
/// 单项工程节点 - 表示独立的工程单元
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "DXGC"]
#[desc = "单项工程"]
#[content = "(DWGC|DXGC)+"]  // 可包含单位工程或子单项工程
pub struct SingleProjectNode {
    // 标识信息
    #[attr]
    pub project_code: Option<String>,      // 项目编码

    #[attr]
    pub project_name: Option<String>,      // 项目名称

    // 面积与造价
    #[attr]
    pub jzmj: Option<f64>,                 // 建筑面积

    #[attr]
    pub total: Option<f64>,                // 总造价

    #[attr]
    pub average: Option<f64>,              // 平均造价

    #[attr]
    pub unitcost: Option<f64>,             // 单位造价

    // 费用明细
    #[attr]
    pub gfee: Option<f64>,                 // 规费

    #[attr]
    pub safe_fee: Option<f64>,             // 安全文明施工费

    #[attr]
    pub sbf: Option<f64>,                  // 设备费

    #[attr]
    pub zbkzj: Option<f64>,                // 招标控制价

    #[attr]
    pub zbj: Option<f64>,                  // 中标价

    // 排序与报表
    #[attr]
    pub sort_no: Option<i32>,              // 排序号

    #[attr]
    pub report_url: Option<String>,        // 报表URL
}
```

### 分部节点（包含大量财务字段）

来自 `price-rs/exts/fbfx/extension-fbfx-csxm/src/nodes/node_definitions.rs`：

```rust
/// 分部节点 - 用于表示工程项目的分部信息
/// 包含 60+ 个财务相关字段，展示了 ModuForge-RS 处理复杂数据模型的能力
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "fb"]
#[desc = "分部"]
#[content = "(fb|qd)*"]  // 可包含子分部或清单
pub struct FbNode {
    // 基本信息
    #[attr]
    pub project_code: String,              // 项目编码

    #[attr]
    pub project_name: String,              // 项目名称

    #[attr]
    pub unit: String,                      // 单位

    #[attr]
    pub quantity: String,                  // 工程量

    #[attr]
    pub quantity_expression: String,       // 工程量表达式

    // 人工费
    #[attr(default = 0.0)]
    pub rfee_price: f64,                   // 人工费单价

    #[attr(default = 0.0)]
    pub rfee_total: f64,                   // 人工费合价

    // 材料费
    #[attr(default = 0.0)]
    pub cfee_price: f64,                   // 材料费单价

    #[attr(default = 0.0)]
    pub cfee_total: f64,                   // 材料费合价

    // 机械费
    #[attr(default = 0.0)]
    pub jfee_price: f64,                   // 机械费单价

    #[attr(default = 0.0)]
    pub jfee_total: f64,                   // 机械费合价

    // 综合费用
    #[attr(default = 0.0)]
    pub total_profit_fee: f64,             // 利润费合价

    #[attr(default = 0.0)]
    pub total_manager_fee: f64,            // 管理费合价

    // 税费计算
    #[attr(default = 0.0)]
    pub jxse_price: f64,                   // 进项税额单价

    #[attr(default = 0.0)]
    pub jxse_total: f64,                   // 进项税额合价

    #[attr(default = 0.0)]
    pub xxse_price: f64,                   // 销项税额单价

    #[attr(default = 0.0)]
    pub xxse_total: f64,                   // 销项税额合价

    // 总价计算
    #[attr(default = 0.0)]
    pub price: f64,                        // 单价

    #[attr(default = 0.0)]
    pub total: f64,                        // 工程造价合价

    // ... 还有 40+ 个其他财务字段
}
```

## 节点池（NodePool）在 Price-RS 中的应用

Price-RS 使用节点池来高效管理数万个工程节点：

```rust
use mf_model::node_pool::{NodePool, NodePoolConfig};
use price_rs::nodes::ProjectNode;

// 配置适合大型工程项目的节点池
let config = NodePoolConfig {
    shard_count: 32,      // 增加分片数以处理大量节点
    cache_size: 5000,     // 大型 LRU 缓存存储热点节点
    gc_threshold: 50000,  // 适应工程项目的节点数量
};

let mut pool = NodePool::with_config(config);

// 创建工程项目
let project = ProjectNode::new()
    .with_name("广州地铁18号线工程")
    .with_code("GZ-METRO-18")
    .with_province("广东省")
    .with_city("广州市");

pool = pool.add_node(project.into());

// 批量添加单项工程
let single_projects = vec![
    SingleProjectNode::new("车站工程", "SP-001"),
    SingleProjectNode::new("隧道工程", "SP-002"),
    SingleProjectNode::new("轨道工程", "SP-003"),
];

for sp in single_projects {
    pool = pool.add_node(sp.into());
}
```

## Schema 验证系统

Price-RS 定义了严格的文档结构规则来确保造价数据的完整性：

```rust
use mf_model::{Schema, NodeType, AttrSpec, AttrType};

pub fn create_price_schema() -> Schema {
    let mut schema = Schema::new();

    // 注册工程项目节点
    schema.register(NodeType {
        name: "GCXM".to_string(),
        desc: Some("工程项目根节点".to_string()),
        content: Some("(DXGC|DWGC)+".to_string()),
        attrs: hashmap! {
            "construct_name" => AttrSpec::required(AttrType::String),
            "construct_code" => AttrSpec::required(AttrType::String),
            "bidding_type" => AttrSpec::optional(AttrType::Number, 1),
            "ss_province" => AttrSpec::optional(AttrType::String),
            "total" => AttrSpec::computed(AttrType::Number), // 计算字段
        },
        group: Some("project".to_string()),
        ..Default::default()
    });

    // 注册分部节点
    schema.register(NodeType {
        name: "fb".to_string(),
        desc: Some("分部节点".to_string()),
        content: Some("(fb|qd)*".to_string()),
        attrs: hashmap! {
            "project_code" => AttrSpec::required(AttrType::String),
            "quantity" => AttrSpec::required(AttrType::String),
            "rfee_price" => AttrSpec::optional(AttrType::Number, 0.0),
            "cfee_price" => AttrSpec::optional(AttrType::Number, 0.0),
            "jfee_price" => AttrSpec::optional(AttrType::Number, 0.0),
            "total" => AttrSpec::computed(AttrType::Number),
        },
        validations: vec![
            Validation::Custom(Box::new(validate_fb_totals)),
        ],
        ..Default::default()
    });

    schema.compile().expect("Schema compilation failed")
}

// 自定义验证函数
fn validate_fb_totals(node: &Node) -> Result<()> {
    let rfee = node.attr("rfee_total").as_f64().unwrap_or(0.0);
    let cfee = node.attr("cfee_total").as_f64().unwrap_or(0.0);
    let jfee = node.attr("jfee_total").as_f64().unwrap_or(0.0);
    let total = node.attr("total").as_f64().unwrap_or(0.0);

    let calculated = rfee + cfee + jfee;

    if (total - calculated).abs() > 0.01 {
        return Err(anyhow!("总价计算错误: {} != {}", total, calculated));
    }

    Ok(())
}
```

## 文档树操作

### 构建工程项目树

```rust
use mf_model::Tree;
use price_rs::nodes::*;

// 构建多层级工程结构
let mut tree = Tree::new();

// 根节点 - 工程项目
let project = ProjectNode::new()
    .with_name("深圳科技园办公楼")
    .with_code("SZ-KJY-2024");

let project_id = tree.add_root(project);

// 第二层 - 单项工程
let main_building = SingleProjectNode::new()
    .with_name("主楼工程")
    .with_code("SP-001")
    .with_area(25000.0)
    .with_total(150000000.0);

let main_id = tree.add_child(project_id, main_building);

// 第三层 - 单位工程
let foundation = UnitProjectNode::new()
    .with_name("基础工程")
    .with_code("UP-001-01")
    .with_total(30000000.0);

tree.add_child(main_id, foundation);

// 第四层 - 分部分项
let fbfx = FbfxNode::new()
    .with_name("土方工程分部");

let fbfx_id = tree.add_child(foundation_id, fbfx);

// 第五层 - 分部
let excavation = FbNode::new()
    .with_code("FB-001")
    .with_name("机械挖土方")
    .with_quantity("15000")
    .with_unit("m³")
    .with_rfee_price(45.0)
    .with_cfee_price(12.0)
    .with_jfee_price(85.0);

tree.add_child(fbfx_id, excavation);
```

### 遍历与查询

```rust
// 深度遍历计算总造价
fn calculate_total_cost(tree: &Tree, node_id: NodeId) -> f64 {
    let node = tree.get_node(node_id).unwrap();
    let mut total = node.attr("total").as_f64().unwrap_or(0.0);

    for child_id in tree.children(node_id) {
        total += calculate_total_cost(tree, child_id);
    }

    total
}

// 查找所有分部节点
fn find_all_fb_nodes(tree: &Tree) -> Vec<NodeId> {
    let mut results = Vec::new();

    tree.traverse_dfs(|node_id, node| {
        if node.node_type == "fb" {
            results.push(node_id);
        }
    });

    results
}

// 按条件筛选节点
fn find_high_cost_items(tree: &Tree, threshold: f64) -> Vec<(NodeId, f64)> {
    let mut items = Vec::new();

    tree.traverse_bfs(|node_id, node| {
        if let Some(total) = node.attr("total").as_f64() {
            if total > threshold {
                items.push((node_id, total));
            }
        }
    });

    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    items
}
```

## 高级特性应用

### 计算属性自动更新

Price-RS 使用插件自动计算和更新造价：

```rust
use mf_macro::mf_plugin;

mf_plugin!(
    auto_calculate_totals,
    append_transaction = async |tr, old_state, new_state| {
        let mut transaction = new_state.tr();

        // 查找所有分部节点
        for node in new_state.doc().find_all("fb") {
            // 获取基础费用
            let rfee = node.attr("rfee_total").as_f64().unwrap_or(0.0);
            let cfee = node.attr("cfee_total").as_f64().unwrap_or(0.0);
            let jfee = node.attr("jfee_total").as_f64().unwrap_or(0.0);

            // 计算管理费（按人工费的15%）
            let manager_fee = rfee * 0.15;

            // 计算利润（按人工费的8%）
            let profit = rfee * 0.08;

            // 计算总价
            let total = rfee + cfee + jfee + manager_fee + profit;

            // 更新节点属性
            transaction.update_node_attrs(&node.id, hashmap!{
                "total_manager_fee" => json!(manager_fee),
                "total_profit_fee" => json!(profit),
                "total" => json!(total),
            })?;
        }

        Ok(Some(transaction))
    }
);
```

### 节点关系验证

确保工程结构的合法性：

```rust
use mf_model::{NodeRelation, NodeConstraint};

impl NodeRelation for FbNode {
    fn allows_child(&self, child_type: &str) -> bool {
        // 分部节点只能包含子分部或清单
        matches!(child_type, "fb" | "qd")
    }

    fn requires_parent(&self) -> Option<&str> {
        // 分部必须在分部分项容器内
        Some("fbfx")
    }

    fn max_depth(&self) -> Option<usize> {
        // 分部最多嵌套3层
        Some(3)
    }
}

impl NodeConstraint for ProjectNode {
    fn validate_children(&self, children: &[Node]) -> Result<()> {
        // 至少要有一个单项或单位工程
        if children.is_empty() {
            return Err(anyhow!("工程项目必须包含至少一个单项或单位工程"));
        }

        // 检查是否有重复的项目编码
        let mut codes = HashSet::new();
        for child in children {
            if let Some(code) = child.attr("project_code").as_str() {
                if !codes.insert(code) {
                    return Err(anyhow!("项目编码重复: {}", code));
                }
            }
        }

        Ok(())
    }
}
```

## 性能优化实践

### 1. 批量操作优化

```rust
// Price-RS 中的批量价格更新
pub fn batch_update_prices(
    pool: &mut NodePool,
    updates: Vec<(NodeId, PriceUpdate)>,
) -> Result<()> {
    // 收集所有更新
    let mut batch = Vec::with_capacity(updates.len());

    for (id, price_update) in updates {
        if let Some(node) = pool.get_node(&id) {
            let mut updated = node.clone();

            // 更新价格字段
            updated.set_attr("rfee_price", price_update.labor);
            updated.set_attr("cfee_price", price_update.material);
            updated.set_attr("jfee_price", price_update.machinery);

            // 重新计算合价
            let quantity = updated.attr("quantity").as_f64().unwrap_or(1.0);
            updated.set_attr("rfee_total", price_update.labor * quantity);
            updated.set_attr("cfee_total", price_update.material * quantity);
            updated.set_attr("jfee_total", price_update.machinery * quantity);

            batch.push((id, updated));
        }
    }

    // 一次性批量更新
    *pool = pool.batch_update(batch);
    Ok(())
}
```

### 2. 懒加载大型工程文档

```rust
use once_cell::sync::OnceCell;
use std::sync::Arc;

/// 懒加载节点包装器，用于处理大型工程文档
pub struct LazyProjectNode {
    id: NodeId,
    basic_info: ProjectBasicInfo,  // 基础信息始终加载
    details: OnceCell<Arc<ProjectDetails>>,  // 详细信息按需加载
    loader: Arc<dyn Fn() -> ProjectDetails + Send + Sync>,
}

impl LazyProjectNode {
    pub fn get_details(&self) -> &ProjectDetails {
        self.details.get_or_init(|| {
            Arc::new((self.loader)())
        })
    }

    pub fn name(&self) -> &str {
        &self.basic_info.name
    }

    pub fn code(&self) -> &str {
        &self.basic_info.code
    }
}
```

### 3. 内存优化策略

```rust
// 使用 Arc 共享大型数据
pub struct OptimizedFbNode {
    // 小数据直接存储
    project_code: String,
    quantity: f64,

    // 大数据使用 Arc 共享
    price_details: Arc<PriceDetails>,
    quantity_expression: Arc<String>,  // 可能很长的表达式
}

// 定期清理缓存
pub fn gc_node_pool(pool: &mut NodePool) {
    let stats = pool.stats();

    if stats.total_nodes > 100000 || stats.memory_usage > 1_000_000_000 {
        pool.gc();  // 触发垃圾回收
        pool.compact();  // 压缩存储
    }
}
```

## 最佳实践总结

基于 Price-RS 项目的实际经验：

1. **分层设计**：将节点按业务逻辑分层（项目→单项→单位→分部→清单）
2. **属性管理**：使用 `Option<T>` 处理可选字段，使用默认值处理数值字段
3. **验证规则**：在 Schema 层面定义业务规则，确保数据一致性
4. **性能优化**：对大型工程文档使用懒加载和批量操作
5. **内存管理**：使用 Arc 共享大型数据，定期进行垃圾回收

Price-RS 项目证明了 ModuForge-RS 能够处理包含数万个节点、每个节点有 60+ 属性的复杂文档，同时保持高性能和类型安全。

下一章：[状态管理](./state-management.md)
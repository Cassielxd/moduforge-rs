# 文档转换

ModuForge-RS 的转换系统提供了强大且灵活的文档操作能力，通过泛型化的 Step 和 Transform 机制实现复杂的文档变更。本章将结合 price-rs 项目的实际应用展示转换系统的强大功能。

## 核心架构

### 泛型框架设计

ModuForge-RS 的转换层采用泛型设计，支持不同的存储后端：

```rust
// 泛型 Step trait - 支持任意容器和 Schema
pub trait StepGeneric<C, S>: Any + Send + Sync + Debug + 'static
where
    C: DataContainer,
    S: SchemaDefinition<Container = C>,
{
    fn name(&self) -> String;

    fn apply(
        &self,
        inner: &mut C::InnerState,
        schema: Arc<S>,
    ) -> TransformResult<StepResult>;

    fn serialize(&self) -> Option<Vec<u8>>;

    fn invert(
        &self,
        inner: &Arc<C::InnerState>,
    ) -> Option<Arc<dyn StepGeneric<C, S>>>;
}

// 泛型 Transform 结构
pub struct TransformGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub base_doc: Arc<C>,
    lazy_doc: LazyDoc<C, S>,
    draft: Option<C::InnerState>,
    pub steps: VectorSync<Arc<dyn StepGeneric<C, S>>>,
    pub invert_steps: VectorSync<Arc<dyn StepGeneric<C, S>>>,
    pub schema: Arc<S>,
    needs_recompute: bool,
}

// 默认实现 - NodePool + Schema
pub type Transform = TransformGeneric<NodePool, Schema>;
```

### 延迟计算机制

Transform 使用延迟计算优化性能：

```rust
#[derive(Debug, Clone)]
enum LazyDoc<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    // 原始文档，未进行任何修改
    Original(Arc<C>),

    // 需要重新计算的状态，包含基础文档和待应用的步骤
    Pending {
        base: Arc<C>,
        steps: VectorSync<Arc<dyn StepGeneric<C, S>>>,
    },

    // 已计算的最新状态
    Computed(Arc<C>),
}
```

## 核心步骤类型

### 节点操作步骤

基于 `crates/transform/src/node_step.rs` 的实际实现：

```rust
use mf_transform::node_step::{AddNodeStep, RemoveNodeStep, MoveNodeStep};
use mf_model::node_definition::NodeTree;

// 添加节点步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddNodeStep {
    pub parent_id: NodeId,
    pub nodes: Vec<NodeTree>,  // NodeTree 包含节点及其子树
}

// 删除节点步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveNodeStep {
    pub parent_id: NodeId,
    pub node_ids: Vec<NodeId>,
}

// 移动节点步骤
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveNodeStep {
    source_parent_id: NodeId,
    target_parent_id: NodeId,
    node_id: NodeId,
    position: Option<usize>,  // None 表示追加到末尾
}
```

### 属性操作步骤

```rust
use mf_transform::attr_step::AttrStep;
use serde_json::Value;
use std::collections::HashMap;

// 设置节点属性
#[derive(Debug, Clone)]
pub struct AttrStep {
    pub node_id: NodeId,
    pub attrs: HashMap<String, Value>,
    pub old_attrs: Option<HashMap<String, Value>>,  // 用于撤销
}
```

### 标记操作步骤

```rust
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};
use mf_model::mark::Mark;

// 添加标记
#[derive(Debug, Clone)]
pub struct AddMarkStep {
    pub from: usize,
    pub to: usize,
    pub mark: Mark,
}

// 删除标记
#[derive(Debug, Clone)]
pub struct RemoveMarkStep {
    pub from: usize,
    pub to: usize,
    pub mark: Mark,
}
```

## Price-RS 实际应用案例

### 基础命令实现

Price-RS 项目中的实际命令实现，展示了如何使用 Transaction 进行文档操作：

#### 插入命令（InsertCommand）

来自 `extension-base-schema/src/command/insert.rs`：

```rust
use mf_derive::impl_command;
use mf_state::Transaction;
use mf_transform::TransformResult;
use std::collections::HashMap;
use serde_json::Value;

#[impl_command(InsertCommand)]
pub async fn insert_commond(
    tr: &mut Transaction,
    parent_id: &Box<str>,
    name: &Box<str>,
    r#type: &String,
    other: &HashMap<String, Value>,
) -> TransformResult<()> {
    // 验证目标父节点是否存在
    if tr.doc().get_node(parent_id).is_none() {
        return Err(anyhow::anyhow!("目标节点不存在"));
    }

    // 使用 Schema 的工厂方法创建节点树
    let factory = tr.schema.factory();
    let nodes = factory.create_tree(
        r#type,           // 节点类型，如 "DXGC", "fb", "qd"
        None,             // 可选的节点ID
        Some(other),      // 属性映射
        vec![],           // 子节点
        None,             // 可选的标记
    )?;

    // 添加节点到指定父节点
    tr.add_node(parent_id.clone(), vec![nodes])?;
    Ok(())
}
```

#### 更新命令（UpdateCommand）

来自 `extension-base-schema/src/command/update.rs`：

```rust
use mf_derive::impl_command;
use mf_model::rpds::HashTrieMapSync;
use mf_state::Transaction;
use mf_transform::TransformResult;
use serde_json::Value;

#[impl_command(UpdateCommand)]
pub async fn update_commond(
    tr: &mut Transaction,
    point_id: &Box<str>,
    values: &HashTrieMapSync<String, Value>,
) -> TransformResult<()> {
    // 直接更新节点属性
    // Transaction 会自动生成对应的 AttrStep
    tr.set_node_attribute(point_id.clone(), values.clone())?;
    Ok(())
}
```

#### 删除命令（DeleteCommand）

来自 `extension-base-schema/src/command/delete.rs`：

```rust
use mf_derive::impl_command;
use mf_state::Transaction;
use mf_transform::TransformResult;

#[impl_command(DeleteCommand)]
pub async fn del_commond(
    tr: &mut Transaction,
    point_id: &Box<str>,
) -> TransformResult<()> {
    let doc = tr.doc();

    // 验证节点存在
    let point_node = doc.get_node(point_id);
    if point_node.is_none() {
        return Err(anyhow::anyhow!("目标节点不存在，删除失败"));
    }

    // 获取父节点
    let parent_node = doc.get_parent_node(point_id);
    if let Some(node) = parent_node {
        // 从父节点中删除目标节点
        tr.remove_node(node.id.clone(), vec![point_id.clone()])?;
    }
    Ok(())
}
```

### 复杂命令实现

更复杂的命令可以组合多个基础操作：

```rust
use mf_state::{Transaction, transaction::CommandGeneric};
use mf_model::{node_pool::NodePool, schema::Schema};

/// 批量插入分部分项命令
#[derive(Debug)]
pub struct BatchInsertFbfxCommand {
    parent_id: String,
    fb_list: Vec<FbData>,
}

#[derive(Debug)]
struct FbData {
    code: String,
    name: String,
    quantity: f64,
    unit: String,
    rfee_price: f64,
    cfee_price: f64,
    jfee_price: f64,
}

#[async_trait::async_trait]
impl CommandGeneric<NodePool, Schema> for BatchInsertFbfxCommand {
    fn name(&self) -> String {
        "batch_insert_fbfx".to_string()
    }

    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        // 验证父节点存在
        if tr.doc().get_node(&self.parent_id).is_none() {
            return Err(anyhow::anyhow!("父节点不存在"));
        }

        // 批量创建分部节点
        for fb_data in &self.fb_list {
            // 准备节点属性
            let mut attrs = HashMap::new();
            attrs.insert("project_code".to_string(), json!(fb_data.code));
            attrs.insert("project_name".to_string(), json!(fb_data.name));
            attrs.insert("quantity".to_string(), json!(fb_data.quantity.to_string()));
            attrs.insert("unit".to_string(), json!(fb_data.unit));
            attrs.insert("rfee_price".to_string(), json!(fb_data.rfee_price));
            attrs.insert("cfee_price".to_string(), json!(fb_data.cfee_price));
            attrs.insert("jfee_price".to_string(), json!(fb_data.jfee_price));

            // 计算合价
            let rfee_total = fb_data.rfee_price * fb_data.quantity;
            let cfee_total = fb_data.cfee_price * fb_data.quantity;
            let jfee_total = fb_data.jfee_price * fb_data.quantity;

            attrs.insert("rfee_total".to_string(), json!(rfee_total));
            attrs.insert("cfee_total".to_string(), json!(cfee_total));
            attrs.insert("jfee_total".to_string(), json!(jfee_total));

            // 计算管理费和利润
            let manager_fee = rfee_total * 0.15;
            let profit_fee = rfee_total * 0.08;
            let total = rfee_total + cfee_total + jfee_total + manager_fee + profit_fee;

            attrs.insert("total_manager_fee".to_string(), json!(manager_fee));
            attrs.insert("total_profit_fee".to_string(), json!(profit_fee));
            attrs.insert("total".to_string(), json!(total));

            // 使用工厂创建节点
            let factory = tr.schema.factory();
            let node = factory.create_tree(
                "fb",
                None,
                Some(&attrs),
                vec![],
                None,
            )?;

            // 添加到父节点
            tr.add_node(self.parent_id.clone().into(), vec![node])?;
        }

        Ok(())
    }
}
```

### 项目结构调整命令

重新组织项目结构的实际应用：

```rust
/// 按专业重组分部分项
#[derive(Debug)]
pub struct ReorganizeFbfxCommand {
    fbfx_id: String,
}

#[async_trait::async_trait]
impl CommandGeneric<NodePool, Schema> for ReorganizeFbfxCommand {
    fn name(&self) -> String {
        "reorganize_fbfx".to_string()
    }

    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        let doc = tr.doc();
        let tree = doc.inner();

        // 收集所有清单节点
        let mut qd_by_specialty: HashMap<String, Vec<String>> = HashMap::new();

        // 遍历分部分项下的所有节点
        if let Some(children) = tree.children(&self.fbfx_id.clone().into()) {
            for child_id in children {
                if let Some(node) = tree.get(&child_id) {
                    if node.node_type == "qd" {
                        // 从清单编码提取专业代码
                        if let Some(code) = node.attrs.get("project_code")
                            .and_then(|v| v.as_str()) {
                            let specialty = &code[0..2.min(code.len())];
                            qd_by_specialty
                                .entry(specialty.to_string())
                                .or_insert_with(Vec::new)
                                .push(child_id.to_string());
                        }
                    }
                }
            }
        }

        // 为每个专业创建分部容器
        for (specialty_code, qd_ids) in qd_by_specialty {
            let specialty_name = match specialty_code.as_str() {
                "01" => "土建工程",
                "02" => "装饰工程",
                "03" => "安装工程",
                "04" => "市政工程",
                _ => "其他工程",
            };

            // 创建分部节点属性
            let mut attrs = HashMap::new();
            attrs.insert("project_code".to_string(), json!(specialty_code));
            attrs.insert("project_name".to_string(), json!(specialty_name));
            attrs.insert("unit".to_string(), json!("项"));

            // 创建分部节点
            let factory = tr.schema.factory();
            let fb_node = factory.create_tree(
                "fb",
                None,
                Some(&attrs),
                vec![],
                None,
            )?;

            // 添加分部节点
            tr.add_node(self.fbfx_id.clone().into(), vec![fb_node.clone()])?;

            // 移动清单到新的分部下
            let fb_node_id = fb_node.0.id;
            for qd_id in qd_ids {
                // 注意：实际的 move_node 方法需要源父节点ID
                // 这里展示的是概念性代码
                tr.move_node(
                    self.fbfx_id.clone().into(),  // 源父节点
                    fb_node_id.clone(),            // 目标父节点
                    qd_id.into(),                  // 要移动的节点
                    None,                          // 追加到末尾
                )?;
            }
        }

        Ok(())
    }
}
```

## 事务（Transaction）系统

事务系统基于 Transform 提供更高级的操作接口：

```rust
use mf_state::transaction::Transaction;

// 创建事务
let mut tr = Transaction::new(&state);

// 设置元数据
tr.set_meta("author", "张工程师");
tr.set_meta("version", "1.0.0");
tr.set_meta("timestamp", chrono::Utc::now());

// 执行节点操作
tr.add_node(parent_id, vec![node1, node2])?;
tr.remove_node(parent_id, vec![old_node_id])?;
tr.move_node(source_parent, target_parent, node_id, None)?;

// 更新属性
tr.set_attrs(node_id, hashmap!{
    "total" => json!(1000000.0),
    "status" => json!("approved"),
})?;

// 应用事务
let new_state = state.apply(tr).await?;
```

## 批量操作优化

### 批量步骤（BatchStep）

处理大量同类操作时使用批量步骤：

```rust
use mf_transform::batch_step::BatchStep;

// 批量更新多个节点的属性
let batch_updates = vec![
    (node_id1, hashmap!{"status" => json!("completed")}),
    (node_id2, hashmap!{"status" => json!("completed")}),
    (node_id3, hashmap!{"status" => json!("completed")}),
];

let batch_step = BatchStep::new_attr_batch(batch_updates);
transform.step(Arc::new(batch_step))?;
```

### Copy-on-Write 优化

Transform 使用 Copy-on-Write 优化内存使用：

```rust
impl<C, S> TransformGeneric<C, S> {
    /// 获取草稿状态，使用 Copy-on-Write
    fn get_draft(&mut self) -> TransformResult<&mut C::InnerState> {
        if self.draft.is_none() {
            // 只有在第一次修改时才克隆
            self.draft = Some(self.base_doc.inner().clone());
        }
        self.draft.as_mut()
            .ok_or_else(|| anyhow::anyhow!("草稿状态未初始化"))
    }
}
```

## 撤销/重做支持

每个步骤都支持生成反向操作：

```rust
impl StepGeneric<NodePool, Schema> for AddNodeStep {
    fn invert(
        &self,
        _: &Arc<Tree>,
    ) -> Option<Arc<dyn StepGeneric<NodePool, Schema>>> {
        // 生成删除步骤作为反向操作
        let top_level_ids: Vec<NodeId> = self.nodes.iter()
            .map(|node_enum| node_enum.0.id.clone())
            .collect();

        if !top_level_ids.is_empty() {
            return Some(Arc::new(RemoveNodeStep::new(
                self.parent_id.clone(),
                top_level_ids,
            )));
        }
        None
    }
}

// 使用反向步骤实现撤销
let invert_steps = transform.invert_steps.clone();
for step in invert_steps.iter().rev() {
    undo_transform.step(step.clone())?;
}
```

## 性能优化最佳实践

### 1. 使用延迟计算

```rust
// Transform 自动使用延迟计算
// 只有在调用 doc() 时才会真正应用步骤
let mut transform = Transform::new(doc, schema);

// 添加多个步骤（不会立即执行）
transform.step(step1)?;
transform.step(step2)?;
transform.step(step3)?;

// 只有在这里才会计算最终状态
let final_doc = transform.doc();
```

### 2. 批量操作

```rust
// 不好：多次单独操作
for (id, attrs) in updates {
    let step = AttrStep::new(id, attrs);
    transform.step(Arc::new(step))?;
}

// 好：使用批量操作
let batch = BatchStep::new_attr_batch(updates);
transform.step(Arc::new(batch))?;
```

### 3. 复用 Transform 实例

```rust
// 在一个转换会话中复用 Transform
let mut transform = Transform::new(doc, schema);

// 执行多个相关操作
transform.reorganize_nodes()?;
transform.update_attributes()?;
transform.calculate_totals()?;

// 一次性获取结果
let result = transform.doc();
```

## 自定义步骤实现

扩展框架以支持特定业务逻辑：

```rust
use mf_transform::step::{StepGeneric, StepResult};

/// 自定义的价格调整步骤
#[derive(Debug, Clone)]
pub struct PriceAdjustmentStep {
    node_ids: Vec<NodeId>,
    adjustment_rate: f64,  // 调整率，如 1.1 表示上调 10%
}

impl StepGeneric<NodePool, Schema> for PriceAdjustmentStep {
    fn name(&self) -> String {
        "price_adjustment_step".to_string()
    }

    fn apply(
        &self,
        tree: &mut Tree,
        schema: Arc<Schema>,
    ) -> TransformResult<StepResult> {
        for node_id in &self.node_ids {
            if let Some(mut node) = tree.get_mut(node_id) {
                // 调整所有价格相关字段
                for (key, value) in node.attrs.iter_mut() {
                    if key.contains("price") || key.contains("total") {
                        if let Some(num) = value.as_f64() {
                            *value = json!(num * self.adjustment_rate);
                        }
                    }
                }
            }
        }

        Ok(StepResult::ok())
    }

    fn serialize(&self) -> Option<Vec<u8>> {
        serde_json::to_vec(self).ok()
    }

    fn invert(
        &self,
        _: &Arc<Tree>,
    ) -> Option<Arc<dyn StepGeneric<NodePool, Schema>>> {
        // 反向操作：使用倒数调整率
        Some(Arc::new(PriceAdjustmentStep {
            node_ids: self.node_ids.clone(),
            adjustment_rate: 1.0 / self.adjustment_rate,
        }))
    }
}
```

## 总结

ModuForge-RS 的转换系统通过泛型设计和延迟计算提供了：

1. **灵活性**：泛型框架支持不同的存储后端
2. **高性能**：延迟计算和 Copy-on-Write 优化
3. **可扩展**：易于实现自定义步骤
4. **可靠性**：内置撤销/重做支持
5. **实用性**：Price-RS 项目验证了在复杂业务场景下的可用性

转换系统是 ModuForge-RS 的核心组件，为文档编辑提供了强大而高效的操作能力。

下一章：[插件系统](./plugins.md)
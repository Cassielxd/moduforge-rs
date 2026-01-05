# 命令系统

ModuForge-RS 提供了一个强大的命令系统，用于实现文档操作的业务逻辑。本指南展示了 Price-RS 项目中的实际命令实现。

## 核心概念

### CommandGeneric 特征

所有命令都必须实现 `CommandGeneric` 特征：

```rust
pub trait CommandGeneric<C, S>
where
    C: ConfigGeneric,
    S: SchemaGeneric<C>,
{
    type Input: DeserializeOwned + Serialize;
    type Output: DeserializeOwned + Serialize;

    fn command(
        &self,
        tr: &mut Transaction<C, S>,
        input: Self::Input,
    ) -> impl Future<Output = Result<Self::Output, Error>>;
}
```

### #[impl_command] 宏

ModuForge-RS 提供了 `#[impl_command]` 宏来简化命令实现：

```rust
use moduforge_transform::impl_command;

#[impl_command(InsertCommand)]
pub async fn insert_commond(
    tr: &mut Transaction,
    parent_id: &Box<str>,
    name: &Box<str>,
    r#type: &String,
    other: &HashMap<String, Value>,
) -> TransformResult<()> {
    // 命令实现
}
```

宏会自动生成所需的 `CommandGeneric` 实现和输入/输出类型定义。

## Price-RS 命令实现

### InsertCommand - 插入节点命令

插入命令用于向文档树中添加新节点：

```rust
use moduforge_transform::{impl_command, Transaction, TransformResult};
use serde_json::Value;
use std::collections::HashMap;

#[impl_command(InsertCommand)]
pub async fn insert_commond(
    tr: &mut Transaction,
    parent_id: &Box<str>,
    name: &Box<str>,
    r#type: &String,
    other: &HashMap<String, Value>,
) -> TransformResult<()> {
    // 1. 验证父节点是否存在
    if tr.doc().get_node(parent_id).is_none() {
        return Err(anyhow::anyhow!("目标节点不存在"));
    }

    // 2. 使用 schema factory 创建节点树
    let factory = tr.schema.factory();
    let nodes = factory.create_tree(r#type, None, Some(other), vec![], None)?;

    // 3. 将新节点添加到父节点下
    tr.add_node(parent_id.clone(), vec![nodes])?;

    Ok(())
}
```

**使用示例：**

```rust
// 插入一个新的单位工程节点
let input = InsertCommandInput {
    parent_id: "project_001".into(),
    name: "土建工程".into(),
    r#type: "UnitProjectNode".to_string(),
    other: HashMap::from([
        ("code".to_string(), json!("A001")),
        ("unit".to_string(), json!("m²")),
        ("quantity".to_string(), json!(1000.0)),
    ]),
};

let result = insert_command.command(&mut transaction, input).await?;
```

### UpdateCommand - 更新节点命令

更新命令用于修改现有节点的属性：

```rust
use moduforge_transform::{impl_command, Transaction, TransformResult};
use serde_json::Value;
use std::collections::HashMap;

#[impl_command(UpdateCommand)]
pub async fn update_command(
    tr: &mut Transaction,
    target_id: &Box<str>,
    attrs: &HashMap<String, Value>,
) -> TransformResult<()> {
    // 1. 验证目标节点是否存在
    if tr.doc().get_node(target_id).is_none() {
        return Err(anyhow::anyhow!("目标节点不存在"));
    }

    // 2. 更新节点属性
    tr.update_node(target_id.clone(), attrs.clone())?;

    Ok(())
}
```

**使用示例：**

```rust
// 更新单位工程的工程量和单价
let input = UpdateCommandInput {
    target_id: "unit_project_001".into(),
    attrs: HashMap::from([
        ("quantity".to_string(), json!(1200.0)),
        ("unit_price".to_string(), json!(350.50)),
        ("total_price".to_string(), json!(420600.0)),
    ]),
};

let result = update_command.command(&mut transaction, input).await?;
```

### DeleteCommand - 删除节点命令

删除命令用于从文档树中移除节点：

```rust
use moduforge_transform::{impl_command, Transaction, TransformResult};

#[impl_command(DeleteCommand)]
pub async fn delete_command(
    tr: &mut Transaction,
    target_id: &Box<str>,
) -> TransformResult<()> {
    // 1. 验证目标节点是否存在
    let node = tr.doc()
        .get_node(target_id)
        .ok_or_else(|| anyhow::anyhow!("目标节点不存在"))?;

    // 2. 检查业务规则（例如：是否有子节点）
    if node.children().len() > 0 {
        return Err(anyhow::anyhow!("不能删除有子节点的节点"));
    }

    // 3. 删除节点
    tr.delete_node(target_id.clone())?;

    Ok(())
}
```

**使用示例：**

```rust
// 删除一个清单节点
let input = DeleteCommandInput {
    target_id: "qd_node_001".into(),
};

let result = delete_command.command(&mut transaction, input).await?;
```

## 复杂命令示例

### CalculatePriceCommand - 价格计算命令

这是一个展示复杂业务逻辑的命令示例：

```rust
#[impl_command(CalculatePriceCommand)]
pub async fn calculate_price_command(
    tr: &mut Transaction,
    project_id: &Box<str>,
    include_tax: &bool,
    tax_rate: &Option<f64>,
) -> TransformResult<PriceCalculation> {
    // 1. 获取项目节点
    let project_node = tr.doc()
        .get_node(project_id)
        .ok_or_else(|| anyhow::anyhow!("项目节点不存在"))?;

    // 2. 递归计算所有子节点的价格
    let mut total_price = 0.0;
    let mut details = Vec::new();

    for child_id in project_node.children() {
        let child = tr.doc().get_node(child_id).unwrap();

        // 根据节点类型进行不同的计算
        match child.r#type.as_str() {
            "UnitProjectNode" => {
                let quantity = child.attrs.get("quantity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let unit_price = child.attrs.get("unit_price")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let price = quantity * unit_price;
                total_price += price;

                details.push(PriceDetail {
                    node_id: child_id.clone(),
                    node_name: child.attrs.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    price,
                });
            }
            "FbNode" => {
                // 处理分部节点的价格汇总
                let fb_price = calculate_fb_price(tr, child_id)?;
                total_price += fb_price;
            }
            _ => {}
        }
    }

    // 3. 应用税率（如果需要）
    let final_price = if *include_tax {
        let rate = tax_rate.unwrap_or(0.13); // 默认13%税率
        total_price * (1.0 + rate)
    } else {
        total_price
    };

    // 4. 更新项目节点的总价
    tr.update_node(
        project_id.clone(),
        HashMap::from([
            ("total_price".to_string(), json!(final_price)),
            ("price_updated_at".to_string(), json!(chrono::Utc::now())),
        ]),
    )?;

    Ok(PriceCalculation {
        total_price: final_price,
        tax_included: *include_tax,
        tax_amount: if *include_tax {
            final_price - total_price
        } else {
            0.0
        },
        details,
    })
}

// 辅助函数：计算分部价格
fn calculate_fb_price(tr: &Transaction, fb_id: &Box<str>) -> TransformResult<f64> {
    let fb_node = tr.doc().get_node(fb_id).unwrap();
    let mut total = 0.0;

    for child_id in fb_node.children() {
        let child = tr.doc().get_node(child_id).unwrap();
        if let Some(price) = child.attrs.get("total_price").and_then(|v| v.as_f64()) {
            total += price;
        }
    }

    Ok(total)
}
```

## 命令注册与执行

### 在扩展中注册命令

使用 `mf_extension!` 宏注册命令：

```rust
use moduforge_extension::mf_extension;

mf_extension!(BaseSchemaExtension, {
    commands: {
        "insert": InsertCommand,
        "update": UpdateCommand,
        "delete": DeleteCommand,
        "calculate_price": CalculatePriceCommand,
    },
    nodes: {
        // 节点注册...
    }
});
```

### 执行命令

通过 Transaction API 执行命令：

```rust
use moduforge_transform::Transaction;

async fn execute_operations() -> Result<()> {
    let mut transaction = Transaction::new(doc, schema);

    // 执行插入命令
    let insert_result = transaction
        .execute_command("insert", InsertCommandInput {
            parent_id: "root".into(),
            name: "新项目".into(),
            r#type: "ProjectNode".to_string(),
            other: HashMap::new(),
        })
        .await?;

    // 执行更新命令
    let update_result = transaction
        .execute_command("update", UpdateCommandInput {
            target_id: insert_result.node_id,
            attrs: HashMap::from([
                ("status".to_string(), json!("active")),
            ]),
        })
        .await?;

    // 执行计算命令
    let calc_result = transaction
        .execute_command("calculate_price", CalculatePriceCommandInput {
            project_id: insert_result.node_id,
            include_tax: true,
            tax_rate: Some(0.13),
        })
        .await?;

    // 提交事务
    transaction.commit().await?;

    Ok(())
}
```

## 命令测试

### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use moduforge_test::create_test_transaction;

    #[tokio::test]
    async fn test_insert_command() {
        let mut tr = create_test_transaction();

        // 准备测试数据
        let input = InsertCommandInput {
            parent_id: "root".into(),
            name: "测试节点".into(),
            r#type: "UnitProjectNode".to_string(),
            other: HashMap::from([
                ("code".to_string(), json!("TEST001")),
            ]),
        };

        // 执行命令
        let result = InsertCommand.command(&mut tr, input).await;

        // 验证结果
        assert!(result.is_ok());

        // 验证节点已创建
        let doc = tr.doc();
        let root = doc.get_node(&"root".into()).unwrap();
        assert_eq!(root.children().len(), 1);

        let child = doc.get_node(&root.children()[0]).unwrap();
        assert_eq!(child.r#type, "UnitProjectNode");
        assert_eq!(
            child.attrs.get("code").unwrap().as_str().unwrap(),
            "TEST001"
        );
    }

    #[tokio::test]
    async fn test_update_command() {
        let mut tr = create_test_transaction_with_data();

        let input = UpdateCommandInput {
            target_id: "node_001".into(),
            attrs: HashMap::from([
                ("status".to_string(), json!("completed")),
                ("updated_at".to_string(), json!("2024-01-01")),
            ]),
        };

        let result = UpdateCommand.command(&mut tr, input).await;
        assert!(result.is_ok());

        let node = tr.doc().get_node(&"node_001".into()).unwrap();
        assert_eq!(
            node.attrs.get("status").unwrap().as_str().unwrap(),
            "completed"
        );
    }

    #[tokio::test]
    async fn test_delete_command_with_children() {
        let mut tr = create_test_transaction_with_hierarchy();

        // 尝试删除有子节点的节点
        let input = DeleteCommandInput {
            target_id: "parent_node".into(),
        };

        let result = DeleteCommand.command(&mut tr, input).await;

        // 应该失败
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "不能删除有子节点的节点"
        );
    }
}
```

## 最佳实践

### 1. 输入验证

始终验证命令输入的有效性：

```rust
#[impl_command(ValidatedCommand)]
pub async fn validated_command(
    tr: &mut Transaction,
    node_id: &Box<str>,
    value: &f64,
) -> TransformResult<()> {
    // 验证数值范围
    if *value < 0.0 || *value > 1000000.0 {
        return Err(anyhow::anyhow!("数值超出有效范围"));
    }

    // 验证节点存在且类型正确
    let node = tr.doc()
        .get_node(node_id)
        .ok_or_else(|| anyhow::anyhow!("节点不存在"))?;

    if node.r#type != "NumericNode" {
        return Err(anyhow::anyhow!("节点类型不支持数值操作"));
    }

    // 执行操作...
    Ok(())
}
```

### 2. 事务一致性

确保命令操作的原子性：

```rust
#[impl_command(AtomicCommand)]
pub async fn atomic_command(
    tr: &mut Transaction,
    operations: &Vec<Operation>,
) -> TransformResult<()> {
    // 使用检查点以支持回滚
    tr.checkpoint();

    for op in operations {
        match execute_operation(tr, op).await {
            Ok(_) => continue,
            Err(e) => {
                // 回滚到检查点
                tr.rollback();
                return Err(e);
            }
        }
    }

    Ok(())
}
```

### 3. 性能优化

对于批量操作，使用批处理优化：

```rust
#[impl_command(BatchCommand)]
pub async fn batch_command(
    tr: &mut Transaction,
    node_ids: &Vec<Box<str>>,
    operation: &String,
) -> TransformResult<BatchResult> {
    let mut results = Vec::new();

    // 批量获取节点以减少查找开销
    let nodes: Vec<_> = node_ids
        .iter()
        .filter_map(|id| tr.doc().get_node(id))
        .collect();

    // 并行处理（如果操作允许）
    for node in nodes {
        let result = process_node(node, operation)?;
        results.push(result);
    }

    Ok(BatchResult {
        processed: results.len(),
        results,
    })
}
```

## 错误处理

### 自定义错误类型

为命令定义特定的错误类型：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("节点不存在: {0}")]
    NodeNotFound(String),

    #[error("权限不足")]
    PermissionDenied,

    #[error("验证失败: {0}")]
    ValidationError(String),

    #[error("业务规则冲突: {0}")]
    BusinessRuleViolation(String),
}

#[impl_command(SafeCommand)]
pub async fn safe_command(
    tr: &mut Transaction,
    node_id: &Box<str>,
) -> Result<(), CommandError> {
    let node = tr.doc()
        .get_node(node_id)
        .ok_or_else(|| CommandError::NodeNotFound(node_id.to_string()))?;

    // 检查权限
    if !has_permission(&node) {
        return Err(CommandError::PermissionDenied);
    }

    // 业务逻辑...
    Ok(())
}
```

## 总结

ModuForge-RS 的命令系统通过 `#[impl_command]` 宏和 `CommandGeneric` 特征提供了：

- **类型安全**：强类型的输入输出定义
- **异步支持**：原生异步命令执行
- **事务管理**：通过 Transaction API 保证一致性
- **扩展性**：轻松添加新命令到扩展中
- **测试友好**：易于编写单元测试和集成测试

Price-RS 项目展示了如何使用这个系统构建复杂的业务逻辑，从简单的 CRUD 操作到复杂的价格计算算法。
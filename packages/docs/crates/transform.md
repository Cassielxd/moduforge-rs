# moduforge-transform 文档

`moduforge-transform` 提供事务式变更管理,包括 Step 抽象、事务处理、补丁生成和变更历史。

## 概述

Transform 层将所有文档变更抽象为可组合的 Step,确保原子性、可回放和增量同步。

## 核心概念

### Step - 变更步骤

```rust
pub trait Step: Send + Sync {
    fn apply(&self, doc: &Document) -> TransformResult<Document>;
    fn invert(&self, doc: &Document) -> TransformResult<Box<dyn Step>>;
    fn merge(&self, other: &dyn Step) -> Option<Box<dyn Step>>;
}
```

### Transaction - 事务

```rust
pub struct Transaction {
    steps: Vec<Box<dyn Step>>,
    doc_before: Document,
    doc_after: Option<Document>,
}
```

## 内置 Step 类型

### 节点操作

```rust
// 添加节点
AddNodeStep::new_single(node, parent_id)

// 删除节点
RemoveNodeStep::new(node_id)

// 更新节点
UpdateNodeStep::new(node_id, new_node)

// 移动节点
MoveNodeStep::new(node_id, new_parent, position)
```

### 标记操作

```rust
// 添加标记
AddMarkStep::new(from, to, mark)

// 移除标记
RemoveMarkStep::new(from, to, mark_type)
```

### 属性操作

```rust
// 设置属性
SetAttrStep::new(node_id, key, value)

// 删除属性
RemoveAttrStep::new(node_id, key)
```

## 使用示例

```rust
use mf_transform::Transaction;
use mf_transform::node_step::AddNodeStep;

// 创建事务
let mut tr = Transaction::new(doc);

// 添加步骤
tr.add_step(Box::new(AddNodeStep::new_single(node, None)));

// 提交
let new_doc = tr.commit()?;
```

## 下一步

- 查看 [moduforge-model](./model.md)
- 查看 [moduforge-state](./state.md)

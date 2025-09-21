# 插件事务共享设计（保留单一事务接口）

## 设计目标
- 仍通过 `Transaction` 作为唯一的修改入口。
- 在 `Transaction` 内部引入共享 Transform，以减少文档 materialize 次数。
- 保持 `State::apply_transaction` 现有语义：`filter_transaction`、`SeenState`、`apply_inner`、`trs` 都不变。

## 关键结构

### SharedTransform
```rust
#[derive(Clone)]
pub struct SharedTransform {
    inner: Arc<RwLock<Transform>>,
}

impl SharedTransform {
    pub fn new(transform: Transform) -> Self {
        Self { inner: Arc::new(RwLock::new(transform)) }
    }

    pub fn apply_step(&self, step: Arc<dyn Step>) -> TransformResult<()> {
        let mut guard = self.inner.write();
        guard.step(step)
    }

    pub fn doc(&self) -> Arc<NodePool> {
        self.inner.read().doc()
    }

    pub fn commit(&self) -> TransformResult<()> {
        self.inner.write().commit()
    }

    pub fn clone_transform(&self) -> Transform {
        self.inner.read().clone()
    }
}
```

### TransactionBody / Transaction
```rust
enum TransactionBody {
    Owned(Transform),              // 旧行为
    Shared(SharedTransform),       // 指向共享 Transform
}

pub struct Transaction {
    pub meta: imbl::HashMap<String, Arc<dyn Any + Send + Sync>>,
    pub id: u64,
    body: TransactionBody,
}

impl Transaction {
    pub fn new(state: &State) -> Self { /* 保持原实现 */ }

    pub fn new_shared(shared: SharedTransform) -> Self {
        Self {
            meta: imbl::HashMap::new(),
            id: get_transaction_id(),
            body: TransactionBody::Shared(shared),
        }
    }

    pub fn step(&mut self, step: Arc<dyn Step>) -> TransformResult<()> {
        match &mut self.body {
            TransactionBody::Owned(transform) => transform.step(step),
            TransactionBody::Shared(shared) => shared.apply_step(step),
        }
    }

    pub fn commit(&mut self) -> TransformResult<()> {
        match &mut self.body {
            TransactionBody::Owned(transform) => transform.commit(),
            TransactionBody::Shared(_) => Ok(()),
        }
    }

    pub fn doc(&self) -> Arc<NodePool> {
        match &self.body {
            TransactionBody::Owned(transform) => transform.doc(),
            TransactionBody::Shared(shared) => shared.doc(),
        }
    }

    pub fn shared_transform(&self) -> Option<SharedTransform> {
        match &self.body {
            TransactionBody::Shared(shared) => Some(shared.clone()),
            _ => None,
        }
    }
}
```

## State::apply_transaction 改动要点
```rust
impl State {
    pub async fn apply_transaction(
        self: &Arc<Self>,
        root_tr: Arc<Transaction>,
    ) -> StateResult<TransactionResult> {
        if !self.filter_transaction(&root_tr, None).await? {
            return Ok(TransactionResult { state: self.clone(), transactions: vec![root_tr] });
        }

        // 创建共享 Transform
        let shared = root_tr
            .shared_transform()
            .unwrap_or_else(|| SharedTransform::new(root_tr.clone_transform()));

        let mut trs = vec![root_tr.clone()];
        let mut new_state = self.apply_inner(&root_tr).await?;
        let sorted = self.sorted_plugins().await;
        let mut seen: Option<Vec<SeenState>> = None;

        for round in 0..MAX_ROUNDS {
            let mut have_new = false;

            for (i, plugin) in sorted.iter().enumerate() {
                let seen_entry = seen.as_ref().map(|v| v[i].clone());
                let old_state = seen_entry.as_ref().map(|s| s.state.clone()).unwrap_or(self.clone());
                let seen_n = seen_entry.as_ref().map(|s| s.n).unwrap_or(0);
                if seen_n >= trs.len() { continue; }

                if let Some(mut tr) = plugin
                    .spec
                    .apply_append_transaction(&trs[seen_n..], &old_state, &new_state)
                    .await?
                {
                    if let TransactionBody::Owned(transform) = &tr.body {
                        // 升级为共享模式
                        tr.body = TransactionBody::Shared(shared.clone());
                        for step in transform.steps.iter() {
                            shared.apply_step(step.clone())?;
                        }
                    }

                    if new_state.filter_transaction(&tr, Some(i)).await? {
                        tr.set_meta("rootTr", root_tr.clone());
                        new_state = new_state.apply_inner(&tr).await?;
                        trs.push(Arc::new(tr));
                        have_new = true;
                    }
                }

                seen.get_or_insert_with(|| {
                    (0..sorted.len())
                        .map(|_| SeenState { state: self.clone(), n: 0 })
                        .collect()
                })[i] = SeenState { state: new_state.clone(), n: trs.len() };
            }

            if !have_new { break; }
        }

        // 统一提交共享 Transform
        shared.commit().ok();
        Arc::make_mut(&mut new_state.node_pool).replace(shared.doc());

        Ok(TransactionResult { state: new_state, transactions: trs })
    }
}
```

## 插件侧注意事项
- 插件仍然通过 `apply_append_transaction` 返回 `Option<Transaction>`。
- 如果插件自身创建的 `Transaction` 是 `Owned` 类型，框架会在 `apply_transaction` 内部将其升级为共享模式，并把步骤重放到共享 Transform 上。
- 插件无需改动返回值或新增 API；区别只在于 `Transaction` 内部如何存储 Transform。

## 效果
- 插件与运行时之间仅通过 `Transaction` 交互，接口完全保持原状。
- 所有追加步骤都写入同一个共享 Transform，并在最后一次性 `commit`，显著减少 `NodePool` materialize 次数。
- `filter_transaction`、`SeenState`、`apply_inner`、`trs` 等流程保持既有语义。

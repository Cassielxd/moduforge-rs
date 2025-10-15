# ModuForge-RS 插件开发指南

ModuForge-RS 的插件体系围绕三类角色展开：

- **PluginTrait**：描述插件的行为（事务过滤、事务扩展、元数据与配置）。
- **StateField**：为插件维护与 `State` 同步的类型安全状态值。
- **PluginSpec / Plugin**：将行为与状态封装为可被运行时加载的实例。

下文以一个“审计”插件为例，展示从零开始到运行时接入的完整路径。

## 1. 定义插件状态

插件状态需要实现 `mf_state::resource::Resource`，以便被统一存放在状态字段中：

```rust
use mf_state::resource::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditState {
    pub event_count: u64,
    pub last_description: Option<String>,
}

impl Resource for AuditState {}
```

## 2. 实现 StateField

`StateField` 负责状态的初始化与更新。新版接口使用关联类型，避免手动 downcast：

```rust
use async_trait::async_trait;
use mf_state::{
    plugin::StateField,
    state::{State, StateConfig},
    transaction::Transaction,
};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct AuditStateField;

#[async_trait]
impl StateField for AuditStateField {
    type Value = AuditState;

    async fn init(
        &self,
        _config: &StateConfig,
        _instance: &State,
    ) -> Arc<Self::Value> {
        Arc::new(AuditState::default())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<Self::Value>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<Self::Value> {
        let mut next = (*value).clone();
        next.event_count += 1;
        if let Some(desc) = tr.get_meta::<String>("description") {
            next.last_description = Some(desc.clone());
        }
        Arc::new(next)
    }
}
```

序列化 / 反序列化若有需要，可重写 `serialize` / `deserialize` 提供持久化能力。

## 3. 实现 PluginTrait

`PluginTrait` 决定插件在事务管线中的行为，并提供元信息：

```rust
use async_trait::async_trait;
use mf_state::plugin::{PluginConfig, PluginMetadata, PluginTrait};
use mf_state::{error::StateResult, state::State, transaction::Transaction};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct AuditPlugin;

#[async_trait]
impl PluginTrait for AuditPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "audit".to_string(),
            version: "1.0.0".to_string(),
            description: Some("记录事务描述与执行次数".to_string()),
            ..PluginMetadata::default()
        }
    }

    fn config(&self) -> PluginConfig {
        PluginConfig { enabled: true, priority: 0, ..PluginConfig::default() }
    }

    async fn append_transaction(
        &self,
        _committed: &[Arc<Transaction>],
        _old_state: &Arc<State>,
        _new_state: &Arc<State>,
    ) -> StateResult<Option<Transaction>> {
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        tr: &Transaction,
        _state: &State,
    ) -> bool {
        tr.get_meta::<String>("description").is_some()
    }
}
```

在审计示例中，我们拒绝没有描述信息的事务。

## 4. 组装 PluginSpec

将 StateField 与 PluginTrait 组合为可注册的插件：

```rust
use mf_state::plugin::{ErasedStateField, Plugin, PluginSpec};
use std::sync::Arc;

pub fn audit_plugin() -> Arc<Plugin> {
    Arc::new(Plugin::new(PluginSpec {
        state_field: Some(Arc::new(AuditStateField::default()) as Arc<dyn ErasedStateField>),
        tr: Arc::new(AuditPlugin::default()) as Arc<_>,
    }))
}
```

> 提示：如不需要状态，可将 `state_field` 设置为 `None`。

## 5. 在运行时注册插件

```rust
use mf_core::{extension::Extension, types::{Extensions, RuntimeOptions}};

let mut extension = Extension::new();
extension.add_plugin(audit_plugin());

let runtime_options = RuntimeOptions::default().add_extension(Extensions::E(extension));
```

随后将 `runtime_options` 传入 `ForgeRuntimeBuilder` / `ForgeAsyncRuntime::create_with_config` 即可。

若希望使用宏减少样板，可参考 `mf_macro::mf_plugin!`、`mf_macro::mf_extension!`：

```rust
use mf_macro::{mf_extension, mf_plugin};

mf_plugin!(audit_plugin_macro,
    metadata = mf_macro::mf_plugin_metadata!("audit"),
    state_field = AuditStateField,
    filter_transaction = |tr: &Transaction, _state: &State| async move {
        tr.get_meta::<String>("description").is_some()
    },
    docs = "声明式定义的审计插件"
);

let extension = mf_extension!(docs = "注册审计插件" => audit_plugin_macro::new());
```

宏会自动完成 `PluginSpec`、`Plugin` 封装与类型擦除。

## 6. 事务元数据与资源表

- 使用 `Transaction::set_meta` / `get_meta` 向插件传递轻量信息。
- 长周期依赖可通过 `State::resources()` 获取 `GlobalResourceManager`，在插件中读取共享数据。

```rust
use mf_state::ops::GlobalResourceManager;

fn fetch_user(manager: &GlobalResourceManager, user_id: &str) -> Option<String> {
    manager.get::<String>("current_user").cloned().filter(|id| id == user_id)
}
```

## 7. 测试插件

```rust
use mf_state::{State, StateConfig};

#[tokio::test]
async fn audit_plugin_tracks_events() {
    let plugin = audit_plugin();
    let config = StateConfig { plugins: Some(vec![plugin]), ..StateConfig::default() };
    let state = State::create(config).await.unwrap();

    let mut tr = state.tr();
    tr.set_meta("description", "create document");
    state.apply(tr).await.unwrap();

    let audit_state = state.get_field("audit").unwrap();
    let snapshot = audit_state.downcast_arc::<AuditState>().unwrap();
    assert_eq!(snapshot.event_count, 1);
}
```
## 8. 最佳实践

- **最小化副作用**：`append_transaction` 返回的新事务务必调用 `commit()`，避免未提交的 Step 残留。
- **合理设置优先级**：`PluginConfig::priority` 越小越先执行，可用于构建过滤链。
- **记录日志**：结合 `tracing` 输出关键信息，定位插件行为。
- **关注性能**：耗时任务放在后台任务或持久化管线，插件函数应保持轻量。
- **结合宏**：对外暴露插件时推荐提供 `mf_plugin!` 版本，方便调用方直接注册。

通过以上步骤，便可以以类型安全、可测试的方式构建适配 ModuForge-RS 运行时的插件体系。

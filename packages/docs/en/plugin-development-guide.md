# ModuForge-RS Plugin Development Guide

The ModuForge-RS plugin system is built around three concepts:

- **PluginTrait** – describes plugin behaviour (transaction filtering, transaction augmentation, metadata, configuration).
- **StateField** – maintains typed plugin state that stays in sync with `State`.
- **PluginSpec / Plugin** – packages behaviour and state into a runtime-loadable unit.

The following sections walk through a complete “audit” plugin from scratch to runtime registration.

## 1. Define plugin state

Plugin state must implement `mf_state::resource::Resource` so it can be stored inside the state field registry:

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

## 2. Implement StateField

`StateField` initialises and updates the plugin state. The current API uses an associated type so no manual downcasting is required:

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

Override `serialize` / `deserialize` if you need persistent plugin state snapshots.

## 3. Implement PluginTrait

`PluginTrait` controls how the plugin participates in the transaction pipeline and supplies metadata:

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
            description: Some("Record transaction descriptions and execution count".to_string()),
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

Here the plugin refuses to run when no human-readable description is provided.

## 4. Build a PluginSpec

Combine the `StateField` and `PluginTrait` into a registerable plugin:

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

> Tip: set `state_field` to `None` for stateless plugins.

## 5. Register the plugin with a runtime

```rust
use mf_core::{extension::Extension, types::{Extensions, RuntimeOptions}};

let mut extension = Extension::new();
extension.add_plugin(audit_plugin());

let runtime_options = RuntimeOptions::default().add_extension(Extensions::E(extension));
```

Pass `runtime_options` to `ForgeRuntimeBuilder` or `ForgeAsyncRuntime::create_with_config` to enable the plugin.

Prefer less boilerplate? Leverage `mf_macro::mf_plugin!` and `mf_macro::mf_extension!`:

```rust
use mf_macro::{mf_extension, mf_plugin};

mf_plugin!(audit_plugin_macro,
    metadata = mf_macro::mf_plugin_metadata!("audit"),
    state_field = AuditStateField,
    filter_transaction = |tr: &Transaction, _state: &State| async move {
        tr.get_meta::<String>("description").is_some()
    },
    docs = "Declarative audit plugin"
);

let extension = mf_extension!(docs = "Register audit plugin" => audit_plugin_macro::new());
```

The macros emit the same `PluginSpec`/`Plugin` structure and handle type erasure for you.

## 6. Metadata and resource access

- Use `Transaction::set_meta` / `get_meta` to pass lightweight data into the plugin.
- For shared services, reach into the `GlobalResourceManager` obtained from `State::resources()`.

```rust
use mf_state::ops::GlobalResourceManager;

fn lookup_user(manager: &GlobalResourceManager, user_id: &str) -> Option<String> {
    manager.get::<String>("current_user").cloned().filter(|id| id == user_id)
}
```

## 7. Testing

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
## 8. Best practices

- **Keep side effects minimal** – always call `commit()` on transactions returned from `append_transaction`.
- **Prioritise execution order** – lower `PluginConfig::priority` values run first; chain filters accordingly.
- **Log important decisions** – integrate with `tracing` to make plugin behaviour observable.
- **Watch performance** – keep plugin callbacks lean; offload heavy work to background tasks.
- **Offer macro wrappers** – if you publish plugins, expose an `mf_plugin!` convenience layer for downstream users.

Following these steps yields type-safe, testable plugins that integrate cleanly with the ModuForge-RS runtime.

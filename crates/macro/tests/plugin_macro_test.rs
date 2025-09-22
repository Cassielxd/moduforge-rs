use mf_macro::{
    mf_plugin, mf_plugin_metadata, mf_plugin_config, mf_plugin_with_config,
};
use mf_state::{
    Transaction, State, StateConfig, error::StateResult, resource::Resource,
};
use mf_state::plugin::{
    PluginMetadata, PluginConfig, PluginTrait, StateField, PluginSpec,
};
use std::sync::Arc;
use async_trait::async_trait;

// 测试事务处理函数
async fn test_append_transaction(
    _trs: &[Arc<Transaction>],
    _old_state: &Arc<State>,
    _new_state: &Arc<State>,
) -> StateResult<Option<Transaction>> {
    println!("处理事务");
    Ok(None)
}

async fn test_filter_transaction(
    _tr: &Transaction,
    _state: &State,
) -> bool {
    true
}

// 测试状态字段
#[derive(Debug, Clone)]
struct TestPluginState {
    counter: u32,
}

impl Resource for TestPluginState {}

// 简化测试，先删除StateField的复杂实现
#[derive(Debug)]
struct TestStateField;

#[async_trait]
impl StateField for TestStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: &State,
    ) -> Arc<dyn Resource> {
        Arc::new(TestPluginState { counter: 0 })
    }

    async fn apply(
        &self,
        _tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        let state = value.downcast_arc::<TestPluginState>().unwrap().clone();
        state
    }
}

// 测试基础插件宏
mf_plugin!(basic_plugin, docs = "基础测试插件");

// 测试带元数据的插件
mf_plugin!(
    metadata_plugin,
    metadata = mf_plugin_metadata!(
        "metadata_plugin",
        version = "1.0.0",
        description = "带元数据的测试插件",
        author = "ModuForge Team",
        tags = ["test", "metadata"]
    ),
    docs = "带完整元数据的测试插件"
);

// 测试带配置的插件
mf_plugin!(
    config_plugin,
    config = mf_plugin_config!(
        enabled = true,
        priority = 10,
        settings = { "debug" => true, "timeout" => 5000 }
    ),
    docs = "带配置的测试插件"
);

// 测试完整功能插件
mf_plugin!(
    full_plugin,
    metadata = mf_plugin_metadata!(
        "full_plugin",
        version = "2.0.0",
        description = "完整功能测试插件",
        author = "ModuForge Team",
        dependencies = ["dep1", "dep2"],
        conflicts = ["conflict1"],
        state_fields = ["counter"],
        tags = ["test", "full"]
    ),
    config = mf_plugin_config!(
        enabled = true,
        priority = 20,
        settings = { "mode" => "test", "level" => 3 }
    ),
    append_transaction = test_append_transaction,
    filter_transaction = test_filter_transaction,
    state_field = TestStateField,
    docs = "具有完整功能的测试插件"
);

// 测试可配置插件
mf_plugin_with_config!(
    configurable_plugin,
    config = {
        name: String,
        enabled: bool,
        priority: i32
    },
    init_fn = |name: String, enabled: bool, priority: i32| {
        let metadata = mf_plugin_metadata!(
            &name,
            version = "1.0.0",
            description = "动态配置插件",
            author = "ModuForge Team"
        );
        let config = mf_plugin_config!(
            enabled = enabled,
            priority = priority
        );

        #[derive(Debug)]
        struct DynamicPlugin {
            metadata: PluginMetadata,
            config: PluginConfig,
        }

        #[async_trait]
        impl PluginTrait for DynamicPlugin {
            fn metadata(&self) -> PluginMetadata {
                self.metadata.clone()
            }

            fn config(&self) -> PluginConfig {
                self.config.clone()
            }

            async fn append_transaction(
                &self,
                _trs: &[Arc<Transaction>],
                _old_state: &Arc<State>,
                _new_state: &Arc<State>,
            ) -> StateResult<Option<Transaction>> {
                Ok(None)
            }
        }

        PluginSpec {
            state_field: None,
            tr: Arc::new(DynamicPlugin { metadata, config }),
        }
    },
    docs = "可动态配置的插件"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_plugin_creation() {
        let plugin = basic_plugin::new();
        assert_eq!(plugin.get_name(), "basic_plugin");

        let metadata = plugin.get_metadata();
        assert_eq!(metadata.name, "basic_plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "Auto-generated plugin");
    }

    #[test]
    fn test_metadata_plugin() {
        let plugin = metadata_plugin::new();
        let metadata = plugin.get_metadata();

        assert_eq!(metadata.name, "metadata_plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "带元数据的测试插件");
        assert_eq!(metadata.author, "ModuForge Team");
        assert_eq!(metadata.tags, vec!["test", "metadata"]);
    }

    #[test]
    fn test_config_plugin() {
        let plugin = config_plugin::new();
        let config = plugin.get_config();

        assert_eq!(config.enabled, true);
        assert_eq!(config.priority, 10);
        assert_eq!(
            config.settings.get("debug").unwrap(),
            &serde_json::json!(true)
        );
        assert_eq!(
            config.settings.get("timeout").unwrap(),
            &serde_json::json!(5000)
        );
    }

    #[test]
    fn test_full_plugin() {
        let plugin = full_plugin::new();
        let metadata = plugin.get_metadata();
        let config = plugin.get_config();

        // 验证元数据
        assert_eq!(metadata.name, "full_plugin");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.description, "完整功能测试插件");
        assert_eq!(metadata.dependencies, vec!["dep1", "dep2"]);
        assert_eq!(metadata.conflicts, vec!["conflict1"]);
        assert_eq!(metadata.state_fields, vec!["counter"]);
        assert_eq!(metadata.tags, vec!["test", "full"]);

        // 验证配置
        assert_eq!(config.enabled, true);
        assert_eq!(config.priority, 20);
        assert_eq!(
            config.settings.get("mode").unwrap(),
            &serde_json::json!("test")
        );
        assert_eq!(
            config.settings.get("level").unwrap(),
            &serde_json::json!(3)
        );
    }

    #[test]
    fn test_configurable_plugin() {
        let plugin =
            configurable_plugin::new("dynamic_plugin".to_string(), true, 15);

        let metadata = plugin.get_metadata();
        let config = plugin.get_config();

        assert_eq!(metadata.name, "dynamic_plugin");
        assert_eq!(metadata.description, "动态配置插件");
        assert_eq!(config.enabled, true);
        assert_eq!(config.priority, 15);
    }

    #[test]
    fn test_plugin_functionality() {
        let plugin = full_plugin::new();
        // 基础功能验证
        assert_eq!(plugin.get_name(), "full_plugin");

        let metadata = plugin.get_metadata();
        assert_eq!(metadata.name, "full_plugin");

        let config = plugin.get_config();
        assert!(config.enabled);
    }

    #[test]
    fn test_plugin_spec_creation() {
        let spec = basic_plugin::spec();
        assert!(spec.state_field.is_none());

        let spec = full_plugin::spec();
        assert!(spec.state_field.is_some());
    }
}

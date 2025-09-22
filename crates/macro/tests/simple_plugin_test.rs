use mf_macro::{mf_plugin, mf_plugin_metadata, mf_plugin_config};

// 测试基础插件宏
mf_plugin!(test_basic_plugin, docs = "基础测试插件");

// 测试带元数据的插件
mf_plugin!(
    test_metadata_plugin,
    metadata = mf_plugin_metadata!(
        "test_metadata_plugin",
        version = "1.0.0",
        description = "测试元数据插件",
        author = "ModuForge Team"
    ),
    docs = "带元数据的测试插件"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_plugin_creation() {
        let plugin = test_basic_plugin::new();
        assert_eq!(plugin.get_name(), "test_basic_plugin");

        let metadata = plugin.get_metadata();
        assert_eq!(metadata.name, "test_basic_plugin");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_metadata_plugin_creation() {
        let plugin = test_metadata_plugin::new();
        assert_eq!(plugin.get_name(), "test_metadata_plugin");

        let metadata = plugin.get_metadata();
        assert_eq!(metadata.name, "test_metadata_plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.description, "测试元数据插件");
        assert_eq!(metadata.author, "ModuForge Team");
    }

    #[test]
    fn test_plugin_spec_creation() {
        let spec = test_basic_plugin::spec();
        assert!(spec.state_field.is_none());

        // 验证 trait 对象
        let metadata = spec.tr.metadata();
        assert_eq!(metadata.name, "test_basic_plugin");
    }
}

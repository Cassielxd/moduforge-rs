//! 完整的XML Schema集成示例
//!
//! 展示如何在实际项目中使用XML schema定义，包括：
//! 1. 配置文件中的XML schema路径
//! 2. Runtime的创建和使用
//! 3. 动态扩展管理
//! 4. 混合使用代码和XML定义的扩展

use mf_core::{
    ForgeRuntime, ForgeAsyncRuntime, ExtensionManager, ExtensionManagerBuilder,
    config::{ForgeConfig, ExtensionConfig},
    types::{Extensions, RuntimeOptions},
    node::Node,
    mark::Mark,
    ForgeResult,
};
use mf_model::{node_type::NodeSpec, mark_type::MarkSpec};
use std::time::Duration;

#[tokio::main]
async fn main() -> ForgeResult<()> {
    println!("=== 完整的XML Schema集成示例 ===\n");

    // 1. 使用配置文件中的XML schema路径
    config_based_schema_example().await?;

    // 2. 直接从XML文件创建Runtime
    direct_xml_runtime_example().await?;

    // 3. 混合使用代码和XML定义的扩展
    mixed_extensions_example().await?;

    // 4. 动态扩展管理示例
    dynamic_extension_example().await?;

    println!("\n=== 所有集成示例执行完成 ===");
    Ok(())
}

/// 1. 使用配置文件中的XML schema路径
async fn config_based_schema_example() -> ForgeResult<()> {
    println!("1. 配置文件中的XML Schema示例:");

    // 创建包含XML schema路径的配置
    let mut config = ForgeConfig::default();
    config.extension = ExtensionConfig {
        load_timeout: Duration::from_secs(10),
        enable_hot_reload: true,
        max_memory_mb: 200,
        enable_sandbox: false,
        xml_schema_paths: vec![
            // 注意：这些路径在实际使用中需要存在
            // "./test-data/xml-schemas/basic-document.xml".to_string(),
            // "./test-data/xml-schemas/table-schema.xml".to_string(),
        ],
        enable_xml_auto_reload: true,
        xml_parse_timeout: Duration::from_secs(5),
    };

    println!("   ✅ 配置创建成功");
    println!(
        "   - XML schema路径数量: {}",
        config.extension.xml_schema_paths.len()
    );
    println!("   - 自动重载: {}", config.extension.enable_xml_auto_reload);
    println!("   - 解析超时: {:?}", config.extension.xml_parse_timeout);

    // 在实际应用中，你可以从这些路径加载schema
    // let extension_manager = ExtensionManager::from_xml_files(&config.extension.xml_schema_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
    // let options = RuntimeOptions::from_extension_manager(extension_manager);
    // let runtime = ForgeRuntime::create_with_config(options, config).await?;

    Ok(())
}

/// 2. 直接从XML文件创建Runtime
async fn direct_xml_runtime_example() -> ForgeResult<()> {
    println!("\n2. 直接从XML创建Runtime示例:");

    // 创建临时XML内容用于演示
    let xml_content = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema top_node="document">
      <nodes>
        <node name="document" group="block">
          <desc>文档根节点</desc>
          <content>section+</content>
          <attrs>
            <attr name="title" default="New Document"/>
            <attr name="language" default="zh-CN"/>
          </attrs>
        </node>
        <node name="section" group="block">
          <desc>章节节点</desc>
          <content>paragraph+</content>
          <attrs>
            <attr name="level" default="1"/>
          </attrs>
        </node>
        <node name="paragraph" group="block">
          <desc>段落节点</desc>
          <content>text*</content>
          <marks>strong</marks>
        </node>
        <node name="text">
          <desc>文本节点</desc>
        </node>
      </nodes>
      <marks>
        <mark name="em" group="formatting">
          <desc>强调标记</desc>
          <spanning>true</spanning>
        </mark>
        <mark name="strong" group="formatting">
          <desc>粗体标记</desc>
          <spanning>true</spanning>
        </mark>
      </marks>
    </schema>
    "#;

    // 从XML内容创建同步Runtime
    let sync_runtime =
        ForgeRuntime::from_xml_content(xml_content, None, None).await?;
    println!("   ✅ 同步Runtime创建成功");

    let schema = sync_runtime.get_schema();
    println!("   - 节点类型数量: {}", schema.nodes.len());
    println!("   - 标记类型数量: {}", schema.marks.len());
    println!(
        "   - 顶级节点: {:?}",
        schema.top_node_type.as_ref().map(|n| &n.name)
    );

    // 从XML内容创建异步Runtime
    let _async_runtime =
        ForgeAsyncRuntime::from_xml_content(xml_content, None, None).await?;
    println!("   ✅ 异步Runtime创建成功");

    Ok(())
}

/// 3. 混合使用代码和XML定义的扩展
async fn mixed_extensions_example() -> ForgeResult<()> {
    println!("\n3. 混合扩展示例:");

    // 代码定义的扩展
    let mut code_node_spec = NodeSpec::default();
    code_node_spec.desc = Some("代码定义的自定义节点".to_string());
    code_node_spec.group = Some("custom".to_string());
    let code_node = Node::create("code_custom_node", code_node_spec);

    let mut code_mark_spec = MarkSpec::default();
    code_mark_spec.desc = Some("代码定义的自定义标记".to_string());
    code_mark_spec.spanning = Some(true);
    let code_mark = Mark::new("code_custom_mark", code_mark_spec);

    // XML定义的扩展
    let xml_content = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema>
      <nodes>
        <node name="xml_custom_node" group="custom">
          <desc>XML定义的自定义节点</desc>
          <content>text*</content>
        </node>
      </nodes>
      <marks>
        <mark name="xml_custom_mark" group="custom">
          <desc>XML定义的自定义标记</desc>
          <spanning>false</spanning>
        </mark>
      </marks>
    </schema>
    "#;

    // 使用Builder模式混合不同来源的扩展
    let extension_manager = ExtensionManagerBuilder::new()
        .add_extension(Extensions::N(code_node))
        .add_extension(Extensions::M(code_mark))
        .add_xml_content(xml_content)
        .build()?;

    println!("   ✅ 混合ExtensionManager创建成功");

    let schema = extension_manager.get_schema();
    println!("   - 总节点数量: {}", schema.nodes.len());
    println!("   - 总标记数量: {}", schema.marks.len());

    // 验证混合后的扩展
    assert!(schema.nodes.contains_key("code_custom_node"));
    assert!(schema.nodes.contains_key("xml_custom_node"));
    assert!(schema.marks.contains_key("code_custom_mark"));
    assert!(schema.marks.contains_key("xml_custom_mark"));

    println!("   ✅ 扩展验证通过");

    // 从ExtensionManager创建Runtime
    let options = RuntimeOptions::from_extension_manager(extension_manager);
    let _runtime = ForgeRuntime::create(options).await?;
    println!("   ✅ Runtime创建成功");

    Ok(())
}

/// 4. 动态扩展管理示例
async fn dynamic_extension_example() -> ForgeResult<()> {
    println!("\n4. 动态扩展管理示例:");

    // 创建初始的ExtensionManager
    let initial_xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema>
      <nodes>
        <node name="base_node">
          <desc>基础节点</desc>
        </node>
      </nodes>
    </schema>
    "#;

    let mut extension_manager = ExtensionManager::from_xml_string(initial_xml)?;
    println!("   ✅ 初始ExtensionManager创建成功");
    println!(
        "   - 初始节点数量: {}",
        extension_manager.get_schema().nodes.len()
    );

    // 动态添加代码定义的扩展
    let dynamic_node = Node::create("dynamic_node", NodeSpec::default());
    extension_manager.add_extensions(vec![Extensions::N(dynamic_node)])?;
    println!("   ✅ 动态添加代码扩展成功");
    println!(
        "   - 当前节点数量: {}",
        extension_manager.get_schema().nodes.len()
    );

    // 动态添加XML定义的扩展
    let additional_xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema>
      <nodes>
        <node name="dynamic_xml_node">
          <desc>动态添加的XML节点</desc>
        </node>
      </nodes>
      <marks>
        <mark name="dynamic_mark">
          <desc>动态添加的标记</desc>
        </mark>
      </marks>
    </schema>
    "#;

    extension_manager.add_xml_content(additional_xml)?;
    println!("   ✅ 动态添加XML扩展成功");
    println!(
        "   - 最终节点数量: {}",
        extension_manager.get_schema().nodes.len()
    );
    println!(
        "   - 最终标记数量: {}",
        extension_manager.get_schema().marks.len()
    );

    // 验证所有扩展都存在
    let schema = extension_manager.get_schema();
    assert!(schema.nodes.contains_key("base_node"));
    assert!(schema.nodes.contains_key("dynamic_node"));
    assert!(schema.nodes.contains_key("dynamic_xml_node"));
    assert!(schema.marks.contains_key("dynamic_mark"));

    println!("   ✅ 动态扩展验证通过");

    Ok(())
}

/// 实际项目中的最佳实践示例
#[allow(dead_code)]
async fn best_practices_example() -> ForgeResult<()> {
    println!("\n5. 最佳实践示例:");

    // 1. 使用配置文件管理schema路径
    let config = ForgeConfig {
        extension: ExtensionConfig {
            xml_schema_paths: vec![
                "./schemas/base.xml".to_string(),
                "./schemas/formatting.xml".to_string(),
                "./schemas/custom.xml".to_string(),
            ],
            enable_xml_auto_reload: true,
            xml_parse_timeout: Duration::from_secs(10),
            ..Default::default()
        },
        ..Default::default()
    };

    // 2. 分层的schema设计
    // - base.xml: 基础节点和标记
    // - formatting.xml: 格式化相关的标记
    // - custom.xml: 项目特定的自定义扩展

    // 3. 错误处理和回退机制
    let extension_manager = match ExtensionManager::from_xml_files(
        &config
            .extension
            .xml_schema_paths
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>(),
    ) {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("XML schema加载失败，使用默认配置: {}", e);
            // 回退到代码定义的基础扩展
            ExtensionManager::new(&vec![])?
        },
    };

    // 4. 性能监控
    let start_time = std::time::Instant::now();
    let options = RuntimeOptions::from_extension_manager(extension_manager);
    let _runtime = ForgeRuntime::create_with_config(options, config).await?;
    let duration = start_time.elapsed();

    println!("   ✅ Runtime创建耗时: {:?}", duration);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_xml_schema_integration() {
        assert!(config_based_schema_example().await.is_ok());
        assert!(direct_xml_runtime_example().await.is_ok());
        assert!(mixed_extensions_example().await.is_ok());
        assert!(dynamic_extension_example().await.is_ok());
    }
}

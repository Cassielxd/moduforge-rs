//! XML Schema 与 Runtime 集成示例
//!
//! 本示例展示如何将XML定义的Schema集成到ModuForge Runtime中使用。

use mf_core::{XmlSchemaParser, XmlSchemaResult, EditorOptionsBuilder};
use mf_model::schema::Schema;

fn main() -> XmlSchemaResult<()> {
    println!("=== XML Schema 与 Runtime 集成示例 ===\n");

    // 1. 从XML创建Schema并集成到Runtime
    schema_runtime_integration()?;

    // 2. 使用Extensions创建Runtime
    extensions_runtime_integration()?;

    println!("\n=== 集成示例执行完成 ===");
    Ok(())
}

/// Schema与Runtime集成示例
fn schema_runtime_integration() -> XmlSchemaResult<()> {
    println!("1. Schema与Runtime集成:");

    // 定义一个完整的文档Schema
    let xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema top_node="doc">
      <nodes>
        <node name="doc" group="block">
          <desc>文档根节点</desc>
          <content>paragraph+</content>
          <marks>_</marks>
          <attrs>
            <attr name="title" default="New Document"/>
            <attr name="author"/>
          </attrs>
        </node>
        <node name="paragraph" group="block">
          <desc>段落节点</desc>
          <content>text*</content>
          <marks>strong em link</marks>
          <attrs>
            <attr name="align" default="left"/>
            <attr name="indent" default="0"/>
          </attrs>
        </node>
        <node name="heading" group="block">
          <desc>标题节点</desc>
          <content>text*</content>
          <marks>strong em</marks>
          <attrs>
            <attr name="level" default="1"/>
          </attrs>
        </node>
        <node name="text">
          <desc>文本节点</desc>
        </node>
      </nodes>
      <marks>
        <mark name="strong" group="formatting">
          <desc>粗体标记</desc>
          <spanning>true</spanning>
        </mark>
        <mark name="em" group="formatting">
          <desc>斜体标记</desc>
          <spanning>true</spanning>
        </mark>
        <mark name="link" group="link">
          <desc>链接标记</desc>
          <spanning>false</spanning>
          <attrs>
            <attr name="href"/>
            <attr name="title"/>
          </attrs>
        </mark>
      </marks>
    </schema>
    "#;

    // 解析XML为SchemaSpec
    let schema_spec = XmlSchemaParser::parse_from_str(xml)?;
    println!("   ✅ XML Schema解析成功");

    // 编译为Schema
    let schema = Schema::compile(schema_spec).map_err(|e| {
        mf_core::XmlSchemaError::InvalidNodeDefinition(format!(
            "Schema编译失败: {e}"
        ))
    })?;
    println!("   ✅ Schema编译成功");

    // 验证Schema结构
    println!("   - 节点类型数量: {}", schema.nodes.len());
    println!("   - 标记类型数量: {}", schema.marks.len());
    println!(
        "   - 顶级节点: {:?}",
        schema.top_node_type.as_ref().map(|n| &n.name)
    );

    // 显示节点详细信息
    for (name, node_type) in &schema.nodes {
        println!(
            "   - 节点 '{}': 组={:?}, 内容={:?}",
            name, node_type.spec.group, node_type.spec.content
        );
    }

    // 显示标记详细信息
    for (name, mark_type) in &schema.marks {
        println!(
            "   - 标记 '{}': 组={:?}, spanning={:?}",
            name, mark_type.spec.group, mark_type.spec.spanning
        );
    }

    Ok(())
}

/// Extensions与Runtime集成示例
fn extensions_runtime_integration() -> XmlSchemaResult<()> {
    println!("\n2. Extensions与Runtime集成:");

    let xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema top_node="doc">
      <nodes>
        <node name="doc" group="block">
          <desc>文档根节点</desc>
          <content>paragraph+</content>
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
        <mark name="strong" group="formatting">
          <desc>粗体标记</desc>
          <spanning>true</spanning>
        </mark>
      </marks>
    </schema>
    "#;

    // 解析为Extensions
    let extensions = XmlSchemaParser::parse_to_extensions(xml)?;
    println!("   ✅ Extensions解析成功，数量: {}", extensions.len());

    // 创建RuntimeOptions并添加Extensions
    let options = EditorOptionsBuilder::new()
        .extensions(extensions)
        .history_limit(100)
        .build();

    println!("   ✅ RuntimeOptions创建成功");
    println!("   - Extensions数量: {}", options.get_extensions().len());
    println!("   - 历史记录限制: {:?}", options.get_history_limit());

    // 分析Extensions内容
    let extensions = options.get_extensions();
    let mut node_names = Vec::new();
    let mut mark_names = Vec::new();

    for ext in &extensions {
        match ext {
            mf_core::types::Extensions::N(node) => {
                node_names.push(node.get_name().to_string());
            },
            mf_core::types::Extensions::M(mark) => {
                mark_names.push(mark.get_name().to_string());
            },
            mf_core::types::Extensions::E(_) => {
                // Extension类型
            },
        }
    }

    println!("   - 节点类型: {node_names:?}");
    println!("   - 标记类型: {mark_names:?}");

    Ok(())
}

/// 演示如何从多个XML文件组合Schema
#[allow(dead_code)]
fn multi_file_schema_composition() -> XmlSchemaResult<()> {
    println!("\n3. 多文件Schema组合:");

    // 这个示例展示如何从多个XML文件组合Schema
    // 实际使用中，你可能有基础Schema和扩展Schema

    let base_schema_xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema top_node="doc">
      <nodes>
        <node name="doc">
          <content>paragraph+</content>
        </node>
        <node name="paragraph">
          <content>text*</content>
        </node>
        <node name="text">
          <desc>文本节点</desc>
        </node>
      </nodes>
    </schema>
    "#;

    let extension_schema_xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema>
      <nodes>
        <node name="heading" group="block">
          <desc>标题节点</desc>
          <content>text*</content>
          <attrs>
            <attr name="level" default="1"/>
          </attrs>
        </node>
      </nodes>
      <marks>
        <mark name="strong">
          <desc>粗体标记</desc>
          <spanning>true</spanning>
        </mark>
      </marks>
    </schema>
    "#;

    // 解析基础Schema
    let mut base_spec = XmlSchemaParser::parse_from_str(base_schema_xml)?;
    println!("   ✅ 基础Schema解析成功");

    // 解析扩展Schema
    let ext_spec = XmlSchemaParser::parse_from_str(extension_schema_xml)?;
    println!("   ✅ 扩展Schema解析成功");

    // 合并Schema（简单的合并策略）
    for (name, node_spec) in ext_spec.nodes {
        base_spec.nodes.insert(name, node_spec);
    }
    for (name, mark_spec) in ext_spec.marks {
        base_spec.marks.insert(name, mark_spec);
    }

    println!("   ✅ Schema合并完成");
    println!("   - 最终节点数量: {}", base_spec.nodes.len());
    println!("   - 最终标记数量: {}", base_spec.marks.len());

    // 编译合并后的Schema
    let _schema = Schema::compile(base_spec).map_err(|e| {
        mf_core::XmlSchemaError::InvalidNodeDefinition(format!(
            "合并Schema编译失败: {e}"
        ))
    })?;
    println!("   ✅ 合并Schema编译成功");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_runtime_integration() {
        assert!(schema_runtime_integration().is_ok());
    }

    #[test]
    fn test_extensions_runtime_integration() {
        assert!(extensions_runtime_integration().is_ok());
    }

    #[test]
    fn test_multi_file_composition() {
        assert!(multi_file_schema_composition().is_ok());
    }
}

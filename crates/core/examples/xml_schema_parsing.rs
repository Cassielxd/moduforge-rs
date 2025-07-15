//! XML Schema 解析示例
//! 
//! 本示例展示如何使用XML格式定义Schema，并将其解析为ModuForge可用的结构。

use mf_core::{XmlSchemaParser, XmlSchemaResult};
use mf_model::schema::Schema;

fn main() -> XmlSchemaResult<()> {
    println!("=== XML Schema 解析示例 ===\n");

    // 1. 基础XML Schema解析
    basic_schema_parsing()?;
    
    // 2. 复杂Schema解析（包含属性和标记）
    complex_schema_parsing()?;
    
    // 3. 解析为Extensions
    extensions_parsing()?;
    
    // 4. 从文件解析
    file_parsing_example()?;

    println!("\n=== 所有示例执行完成 ===");
    Ok(())
}

/// 基础Schema解析示例
fn basic_schema_parsing() -> XmlSchemaResult<()> {
    println!("1. 基础Schema解析:");
    
    let xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema top_node="doc">
      <nodes>
        <node name="doc">
          <desc>文档根节点</desc>
          <content>paragraph+</content>
        </node>
        <node name="paragraph">
          <desc>段落节点</desc>
          <content>text*</content>
        </node>
        <node name="text">
          <desc>文本节点</desc>
        </node>
      </nodes>
    </schema>
    "#;

    // 解析为SchemaSpec
    let schema_spec = XmlSchemaParser::parse_from_str(xml)?;
    println!("   ✅ 解析成功");
    println!("   - 顶级节点: {:?}", schema_spec.top_node);
    println!("   - 节点数量: {}", schema_spec.nodes.len());
    println!("   - 标记数量: {}", schema_spec.marks.len());

    // 编译为Schema
    let schema = Schema::compile(schema_spec).map_err(|e| {
        mf_core::XmlSchemaError::InvalidNodeDefinition(format!("Schema编译失败: {}", e))
    })?;
    println!("   ✅ Schema编译成功");
    println!("   - 编译后节点数量: {}", schema.nodes.len());
    
    Ok(())
}

/// 复杂Schema解析示例
fn complex_schema_parsing() -> XmlSchemaResult<()> {
    println!("\n2. 复杂Schema解析（包含属性和标记）:");
    
    let xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <schema top_node="doc">
      <nodes>
        <node name="doc" group="block">
          <desc>文档根节点</desc>
          <content>paragraph+</content>
          <marks>_</marks>
          <attrs>
            <attr name="title" default="Untitled Document"/>
            <attr name="version" default="1.0"/>
          </attrs>
        </node>
        <node name="paragraph" group="block">
          <desc>段落节点</desc>
          <content>inline*</content>
          <marks>strong em link</marks>
          <attrs>
            <attr name="align" default="left"/>
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
          <attrs>
            <attr name="weight" default="bold"/>
          </attrs>
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
            <attr name="target" default="_self"/>
          </attrs>
        </mark>
      </marks>
    </schema>
    "#;

    let schema_spec = XmlSchemaParser::parse_from_str(xml)?;
    println!("   ✅ 复杂Schema解析成功");
    
    // 验证节点属性
    let doc_node = schema_spec.nodes.get("doc").unwrap();
    println!("   - doc节点组: {:?}", doc_node.group);
    println!("   - doc节点属性数量: {}", doc_node.attrs.as_ref().map_or(0, |a| a.len()));
    
    // 验证标记
    println!("   - 标记数量: {}", schema_spec.marks.len());
    let strong_mark = schema_spec.marks.get("strong").unwrap();
    println!("   - strong标记spanning: {:?}", strong_mark.spanning);
    
    Ok(())
}

/// Extensions解析示例
fn extensions_parsing() -> XmlSchemaResult<()> {
    println!("\n3. 解析为Extensions:");
    
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

    let extensions = XmlSchemaParser::parse_to_extensions(xml)?;
    println!("   ✅ Extensions解析成功");
    println!("   - Extensions数量: {}", extensions.len());
    
    // 统计不同类型的扩展
    let mut node_count = 0;
    let mut mark_count = 0;
    let mut extension_count = 0;
    
    for ext in &extensions {
        match ext {
            mf_core::types::Extensions::N(_) => node_count += 1,
            mf_core::types::Extensions::M(_) => mark_count += 1,
            mf_core::types::Extensions::E(_) => extension_count += 1,
        }
    }
    
    println!("   - 节点扩展: {}", node_count);
    println!("   - 标记扩展: {}", mark_count);
    println!("   - 其他扩展: {}", extension_count);
    
    Ok(())
}

/// 文件解析示例
fn file_parsing_example() -> XmlSchemaResult<()> {
    println!("\n4. 从文件解析示例:");
    
    // 检查示例文件是否存在
    let file_path = "test-data/xml-schemas/basic-document.xml";
    if std::path::Path::new(file_path).exists() {
        let schema_spec = XmlSchemaParser::parse_from_file(file_path)?;
        println!("   ✅ 从文件解析成功: {}", file_path);
        println!("   - 节点数量: {}", schema_spec.nodes.len());
        println!("   - 标记数量: {}", schema_spec.marks.len());
        
        // 也可以解析为Extensions
        let extensions = XmlSchemaParser::parse_extensions_from_file(file_path)?;
        println!("   - Extensions数量: {}", extensions.len());
    } else {
        println!("   ⚠️  示例文件不存在: {}", file_path);
        println!("   提示: 请确保在项目根目录运行此示例");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_schema_parsing_examples() {
        // 测试基础解析
        assert!(basic_schema_parsing().is_ok());
        
        // 测试复杂解析
        assert!(complex_schema_parsing().is_ok());
        
        // 测试Extensions解析
        assert!(extensions_parsing().is_ok());
    }
}

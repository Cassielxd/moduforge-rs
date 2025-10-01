//! XML Schema 解析示例
//!
//! 本示例展示如何使用XML格式定义Schema，并将其解析为ModuForge可用的结构。

use mf_core::{XmlSchemaParser, XmlSchemaResult, Extensions};
use mf_model::schema::{Schema, SchemaSpec};

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
        mf_core::XmlSchemaError::InvalidNodeDefinition(format!(
            "Schema编译失败: {e}"
        ))
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
    println!(
        "   - doc节点属性数量: {}",
        doc_node.attrs.as_ref().map_or(0, |a| a.len())
    );

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

    println!("   - 节点扩展: {node_count}");
    println!("   - 标记扩展: {mark_count}");
    println!("   - 其他扩展: {extension_count}");

    Ok(())
}

/// 文件解析示例 - 多文件解析
fn file_parsing_example() -> XmlSchemaResult<()> {
    println!("\n4. 多文件解析示例:");

    let current_dir = std::env::current_dir().unwrap();
    println!("   当前工作目录: {current_dir:?}");

    let file_path = "test-data/xml-schemas/multi-file/main-schema.xml";
    if std::path::Path::new(file_path).exists() {
        println!("   ✅ 找到多文件schema: {file_path}");

        // 检查引用的文件是否存在
        let base_dir = std::path::Path::new(file_path).parent().unwrap();
        let ref_files = [
            "base-nodes.xml",
            "formatting-marks.xml",
            "link-marks.xml",
            "table-extension.xml",
        ];
        for ref_file in &ref_files {
            let ref_path = base_dir.join(ref_file);
            if ref_path.exists() {
                println!("   ✅ 引用文件存在: {ref_file}");
            } else {
                println!("   ❌ 引用文件缺失: {ref_file}");
            }
        }

        // 使用多文件解析方法，支持import/include
        println!("   开始多文件解析...");
        let schema_spec = XmlSchemaParser::parse_multi_file(file_path)?;
        println!("   ✅ 多文件解析成功");
        println!("   - 节点数量: {}", schema_spec.nodes.len());
        println!("   - 标记数量: {}", schema_spec.marks.len());
        println!("   - 顶级节点: {:?}", schema_spec.top_node);

        // 显示具体的节点和标记
        println!(
            "   - 节点类型: {:?}",
            schema_spec.nodes.keys().collect::<Vec<_>>()
        );
        println!(
            "   - 标记类型: {:?}",
            schema_spec.marks.keys().collect::<Vec<_>>()
        );

        // 解析为Extensions（支持多文件）
        let extensions =
            XmlSchemaParser::parse_multi_file_to_extensions(file_path)?;
        println!("   - Extensions数量: {}", extensions.len());

        // 分析Extensions类型
        let mut node_count = 0;
        let mut mark_count = 0;
        let mut extension_count = 0;
        let mut has_global_attrs = false;

        for ext in &extensions {
            match ext {
                Extensions::N(_) => node_count += 1,
                Extensions::M(_) => mark_count += 1,
                Extensions::E(extension) => {
                    extension_count += 1;
                    if !extension.get_global_attributes().is_empty() {
                        has_global_attrs = true;
                        println!(
                            "   - 发现全局属性: {} 个",
                            extension.get_global_attributes().len()
                        );
                    }
                },
            }
        }

        println!(
            "   - 节点扩展: {node_count}, 标记扩展: {mark_count}, 其他扩展: {extension_count}"
        );
        println!("   - 包含全局属性: {has_global_attrs}");

        // 测试不同类型的默认值解析
        test_attribute_types(&schema_spec);

        return Ok(());
    }

    println!("   ⚠️  多文件schema不存在: {file_path}");
    println!("   提示: 请确保多文件测试数据存在");
    Ok(())
}

/// 测试不同类型的属性默认值解析
fn test_attribute_types(schema_spec: &SchemaSpec) {
    println!("\n🔍 属性类型解析测试:");

    // 测试 codeblock 节点的属性
    if let Some(codeblock) = schema_spec.nodes.get("codeblock") {
        if let Some(attrs) = &codeblock.attrs {
            for (attr_name, attr_spec) in attrs {
                if let Some(default_value) = &attr_spec.default {
                    let type_name = match default_value {
                        serde_json::Value::Bool(b) => format!("Bool({b})"),
                        serde_json::Value::Number(n) => {
                            format!("Number({n})")
                        },
                        serde_json::Value::String(s) => {
                            format!("String(\"{s}\")")
                        },
                        serde_json::Value::Null => "Null".to_string(),
                        serde_json::Value::Array(_) => "Array".to_string(),
                        serde_json::Value::Object(_) => "Object".to_string(),
                    };
                    println!("   ✅ codeblock.{attr_name}: {type_name}");
                } else {
                    println!("   ⚪ codeblock.{attr_name}: None");
                }
            }
        }
    }

    // 测试 list 节点的属性
    if let Some(list) = schema_spec.nodes.get("list") {
        if let Some(attrs) = &list.attrs {
            for (attr_name, attr_spec) in attrs {
                if let Some(default_value) = &attr_spec.default {
                    let type_name = match default_value {
                        serde_json::Value::Bool(b) => format!("Bool({b})"),
                        serde_json::Value::Number(n) => {
                            format!("Number({n})")
                        },
                        serde_json::Value::String(s) => {
                            format!("String(\"{s}\")")
                        },
                        serde_json::Value::Null => "Null".to_string(),
                        serde_json::Value::Array(_) => "Array".to_string(),
                        serde_json::Value::Object(_) => "Object".to_string(),
                    };
                    println!("   ✅ list.{attr_name}: {type_name}");
                } else {
                    println!("   ⚪ list.{attr_name}: None");
                }
            }
        }
    }

    // 测试 highlight 标记的属性
    if let Some(highlight) = schema_spec.marks.get("highlight") {
        if let Some(attrs) = &highlight.attrs {
            for (attr_name, attr_spec) in attrs {
                if let Some(default_value) = &attr_spec.default {
                    let type_name = match default_value {
                        serde_json::Value::Bool(b) => format!("Bool({b})"),
                        serde_json::Value::Number(n) => {
                            format!("Number({n})")
                        },
                        serde_json::Value::String(s) => {
                            format!("String(\"{s}\")")
                        },
                        serde_json::Value::Null => "Null".to_string(),
                        serde_json::Value::Array(_) => "Array".to_string(),
                        serde_json::Value::Object(_) => "Object".to_string(),
                    };
                    println!("   ✅ highlight.{attr_name}: {type_name}");
                } else {
                    println!("   ⚪ highlight.{attr_name}: None");
                }
            }
        }
    }
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

use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Cursor;
use std::sync::Arc;

use mf_model::schema::{AttributeSpec, SchemaSpec};

use crate::{mark::Mark, node::Node, types::{Extensions, GlobalAttributeItem}};

use super::error::{XmlSchemaError, XmlSchemaResult};

/// XML Schema 序列化器
pub struct XmlSchemaSerializer;

impl XmlSchemaSerializer {
    /// 将 `SchemaSpec` 序列化为 XML 字符串（基础版本，不包含全局属性）
    pub fn schema_spec_to_string(schema: &SchemaSpec) -> XmlSchemaResult<String> {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        // <schema>
        let mut root = BytesStart::new("schema");
        if let Some(top) = &schema.top_node {
            root.push_attribute(("top_node", top.as_str()));
        }
        writer.write_event(Event::Start(root)).map_err(map_io)?;

        // <nodes>
        if !schema.nodes.is_empty() {
            writer
                .write_event(Event::Start(BytesStart::new("nodes")))
                .map_err(map_io)?;

            for (name, spec) in &schema.nodes {
                let mut el = BytesStart::new("node");
                el.push_attribute(("name", name.as_str()));
                if let Some(group) = &spec.group {
                    el.push_attribute(("group", group.as_str()));
                }
                if let Some(desc) = &spec.desc {
                    el.push_attribute(("desc", desc.as_str()));
                }
                if let Some(content) = &spec.content {
                    el.push_attribute(("content", content.as_str()));
                }
                if let Some(marks) = &spec.marks {
                    el.push_attribute(("marks", marks.as_str()));
                }

                writer.write_event(Event::Start(el)).map_err(map_io)?;

                if let Some(attrs) = &spec.attrs {
                    writer
                        .write_event(Event::Start(BytesStart::new("attrs")))
                        .map_err(map_io)?;

                    for (attr_name, attr_spec) in attrs {
                        write_attr_element(&mut writer, attr_name, attr_spec)?;
                    }

                    writer
                        .write_event(Event::End(BytesEnd::new("attrs")))
                        .map_err(map_io)?;
                }

                writer
                    .write_event(Event::End(BytesEnd::new("node")))
                    .map_err(map_io)?;
            }

            writer
                .write_event(Event::End(BytesEnd::new("nodes")))
                .map_err(map_io)?;
        }

        // <marks>
        if !schema.marks.is_empty() {
            writer
                .write_event(Event::Start(BytesStart::new("marks")))
                .map_err(map_io)?;

            for (name, spec) in &schema.marks {
                let mut el = BytesStart::new("mark");
                el.push_attribute(("name", name.as_str()));
                if let Some(group) = &spec.group {
                    el.push_attribute(("group", group.as_str()));
                }
                if let Some(desc) = &spec.desc {
                    el.push_attribute(("desc", desc.as_str()));
                }
                if let Some(excludes) = &spec.excludes {
                    el.push_attribute(("excludes", excludes.as_str()));
                }
                if let Some(spanning) = spec.spanning {
                    if spanning {
                        el.push_attribute(("spanning", "true"));
                    }
                }

                writer.write_event(Event::Start(el)).map_err(map_io)?;

                if let Some(attrs) = &spec.attrs {
                    writer
                        .write_event(Event::Start(BytesStart::new("attrs")))
                        .map_err(map_io)?;

                    for (attr_name, attr_spec) in attrs {
                        write_attr_element(&mut writer, attr_name, attr_spec)?;
                    }

                    writer
                        .write_event(Event::End(BytesEnd::new("attrs")))
                        .map_err(map_io)?;
                }

                writer
                    .write_event(Event::End(BytesEnd::new("mark")))
                    .map_err(map_io)?;
            }

            writer
                .write_event(Event::End(BytesEnd::new("marks")))
                .map_err(map_io)?;
        }

        // </schema>
        writer
            .write_event(Event::End(BytesEnd::new("schema")))
            .map_err(map_io)?;

        let buf = writer.into_inner().into_inner();
        let xml = String::from_utf8(buf).map_err(|e| {
            XmlSchemaError::XmlParseError(quick_xml::Error::Io(Arc::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.utf8_error().to_string(),
            ))))
        })?;
        Ok(xml)
    }

    /// 将 `Extensions` 列表序列化为 XML 字符串（包含全局属性）
    /// 可选传入 `top_node` 覆盖顶层节点名称
    pub fn extensions_to_string(
        extensions: &[Extensions],
        top_node: Option<&str>,
    ) -> XmlSchemaResult<String> {
        let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

        let mut root = BytesStart::new("schema");
        if let Some(top) = top_node {
            root.push_attribute(("top_node", top));
        }
        writer.write_event(Event::Start(root)).map_err(map_io)?;

        // 收集 nodes / marks / global_attributes
        let mut nodes: Vec<&Node> = Vec::new();
        let mut marks: Vec<&Mark> = Vec::new();
        let mut global_attrs_sets: Vec<&[GlobalAttributeItem]> = Vec::new();

        for ext in extensions {
            match ext {
                Extensions::N(n) => nodes.push(n),
                Extensions::M(m) => marks.push(m),
                Extensions::E(e) => global_attrs_sets.push(e.get_global_attributes()),
            }
        }

        if !nodes.is_empty() {
            writer
                .write_event(Event::Start(BytesStart::new("nodes")))
                .map_err(map_io)?;
            for n in nodes {
                let mut el = BytesStart::new("node");
                el.push_attribute(("name", n.get_name()));
                if let Some(group) = &n.r#type.group {
                    el.push_attribute(("group", group.as_str()));
                }
                if let Some(desc) = &n.r#type.desc {
                    el.push_attribute(("desc", desc.as_str()));
                }
                if let Some(content) = &n.r#type.content {
                    el.push_attribute(("content", content.as_str()));
                }
                if let Some(marks_attr) = &n.r#type.marks {
                    el.push_attribute(("marks", marks_attr.as_str()));
                }
                writer.write_event(Event::Start(el)).map_err(map_io)?;

                if let Some(attrs) = &n.r#type.attrs {
                    writer
                        .write_event(Event::Start(BytesStart::new("attrs")))
                        .map_err(map_io)?;
                    for (attr_name, attr_spec) in attrs {
                        write_attr_element(&mut writer, attr_name, attr_spec)?;
                    }
                    writer
                        .write_event(Event::End(BytesEnd::new("attrs")))
                        .map_err(map_io)?;
                }

                writer
                    .write_event(Event::End(BytesEnd::new("node")))
                    .map_err(map_io)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("nodes")))
                .map_err(map_io)?;
        }

        if !marks.is_empty() {
            writer
                .write_event(Event::Start(BytesStart::new("marks")))
                .map_err(map_io)?;
            for m in marks {
                let mut el = BytesStart::new("mark");
                el.push_attribute(("name", m.get_name()));
                if let Some(group) = &m.r#type.group {
                    el.push_attribute(("group", group.as_str()));
                }
                if let Some(desc) = &m.r#type.desc {
                    el.push_attribute(("desc", desc.as_str()));
                }
                if let Some(excludes) = &m.r#type.excludes {
                    el.push_attribute(("excludes", excludes.as_str()));
                }
                if let Some(spanning) = m.r#type.spanning {
                    if spanning {
                        el.push_attribute(("spanning", "true"));
                    }
                }
                writer.write_event(Event::Start(el)).map_err(map_io)?;

                if let Some(attrs) = &m.r#type.attrs {
                    writer
                        .write_event(Event::Start(BytesStart::new("attrs")))
                        .map_err(map_io)?;
                    for (attr_name, attr_spec) in attrs {
                        write_attr_element(&mut writer, attr_name, attr_spec)?;
                    }
                    writer
                        .write_event(Event::End(BytesEnd::new("attrs")))
                        .map_err(map_io)?;
                }

                writer
                    .write_event(Event::End(BytesEnd::new("mark")))
                    .map_err(map_io)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("marks")))
                .map_err(map_io)?;
        }

        // <global_attributes>
        let mut wrote_global = false;
        for set in global_attrs_sets {
            if set.is_empty() {
                continue;
            }
            if !wrote_global {
                writer
                    .write_event(Event::Start(BytesStart::new("global_attributes")))
                    .map_err(map_io)?;
                wrote_global = true;
            }
            for item in set.iter() {
                write_global_attribute(&mut writer, item)?;
            }
        }
        if wrote_global {
            writer
                .write_event(Event::End(BytesEnd::new("global_attributes")))
                .map_err(map_io)?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("schema")))
            .map_err(map_io)?;

        let buf = writer.into_inner().into_inner();
        let xml = String::from_utf8(buf).map_err(|e| {
            XmlSchemaError::XmlParseError(quick_xml::Error::Io(Arc::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.utf8_error().to_string(),
            ))))
        })?;
        Ok(xml)
    }
}

fn write_attr_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    name: &str,
    attr: &AttributeSpec,
) -> XmlSchemaResult<()> {
    let mut el = BytesStart::new("attr");
    el.push_attribute(("name", name));
    if let Some(default) = &attr.default {
        let s = value_to_attr_string(default);
        el.push_attribute(("default", s.as_str()));
    }
    writer.write_event(Event::Empty(el)).map_err(map_io)?;
    Ok(())
}

fn write_global_attribute(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    item: &GlobalAttributeItem,
) -> XmlSchemaResult<()> {
    let mut el = BytesStart::new("global_attribute");
    let types_value = if item.types.len() == 1 && item.types[0] == "*" {
        "*".to_string()
    } else {
        item.types.join(" ")
    };
    el.push_attribute(("types", types_value.as_str()));
    writer.write_event(Event::Start(el)).map_err(map_io)?;

    if !item.attributes.is_empty() {
        for (name, spec) in &item.attributes {
            write_attr_element(writer, name, spec)?;
        }
    }

    writer.write_event(Event::End(BytesEnd::new("global_attribute"))).map_err(map_io)?;
    Ok(())
}

fn value_to_attr_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn map_io(err: quick_xml::Error) -> XmlSchemaError {
    XmlSchemaError::XmlParseError(err)
}



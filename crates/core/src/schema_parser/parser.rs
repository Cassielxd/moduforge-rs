use std::collections::HashMap;

use mf_model::{
    mark_type::MarkSpec,
    node_type::NodeSpec,
    schema::{AttributeSpec, SchemaSpec},
};
use serde_json::Value;

use crate::{
    extension::Extension,
    mark::Mark,
    node::Node,
    types::{Extensions, GlobalAttributeItem},
};

use super::error::{XmlSchemaError, XmlSchemaResult};
use super::types::{
    deserialize_optional_bool, deserialize_optional_value, XmlAttr, XmlAttrs,
    XmlGlobalAttribute, XmlGlobalAttributes, XmlInclude, XmlIncludes, XmlImport,
    XmlImports, XmlMark, XmlMarks, XmlNode, XmlNodes, XmlSchema,
    XmlSchemaWithReferences,
};

/// XML Schema 解析器
pub struct XmlSchemaParser;

/// 多文件解析上下文
#[derive(Debug, Clone)]
pub struct MultiFileParseContext {
    pub base_path: std::path::PathBuf,
    pub parsed_files: std::collections::HashSet<std::path::PathBuf>,
    pub max_depth: usize,
    pub current_depth: usize,
}

impl XmlSchemaParser {
    pub fn parse_from_str(xml_content: &str) -> XmlSchemaResult<SchemaSpec> {
        let xml_schema: XmlSchema = quick_xml::de::from_str(xml_content)?;
        Self::convert_to_schema_spec(xml_schema)
    }

    pub fn parse_from_file(file_path: &str) -> XmlSchemaResult<SchemaSpec> {
        let xml_content = std::fs::read_to_string(file_path).map_err(|e| {
            XmlSchemaError::XmlParseError(quick_xml::Error::Io(e.into()))
        })?;
        Self::parse_from_str(&xml_content)
    }

    pub fn parse_to_extensions(xml_content: &str) -> XmlSchemaResult<Vec<Extensions>> {
        if let Ok(xml_schema_with_refs) =
            quick_xml::de::from_str::<XmlSchemaWithReferences>(xml_content)
        {
            Self::convert_xml_schema_with_refs_to_extensions(xml_schema_with_refs)
        } else {
            let xml_schema: XmlSchema = quick_xml::de::from_str(xml_content)?;
            Self::convert_to_extensions(xml_schema)
        }
    }

    pub fn parse_extensions_from_file(file_path: &str) -> XmlSchemaResult<Vec<Extensions>> {
        let xml_content = std::fs::read_to_string(file_path).map_err(|e| {
            XmlSchemaError::XmlParseError(quick_xml::Error::Io(e.into()))
        })?;
        Self::parse_to_extensions(&xml_content)
    }

    pub fn parse_multi_file(file_path: &str) -> XmlSchemaResult<SchemaSpec> {
        let root_path = std::path::Path::new(file_path).canonicalize().map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法找到文件 {}: {}", file_path, e))
        })?;

        let mut context = MultiFileParseContext {
            base_path: root_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf(),
            parsed_files: std::collections::HashSet::new(),
            max_depth: 10,
            current_depth: 0,
        };

        Self::parse_file_with_context_new(&root_path, &mut context)
    }

    fn parse_file_with_context_new(
        file_path: &std::path::Path,
        context: &mut MultiFileParseContext,
    ) -> XmlSchemaResult<SchemaSpec> {
        if context.current_depth >= context.max_depth {
            return Err(XmlSchemaError::CircularReference(format!(
                "解析深度超过限制: {}",
                context.max_depth
            )));
        }

        let canonical_path = file_path.canonicalize().map_err(|e| {
            XmlSchemaError::FileNotFound(format!(
                "无法解析文件路径 {:?}: {}",
                file_path, e
            ))
        })?;

        if context.parsed_files.contains(&canonical_path) {
            return Err(XmlSchemaError::CircularReference(format!(
                "检测到循环引用: {:?}",
                canonical_path
            )));
        }

        context.parsed_files.insert(canonical_path.clone());
        context.current_depth += 1;

        let xml_content = std::fs::read_to_string(&canonical_path).map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法读取文件 {:?}: {}", canonical_path, e))
        })?;

        let xml_schema: XmlSchemaWithReferences = quick_xml::de::from_str(&xml_content)?;

        let mut merged_spec = SchemaSpec {
            nodes: HashMap::new(),
            marks: HashMap::new(),
            top_node: xml_schema.top_node.clone(),
        };

        let old_base_path = context.base_path.clone();
        if let Some(parent) = canonical_path.parent() {
            context.base_path = parent.to_path_buf();
        }

        if let Some(imports) = xml_schema.imports {
            for import in imports.imports {
                let import_path = Self::resolve_relative_path(&context.base_path, &import.src)?;
                let imported_spec = Self::parse_file_with_context_new(&import_path, context)?;
                Self::merge_schema_spec(&mut merged_spec, imported_spec, false)?;
            }
        }

        if let Some(includes) = xml_schema.includes {
            for include in includes.includes {
                let include_path = Self::resolve_relative_path(&context.base_path, &include.src)?;
                let included_spec = Self::parse_file_with_context_new(&include_path, context)?;
                Self::merge_schema_spec(&mut merged_spec, included_spec, true)?;
            }
        }

        context.base_path = old_base_path;

        let current_spec = Self::convert_xml_schema_to_spec(XmlSchema {
            top_node: xml_schema.top_node,
            nodes: xml_schema.nodes,
            marks: xml_schema.marks,
        })?;

        Self::merge_schema_spec(&mut merged_spec, current_spec, true)?;

        context.current_depth -= 1;
        Ok(merged_spec)
    }

    fn resolve_relative_path(
        base_path: &std::path::Path,
        relative_path: &str,
    ) -> XmlSchemaResult<std::path::PathBuf> {
        let path = if std::path::Path::new(relative_path).is_absolute() {
            std::path::PathBuf::from(relative_path)
        } else {
            base_path.join(relative_path)
        };

        path.canonicalize().map_err(|e| {
            XmlSchemaError::PathResolutionError(format!(
                "无法解析路径 {} (基于 {:?}): {}",
                relative_path, base_path, e
            ))
        })
    }

    pub fn parse_multi_file_to_extensions(
        file_path: &str,
    ) -> XmlSchemaResult<Vec<Extensions>> {
        let root_path = std::path::Path::new(file_path).canonicalize().map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法找到文件 {}: {}", file_path, e))
        })?;

        let mut context = MultiFileParseContext {
            base_path: root_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf(),
            parsed_files: std::collections::HashSet::new(),
            max_depth: 10,
            current_depth: 0,
        };

        Self::parse_file_to_extensions_with_context_new(&root_path, &mut context)
    }

    fn parse_file_to_extensions_with_context_new(
        file_path: &std::path::Path,
        context: &mut MultiFileParseContext,
    ) -> XmlSchemaResult<Vec<Extensions>> {
        if context.current_depth >= context.max_depth {
            return Err(XmlSchemaError::CircularReference(format!(
                "解析深度超过限制: {}",
                context.max_depth
            )));
        }

        let canonical_path = file_path.canonicalize().map_err(|e| {
            XmlSchemaError::FileNotFound(format!(
                "无法解析文件路径 {:?}: {}",
                file_path, e
            ))
        })?;

        if context.parsed_files.contains(&canonical_path) {
            return Err(XmlSchemaError::CircularReference(format!(
                "检测到循环引用: {:?}",
                canonical_path
            )));
        }

        context.parsed_files.insert(canonical_path.clone());
        context.current_depth += 1;

        let xml_content = std::fs::read_to_string(&canonical_path).map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法读取文件 {:?}: {}", canonical_path, e))
        })?;

        let xml_schema: XmlSchemaWithReferences = quick_xml::de::from_str(&xml_content)?;

        let mut all_extensions = Vec::new();

        let old_base_path = context.base_path.clone();
        if let Some(parent) = canonical_path.parent() {
            context.base_path = parent.to_path_buf();
        }

        if let Some(imports) = &xml_schema.imports {
            for import in &imports.imports {
                let import_path = Self::resolve_relative_path(&context.base_path, &import.src)?;
                let imported_extensions = Self::parse_file_to_extensions_with_context_new(
                    &import_path,
                    context,
                )?;
                all_extensions.extend(imported_extensions);
            }
        }

        if let Some(includes) = &xml_schema.includes {
            for include in &includes.includes {
                let include_path = Self::resolve_relative_path(&context.base_path, &include.src)?;
                let included_extensions = Self::parse_file_to_extensions_with_context_new(
                    &include_path,
                    context,
                )?;
                all_extensions.extend(included_extensions);
            }
        }

        context.base_path = old_base_path;

        let current_schema = XmlSchema {
            top_node: xml_schema.top_node,
            nodes: xml_schema.nodes,
            marks: xml_schema.marks,
        };
        let current_extensions = Self::convert_to_extensions(current_schema)?;
        all_extensions.extend(current_extensions);

        if let Some(xml_global_attrs) = &xml_schema.global_attributes {
            let mut extension = Extension::new();
            for xml_global_attr in &xml_global_attrs.global_attributes {
                let global_attr_item =
                    Self::convert_xml_global_attribute_to_item(xml_global_attr.clone())?;
                extension.add_global_attribute(global_attr_item);
            }
            all_extensions.push(Extensions::E(extension));
        }

        context.current_depth -= 1;
        Ok(all_extensions)
    }

    pub fn parse_file_to_extensions_with_context(
        file_path: &str,
        context: &mut MultiFileParseContext,
    ) -> XmlSchemaResult<Vec<Extensions>> {
        if context.current_depth >= context.max_depth {
            return Err(XmlSchemaError::CircularReference(format!(
                "解析深度超过限制: {}",
                context.max_depth
            )));
        }

        let file_path_buf = if std::path::Path::new(file_path).is_absolute() {
            std::path::PathBuf::from(file_path)
        } else {
            context.base_path.join(file_path)
        };

        let canonical_path = file_path_buf.canonicalize().map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法解析文件路径 {}: {}", file_path, e))
        })?;

        if context.parsed_files.contains(&canonical_path) {
            return Err(XmlSchemaError::CircularReference(format!(
                "检测到循环引用: {:?}",
                canonical_path
            )));
        }

        context.parsed_files.insert(canonical_path.clone());
        context.current_depth += 1;

        let xml_content = std::fs::read_to_string(&canonical_path).map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法读取文件 {:?}: {}", canonical_path, e))
        })?;

        let xml_schema: XmlSchemaWithReferences = quick_xml::de::from_str(&xml_content)?;

        let mut all_extensions = Vec::new();

        let old_base_path = context.base_path.clone();
        if let Some(parent) = canonical_path.parent() {
            context.base_path = parent.to_path_buf();
        }

        if let Some(imports) = &xml_schema.imports {
            for import in &imports.imports {
                let imported_extensions =
                    Self::parse_file_to_extensions_with_context(&import.src, context)?;
                all_extensions.extend(imported_extensions);
            }
        }

        if let Some(includes) = &xml_schema.includes {
            for include in &includes.includes {
                let included_extensions =
                    Self::parse_file_to_extensions_with_context(&include.src, context)?;
                all_extensions.extend(included_extensions);
            }
        }

        context.base_path = old_base_path;

        let current_schema = XmlSchema {
            top_node: xml_schema.top_node,
            nodes: xml_schema.nodes,
            marks: xml_schema.marks,
        };
        let current_extensions = Self::convert_to_extensions(current_schema)?;
        all_extensions.extend(current_extensions);

        if let Some(xml_global_attrs) = &xml_schema.global_attributes {
            let mut extension = Extension::new();
            for xml_global_attr in &xml_global_attrs.global_attributes {
                let global_attr_item =
                    Self::convert_xml_global_attribute_to_item(xml_global_attr.clone())?;
                extension.add_global_attribute(global_attr_item);
            }
            all_extensions.push(Extensions::E(extension));
        }

        context.current_depth -= 1;
        Ok(all_extensions)
    }

    pub fn parse_file_with_context(
        file_path: &str,
        context: &mut MultiFileParseContext,
    ) -> XmlSchemaResult<SchemaSpec> {
        if context.current_depth >= context.max_depth {
            return Err(XmlSchemaError::CircularReference(format!(
                "解析深度超过限制: {}",
                context.max_depth
            )));
        }

        let file_path_buf = if std::path::Path::new(file_path).is_absolute() {
            std::path::PathBuf::from(file_path)
        } else {
            context.base_path.join(file_path)
        };

        let canonical_path = file_path_buf.canonicalize().map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法解析文件路径 {}: {}", file_path, e))
        })?;

        if context.parsed_files.contains(&canonical_path) {
            return Err(XmlSchemaError::CircularReference(format!(
                "检测到循环引用: {:?}",
                canonical_path
            )));
        }

        context.parsed_files.insert(canonical_path.clone());
        context.current_depth += 1;

        let xml_content = std::fs::read_to_string(&canonical_path).map_err(|e| {
            XmlSchemaError::FileNotFound(format!("无法读取文件 {:?}: {}", canonical_path, e))
        })?;

        let xml_schema: XmlSchemaWithReferences = quick_xml::de::from_str(&xml_content)?;

        let mut merged_spec = SchemaSpec {
            nodes: HashMap::new(),
            marks: HashMap::new(),
            top_node: xml_schema.top_node.clone(),
        };

        let old_base_path = context.base_path.clone();
        if let Some(parent) = canonical_path.parent() {
            context.base_path = parent.to_path_buf();
        }

        if let Some(imports) = xml_schema.imports {
            for import in imports.imports {
                let imported_spec = Self::parse_file_with_context(&import.src, context)?;
                Self::merge_schema_spec(&mut merged_spec, imported_spec, false)?;
            }
        }

        if let Some(includes) = xml_schema.includes {
            for include in includes.includes {
                let included_spec = Self::parse_file_with_context(&include.src, context)?;
                Self::merge_schema_spec(&mut merged_spec, included_spec, true)?;
            }
        }

        context.base_path = old_base_path;

        let current_spec = Self::convert_xml_schema_to_spec(XmlSchema {
            top_node: xml_schema.top_node,
            nodes: xml_schema.nodes,
            marks: xml_schema.marks,
        })?;
        Self::merge_schema_spec(&mut merged_spec, current_spec, true)?;

        context.current_depth -= 1;
        Ok(merged_spec)
    }

    pub fn convert_to_extensions_from_spec(
        schema_spec: SchemaSpec,
    ) -> XmlSchemaResult<Vec<Extensions>> {
        let xml_schema = XmlSchema {
            top_node: schema_spec.top_node,
            nodes: Some(XmlNodes {
                nodes: schema_spec
                    .nodes
                    .into_iter()
                    .map(|(name, spec)| XmlNode {
                        name,
                        group: spec.group,
                        desc: spec.desc,
                        content: spec.content,
                        marks: spec.marks,
                        attrs: spec.attrs.map(|attrs| XmlAttrs {
                            attrs: attrs
                                .into_iter()
                                .map(|(name, attr_spec)| XmlAttr {
                                    name,
                                    default: attr_spec.default,
                                })
                                .collect(),
                        }),
                    })
                    .collect(),
            }),
            marks: Some(XmlMarks {
                marks: schema_spec
                    .marks
                    .into_iter()
                    .map(|(name, spec)| XmlMark {
                        name,
                        group: spec.group,
                        desc: spec.desc,
                        excludes: spec.excludes,
                        spanning: spec.spanning,
                        attrs: spec.attrs.map(|attrs| XmlAttrs {
                            attrs: attrs
                                .into_iter()
                                .map(|(name, attr_spec)| XmlAttr {
                                    name,
                                    default: attr_spec.default,
                                })
                                .collect(),
                        }),
                    })
                    .collect(),
            }),
        };

        Self::convert_to_extensions(xml_schema)
    }

    fn merge_schema_spec(
        target: &mut SchemaSpec,
        source: SchemaSpec,
        allow_override: bool,
    ) -> XmlSchemaResult<()> {
        for (name, node_spec) in source.nodes {
            if target.nodes.contains_key(&name) && !allow_override {
                return Err(XmlSchemaError::DuplicateNodeName(format!(
                    "节点 '{}' 已存在，不允许覆盖",
                    name
                )));
            }
            target.nodes.insert(name, node_spec);
        }

        for (name, mark_spec) in source.marks {
            if target.marks.contains_key(&name) && !allow_override {
                return Err(XmlSchemaError::DuplicateMarkName(format!(
                    "标记 '{}' 已存在，不允许覆盖",
                    name
                )));
            }
            target.marks.insert(name, mark_spec);
        }

        if target.top_node.is_none() && source.top_node.is_some() {
            target.top_node = source.top_node;
        }

        Ok(())
    }

    fn convert_xml_schema_to_spec(xml_schema: XmlSchema) -> XmlSchemaResult<SchemaSpec> {
        Self::convert_to_schema_spec(xml_schema)
    }

    fn convert_to_extensions(xml_schema: XmlSchema) -> XmlSchemaResult<Vec<Extensions>> {
        let mut extensions = Vec::new();

        if let Some(xml_nodes) = xml_schema.nodes {
            for xml_node in xml_nodes.nodes {
                let mut node = Node::create(&xml_node.name, NodeSpec::default());
                if let Some(group) = xml_node.group {
                    node.r#type.group = Some(group);
                }
                if let Some(desc) = xml_node.desc {
                    node.set_desc(&desc);
                }
                if let Some(content) = xml_node.content {
                    node.set_content(&content);
                }
                if let Some(marks) = xml_node.marks {
                    node.set_marks(marks);
                }
                if let Some(xml_attrs) = xml_node.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    node.set_attrs(attrs);
                }
                extensions.push(Extensions::N(node));
            }
        }

        if let Some(xml_marks) = xml_schema.marks {
            for xml_mark in xml_marks.marks {
                let mut mark = Mark::new(&xml_mark.name, MarkSpec::default());
                if let Some(group) = xml_mark.group {
                    mark.r#type.group = Some(group);
                }
                if let Some(desc) = xml_mark.desc {
                    mark.set_desc(&desc);
                }
                if let Some(excludes) = xml_mark.excludes {
                    mark.r#type.excludes = Some(excludes);
                }
                if let Some(spanning) = xml_mark.spanning {
                    mark.r#type.spanning = Some(spanning);
                }
                if let Some(xml_attrs) = xml_mark.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    mark.set_attrs(attrs);
                }
                extensions.push(Extensions::M(mark));
            }
        }

        let extension = Extension::new();
        extensions.push(Extensions::E(extension));

        Ok(extensions)
    }

    fn convert_xml_schema_with_refs_to_extensions(
        xml_schema: XmlSchemaWithReferences,
    ) -> XmlSchemaResult<Vec<Extensions>> {
        let mut extensions = Vec::new();

        if let Some(xml_nodes) = xml_schema.nodes {
            for xml_node in xml_nodes.nodes {
                let mut node = Node::create(&xml_node.name, NodeSpec::default());
                if let Some(group) = xml_node.group {
                    node.r#type.group = Some(group);
                }
                if let Some(desc) = xml_node.desc {
                    node.set_desc(&desc);
                }
                if let Some(content) = xml_node.content {
                    node.set_content(&content);
                }
                if let Some(marks) = xml_node.marks {
                    node.set_marks(marks);
                }
                if let Some(xml_attrs) = xml_node.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    node.set_attrs(attrs);
                }
                extensions.push(Extensions::N(node));
            }
        }

        if let Some(xml_marks) = xml_schema.marks {
            for xml_mark in xml_marks.marks {
                let mut mark = Mark::new(&xml_mark.name, MarkSpec::default());
                if let Some(group) = xml_mark.group {
                    mark.r#type.group = Some(group);
                }
                if let Some(desc) = xml_mark.desc {
                    mark.set_desc(&desc);
                }
                if let Some(excludes) = xml_mark.excludes {
                    mark.r#type.excludes = Some(excludes);
                }
                if let Some(spanning) = xml_mark.spanning {
                    mark.r#type.spanning = Some(spanning);
                }
                if let Some(xml_attrs) = xml_mark.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    mark.set_attrs(attrs);
                }
                extensions.push(Extensions::M(mark));
            }
        }

        if let Some(xml_global_attrs) = xml_schema.global_attributes {
            let mut extension = Extension::new();
            for xml_global_attr in xml_global_attrs.global_attributes {
                let global_attr_item = Self::convert_xml_global_attribute_to_item(xml_global_attr)?;
                extension.add_global_attribute(global_attr_item);
            }
            extensions.push(Extensions::E(extension));
        } else {
            let extension = Extension::new();
            extensions.push(Extensions::E(extension));
        }

        Ok(extensions)
    }

    fn convert_to_schema_spec(xml_schema: XmlSchema) -> XmlSchemaResult<SchemaSpec> {
        let mut nodes = HashMap::new();
        let mut marks = HashMap::new();

        if let Some(xml_nodes) = xml_schema.nodes {
            for xml_node in xml_nodes.nodes {
                if nodes.contains_key(&xml_node.name) {
                    return Err(XmlSchemaError::DuplicateNodeName(xml_node.name.clone()));
                }
                let node_name = xml_node.name.clone();
                let node_spec = Self::convert_xml_node_to_spec(xml_node)?;
                nodes.insert(node_name, node_spec);
            }
        }

        if let Some(xml_marks) = xml_schema.marks {
            for xml_mark in xml_marks.marks {
                if marks.contains_key(&xml_mark.name) {
                    return Err(XmlSchemaError::DuplicateMarkName(xml_mark.name.clone()));
                }
                let mark_name = xml_mark.name.clone();
                let mark_spec = Self::convert_xml_mark_to_spec(xml_mark)?;
                marks.insert(mark_name, mark_spec);
            }
        }

        Ok(SchemaSpec { nodes, marks, top_node: xml_schema.top_node })
    }

    fn convert_xml_node_to_spec(xml_node: XmlNode) -> XmlSchemaResult<NodeSpec> {
        let attrs = if let Some(xml_attrs) = xml_node.attrs {
            Some(Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?)
        } else {
            None
        };

        Ok(NodeSpec {
            content: xml_node.content,
            marks: xml_node.marks,
            group: xml_node.group,
            desc: xml_node.desc,
            attrs,
        })
    }

    fn convert_xml_mark_to_spec(xml_mark: XmlMark) -> XmlSchemaResult<MarkSpec> {
        let attrs = if let Some(xml_attrs) = xml_mark.attrs {
            Some(Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?)
        } else {
            None
        };

        Ok(MarkSpec { attrs, excludes: xml_mark.excludes, group: xml_mark.group, spanning: xml_mark.spanning, desc: xml_mark.desc })
    }

    fn convert_xml_attrs_to_spec(
        xml_attrs: Vec<XmlAttr>,
    ) -> XmlSchemaResult<HashMap<String, AttributeSpec>> {
        let mut attrs = HashMap::new();
        for xml_attr in xml_attrs {
            let default_value = if let Some(default_value) = xml_attr.default {
                Some(default_value)
            } else {
                None
            };

            attrs.insert(xml_attr.name.clone(), AttributeSpec { default: default_value });
        }
        Ok(attrs)
    }

    pub fn parse_attribute_value(value_str: &str) -> XmlSchemaResult<Value> {
        if let Ok(json_value) = serde_json::from_str::<Value>(value_str) {
            return Ok(json_value);
        }
        Ok(Value::String(value_str.to_string()))
    }

    fn convert_xml_global_attribute_to_item(
        xml_global_attr: XmlGlobalAttribute,
    ) -> XmlSchemaResult<GlobalAttributeItem> {
        let types = if xml_global_attr.types.trim() == "*" {
            vec!["*".to_string()]
        } else {
            xml_global_attr
                .types
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        let attributes = Self::convert_xml_attrs_to_spec(xml_global_attr.attrs)?;
        Ok(GlobalAttributeItem { types, attributes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_schema() {
        let xml = r#"
        <?xml version=\"1.0\" encoding=\"UTF-8\"?>
        <schema top_node=\"doc\">
          <nodes>
            <node name=\"doc\" desc=\"文档根节点\" content=\"paragraph+\"/>
            <node name=\"paragraph\" desc=\"段落节点\" content=\"text*\"/>
            <node name=\"text\" desc=\"文本节点\"/>
          </nodes>
        </schema>
        "#;

        let result = XmlSchemaParser::parse_from_str(xml);
        assert!(result.is_ok());
        let schema_spec = result.unwrap();
        assert_eq!(schema_spec.top_node, Some("doc".to_string()));
        assert_eq!(schema_spec.nodes.len(), 3);
        assert!(schema_spec.nodes.contains_key("doc"));
        assert!(schema_spec.nodes.contains_key("paragraph"));
        assert!(schema_spec.nodes.contains_key("text"));
    }
}



//! XML Schema 解析器模块
//!
//! 该模块提供从XML格式解析Schema定义的功能，支持将XML schema定义
//! 转换为ModuForge的SchemaSpec结构。
//!
//! # XML Schema 格式
//!
//! ## 基础格式
//! ```xml
//! <?xml version="1.0" encoding="UTF-8"?>
//! <schema top_node="doc">
//!   <nodes>
//!     <node name="doc" group="block">
//!       <desc>文档根节点</desc>
//!       <content>paragraph+</content>
//!       <marks>_</marks>
//!       <attrs>
//!         <attr name="title" default="Untitled Document"/>
//!         <attr name="version" default="1.0"/>
//!       </attrs>
//!     </node>
//!   </nodes>
//! </schema>
//! ```
//!
//! ## 多文件引用格式
//! ```xml
//! <?xml version="1.0" encoding="UTF-8"?>
//! <schema top_node="doc">
//!   <!-- 导入其他schema文件 -->
//!   <imports>
//!     <import src="./base-nodes.xml"/>
//!     <import src="./formatting-marks.xml"/>
//!     <import src="../common/table-nodes.xml"/>
//!   </imports>
//!
//!   <!-- 包含其他schema内容（内联合并） -->
//!   <includes>
//!     <include src="./extensions.xml"/>
//!   </includes>
//!
//!   <!-- 全局属性定义 -->
//!   <global_attributes>
//!     <global_attribute types="paragraph heading">
//!       <attr name="id"/>
//!       <attr name="class"/>
//!       <attr name="style"/>
//!     </global_attribute>
//!     <global_attribute types="*">
//!       <attr name="data-custom"/>
//!     </global_attribute>
//!   </global_attributes>
//!
//!   <!-- 本文件定义的节点和标记 -->
//!   <nodes>
//!     <node name="custom_node">
//!       <desc>自定义节点</desc>
//!     </node>
//!   </nodes>
//! </schema>
//! ```

use mf_model::schema::{SchemaSpec, AttributeSpec};
use mf_model::node_type::NodeSpec;
use mf_model::mark_type::MarkSpec;
use crate::node::Node;
use crate::mark::Mark;
use crate::extension::Extension;
use crate::types::{Extensions, GlobalAttributeItem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// XML Schema 解析错误类型
#[derive(Error, Debug)]
pub enum XmlSchemaError {
    #[error("XML 解析错误: {0}")]
    XmlParseError(#[from] quick_xml::Error),
    
    #[error("XML 反序列化错误: {0}")]
    DeserializeError(#[from] quick_xml::DeError),
    
    #[error("JSON 值解析错误: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("缺少必需的属性: {0}")]
    MissingAttribute(String),
    
    #[error("无效的节点定义: {0}")]
    InvalidNodeDefinition(String),
    
    #[error("无效的标记定义: {0}")]
    InvalidMarkDefinition(String),
    
    #[error("重复的节点名称: {0}")]
    DuplicateNodeName(String),
    
    #[error("重复的标记名称: {0}")]
    DuplicateMarkName(String),

    #[error("文件引用错误: {0}")]
    FileReferenceError(String),

    #[error("循环引用检测到: {0}")]
    CircularReference(String),

    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("相对路径解析错误: {0}")]
    PathResolutionError(String),
}

/// XML Schema 解析结果类型
pub type XmlSchemaResult<T> = Result<T, XmlSchemaError>;

/// XML Schema 解析器
pub struct XmlSchemaParser;

/// 多文件解析上下文
#[derive(Debug, Clone)]
pub struct MultiFileParseContext {
    /// 当前文件的基础路径
    pub base_path: std::path::PathBuf,
    /// 已解析的文件路径集合（用于循环引用检测）
    pub parsed_files: std::collections::HashSet<std::path::PathBuf>,
    /// 解析深度限制
    pub max_depth: usize,
    /// 当前解析深度
    pub current_depth: usize,
}

impl XmlSchemaParser {
    /// 从XML字符串解析Schema定义
    ///
    /// # 参数
    /// * `xml_content` - XML格式的schema定义字符串
    ///
    /// # 返回值
    /// * `XmlSchemaResult<SchemaSpec>` - 解析后的SchemaSpec或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::schema_parser::XmlSchemaParser;
    /// 
    /// let xml = r#"
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <schema top_node="doc">
    ///   <nodes>
    ///     <node name="doc">
    ///       <content>paragraph+</content>
    ///     </node>
    ///   </nodes>
    /// </schema>
    /// "#;
    /// 
    /// let schema_spec = XmlSchemaParser::parse_from_str(xml)?;
    /// ```
    pub fn parse_from_str(xml_content: &str) -> XmlSchemaResult<SchemaSpec> {
        let xml_schema: XmlSchema = quick_xml::de::from_str(xml_content)?;
        Self::convert_to_schema_spec(xml_schema)
    }

    /// 从XML文件解析Schema定义
    ///
    /// # 参数
    /// * `file_path` - XML文件路径
    ///
    /// # 返回值
    /// * `XmlSchemaResult<SchemaSpec>` - 解析后的SchemaSpec或错误
    pub fn parse_from_file(file_path: &str) -> XmlSchemaResult<SchemaSpec> {
        let xml_content = std::fs::read_to_string(file_path)
            .map_err(|e| XmlSchemaError::XmlParseError(quick_xml::Error::Io(e.into())))?;
        Self::parse_from_str(&xml_content)
    }

    /// 从XML字符串解析为Extensions列表
    ///
    /// # 参数
    /// * `xml_content` - XML格式的schema定义字符串
    ///
    /// # 返回值
    /// * `XmlSchemaResult<Vec<Extensions>>` - 解析后的Extensions列表或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::schema_parser::XmlSchemaParser;
    ///
    /// let xml = r#"
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <schema top_node="doc">
    ///   <nodes>
    ///     <node name="doc">
    ///       <content>paragraph+</content>
    ///     </node>
    ///   </nodes>
    /// </schema>
    /// "#;
    ///
    /// let extensions = XmlSchemaParser::parse_to_extensions(xml)?;
    /// ```
    pub fn parse_to_extensions(xml_content: &str) -> XmlSchemaResult<Vec<Extensions>> {
        // 尝试解析为支持引用的完整格式
        if let Ok(xml_schema_with_refs) = quick_xml::de::from_str::<XmlSchemaWithReferences>(xml_content) {
            Self::convert_xml_schema_with_refs_to_extensions(xml_schema_with_refs)
        } else {
            // 回退到基础格式
            let xml_schema: XmlSchema = quick_xml::de::from_str(xml_content)?;
            Self::convert_to_extensions(xml_schema)
        }
    }

    /// 从XML文件解析为Extensions列表
    ///
    /// # 参数
    /// * `file_path` - XML文件路径
    ///
    /// # 返回值
    /// * `XmlSchemaResult<Vec<Extensions>>` - 解析后的Extensions列表或错误
    pub fn parse_extensions_from_file(file_path: &str) -> XmlSchemaResult<Vec<Extensions>> {
        let xml_content = std::fs::read_to_string(file_path)
            .map_err(|e| XmlSchemaError::XmlParseError(quick_xml::Error::Io(e.into())))?;
        Self::parse_to_extensions(&xml_content)
    }

    /// 从XML文件解析Schema定义（支持多文件引用）
    ///
    /// # 参数
    /// * `file_path` - 主XML文件路径
    ///
    /// # 返回值
    /// * `XmlSchemaResult<SchemaSpec>` - 解析后的SchemaSpec或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::schema_parser::XmlSchemaParser;
    ///
    /// // 解析支持import/include的schema文件
    /// let schema_spec = XmlSchemaParser::parse_multi_file("./schemas/main.xml")?;
    /// ```
    pub fn parse_multi_file(file_path: &str) -> XmlSchemaResult<SchemaSpec> {
        let base_path = std::path::Path::new(file_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();

        let mut context = MultiFileParseContext {
            base_path,
            parsed_files: std::collections::HashSet::new(),
            max_depth: 10, // 最大引用深度
            current_depth: 0,
        };

        Self::parse_file_with_context(file_path, &mut context)
    }

    /// 从XML文件解析为Extensions列表（支持多文件引用）
    ///
    /// # 参数
    /// * `file_path` - 主XML文件路径
    ///
    /// # 返回值
    /// * `XmlSchemaResult<Vec<Extensions>>` - 解析后的Extensions列表或错误
    pub fn parse_multi_file_to_extensions(file_path: &str) -> XmlSchemaResult<Vec<Extensions>> {
        let base_path = std::path::Path::new(file_path)
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();

        let mut context = MultiFileParseContext {
            base_path,
            parsed_files: std::collections::HashSet::new(),
            max_depth: 10,
            current_depth: 0,
        };

        Self::parse_file_to_extensions_with_context(file_path, &mut context)
    }

    /// 使用上下文解析文件为Extensions（支持global_attributes）
    fn parse_file_to_extensions_with_context(
        file_path: &str,
        context: &mut MultiFileParseContext
    ) -> XmlSchemaResult<Vec<Extensions>> {
        // 检查解析深度
        if context.current_depth >= context.max_depth {
            return Err(XmlSchemaError::CircularReference(
                format!("解析深度超过限制: {}", context.max_depth)
            ));
        }

        // 解析文件路径
        let file_path_buf = if std::path::Path::new(file_path).is_absolute() {
            std::path::PathBuf::from(file_path)
        } else {
            context.base_path.join(file_path)
        };

        let canonical_path = file_path_buf.canonicalize()
            .map_err(|e| XmlSchemaError::FileNotFound(
                format!("无法解析文件路径 {}: {}", file_path, e)
            ))?;

        // 检查循环引用
        if context.parsed_files.contains(&canonical_path) {
            return Err(XmlSchemaError::CircularReference(
                format!("检测到循环引用: {:?}", canonical_path)
            ));
        }

        // 标记文件为已解析
        context.parsed_files.insert(canonical_path.clone());
        context.current_depth += 1;

        // 读取并解析文件
        let xml_content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| XmlSchemaError::FileNotFound(
                format!("无法读取文件 {:?}: {}", canonical_path, e)
            ))?;

        let xml_schema: XmlSchemaWithReferences = quick_xml::de::from_str(&xml_content)?;

        let mut all_extensions = Vec::new();

        // 更新基础路径为当前文件的目录
        let old_base_path = context.base_path.clone();
        if let Some(parent) = canonical_path.parent() {
            context.base_path = parent.to_path_buf();
        }

        // 处理imports（导入其他schema）
        if let Some(imports) = &xml_schema.imports {
            for import in &imports.imports {
                let imported_extensions = Self::parse_file_to_extensions_with_context(&import.src, context)?;
                all_extensions.extend(imported_extensions);
            }
        }

        // 处理includes（包含其他schema内容）
        if let Some(includes) = &xml_schema.includes {
            for include in &includes.includes {
                let included_extensions = Self::parse_file_to_extensions_with_context(&include.src, context)?;
                all_extensions.extend(included_extensions);
            }
        }

        // 恢复基础路径
        context.base_path = old_base_path;

        // 处理当前文件的节点和标记
        let current_schema = XmlSchema {
            top_node: xml_schema.top_node,
            nodes: xml_schema.nodes,
            marks: xml_schema.marks,
        };
        let current_extensions = Self::convert_to_extensions(current_schema)?;
        all_extensions.extend(current_extensions);

        // 处理全局属性
        if let Some(xml_global_attrs) = &xml_schema.global_attributes {
            let mut extension = Extension::new();
            for xml_global_attr in &xml_global_attrs.global_attributes {
                let global_attr_item = Self::convert_xml_global_attribute_to_item(xml_global_attr.clone())?;
                extension.add_global_attribute(global_attr_item);
            }
            all_extensions.push(Extensions::E(extension));
        }

        // 恢复解析深度
        context.current_depth -= 1;

        Ok(all_extensions)
    }

    /// 使用上下文解析文件（支持引用）
    fn parse_file_with_context(
        file_path: &str,
        context: &mut MultiFileParseContext
    ) -> XmlSchemaResult<SchemaSpec> {
        // 检查解析深度
        if context.current_depth >= context.max_depth {
            return Err(XmlSchemaError::CircularReference(
                format!("解析深度超过限制: {}", context.max_depth)
            ));
        }

        // 解析文件路径
        let file_path_buf = if std::path::Path::new(file_path).is_absolute() {
            std::path::PathBuf::from(file_path)
        } else {
            context.base_path.join(file_path)
        };

        let canonical_path = file_path_buf.canonicalize()
            .map_err(|e| XmlSchemaError::FileNotFound(
                format!("无法解析文件路径 {}: {}", file_path, e)
            ))?;

        // 检查循环引用
        if context.parsed_files.contains(&canonical_path) {
            return Err(XmlSchemaError::CircularReference(
                format!("检测到循环引用: {:?}", canonical_path)
            ));
        }

        // 标记文件为已解析
        context.parsed_files.insert(canonical_path.clone());
        context.current_depth += 1;

        // 读取并解析文件
        let xml_content = std::fs::read_to_string(&canonical_path)
            .map_err(|e| XmlSchemaError::FileNotFound(
                format!("无法读取文件 {:?}: {}", canonical_path, e)
            ))?;

        let xml_schema: XmlSchemaWithReferences = quick_xml::de::from_str(&xml_content)?;

        // 处理引用文件
        let mut merged_spec = SchemaSpec {
            nodes: HashMap::new(),
            marks: HashMap::new(),
            top_node: xml_schema.top_node.clone(),
        };

        // 收集全局属性
        let mut global_attributes: Vec<GlobalAttributeItem> = Vec::new();

        // 处理当前文件的全局属性
        if let Some(xml_global_attrs) = &xml_schema.global_attributes {
            for xml_global_attr in &xml_global_attrs.global_attributes {
                let global_attr_item = Self::convert_xml_global_attribute_to_item(xml_global_attr.clone())?;
                global_attributes.push(global_attr_item);
            }
        }

        // 更新基础路径为当前文件的目录
        let old_base_path = context.base_path.clone();
        if let Some(parent) = canonical_path.parent() {
            context.base_path = parent.to_path_buf();
        }

        // 处理imports（导入其他schema）
        if let Some(imports) = xml_schema.imports {
            for import in imports.imports {
                let imported_spec = Self::parse_file_with_context(&import.src, context)?;
                Self::merge_schema_spec(&mut merged_spec, imported_spec, false)?;
            }
        }

        // 处理includes（包含其他schema内容）
        if let Some(includes) = xml_schema.includes {
            for include in includes.includes {
                let included_spec = Self::parse_file_with_context(&include.src, context)?;
                Self::merge_schema_spec(&mut merged_spec, included_spec, true)?;
            }
        }

        // 恢复基础路径
        context.base_path = old_base_path;

        // 合并当前文件的定义
        let current_spec = Self::convert_xml_schema_to_spec(XmlSchema {
            top_node: xml_schema.top_node,
            nodes: xml_schema.nodes,
            marks: xml_schema.marks,
        })?;

        Self::merge_schema_spec(&mut merged_spec, current_spec, true)?;

        // 恢复解析深度
        context.current_depth -= 1;

        // 注意：这里返回的是SchemaSpec，不包含global_attributes
        // global_attributes需要在更高层次处理，或者我们需要扩展返回类型
        // 目前先返回基本的SchemaSpec
        Ok(merged_spec)
    }

    /// 从SchemaSpec转换为Extensions
    fn convert_to_extensions_from_spec(schema_spec: SchemaSpec) -> XmlSchemaResult<Vec<Extensions>> {
        let xml_schema = XmlSchema {
            top_node: schema_spec.top_node,
            nodes: Some(XmlNodes {
                nodes: schema_spec.nodes.into_iter().map(|(name, spec)| {
                    XmlNode {
                        name,
                        group: spec.group,
                        desc: spec.desc,
                        content: spec.content,
                        marks: spec.marks,
                        attrs: spec.attrs.map(|attrs| XmlAttrs {
                            attrs: attrs.into_iter().map(|(name, attr_spec)| {
                                XmlAttr {
                                    name,
                                    default: attr_spec.default.map(|v| v.to_string()),
                                }
                            }).collect(),
                        }),
                    }
                }).collect(),
            }),
            marks: Some(XmlMarks {
                marks: schema_spec.marks.into_iter().map(|(name, spec)| {
                    XmlMark {
                        name,
                        group: spec.group,
                        desc: spec.desc,
                        excludes: spec.excludes,
                        spanning: spec.spanning,
                        attrs: spec.attrs.map(|attrs| XmlAttrs {
                            attrs: attrs.into_iter().map(|(name, attr_spec)| {
                                XmlAttr {
                                    name,
                                    default: attr_spec.default.map(|v| v.to_string()),
                                }
                            }).collect(),
                        }),
                    }
                }).collect(),
            }),
        };

        Self::convert_to_extensions(xml_schema)
    }

    /// 合并两个SchemaSpec
    fn merge_schema_spec(
        target: &mut SchemaSpec,
        source: SchemaSpec,
        allow_override: bool,
    ) -> XmlSchemaResult<()> {
        // 合并节点定义
        for (name, node_spec) in source.nodes {
            if target.nodes.contains_key(&name) && !allow_override {
                return Err(XmlSchemaError::DuplicateNodeName(
                    format!("节点 '{}' 已存在，不允许覆盖", name)
                ));
            }
            target.nodes.insert(name, node_spec);
        }

        // 合并标记定义
        for (name, mark_spec) in source.marks {
            if target.marks.contains_key(&name) && !allow_override {
                return Err(XmlSchemaError::DuplicateMarkName(
                    format!("标记 '{}' 已存在，不允许覆盖", name)
                ));
            }
            target.marks.insert(name, mark_spec);
        }

        // 如果目标没有top_node，使用源的top_node
        if target.top_node.is_none() && source.top_node.is_some() {
            target.top_node = source.top_node;
        }

        Ok(())
    }

    /// 将XmlSchema转换为SchemaSpec（不处理引用）
    fn convert_xml_schema_to_spec(xml_schema: XmlSchema) -> XmlSchemaResult<SchemaSpec> {
        Self::convert_to_schema_spec(xml_schema)
    }

    /// 将XML Schema结构转换为Extensions列表
    fn convert_to_extensions(xml_schema: XmlSchema) -> XmlSchemaResult<Vec<Extensions>> {
        let mut extensions = Vec::new();

        // 转换节点定义为Node Extensions
        if let Some(xml_nodes) = xml_schema.nodes {
            for xml_node in xml_nodes.nodes {
                let mut node = Node::create(&xml_node.name, NodeSpec::default());

                // 设置节点属性
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

                // 设置属性
                if let Some(xml_attrs) = xml_node.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    node.set_attrs(attrs);
                }

                extensions.push(Extensions::N(node));
            }
        }

        // 转换标记定义为Mark Extensions
        if let Some(xml_marks) = xml_schema.marks {
            for xml_mark in xml_marks.marks {
                let mut mark = Mark::new(&xml_mark.name, MarkSpec::default());

                // 设置标记属性
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

                // 设置属性
                if let Some(xml_attrs) = xml_mark.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    mark.set_attrs(attrs);
                }

                extensions.push(Extensions::M(mark));
            }
        }

        // 创建一个Extension来包含全局配置
        let extension = Extension::new();

        // 注意：这里的xml_schema是XmlSchema类型，不包含global_attributes
        // global_attributes的处理在多文件解析的parse_file_with_context中进行

        extensions.push(Extensions::E(extension));

        Ok(extensions)
    }

    /// 将支持引用的XML Schema结构转换为Extensions列表
    fn convert_xml_schema_with_refs_to_extensions(xml_schema: XmlSchemaWithReferences) -> XmlSchemaResult<Vec<Extensions>> {
        let mut extensions = Vec::new();

        // 转换节点定义为Node Extensions
        if let Some(xml_nodes) = xml_schema.nodes {
            for xml_node in xml_nodes.nodes {
                let mut node = Node::create(&xml_node.name, NodeSpec::default());

                // 设置节点属性
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

                // 设置属性
                if let Some(xml_attrs) = xml_node.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    node.set_attrs(attrs);
                }

                extensions.push(Extensions::N(node));
            }
        }

        // 转换标记定义为Mark Extensions
        if let Some(xml_marks) = xml_schema.marks {
            for xml_mark in xml_marks.marks {
                let mut mark = Mark::new(&xml_mark.name, MarkSpec::default());

                // 设置标记属性
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

                // 设置属性
                if let Some(xml_attrs) = xml_mark.attrs {
                    let attrs = Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?;
                    mark.set_attrs(attrs);
                }

                extensions.push(Extensions::M(mark));
            }
        }

        // 处理全局属性
        if let Some(xml_global_attrs) = xml_schema.global_attributes {
            let mut extension = Extension::new();
            for xml_global_attr in xml_global_attrs.global_attributes {
                let global_attr_item = Self::convert_xml_global_attribute_to_item(xml_global_attr)?;
                extension.add_global_attribute(global_attr_item);
            }
            extensions.push(Extensions::E(extension));
        } else {
            // 即使没有全局属性，也添加一个空的Extension
            let extension = Extension::new();
            extensions.push(Extensions::E(extension));
        }

        Ok(extensions)
    }

    /// 将XML Schema结构转换为SchemaSpec
    fn convert_to_schema_spec(xml_schema: XmlSchema) -> XmlSchemaResult<SchemaSpec> {
        let mut nodes = HashMap::new();
        let mut marks = HashMap::new();

        // 转换节点定义
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

        // 转换标记定义
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

        Ok(SchemaSpec {
            nodes,
            marks,
            top_node: xml_schema.top_node,
        })
    }

    /// 将XML节点定义转换为NodeSpec
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

    /// 将XML标记定义转换为MarkSpec
    fn convert_xml_mark_to_spec(xml_mark: XmlMark) -> XmlSchemaResult<MarkSpec> {
        let attrs = if let Some(xml_attrs) = xml_mark.attrs {
            Some(Self::convert_xml_attrs_to_spec(xml_attrs.attrs)?)
        } else {
            None
        };

        Ok(MarkSpec {
            attrs,
            excludes: xml_mark.excludes,
            group: xml_mark.group,
            spanning: xml_mark.spanning,
            desc: xml_mark.desc,
        })
    }

    /// 将XML属性定义转换为AttributeSpec映射
    fn convert_xml_attrs_to_spec(
        xml_attrs: Vec<XmlAttr>
    ) -> XmlSchemaResult<HashMap<String, AttributeSpec>> {
        let mut attrs = HashMap::new();
        
        for xml_attr in xml_attrs {
            let default_value = if let Some(default_str) = xml_attr.default {
                Some(Self::parse_attribute_value(&default_str)?)
            } else {
                None
            };

            attrs.insert(
                xml_attr.name.clone(),
                AttributeSpec {
                    default: default_value,
                }
            );
        }

        Ok(attrs)
    }

    /// 解析属性默认值
    fn parse_attribute_value(value_str: &str) -> XmlSchemaResult<Value> {
        // 尝试解析为JSON值
        if let Ok(json_value) = serde_json::from_str::<Value>(value_str) {
            return Ok(json_value);
        }

        // 如果不是有效的JSON，则作为字符串处理
        Ok(Value::String(value_str.to_string()))
    }

    /// 将XML全局属性定义转换为GlobalAttributeItem
    fn convert_xml_global_attribute_to_item(
        xml_global_attr: XmlGlobalAttribute
    ) -> XmlSchemaResult<GlobalAttributeItem> {
        // 解析类型列表
        let types = if xml_global_attr.types.trim() == "*" {
            vec!["*".to_string()]
        } else {
            xml_global_attr.types
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        };

        // 转换属性
        let attributes = Self::convert_xml_attrs_to_spec(xml_global_attr.attrs)?;

        Ok(GlobalAttributeItem {
            types,
            attributes,
        })
    }
}

/// XML Schema 根结构
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "schema")]
struct XmlSchema {
    #[serde(rename = "@top_node")]
    top_node: Option<String>,

    nodes: Option<XmlNodes>,
    marks: Option<XmlMarks>,
}

/// 支持引用的XML Schema 根结构
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "schema")]
struct XmlSchemaWithReferences {
    #[serde(rename = "@top_node")]
    top_node: Option<String>,

    imports: Option<XmlImports>,
    includes: Option<XmlIncludes>,
    global_attributes: Option<XmlGlobalAttributes>,
    nodes: Option<XmlNodes>,
    marks: Option<XmlMarks>,
}

/// XML 导入集合
#[derive(Debug, Deserialize, Serialize)]
struct XmlImports {
    #[serde(rename = "import")]
    imports: Vec<XmlImport>,
}

/// XML 包含集合
#[derive(Debug, Deserialize, Serialize)]
struct XmlIncludes {
    #[serde(rename = "include")]
    includes: Vec<XmlInclude>,
}

/// XML 导入定义
#[derive(Debug, Deserialize, Serialize)]
struct XmlImport {
    #[serde(rename = "@src")]
    src: String,
}

/// XML 包含定义
#[derive(Debug, Deserialize, Serialize)]
struct XmlInclude {
    #[serde(rename = "@src")]
    src: String,
}

/// XML 全局属性集合
#[derive(Debug, Deserialize, Serialize)]
struct XmlGlobalAttributes {
    #[serde(rename = "global_attribute")]
    global_attributes: Vec<XmlGlobalAttribute>,
}

/// XML 全局属性定义
#[derive(Debug, Clone, Deserialize, Serialize)]
struct XmlGlobalAttribute {
    #[serde(rename = "@types")]
    types: String, // 空格分隔的类型列表，如 "paragraph heading" 或 "*"

    #[serde(rename = "attr")]
    attrs: Vec<XmlAttr>,
}

/// XML 节点集合
#[derive(Debug, Deserialize, Serialize)]
struct XmlNodes {
    #[serde(rename = "node")]
    nodes: Vec<XmlNode>,
}

/// XML 标记集合
#[derive(Debug, Deserialize, Serialize)]
struct XmlMarks {
    #[serde(rename = "mark")]
    marks: Vec<XmlMark>,
}

/// XML 节点定义
#[derive(Debug, Deserialize, Serialize)]
struct XmlNode {
    #[serde(rename = "@name")]
    name: String,
    
    #[serde(rename = "@group")]
    group: Option<String>,
    
    desc: Option<String>,
    content: Option<String>,
    marks: Option<String>,
    attrs: Option<XmlAttrs>,
}

/// XML 标记定义
#[derive(Debug, Deserialize, Serialize)]
struct XmlMark {
    #[serde(rename = "@name")]
    name: String,
    
    #[serde(rename = "@group")]
    group: Option<String>,
    
    desc: Option<String>,
    excludes: Option<String>,
    spanning: Option<bool>,
    attrs: Option<XmlAttrs>,
}

/// XML 属性集合
#[derive(Debug, Deserialize, Serialize)]
struct XmlAttrs {
    #[serde(rename = "attr")]
    attrs: Vec<XmlAttr>,
}

/// XML 属性定义
#[derive(Debug, Clone, Deserialize, Serialize)]
struct XmlAttr {
    #[serde(rename = "@name")]
    name: String,
    
    #[serde(rename = "@default")]
    default: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_schema() {
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

        let result = XmlSchemaParser::parse_from_str(xml);
        assert!(result.is_ok());

        let schema_spec = result.unwrap();
        assert_eq!(schema_spec.top_node, Some("doc".to_string()));
        assert_eq!(schema_spec.nodes.len(), 3);
        assert!(schema_spec.nodes.contains_key("doc"));
        assert!(schema_spec.nodes.contains_key("paragraph"));
        assert!(schema_spec.nodes.contains_key("text"));
    }

    #[test]
    fn test_parse_schema_with_attributes_and_marks() {
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
              <marks>strong em</marks>
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
              <excludes>strong</excludes>
            </mark>
          </marks>
        </schema>
        "#;

        let result = XmlSchemaParser::parse_from_str(xml);
        assert!(result.is_ok());

        let schema_spec = result.unwrap();

        // 验证基本结构
        assert_eq!(schema_spec.top_node, Some("doc".to_string()));
        assert_eq!(schema_spec.nodes.len(), 3);
        assert_eq!(schema_spec.marks.len(), 2);

        // 验证节点属性
        let doc_node = schema_spec.nodes.get("doc").unwrap();
        assert_eq!(doc_node.group, Some("block".to_string()));
        assert_eq!(doc_node.content, Some("paragraph+".to_string()));
        assert_eq!(doc_node.marks, Some("_".to_string()));
        assert!(doc_node.attrs.is_some());

        let doc_attrs = doc_node.attrs.as_ref().unwrap();
        assert_eq!(doc_attrs.len(), 2);
        assert!(doc_attrs.contains_key("title"));
        assert!(doc_attrs.contains_key("version"));

        // 验证标记定义
        let strong_mark = schema_spec.marks.get("strong").unwrap();
        assert_eq!(strong_mark.group, Some("formatting".to_string()));
        assert_eq!(strong_mark.spanning, Some(true));
        assert!(strong_mark.attrs.is_some());

        let em_mark = schema_spec.marks.get("em").unwrap();
        assert_eq!(em_mark.excludes, Some("strong".to_string()));
    }

    #[test]
    fn test_parse_attribute_values() {
        // 测试不同类型的属性默认值解析
        let result = XmlSchemaParser::parse_attribute_value("\"hello world\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("hello world".to_string()));

        let result = XmlSchemaParser::parse_attribute_value("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Number(serde_json::Number::from(42)));

        let result = XmlSchemaParser::parse_attribute_value("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));

        // 非JSON格式的字符串
        let result = XmlSchemaParser::parse_attribute_value("plain text");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("plain text".to_string()));
    }

    #[test]
    fn test_duplicate_node_error() {
        let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <nodes>
            <node name="duplicate">
              <desc>第一个节点</desc>
            </node>
            <node name="duplicate">
              <desc>重复的节点</desc>
            </node>
          </nodes>
        </schema>
        "#;

        let result = XmlSchemaParser::parse_from_str(xml);
        assert!(result.is_err());

        if let Err(XmlSchemaError::DuplicateNodeName(name)) = result {
            assert_eq!(name, "duplicate");
        } else {
            panic!("Expected DuplicateNodeName error");
        }
    }

    #[test]
    fn test_integration_with_schema_compilation() {
        // 测试XML解析后能否成功编译为Schema
        let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema top_node="doc">
          <nodes>
            <node name="doc" group="block">
              <desc>文档根节点</desc>
              <content>paragraph+</content>
              <marks>_</marks>
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

        // 解析XML为SchemaSpec
        let schema_spec = XmlSchemaParser::parse_from_str(xml).unwrap();

        // 编译为Schema
        use mf_model::schema::Schema;
        let schema_result = Schema::compile(schema_spec);
        assert!(schema_result.is_ok());

        let schema = schema_result.unwrap();
        assert_eq!(schema.nodes.len(), 3);
        assert_eq!(schema.marks.len(), 1);
        assert!(schema.top_node_type.is_some());

        // 验证编译后的节点类型
        let doc_type = schema.nodes.get("doc").unwrap();
        assert_eq!(doc_type.name, "doc");
        assert!(doc_type.content_match.is_some());

        let paragraph_type = schema.nodes.get("paragraph").unwrap();
        assert_eq!(paragraph_type.name, "paragraph");

        let text_type = schema.nodes.get("text").unwrap();
        assert_eq!(text_type.name, "text");

        // 验证标记类型
        let strong_mark = schema.marks.get("strong").unwrap();
        assert_eq!(strong_mark.name, "strong");
    }

    #[test]
    fn test_parse_to_extensions() {
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
            <mark name="strong" group="formatting">
              <desc>粗体标记</desc>
              <spanning>true</spanning>
              <attrs>
                <attr name="weight" default="bold"/>
              </attrs>
            </mark>
          </marks>
        </schema>
        "#;

        let result = XmlSchemaParser::parse_to_extensions(xml);
        assert!(result.is_ok());

        let extensions = result.unwrap();

        // 应该有3个节点 + 1个标记 + 1个Extension = 5个扩展
        assert_eq!(extensions.len(), 5);

        // 验证节点扩展
        let mut node_count = 0;
        let mut mark_count = 0;
        let mut extension_count = 0;

        for ext in &extensions {
            match ext {
                Extensions::N(node) => {
                    node_count += 1;
                    match node.get_name() {
                        "doc" => {
                            assert_eq!(node.r#type.group, Some("block".to_string()));
                            assert_eq!(node.r#type.content, Some("paragraph+".to_string()));
                            assert_eq!(node.r#type.marks, Some("_".to_string()));
                            assert!(node.r#type.attrs.is_some());
                        },
                        "paragraph" => {
                            assert_eq!(node.r#type.group, Some("block".to_string()));
                            assert_eq!(node.r#type.content, Some("text*".to_string()));
                            assert_eq!(node.r#type.marks, Some("strong".to_string()));
                        },
                        "text" => {
                            assert_eq!(node.r#type.desc, Some("文本节点".to_string()));
                        },
                        _ => panic!("Unexpected node name: {}", node.get_name()),
                    }
                },
                Extensions::M(mark) => {
                    mark_count += 1;
                    assert_eq!(mark.get_name(), "strong");
                    assert_eq!(mark.r#type.group, Some("formatting".to_string()));
                    assert_eq!(mark.r#type.spanning, Some(true));
                    assert!(mark.r#type.attrs.is_some());
                },
                Extensions::E(_) => {
                    extension_count += 1;
                },
            }
        }

        assert_eq!(node_count, 3);
        assert_eq!(mark_count, 1);
        assert_eq!(extension_count, 1);
    }

    #[test]
    fn test_multi_file_parsing() {
        // 创建临时目录和文件进行测试
        let temp_dir = std::env::temp_dir().join("xml_schema_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // 创建基础节点文件
        let base_nodes_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
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
        </schema>
        "#;

        let base_nodes_path = temp_dir.join("base-nodes.xml");
        std::fs::write(&base_nodes_path, base_nodes_content).unwrap();

        // 创建标记文件
        let marks_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <marks>
            <mark name="strong" group="formatting">
              <desc>粗体标记</desc>
              <spanning>true</spanning>
            </mark>
          </marks>
        </schema>
        "#;

        let marks_path = temp_dir.join("marks.xml");
        std::fs::write(&marks_path, marks_content).unwrap();

        // 创建主文件
        let main_content = format!(r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema top_node="doc">
          <imports>
            <import src="{}"/>
            <import src="{}"/>
          </imports>
          <nodes>
            <node name="custom" group="block">
              <desc>自定义节点</desc>
            </node>
          </nodes>
        </schema>
        "#,
        base_nodes_path.file_name().unwrap().to_str().unwrap(),
        marks_path.file_name().unwrap().to_str().unwrap());

        let main_path = temp_dir.join("main.xml");
        std::fs::write(&main_path, main_content).unwrap();

        // 测试多文件解析
        let result = XmlSchemaParser::parse_multi_file(main_path.to_str().unwrap());
        assert!(result.is_ok());

        let schema_spec = result.unwrap();
        assert_eq!(schema_spec.top_node, Some("doc".to_string()));
        assert_eq!(schema_spec.nodes.len(), 4); // doc, paragraph, text, custom
        assert_eq!(schema_spec.marks.len(), 1); // strong

        // 验证节点存在
        assert!(schema_spec.nodes.contains_key("doc"));
        assert!(schema_spec.nodes.contains_key("paragraph"));
        assert!(schema_spec.nodes.contains_key("text"));
        assert!(schema_spec.nodes.contains_key("custom"));

        // 验证标记存在
        assert!(schema_spec.marks.contains_key("strong"));

        // 清理临时文件
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_circular_reference_detection() {
        let temp_dir = std::env::temp_dir().join("xml_schema_circular_test");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // 创建循环引用的文件
        let file_a_content = format!(r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <imports>
            <import src="{}"/>
          </imports>
          <nodes>
            <node name="node_a">
              <desc>节点A</desc>
            </node>
          </nodes>
        </schema>
        "#, "file-b.xml");

        let file_b_content = format!(r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <imports>
            <import src="{}"/>
          </imports>
          <nodes>
            <node name="node_b">
              <desc>节点B</desc>
            </node>
          </nodes>
        </schema>
        "#, "file-a.xml");

        let file_a_path = temp_dir.join("file-a.xml");
        let file_b_path = temp_dir.join("file-b.xml");

        std::fs::write(&file_a_path, file_a_content).unwrap();
        std::fs::write(&file_b_path, file_b_content).unwrap();

        // 测试循环引用检测
        let result = XmlSchemaParser::parse_multi_file(file_a_path.to_str().unwrap());
        assert!(result.is_err());

        if let Err(XmlSchemaError::CircularReference(_)) = result {
            // 正确检测到循环引用
        } else {
            panic!("应该检测到循环引用错误");
        }

        // 清理临时文件
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_global_attributes_parsing() {
        let xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema top_node="doc">
          <global_attributes>
            <global_attribute types="paragraph heading">
              <attr name="id"/>
              <attr name="class"/>
              <attr name="style"/>
            </global_attribute>
            <global_attribute types="*">
              <attr name="data-custom" default="default-value"/>
            </global_attribute>
          </global_attributes>
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

        let result = XmlSchemaParser::parse_to_extensions(xml);
        assert!(result.is_ok());

        let extensions = result.unwrap();

        // 查找Extension类型的扩展
        let mut found_extension = false;
        for ext in &extensions {
            if let Extensions::E(extension) = ext {
                let global_attrs = extension.get_global_attributes();
                if !global_attrs.is_empty() {
                    found_extension = true;

                    // 验证全局属性
                    assert_eq!(global_attrs.len(), 2);

                    // 验证第一个全局属性（针对paragraph和heading）
                    let first_attr = &global_attrs[0];
                    assert_eq!(first_attr.types, vec!["paragraph", "heading"]);
                    assert_eq!(first_attr.attributes.len(), 3);
                    assert!(first_attr.attributes.contains_key("id"));
                    assert!(first_attr.attributes.contains_key("class"));
                    assert!(first_attr.attributes.contains_key("style"));

                    // 验证第二个全局属性（针对所有类型）
                    let second_attr = &global_attrs[1];
                    assert_eq!(second_attr.types, vec!["*"]);
                    assert_eq!(second_attr.attributes.len(), 1);
                    assert!(second_attr.attributes.contains_key("data-custom"));

                    // 验证默认值
                    let data_custom_attr = &second_attr.attributes["data-custom"];
                    assert!(data_custom_attr.default.is_some());
                    break;
                }
            }
        }

        assert!(found_extension, "应该找到包含全局属性的Extension");
    }
}

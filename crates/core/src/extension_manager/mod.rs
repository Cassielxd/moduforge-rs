use std::sync::Arc;
use std::time::Instant;

use mf_model::schema::Schema;
use mf_state::plugin::Plugin;

use crate::{
    helpers::get_schema_by_resolved_extensions::get_schema_by_resolved_extensions,
    metrics, types::Extensions, ForgeResult, XmlSchemaParser, extension::OpFn,
};
/// 扩展管理器
pub struct ExtensionManager {
    plugins: Vec<Arc<Plugin>>,
    schema: Arc<Schema>,
    op_fns: OpFn,
}
/// ExtensionManager构建器
///
/// 提供灵活的方式来配置和创建ExtensionManager，支持多种扩展来源
#[derive(Default)]
pub struct ExtensionManagerBuilder {
    extensions: Vec<Extensions>,
    xml_files: Vec<String>,
    xml_contents: Vec<String>,
}

impl ExtensionManagerBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加代码定义的扩展
    ///
    /// # 参数
    /// * `extension` - 要添加的扩展
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::{ExtensionManagerBuilder, types::Extensions, node::Node};
    /// use mf_model::node_definition::NodeSpec;
    ///
    /// let node = Node::create("custom_node", NodeSpec::default());
    /// let manager = ExtensionManagerBuilder::new()
    ///     .add_extension(Extensions::N(node))
    ///     .build()?;
    /// ```
    pub fn add_extension(
        mut self,
        extension: Extensions,
    ) -> Self {
        self.extensions.push(extension);
        self
    }

    /// 批量添加代码定义的扩展
    ///
    /// # 参数
    /// * `extensions` - 要添加的扩展列表
    pub fn add_extensions(
        mut self,
        extensions: Vec<Extensions>,
    ) -> Self {
        self.extensions.extend(extensions);
        self
    }

    /// 添加XML文件路径
    ///
    /// # 参数
    /// * `xml_file_path` - XML schema文件路径
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ExtensionManagerBuilder;
    ///
    /// let manager = ExtensionManagerBuilder::new()
    ///     .add_xml_file("./schemas/base-nodes.xml")
    ///     .add_xml_file("./schemas/custom-marks.xml")
    ///     .build()?;
    /// ```
    pub fn add_xml_file<P: AsRef<str>>(
        mut self,
        xml_file_path: P,
    ) -> Self {
        self.xml_files.push(xml_file_path.as_ref().to_string());
        self
    }

    /// 批量添加XML文件路径
    ///
    /// # 参数
    /// * `xml_file_paths` - XML schema文件路径列表
    pub fn add_xml_files<P: AsRef<str>>(
        mut self,
        xml_file_paths: &[P],
    ) -> Self {
        for path in xml_file_paths {
            self.xml_files.push(path.as_ref().to_string());
        }
        self
    }

    /// 添加XML内容字符串
    ///
    /// # 参数
    /// * `xml_content` - XML schema内容
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ExtensionManagerBuilder;
    ///
    /// let xml = r#"
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <schema>
    ///   <nodes>
    ///     <node name="custom">
    ///       <desc>自定义节点</desc>
    ///     </node>
    ///   </nodes>
    /// </schema>
    /// "#;
    ///
    /// let manager = ExtensionManagerBuilder::new()
    ///     .add_xml_content(xml)
    ///     .build()?;
    /// ```
    pub fn add_xml_content<S: AsRef<str>>(
        mut self,
        xml_content: S,
    ) -> Self {
        self.xml_contents.push(xml_content.as_ref().to_string());
        self
    }

    /// 批量添加XML内容字符串
    ///
    /// # 参数
    /// * `xml_contents` - XML schema内容列表
    pub fn add_xml_contents<S: AsRef<str>>(
        mut self,
        xml_contents: &[S],
    ) -> Self {
        for content in xml_contents {
            self.xml_contents.push(content.as_ref().to_string());
        }
        self
    }

    /// 构建ExtensionManager
    ///
    /// # 返回值
    /// * `ForgeResult<ExtensionManager>` - 构建的ExtensionManager实例或错误
    pub fn build(self) -> ForgeResult<ExtensionManager> {
        let start_time = Instant::now();
        let mut all_extensions = self.extensions;

        // 解析XML文件
        for xml_file in &self.xml_files {
            let extensions =
                XmlSchemaParser::parse_multi_file_to_extensions(xml_file)
                    .map_err(|e| {
                        crate::error::error_utils::config_error(format!(
                            "解析XML文件 {xml_file} 失败: {e}"
                        ))
                    })?;
            all_extensions.extend(extensions);
        }

        // 解析XML内容
        for xml_content in &self.xml_contents {
            let extensions = XmlSchemaParser::parse_to_extensions(xml_content)
                .map_err(|e| {
                    crate::error::error_utils::config_error(format!(
                        "解析XML内容失败: {e}"
                    ))
                })?;
            all_extensions.extend(extensions);
        }

        metrics::xml_parsing_duration(start_time.elapsed());

        // 创建ExtensionManager
        ExtensionManager::new(&all_extensions)
    }
}

impl ExtensionManager {
    /// 创建ExtensionManager构建器
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ExtensionManager;
    ///
    /// let manager = ExtensionManager::builder()
    ///     .add_xml_file("./schemas/main.xml")
    ///     .build()?;
    /// ```
    pub fn builder() -> ExtensionManagerBuilder {
        ExtensionManagerBuilder::new()
    }

    pub fn new(extensions: &Vec<Extensions>) -> ForgeResult<Self> {
        let start_time = Instant::now();
        let schema = Arc::new(get_schema_by_resolved_extensions(extensions)?);
        let mut plugins = vec![];
        let mut op_fns = vec![];
        let mut extension_count = 0;
        let mut plugin_count = 0;
        for extension in extensions {
            if let Extensions::E(extension) = extension {
                extension_count += 1;
                for plugin in extension.get_plugins() {
                    plugin_count += 1;
                    plugins.push(plugin.clone());
                }
                for op_fn in extension.get_op_fns() {
                    op_fns.push(op_fn.clone());
                }
            }
        }

        metrics::extensions_loaded(extension_count);
        metrics::plugins_loaded(plugin_count);
        metrics::extension_manager_creation_duration(start_time.elapsed());

        Ok(ExtensionManager { schema, plugins, op_fns })
    }

    /// 从XML文件创建ExtensionManager（便捷方法）
    ///
    /// # 参数
    /// * `xml_file_path` - XML schema文件路径
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 返回ExtensionManager实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ExtensionManager;
    ///
    /// let manager = ExtensionManager::from_xml_file("./schemas/main.xml")?;
    /// ```
    pub fn from_xml_file(xml_file_path: &str) -> ForgeResult<Self> {
        Self::builder().add_xml_file(xml_file_path).build()
    }

    /// 从XML字符串创建ExtensionManager（便捷方法）
    ///
    /// # 参数
    /// * `xml_content` - XML schema内容字符串
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 返回ExtensionManager实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ExtensionManager;
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
    /// let manager = ExtensionManager::from_xml_string(xml)?;
    /// ```
    pub fn from_xml_string(xml_content: &str) -> ForgeResult<Self> {
        Self::builder().add_xml_content(xml_content).build()
    }

    /// 从多个XML文件创建ExtensionManager（便捷方法）
    ///
    /// # 参数
    /// * `xml_file_paths` - XML schema文件路径列表
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 返回ExtensionManager实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::ExtensionManager;
    ///
    /// let files = vec![
    ///     "./schemas/base-nodes.xml",
    ///     "./schemas/formatting-marks.xml",
    ///     "./schemas/custom-extensions.xml"
    /// ];
    ///
    /// let manager = ExtensionManager::from_xml_files(&files)?;
    /// ```
    pub fn from_xml_files(xml_file_paths: &[&str]) -> ForgeResult<Self> {
        Self::builder().add_xml_files(xml_file_paths).build()
    }

    /// 从Extensions和XML文件混合创建ExtensionManager（便捷方法）
    ///
    /// # 参数
    /// * `extensions` - 已有的Extensions列表
    /// * `xml_file_paths` - 额外的XML schema文件路径列表
    ///
    /// # 返回值
    /// * `ForgeResult<Self>` - 返回ExtensionManager实例或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::{ExtensionManager, types::Extensions};
    ///
    /// let existing_extensions = vec![/* ... */];
    /// let xml_files = vec!["./schemas/additional.xml"];
    ///
    /// let manager = ExtensionManager::from_mixed_sources(
    ///     &existing_extensions,
    ///     &xml_files
    /// )?;
    /// ```
    pub fn from_mixed_sources(
        extensions: &[Extensions],
        xml_file_paths: &[&str],
    ) -> ForgeResult<Self> {
        Self::builder()
            .add_extensions(extensions.to_vec())
            .add_xml_files(xml_file_paths)
            .build()
    }
    pub fn get_op_fns(&self) -> &OpFn {
        &self.op_fns
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
    pub fn get_plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.plugins
    }

    /// 添加从快照恢复的插件
    pub fn add_restored_plugins(
        &mut self,
        plugins: Vec<std::sync::Arc<mf_state::plugin::Plugin>>,
    ) -> ForgeResult<()> {
        self.plugins.extend(plugins);
        Ok(())
    }

    /// 转换为泛型版本
    ///
    /// 将具体的 ExtensionManager 转换为 ExtensionManagerGeneric<NodePool, Schema>
    pub fn to_generic(
        &self,
    ) -> crate::generic::ExtensionManagerGeneric<
        mf_model::node_pool::NodePool,
        mf_model::schema::Schema,
    > {
        // 由于 OpFn 已经是 OpFnGeneric<NodePool, Schema> 的类型别名，直接克隆即可
        crate::generic::ExtensionManagerGeneric::new(
            self.plugins.clone(),
            self.schema.clone(),
            self.op_fns.clone(),
        )
    }
}

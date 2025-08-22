use std::sync::Arc;
use std::time::Instant;

use mf_model::schema::Schema;
use mf_state::{ops::GlobalResourceManager, plugin::Plugin};

use crate::{
    helpers::get_schema_by_resolved_extensions::get_schema_by_resolved_extensions,
    metrics, types::Extensions, ForgeResult, XmlSchemaParser,
};
/// 扩展管理器
pub struct ExtensionManager {
    plugins: Vec<Arc<Plugin>>,
    schema: Arc<Schema>,
    op_fns: Vec<
        Arc<dyn Fn(&GlobalResourceManager) -> ForgeResult<()> + Send + Sync>,
    >,
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
    /// use mf_model::node_type::NodeSpec;
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
                            "解析XML文件 {} 失败: {}",
                            xml_file, e
                        ))
                    })?;
            all_extensions.extend(extensions);
        }

        // 解析XML内容
        for xml_content in &self.xml_contents {
            let extensions = XmlSchemaParser::parse_to_extensions(xml_content)
                .map_err(|e| {
                    crate::error::error_utils::config_error(format!(
                        "解析XML内容失败: {}",
                        e
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
    pub fn get_op_fns(
        &self
    ) -> &Vec<
        Arc<dyn Fn(&GlobalResourceManager) -> ForgeResult<()> + Send + Sync>,
    > {
        &self.op_fns
    }

    pub fn get_schema(&self) -> Arc<Schema> {
        self.schema.clone()
    }
    pub fn get_plugins(&self) -> &Vec<Arc<Plugin>> {
        &self.plugins
    }

    /// 动态添加扩展并重新构建schema
    ///
    /// # 参数
    /// * `new_extensions` - 要添加的新扩展列表
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    ///
    /// # 示例
    /// ```rust
    /// use mf_core::{ExtensionManager, types::Extensions, node::Node};
    /// use mf_model::node_type::NodeSpec;
    ///
    /// let mut manager = ExtensionManager::new(&vec![])?;
    ///
    /// let new_node = Node::create("dynamic_node", NodeSpec::default());
    /// manager.add_extensions(vec![Extensions::N(new_node)])?;
    /// ```
    pub fn add_extensions(
        &mut self,
        new_extensions: Vec<Extensions>,
    ) -> ForgeResult<()> {
        // 获取当前所有扩展
        let mut all_extensions = Vec::new();

        // 从当前schema重建扩展列表（这是一个简化的实现）
        // 在实际应用中，你可能需要保存原始的扩展列表
        for (name, node_type) in &self.schema.nodes {
            let node = crate::node::Node::create(name, node_type.spec.clone());
            all_extensions.push(Extensions::N(node));
        }

        for (name, mark_type) in &self.schema.marks {
            let mark = crate::mark::Mark::new(name, mark_type.spec.clone());
            all_extensions.push(Extensions::M(mark));
        }

        // 添加新扩展
        all_extensions.extend(new_extensions);

        // 重新构建ExtensionManager
        let new_manager = Self::new(&all_extensions)?;

        // 更新当前实例
        self.schema = new_manager.schema;
        self.plugins = new_manager.plugins;
        self.op_fns = new_manager.op_fns;

        Ok(())
    }

    /// 动态添加XML文件扩展
    ///
    /// # 参数
    /// * `xml_file_path` - XML schema文件路径
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub fn add_xml_file(
        &mut self,
        xml_file_path: &str,
    ) -> ForgeResult<()> {
        let extensions =
            XmlSchemaParser::parse_multi_file_to_extensions(xml_file_path)
                .map_err(|e| {
                    crate::error::error_utils::config_error(format!(
                        "解析XML文件 {} 失败: {}",
                        xml_file_path, e
                    ))
                })?;

        self.add_extensions(extensions)
    }

    /// 添加从快照恢复的插件
    pub fn add_restored_plugins(
        &mut self,
        plugins: Vec<std::sync::Arc<mf_state::plugin::Plugin>>,
    ) -> ForgeResult<()> {
        self.plugins.extend(plugins);
        Ok(())
    }

    /// 动态添加XML内容扩展
    ///
    /// # 参数
    /// * `xml_content` - XML schema内容
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub fn add_xml_content(
        &mut self,
        xml_content: &str,
    ) -> ForgeResult<()> {
        let extensions = XmlSchemaParser::parse_to_extensions(xml_content)
            .map_err(|e| {
                crate::error::error_utils::config_error(format!(
                    "解析XML内容失败: {}",
                    e
                ))
            })?;

        self.add_extensions(extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_manager_from_xml_string() {
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

        let result = ExtensionManager::from_xml_string(xml);
        assert!(result.is_ok());

        let manager = result.unwrap();
        let schema = manager.get_schema();

        // 验证schema结构
        assert_eq!(schema.nodes.len(), 3);
        assert_eq!(schema.marks.len(), 1);
        assert!(schema.nodes.contains_key("doc"));
        assert!(schema.nodes.contains_key("paragraph"));
        assert!(schema.nodes.contains_key("text"));
        assert!(schema.marks.contains_key("strong"));
    }

    #[test]
    fn test_extension_manager_from_xml_files() {
        // 创建临时目录和文件进行测试
        let temp_dir = std::env::temp_dir().join("extension_manager_test");
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
            <mark name="em" group="formatting">
              <desc>斜体标记</desc>
              <spanning>true</spanning>
            </mark>
          </marks>
        </schema>
        "#;

        let marks_path = temp_dir.join("marks.xml");
        std::fs::write(&marks_path, marks_content).unwrap();

        // 测试从多个文件创建ExtensionManager
        let files = vec![
            base_nodes_path.to_str().unwrap(),
            marks_path.to_str().unwrap(),
        ];

        let result = ExtensionManager::from_xml_files(&files);
        assert!(result.is_ok());

        let manager = result.unwrap();
        let schema = manager.get_schema();

        // 验证合并后的schema
        assert_eq!(schema.nodes.len(), 3); // doc, paragraph, text
        assert_eq!(schema.marks.len(), 2); // strong, em

        // 清理临时文件
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_extension_manager_builder_mixed_sources() {
        use crate::node::Node;
        use crate::mark::Mark;
        use mf_model::node_type::NodeSpec;
        use mf_model::mark_type::MarkSpec;

        // 创建代码定义的扩展
        let code_node = Node::create("code_node", NodeSpec::default());
        let code_mark = Mark::new("code_mark", MarkSpec::default());

        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <nodes>
            <node name="xml_node" group="block">
              <desc>XML定义的节点</desc>
            </node>
          </nodes>
          <marks>
            <mark name="xml_mark" group="formatting">
              <desc>XML定义的标记</desc>
            </mark>
          </marks>
        </schema>
        "#;

        // 使用Builder模式混合不同来源的扩展
        let result = ExtensionManager::builder()
            .add_extension(Extensions::N(code_node))
            .add_extension(Extensions::M(code_mark))
            .add_xml_content(xml_content)
            .build();

        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());

        let manager = result.unwrap();
        let schema = manager.get_schema();

        // 验证混合后的schema
        assert_eq!(schema.nodes.len(), 2); // code_node, xml_node
        assert_eq!(schema.marks.len(), 2); // code_mark, xml_mark

        assert!(schema.nodes.contains_key("code_node"));
        assert!(schema.nodes.contains_key("xml_node"));
        assert!(schema.marks.contains_key("code_mark"));
        assert!(schema.marks.contains_key("xml_mark"));
    }

    #[test]
    fn test_dynamic_extension_addition() {
        use crate::node::Node;
        use mf_model::node_type::NodeSpec;

        // 创建初始的ExtensionManager
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <nodes>
            <node name="initial_node">
              <desc>初始节点</desc>
            </node>
          </nodes>
        </schema>
        "#;

        let mut manager =
            ExtensionManager::from_xml_string(xml_content).unwrap();

        // 验证初始状态
        assert_eq!(manager.get_schema().nodes.len(), 1);
        assert!(manager.get_schema().nodes.contains_key("initial_node"));

        // 动态添加新的扩展
        let new_node = Node::create("dynamic_node", NodeSpec::default());
        let result = manager.add_extensions(vec![Extensions::N(new_node)]);
        assert!(result.is_ok());

        // 验证添加后的状态
        assert_eq!(manager.get_schema().nodes.len(), 2);
        assert!(manager.get_schema().nodes.contains_key("initial_node"));
        assert!(manager.get_schema().nodes.contains_key("dynamic_node"));

        // 动态添加XML内容
        let additional_xml = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <nodes>
            <node name="xml_dynamic_node">
              <desc>动态添加的XML节点</desc>
            </node>
          </nodes>
        </schema>
        "#;

        let result = manager.add_xml_content(additional_xml);
        assert!(result.is_ok());

        // 验证最终状态
        assert_eq!(manager.get_schema().nodes.len(), 3);
        assert!(manager.get_schema().nodes.contains_key("initial_node"));
        assert!(manager.get_schema().nodes.contains_key("dynamic_node"));
        assert!(manager.get_schema().nodes.contains_key("xml_dynamic_node"));
    }
}

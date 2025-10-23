//! 运行时公共辅助函数
//!
//! 此模块提供了多个运行时实现之间共享的公共功能，包括：
//! - 扩展管理器的创建和合并逻辑
//! - 中间件执行辅助函数
//!
//! 这些函数被以下运行时使用：
//! - ForgeRuntime (runtime.rs)
//! - ForgeActorRuntime (actor_runtime.rs)
//! - ForgeAsyncRuntime (async_runtime.rs)

use crate::{
    config::ForgeConfig, debug::debug, extension_manager::ExtensionManager,
    types::RuntimeOptions, ForgeResult,
};

/// 扩展管理器辅助函数
///
/// 提供扩展管理器的创建和合并功能，确保所有运行时使用相同的逻辑
pub struct ExtensionManagerHelper;

impl ExtensionManagerHelper {
    /// 创建扩展管理器 - 自动处理XML schema配置并合并代码扩展
    ///
    /// 优先级顺序：
    /// 1. 使用 `config.extension.xml_schema_paths` 中配置的路径
    /// 2. 如果没有配置，尝试加载默认的 `schema/main.xml`
    /// 3. 如果都没有，使用代码定义的扩展
    ///
    /// # 参数
    /// * `runtime_options` - 运行时选项（包含代码定义的扩展）
    /// * `forge_config` - Forge配置（包含XML路径配置）
    ///
    /// # 返回值
    /// * `ForgeResult<ExtensionManager>` - 扩展管理器实例或错误
    pub fn create_extension_manager(
        runtime_options: &RuntimeOptions,
        forge_config: &ForgeConfig,
    ) -> ForgeResult<ExtensionManager> {
        // 检查是否有配置的XML schema路径
        if !forge_config.extension.xml_schema_paths.is_empty() {
            debug!(
                "使用配置的XML schema路径: {:?}",
                forge_config.extension.xml_schema_paths
            );

            // 转换为字符串引用
            let paths: Vec<&str> = forge_config
                .extension
                .xml_schema_paths
                .iter()
                .map(|s| s.as_str())
                .collect();
            let extension_manager = ExtensionManager::from_xml_files(&paths)?;

            // 合并现有的扩展
            let merged_extensions = Self::merge_extensions_with_xml(
                runtime_options,
                extension_manager,
            )?;
            return Ok(merged_extensions);
        }

        // 检查默认的 schema/main.xml 文件
        let default_schema_path = "schema/main.xml";
        if std::path::Path::new(default_schema_path).exists() {
            debug!("使用默认的 schema 文件: {}", default_schema_path);
            let extension_manager =
                ExtensionManager::from_xml_file(default_schema_path)?;
            let merged_extensions = Self::merge_extensions_with_xml(
                runtime_options,
                extension_manager,
            )?;
            return Ok(merged_extensions);
        }

        // 没有找到任何XML schema，使用默认配置
        debug!("未找到XML schema配置，使用默认扩展");
        ExtensionManager::new(&runtime_options.get_extensions())
    }

    /// 合并XML扩展和代码扩展
    ///
    /// 合并策略：
    /// 1. XML扩展优先（节点和标记定义）
    /// 2. 代码扩展补充（Extension类型，包含插件）
    /// 3. 避免重复添加相同名称的节点/标记
    ///
    /// # 参数
    /// * `runtime_options` - 运行时选项（包含代码定义的扩展）
    /// * `xml_extension_manager` - 从XML加载的扩展管理器
    ///
    /// # 返回值
    /// * `ForgeResult<ExtensionManager>` - 合并后的扩展管理器
    pub fn merge_extensions_with_xml(
        runtime_options: &RuntimeOptions,
        xml_extension_manager: ExtensionManager,
    ) -> ForgeResult<ExtensionManager> {
        let schema = xml_extension_manager.get_schema();
        let mut all_extensions = Vec::new();
        let factory = schema.factory();
        let (nodes, marks) = factory.definitions();
        // 先添加XML扩展（优先级更高）
        for (name, node_type) in nodes {
            let node = crate::node::Node::create(name, node_type.spec.clone());
            all_extensions.push(crate::types::Extensions::N(node));
        }

        for (name, mark_type) in marks {
            let mark = crate::mark::Mark::new(name, mark_type.spec.clone());
            all_extensions.push(crate::types::Extensions::M(mark));
        }

        // 再添加代码扩展（避免重复）
        for ext in runtime_options.get_extensions() {
            let name = match &ext {
                crate::types::Extensions::N(node) => &node.name,
                crate::types::Extensions::M(mark) => &mark.name,
                crate::types::Extensions::E(_) => {
                    // 直接添加Extension扩展（包含插件），不需要检查重复
                    all_extensions.push(ext);
                    continue;
                },
            };

            // 检查是否已经存在
            let exists = match &ext {
                crate::types::Extensions::N(_) => nodes.contains_key(name),
                crate::types::Extensions::M(_) => marks.contains_key(name),
                crate::types::Extensions::E(_) => false, // Extension扩展总是添加
            };

            if !exists {
                all_extensions.push(ext);
            }
        }

        ExtensionManager::new(&all_extensions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_manager_helper_creation() {
        // 基本创建测试
        let options = RuntimeOptions::default();
        let config = ForgeConfig::default();

        // 没有XML文件时应该使用代码扩展
        let result =
            ExtensionManagerHelper::create_extension_manager(&options, &config);
        assert!(result.is_ok());
    }
}

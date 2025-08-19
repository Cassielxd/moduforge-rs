use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::{config::ForgeConfig, extension_manager::ExtensionManager, ForgeResult};

/// 核心快照数据结构
///
/// 包含运行时需要的所有预编译数据，避免重复初始化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSnapshot {
    /// 快照版本，用于兼容性检查
    pub version: String,
    /// 快照创建时间戳
    pub created_at: u64,
    /// 预编译的配置
    pub config: ForgeConfig,
    /// 序列化的 Schema (使用 JSON 格式)
    pub schema_json: String,
    /// 扩展映射表（名称到类型的映射）
    pub extension_registry: HashMap<String, ExtensionType>,
    /// XML Schema 缓存
    pub xml_schema_cache: HashMap<String, String>,
    /// 预计算的哈希值（用于快速验证）
    pub content_hash: u64,
}

/// 扩展类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtensionType {
    Node { group: Option<String>, content_spec: String },
    Mark { group: Option<String>, spanning: bool },
}

/// 可序列化的 Schema 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSchema {
    pub nodes: HashMap<String, SerializableNodeType>,
    pub marks: HashMap<String, SerializableMarkType>,
    pub top_node: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableNodeType {
    pub group: Option<String>,
    pub content_spec: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMarkType {
    pub group: Option<String>,
    pub spanning: bool,
}

impl SerializableSchema {
    pub fn from_schema(schema: &mf_model::schema::Schema) -> Self {
        let mut nodes = HashMap::new();
        let mut marks = HashMap::new();

        for (name, node_type) in &schema.nodes {
            nodes.insert(
                name.clone(),
                SerializableNodeType {
                    group: node_type.spec.group.clone(),
                    content_spec: format!("{:?}", node_type.spec.content),
                },
            );
        }

        for (name, mark_type) in &schema.marks {
            marks.insert(
                name.clone(),
                SerializableMarkType {
                    group: mark_type.spec.group.clone(),
                    spanning: mark_type.spec.spanning.unwrap_or(false),
                },
            );
        }

        Self { nodes, marks, top_node: schema.spec.top_node.clone() }
    }
}

/// 快照构建器
pub struct SnapshotBuilder {
    config: Option<ForgeConfig>,
    xml_schema_paths: Vec<String>,
    custom_extensions: Vec<crate::types::Extensions>,
}

impl SnapshotBuilder {
    /// 创建新的快照构建器
    pub fn new() -> Self {
        Self {
            config: None,
            xml_schema_paths: Vec::new(),
            custom_extensions: Vec::new(),
        }
    }

    /// 设置配置
    pub fn with_config(
        mut self,
        config: ForgeConfig,
    ) -> Self {
        self.config = Some(config);
        self
    }

    /// 添加 XML Schema 文件
    pub fn add_xml_schema(
        mut self,
        path: String,
    ) -> Self {
        self.xml_schema_paths.push(path);
        self
    }

    /// 添加自定义扩展
    pub fn add_extension(
        mut self,
        extension: crate::types::Extensions,
    ) -> Self {
        self.custom_extensions.push(extension);
        self
    }

    /// 构建快照
    pub fn build(self) -> ForgeResult<CoreSnapshot> {
        let start_time = Instant::now();
        println!("正在构建核心快照...");

        // 1. 获取配置
        let config = self.config.unwrap_or_default();

        // 2. 构建扩展管理器
        let mut extension_builder = ExtensionManager::builder();

        // 添加 XML Schema
        for path in &self.xml_schema_paths {
            extension_builder = extension_builder.add_xml_file(path);
        }

        // 添加自定义扩展
        extension_builder =
            extension_builder.add_extensions(self.custom_extensions);

        let extension_manager = extension_builder.build()?;
        let schema = extension_manager.get_schema();

        // 3. 序列化核心数据（使用 JSON 格式避免 Serialize trait 问题）
        let schema_json =
            serde_json::to_string(&SerializableSchema::from_schema(&schema))
                .map_err(|e| {
                    crate::error::error_utils::config_error(format!(
                        "序列化 Schema 失败: {}",
                        e
                    ))
                })?;

        // 4. 构建扩展注册表
        let mut extension_registry = HashMap::new();

        for (name, node_type) in &schema.nodes {
            extension_registry.insert(
                name.clone(),
                ExtensionType::Node {
                    group: node_type.spec.group.clone(),
                    content_spec: format!("{:?}", node_type.spec.content),
                },
            );
        }

        for (name, mark_type) in &schema.marks {
            extension_registry.insert(
                name.clone(),
                ExtensionType::Mark {
                    group: mark_type.spec.group.clone(),
                    spanning: mark_type.spec.spanning.unwrap_or(false),
                },
            );
        }

        // 5. 缓存 XML Schema
        let mut xml_schema_cache = HashMap::new();
        for path in &self.xml_schema_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                xml_schema_cache.insert(path.clone(), content);
            }
        }

        // 6. 计算内容哈希
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        schema_json.hash(&mut hasher);
        config.to_json().unwrap_or_default().hash(&mut hasher);
        let content_hash = hasher.finish();

        let snapshot = CoreSnapshot {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            config,
            schema_json,
            extension_registry,
            xml_schema_cache,
            content_hash,
        };

        println!("快照构建完成，耗时: {:?}", start_time.elapsed());
        Ok(snapshot)
    }
}

/// 快照管理器
pub struct SnapshotManager;

impl SnapshotManager {
    /// 保存快照到文件
    pub fn save_to_file(
        snapshot: &CoreSnapshot,
        path: &str,
    ) -> ForgeResult<()> {
        let data = bincode::serialize(snapshot).map_err(|e| {
            crate::error::error_utils::config_error(format!(
                "序列化快照失败: {}",
                e
            ))
        })?;

        std::fs::write(path, data).map_err(|e| {
            crate::error::error_utils::config_error(format!(
                "写入快照文件失败: {}",
                e
            ))
        })?;

        println!("快照已保存到: {}", path);
        Ok(())
    }

    /// 从文件加载快照
    pub fn load_from_file(path: &str) -> ForgeResult<CoreSnapshot> {
        let data = std::fs::read(path).map_err(|e| {
            crate::error::error_utils::config_error(format!(
                "读取快照文件失败: {}",
                e
            ))
        })?;

        let snapshot: CoreSnapshot =
            bincode::deserialize(&data).map_err(|e| {
                crate::error::error_utils::config_error(format!(
                    "反序列化快照失败: {}",
                    e
                ))
            })?;

        // 验证快照版本
        if snapshot.version != env!("CARGO_PKG_VERSION") {
            return Err(crate::error::error_utils::config_error(format!(
                "快照版本不匹配: {} vs {}",
                snapshot.version,
                env!("CARGO_PKG_VERSION")
            )));
        }

        Ok(snapshot)
    }

    /// 验证快照有效性
    pub fn validate_snapshot(snapshot: &CoreSnapshot) -> ForgeResult<()> {
        // 1. 检查版本兼容性
        let current_version = env!("CARGO_PKG_VERSION");
        if snapshot.version != current_version {
            return Err(crate::error::error_utils::config_error(format!(
                "快照版本 {} 与当前版本 {} 不兼容",
                snapshot.version, current_version
            )));
        }

        // 2. 检查数据完整性
        if snapshot.schema_json.is_empty() {
            return Err(crate::error::error_utils::config_error(
                "快照中 Schema 数据为空".to_string(),
            ));
        }

        // 3. 验证配置
        snapshot.config.validate().map_err(|e| {
            crate::error::error_utils::config_error(format!(
                "快照配置无效: {}",
                e
            ))
        })?;

        Ok(())
    }

    /// 从快照恢复 ExtensionManager
    pub fn restore_extension_manager(
        snapshot: &CoreSnapshot
    ) -> ForgeResult<ExtensionManager> {
        let start_time = Instant::now();

        // 从快照重建扩展管理器
        let mut extensions = Vec::new();

        // 从扩展注册表重建扩展
        for (name, ext_type) in &snapshot.extension_registry {
            match ext_type {
                ExtensionType::Node { .. } => {
                    // 创建基础节点扩展
                    let node =
                        crate::node::Node::create(name, Default::default());
                    extensions.push(crate::types::Extensions::N(node));
                },
                ExtensionType::Mark { .. } => {
                    // 创建基础标记扩展
                    let mark = crate::mark::Mark::new(name, Default::default());
                    extensions.push(crate::types::Extensions::M(mark));
                },
            }
        }

        // 使用扩展列表创建基础管理器
        let manager = ExtensionManager::new(&extensions)?;

        crate::metrics::snapshot_restore_duration(start_time.elapsed());
        Ok(manager)
    }

    /// 检查快照是否需要更新
    pub fn needs_update(
        snapshot: &CoreSnapshot,
        xml_paths: &[String],
    ) -> bool {
        // 检查 XML 文件是否发生变化
        for path in xml_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_secs = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    if modified_secs > snapshot.created_at {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_build_and_restore() {
        let temp_dir = tempdir().unwrap();
        let xml_path = temp_dir.path().join("test.xml");

        // 创建测试 XML
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <nodes>
            <node name="doc">
              <content>paragraph+</content>
            </node>
          </nodes>
        </schema>
        "#;

        fs::write(&xml_path, xml_content).unwrap();

        // 构建快照
        let snapshot = SnapshotBuilder::new()
            .with_config(ForgeConfig::development())
            .add_xml_schema(xml_path.to_string_lossy().to_string())
            .build()
            .unwrap();

        // 验证快照
        assert!(SnapshotManager::validate_snapshot(&snapshot).is_ok());

        // 从快照恢复
        let manager =
            SnapshotManager::restore_extension_manager(&snapshot).unwrap();
        assert_eq!(manager.get_schema().nodes.len(), 1);
        assert!(manager.get_schema().nodes.contains_key("doc"));
    }

    #[test]
    fn test_snapshot_file_operations() {
        let temp_dir = tempdir().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.bin");
        let xml_path = temp_dir.path().join("test.xml");

        // 创建测试 XML
        let xml_content = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <schema>
          <nodes>
            <node name="doc">
              <content>paragraph+</content>
            </node>
          </nodes>
        </schema>
        "#;

        fs::write(&xml_path, xml_content).unwrap();

        // 创建快照
        let original_snapshot = SnapshotBuilder::new()
            .with_config(ForgeConfig::testing())
            .add_xml_schema(xml_path.to_string_lossy().to_string())
            .build()
            .unwrap();

        // 保存到文件
        SnapshotManager::save_to_file(
            &original_snapshot,
            snapshot_path.to_string_lossy().as_ref(),
        )
        .unwrap();

        // 从文件加载
        let loaded_snapshot = SnapshotManager::load_from_file(
            snapshot_path.to_string_lossy().as_ref(),
        )
        .unwrap();

        // 验证内容一致
        assert_eq!(original_snapshot.version, loaded_snapshot.version);
        assert_eq!(
            original_snapshot.content_hash,
            loaded_snapshot.content_hash
        );
    }
}

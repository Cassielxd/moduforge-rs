//! 构建时快照生成工具
//! 
//! 提供 build.rs 脚本使用的快照生成功能

use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::snapshot::{SnapshotBuilder, SnapshotManager, CoreSnapshot};
use crate::config::{ForgeConfig, Environment};
use crate::ForgeResult;

/// 构建时快照配置
#[derive(Debug, Clone)]
pub struct BuildSnapshotConfig {
    /// 输出目录
    pub output_dir: PathBuf,
    /// 快照文件名
    pub snapshot_name: String,
    /// XML Schema 目录
    pub schema_dir: Option<PathBuf>,
    /// 目标环境
    pub target_environment: Environment,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 自定义配置覆盖
    pub config_overrides: HashMap<String, String>,
}

impl Default for BuildSnapshotConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("target/snapshots"),
            snapshot_name: "core_snapshot.bin".to_string(),
            schema_dir: None,
            target_environment: Environment::Production,
            enable_compression: true,
            config_overrides: HashMap::new(),
        }
    }
}

impl BuildSnapshotConfig {
    /// 从环境变量创建配置
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // 输出目录
        if let Ok(dir) = env::var("MODUFORGE_SNAPSHOT_OUTPUT_DIR") {
            config.output_dir = PathBuf::from(dir);
        }
        
        // 快照名称
        if let Ok(name) = env::var("MODUFORGE_SNAPSHOT_NAME") {
            config.snapshot_name = name;
        }
        
        // Schema 目录
        if let Ok(dir) = env::var("MODUFORGE_SCHEMA_DIR") {
            config.schema_dir = Some(PathBuf::from(dir));
        }
        
        // 目标环境
        if let Ok(env_str) = env::var("MODUFORGE_TARGET_ENV") {
            config.target_environment = match env_str.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "testing" | "test" => Environment::Testing,
                "production" | "prod" => Environment::Production,
                _ => Environment::Production,
            };
        }
        
        // 压缩选项
        if let Ok(compress) = env::var("MODUFORGE_ENABLE_COMPRESSION") {
            config.enable_compression = compress.to_lowercase() == "true";
        }
        
        config
    }
    
    /// 设置输出目录
    pub fn with_output_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.output_dir = dir.as_ref().to_path_buf();
        self
    }
    
    /// 设置快照名称
    pub fn with_snapshot_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.snapshot_name = name.as_ref().to_string();
        self
    }
    
    /// 设置 Schema 目录
    pub fn with_schema_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.schema_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    
    /// 设置目标环境
    pub fn with_environment(mut self, env: Environment) -> Self {
        self.target_environment = env;
        self
    }
    
    /// 添加配置覆盖
    pub fn with_config_override<K: AsRef<str>, V: AsRef<str>>(
        mut self, 
        key: K, 
        value: V
    ) -> Self {
        self.config_overrides.insert(
            key.as_ref().to_string(), 
            value.as_ref().to_string()
        );
        self
    }
}

/// 构建时快照生成器
pub struct BuildTimeSnapshotGenerator {
    config: BuildSnapshotConfig,
}

impl BuildTimeSnapshotGenerator {
    /// 创建新的生成器
    pub fn new(config: BuildSnapshotConfig) -> Self {
        Self { config }
    }
    
    /// 使用默认配置创建
    pub fn default() -> Self {
        Self::new(BuildSnapshotConfig::default())
    }
    
    /// 从环境变量创建
    pub fn from_env() -> Self {
        Self::new(BuildSnapshotConfig::from_env())
    }
    
    /// 生成快照
    pub fn generate(&self) -> ForgeResult<CoreSnapshot> {
        println!("cargo:warning=开始生成核心快照...");
        
        // 1. 确保输出目录存在
        std::fs::create_dir_all(&self.config.output_dir)
            .map_err(|e| crate::error::error_utils::config_error(
                format!("创建输出目录失败: {}", e)
            ))?;
        
        // 2. 构建基础配置
        let mut config = ForgeConfig::for_environment(self.config.target_environment);
        
        // 应用配置覆盖
        self.apply_config_overrides(&mut config)?;
        
        // 3. 查找 XML Schema 文件
        let xml_schemas = self.discover_xml_schemas()?;
        
        // 4. 构建快照
        let mut builder = SnapshotBuilder::new().with_config(config);
        
        for schema_path in xml_schemas {
            println!("cargo:warning=添加 XML Schema: {}", schema_path.display());
            builder = builder.add_xml_schema(schema_path.to_string_lossy().to_string());
        }
        
        let snapshot = builder.build()?;
        
        // 5. 保存快照
        let snapshot_path = self.config.output_dir.join(&self.config.snapshot_name);
        SnapshotManager::save_to_file(&snapshot, snapshot_path.to_str().unwrap())?;
        
        println!("cargo:warning=快照生成完成: {}", snapshot_path.display());
        
        // 6. 生成 Cargo 重新编译提示
        self.emit_rerun_if_changed()?;
        
        Ok(snapshot)
    }
    
    /// 发现 XML Schema 文件
    fn discover_xml_schemas(&self) -> ForgeResult<Vec<PathBuf>> {
        let mut schemas = Vec::new();
        
        // 从配置的 schema 目录查找
        if let Some(schema_dir) = &self.config.schema_dir {
            if schema_dir.exists() {
                self.scan_directory_for_schemas(schema_dir, &mut schemas)?;
            }
        }
        
        // 从标准位置查找
        let standard_locations = vec![
            "schemas",
            "schema", 
            "src/schemas",
            "resources/schemas",
        ];
        
        for location in standard_locations {
            let path = PathBuf::from(location);
            if path.exists() {
                self.scan_directory_for_schemas(&path, &mut schemas)?;
            }
        }
        
        Ok(schemas)
    }
    
    /// 扫描目录查找 Schema 文件
    fn scan_directory_for_schemas(
        &self, 
        dir: &Path, 
        schemas: &mut Vec<PathBuf>
    ) -> ForgeResult<()> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| crate::error::error_utils::config_error(
                format!("读取目录失败 {}: {}", dir.display(), e)
            ))?;
            
        for entry in entries {
            let entry = entry.map_err(|e| crate::error::error_utils::config_error(
                format!("读取目录项失败: {}", e)
            ))?;
            
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "xml" {
                        schemas.push(path);
                    }
                }
            } else if path.is_dir() {
                // 递归扫描子目录
                self.scan_directory_for_schemas(&path, schemas)?;
            }
        }
        
        Ok(())
    }
    
    /// 应用配置覆盖
    fn apply_config_overrides(&self, config: &mut ForgeConfig) -> ForgeResult<()> {
        for (key, value) in &self.config.config_overrides {
            match key.as_str() {
                "processor.max_queue_size" => {
                    if let Ok(size) = value.parse::<usize>() {
                        config.processor.max_queue_size = size;
                    }
                },
                "processor.max_concurrent_tasks" => {
                    if let Ok(tasks) = value.parse::<usize>() {
                        config.processor.max_concurrent_tasks = tasks;
                    }
                },
                "performance.enable_monitoring" => {
                    config.performance.enable_monitoring = value.to_lowercase() == "true";
                },
                "performance.middleware_timeout_ms" => {
                    if let Ok(timeout) = value.parse::<u64>() {
                        config.performance.middleware_timeout_ms = timeout;
                    }
                },
                _ => {
                    println!("cargo:warning=未知配置覆盖: {} = {}", key, value);
                },
            }
        }
        
        Ok(())
    }
    
    /// 发出 Cargo 重新编译提示
    fn emit_rerun_if_changed(&self) -> ForgeResult<()> {
        // 监听 Schema 目录变化
        if let Some(schema_dir) = &self.config.schema_dir {
            if schema_dir.exists() {
                println!("cargo:rerun-if-changed={}", schema_dir.display());
            }
        }
        
        // 监听标准 Schema 位置
        let standard_locations = vec!["schemas", "schema", "src/schemas"];
        for location in standard_locations {
            let path = PathBuf::from(location);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
        
        // 监听环境变量
        println!("cargo:rerun-if-env-changed=MODUFORGE_SNAPSHOT_OUTPUT_DIR");
        println!("cargo:rerun-if-env-changed=MODUFORGE_SCHEMA_DIR");
        println!("cargo:rerun-if-env-changed=MODUFORGE_TARGET_ENV");
        
        Ok(())
    }
}

/// 构建脚本便捷函数
/// 
/// 在 build.rs 中使用：
/// 
/// ```rust
/// use mf_core::build_tools::generate_snapshot;
/// 
/// fn main() {
///     if let Err(e) = generate_snapshot() {
///         panic!("生成快照失败: {}", e);
///     }
/// }
/// ```
pub fn generate_snapshot() -> ForgeResult<()> {
    let generator = BuildTimeSnapshotGenerator::from_env();
    generator.generate()?;
    Ok(())
}

/// 高级构建脚本函数，允许自定义配置
/// 
/// ```rust
/// use mf_core::build_tools::{generate_snapshot_with_config, BuildSnapshotConfig};
/// use mf_core::config::Environment;
/// 
/// fn main() {
///     let config = BuildSnapshotConfig::default()
///         .with_environment(Environment::Production)
///         .with_schema_dir("custom_schemas")
///         .with_config_override("processor.max_queue_size", "5000");
///         
///     if let Err(e) = generate_snapshot_with_config(config) {
///         panic!("生成快照失败: {}", e);
///     }
/// }
/// ```
pub fn generate_snapshot_with_config(config: BuildSnapshotConfig) -> ForgeResult<()> {
    let generator = BuildTimeSnapshotGenerator::new(config);
    generator.generate()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    
    #[test]
    fn test_build_snapshot_config() {
        let config = BuildSnapshotConfig::default()
            .with_snapshot_name("test.bin")
            .with_environment(Environment::Testing)
            .with_config_override("processor.max_queue_size", "1000");
            
        assert_eq!(config.snapshot_name, "test.bin");
        assert_eq!(config.target_environment, Environment::Testing);
        assert_eq!(config.config_overrides.get("processor.max_queue_size"), Some(&"1000".to_string()));
    }
    
    #[test]
    fn test_xml_schema_discovery() {
        let temp_dir = tempdir().unwrap();
        let schema_dir = temp_dir.path().join("schemas");
        fs::create_dir_all(&schema_dir).unwrap();
        
        // 创建测试 XML 文件
        let xml_content = r#"<?xml version="1.0"?><schema></schema>"#;
        fs::write(schema_dir.join("test1.xml"), xml_content).unwrap();
        fs::write(schema_dir.join("test2.xml"), xml_content).unwrap();
        fs::write(schema_dir.join("readme.txt"), "not xml").unwrap();
        
        let config = BuildSnapshotConfig::default()
            .with_schema_dir(&schema_dir)
            .with_output_dir(temp_dir.path().join("output"));
            
        let generator = BuildTimeSnapshotGenerator::new(config);
        let schemas = generator.discover_xml_schemas().unwrap();
        
        assert_eq!(schemas.len(), 2);
        assert!(schemas.iter().any(|p| p.file_name().unwrap() == "test1.xml"));
        assert!(schemas.iter().any(|p| p.file_name().unwrap() == "test2.xml"));
    }
    
    #[test]
    fn test_config_overrides() {
        let temp_dir = tempdir().unwrap();
        let config = BuildSnapshotConfig::default()
            .with_output_dir(&temp_dir)
            .with_config_override("processor.max_queue_size", "2000")
            .with_config_override("performance.enable_monitoring", "true");
            
        let generator = BuildTimeSnapshotGenerator::new(config);
        let mut forge_config = ForgeConfig::default();
        
        generator.apply_config_overrides(&mut forge_config).unwrap();
        
        assert_eq!(forge_config.processor.max_queue_size, 2000);
        assert_eq!(forge_config.performance.enable_monitoring, true);
    }
}
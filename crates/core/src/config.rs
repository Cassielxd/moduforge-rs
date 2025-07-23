//! 统一配置管理模块
//!
//! 该模块提供了 ModuForge 核心模块的统一配置管理，解决了以下问题：
//!
//! ## 主要功能
//!
//! 1. **统一配置结构**：将分散在各个模块中的配置统一管理
//! 2. **环境适配**：提供开发、测试、生产环境的预设配置
//! 3. **配置验证**：确保配置值的合理性和一致性
//! 4. **配置继承**：支持配置的层级覆盖和继承
//! 5. **运行时调整**：支持运行时动态调整部分配置
//!
//! ## 使用示例
//!
//! ```rust
//! use mf_core::config::{ForgeConfig, Environment};
//!
//! // 使用预设环境配置
//! let config = ForgeConfig::for_environment(Environment::Production);
//!
//! // 自定义配置
//! let config = ForgeConfig::builder()
//!     .processor_config(ProcessorConfig {
//!         max_queue_size: 5000,
//!         max_concurrent_tasks: 20,
//!         ..Default::default()
//!     })
//!     .performance_config(PerformanceConfig {
//!         enable_monitoring: true,
//!         middleware_timeout_ms: 1000,
//!         ..Default::default()
//!     })
//!     .build();
//! ```

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// 运行环境类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    /// 开发环境 - 较长超时时间，详细日志
    Development,
    /// 测试环境 - 中等超时时间，适度日志
    Testing,
    /// 生产环境 - 较短超时时间，精简日志
    Production,
    /// 自定义环境
    Custom,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

/// 任务处理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// 任务队列的最大容量
    pub max_queue_size: usize,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
    /// 单个任务的最大执行时间
    pub task_timeout: Duration,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试延迟时间
    pub retry_delay: Duration,
    /// 任务清理超时时间（用于优雅关闭）
    pub cleanup_timeout: Duration,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            max_concurrent_tasks: 10,
            task_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            cleanup_timeout: Duration::from_secs(30),
        }
    }
}

/// 性能监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 是否启用性能监控
    pub enable_monitoring: bool,
    /// 中间件执行超时时间（毫秒）
    pub middleware_timeout_ms: u64,
    /// 性能日志记录阈值（毫秒）
    pub log_threshold_ms: u64,
    /// 任务接收超时时间（毫秒）
    pub task_receive_timeout_ms: u64,
    /// 是否启用详细性能日志
    pub enable_detailed_logging: bool,
    /// 性能指标采样率（0.0-1.0）
    pub metrics_sampling_rate: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: false,
            middleware_timeout_ms: 500,
            log_threshold_ms: 50,
            task_receive_timeout_ms: 5000,
            enable_detailed_logging: false,
            metrics_sampling_rate: 1.0,
        }
    }
}

/// 事件系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// 事件队列最大大小
    pub max_queue_size: usize,
    /// 事件处理超时时间
    pub handler_timeout: Duration,
    /// 是否启用事件持久化
    pub enable_persistence: bool,
    /// 事件批处理大小
    pub batch_size: usize,
    /// 事件处理器最大并发数
    pub max_concurrent_handlers: usize,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 10000,
            handler_timeout: Duration::from_secs(5),
            enable_persistence: false,
            batch_size: 100,
            max_concurrent_handlers: 5,
        }
    }
}

/// 历史记录配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// 历史记录最大条数
    pub max_entries: usize,
    /// 是否启用历史记录压缩
    pub enable_compression: bool,
    /// 历史记录持久化间隔
    pub persistence_interval: Duration,
    /// 是否启用增量快照
    pub enable_incremental_snapshots: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 100,
            enable_compression: false,
            persistence_interval: Duration::from_secs(60),
            enable_incremental_snapshots: false,
        }
    }
}

/// 扩展系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// 扩展加载超时时间
    pub load_timeout: Duration,
    /// 是否启用扩展热重载
    pub enable_hot_reload: bool,
    /// 扩展最大内存使用量（MB）
    pub max_memory_mb: usize,
    /// 是否启用扩展沙箱
    pub enable_sandbox: bool,
    /// XML schema文件路径列表
    pub xml_schema_paths: Vec<String>,
    /// 是否启用XML schema自动重载
    pub enable_xml_auto_reload: bool,
    /// XML解析超时时间
    pub xml_parse_timeout: Duration,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            load_timeout: Duration::from_secs(10),
            enable_hot_reload: false,
            max_memory_mb: 100,
            enable_sandbox: true,
            xml_schema_paths: Vec::new(),
            enable_xml_auto_reload: false,
            xml_parse_timeout: Duration::from_secs(5),
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 缓存最大条目数
    pub max_entries: usize,
    /// 缓存条目过期时间
    pub entry_ttl: Duration,
    /// 是否启用 LRU 淘汰策略
    pub enable_lru: bool,
    /// 缓存清理间隔
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            entry_ttl: Duration::from_secs(300), // 5分钟
            enable_lru: true,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

/// 统一的 Forge 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    /// 运行环境
    pub environment: Environment,
    /// 任务处理器配置
    pub processor: ProcessorConfig,
    /// 性能监控配置
    pub performance: PerformanceConfig,
    /// 事件系统配置
    pub event: EventConfig,
    /// 历史记录配置
    pub history: HistoryConfig,
    /// 扩展系统配置
    pub extension: ExtensionConfig,
    /// 缓存配置
    pub cache: CacheConfig,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            environment: Environment::default(),
            processor: ProcessorConfig::default(),
            performance: PerformanceConfig::default(),
            event: EventConfig::default(),
            history: HistoryConfig::default(),
            extension: ExtensionConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl ForgeConfig {
    /// 为指定环境创建配置
    pub fn for_environment(env: Environment) -> Self {
        match env {
            Environment::Development => Self::development(),
            Environment::Testing => Self::testing(),
            Environment::Production => Self::production(),
            Environment::Custom => Self::default(),
        }
    }

    /// 开发环境配置
    pub fn development() -> Self {
        Self {
            environment: Environment::Development,
            processor: ProcessorConfig {
                max_queue_size: 500,
                max_concurrent_tasks: 5,
                task_timeout: Duration::from_secs(60),
                max_retries: 5,
                retry_delay: Duration::from_secs(2),
                cleanup_timeout: Duration::from_secs(60),
            },
            performance: PerformanceConfig {
                enable_monitoring: true,
                middleware_timeout_ms: 10000, // 10秒
                log_threshold_ms: 100,
                task_receive_timeout_ms: 30000, // 30秒
                enable_detailed_logging: true,
                metrics_sampling_rate: 1.0,
            },
            event: EventConfig {
                max_queue_size: 5000,
                handler_timeout: Duration::from_secs(10),
                enable_persistence: false,
                batch_size: 50,
                max_concurrent_handlers: 3,
            },
            history: HistoryConfig {
                max_entries: 200,
                enable_compression: false,
                persistence_interval: Duration::from_secs(30),
                enable_incremental_snapshots: false,
            },
            extension: ExtensionConfig {
                load_timeout: Duration::from_secs(30),
                enable_hot_reload: true,
                max_memory_mb: 200,
                enable_sandbox: false, // 开发环境关闭沙箱便于调试
                xml_schema_paths: Vec::new(),
                enable_xml_auto_reload: true,
                xml_parse_timeout: Duration::from_secs(10),
            },
            cache: CacheConfig {
                max_entries: 500,
                entry_ttl: Duration::from_secs(600), // 10分钟
                enable_lru: true,
                cleanup_interval: Duration::from_secs(30),
            },
        }
    }

    /// 测试环境配置
    pub fn testing() -> Self {
        Self {
            environment: Environment::Testing,
            processor: ProcessorConfig {
                max_queue_size: 100,
                max_concurrent_tasks: 3,
                task_timeout: Duration::from_secs(10),
                max_retries: 2,
                retry_delay: Duration::from_millis(500),
                cleanup_timeout: Duration::from_secs(10),
            },
            performance: PerformanceConfig {
                enable_monitoring: true,
                middleware_timeout_ms: 2000, // 2秒
                log_threshold_ms: 50,
                task_receive_timeout_ms: 5000, // 5秒
                enable_detailed_logging: false,
                metrics_sampling_rate: 0.1, // 10% 采样
            },
            event: EventConfig {
                max_queue_size: 1000,
                handler_timeout: Duration::from_secs(2),
                enable_persistence: false,
                batch_size: 20,
                max_concurrent_handlers: 2,
            },
            history: HistoryConfig {
                max_entries: 50,
                enable_compression: false,
                persistence_interval: Duration::from_secs(10),
                enable_incremental_snapshots: false,
            },
            extension: ExtensionConfig {
                load_timeout: Duration::from_secs(5),
                enable_hot_reload: false,
                max_memory_mb: 50,
                enable_sandbox: true,
                xml_schema_paths: Vec::new(),
                enable_xml_auto_reload: false,
                xml_parse_timeout: Duration::from_secs(3),
            },
            cache: CacheConfig {
                max_entries: 100,
                entry_ttl: Duration::from_secs(60), // 1分钟
                enable_lru: true,
                cleanup_interval: Duration::from_secs(10),
            },
        }
    }

    /// 生产环境配置
    pub fn production() -> Self {
        Self {
            environment: Environment::Production,
            processor: ProcessorConfig {
                max_queue_size: 10000,
                max_concurrent_tasks: 50,
                task_timeout: Duration::from_secs(30),
                max_retries: 3,
                retry_delay: Duration::from_millis(100),
                cleanup_timeout: Duration::from_secs(30),
            },
            performance: PerformanceConfig {
                enable_monitoring: true,
                middleware_timeout_ms: 1000, // 1秒
                log_threshold_ms: 50,
                task_receive_timeout_ms: 5000, // 5秒
                enable_detailed_logging: false,
                metrics_sampling_rate: 0.01, // 1% 采样
            },
            event: EventConfig {
                max_queue_size: 50000,
                handler_timeout: Duration::from_secs(5),
                enable_persistence: true,
                batch_size: 500,
                max_concurrent_handlers: 10,
            },
            history: HistoryConfig {
                max_entries: 1000,
                enable_compression: true,
                persistence_interval: Duration::from_secs(300), // 5分钟
                enable_incremental_snapshots: true,
            },
            extension: ExtensionConfig {
                load_timeout: Duration::from_secs(10),
                enable_hot_reload: false,
                max_memory_mb: 500,
                enable_sandbox: true,
                xml_schema_paths: Vec::new(),
                enable_xml_auto_reload: false,
                xml_parse_timeout: Duration::from_secs(5),
            },
            cache: CacheConfig {
                max_entries: 10000,
                entry_ttl: Duration::from_secs(1800), // 30分钟
                enable_lru: true,
                cleanup_interval: Duration::from_secs(300), // 5分钟
            },
        }
    }

    /// 创建配置构建器
    pub fn builder() -> ForgeConfigBuilder {
        ForgeConfigBuilder::new()
    }

    /// 验证配置的合理性
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // 验证处理器配置
        if self.processor.max_queue_size == 0 {
            return Err(ConfigValidationError::InvalidValue {
                field: "processor.max_queue_size".to_string(),
                value: "0".to_string(),
                reason: "队列大小必须大于0".to_string(),
            });
        }

        if self.processor.max_concurrent_tasks == 0 {
            return Err(ConfigValidationError::InvalidValue {
                field: "processor.max_concurrent_tasks".to_string(),
                value: "0".to_string(),
                reason: "并发任务数必须大于0".to_string(),
            });
        }

        if self.processor.task_timeout.is_zero() {
            return Err(ConfigValidationError::InvalidValue {
                field: "processor.task_timeout".to_string(),
                value: "0".to_string(),
                reason: "任务超时时间必须大于0".to_string(),
            });
        }

        // 验证性能配置
        if self.performance.middleware_timeout_ms == 0 {
            return Err(ConfigValidationError::InvalidValue {
                field: "performance.middleware_timeout_ms".to_string(),
                value: "0".to_string(),
                reason: "中间件超时时间必须大于0".to_string(),
            });
        }

        if !(0.0..=1.0).contains(&self.performance.metrics_sampling_rate) {
            return Err(ConfigValidationError::InvalidValue {
                field: "performance.metrics_sampling_rate".to_string(),
                value: self.performance.metrics_sampling_rate.to_string(),
                reason: "采样率必须在0.0到1.0之间".to_string(),
            });
        }

        // 验证事件配置
        if self.event.max_queue_size == 0 {
            return Err(ConfigValidationError::InvalidValue {
                field: "event.max_queue_size".to_string(),
                value: "0".to_string(),
                reason: "事件队列大小必须大于0".to_string(),
            });
        }

        // 验证历史记录配置
        if self.history.max_entries == 0 {
            return Err(ConfigValidationError::InvalidValue {
                field: "history.max_entries".to_string(),
                value: "0".to_string(),
                reason: "历史记录条数必须大于0".to_string(),
            });
        }

        // 验证缓存配置
        if self.cache.max_entries == 0 {
            return Err(ConfigValidationError::InvalidValue {
                field: "cache.max_entries".to_string(),
                value: "0".to_string(),
                reason: "缓存条目数必须大于0".to_string(),
            });
        }

        Ok(())
    }

    /// 获取环境特定的配置调整建议
    pub fn get_tuning_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        match self.environment {
            Environment::Development => {
                if !self.performance.enable_detailed_logging {
                    suggestions
                        .push("开发环境建议启用详细日志记录".to_string());
                }
                if self.extension.enable_sandbox {
                    suggestions
                        .push("开发环境可以关闭扩展沙箱以便调试".to_string());
                }
            },
            Environment::Production => {
                if self.performance.enable_detailed_logging {
                    suggestions.push(
                        "生产环境建议关闭详细日志记录以提高性能".to_string(),
                    );
                }
                if self.performance.metrics_sampling_rate > 0.1 {
                    suggestions.push(
                        "生产环境建议降低指标采样率以减少开销".to_string(),
                    );
                }
                if !self.extension.enable_sandbox {
                    suggestions.push(
                        "生产环境建议启用扩展沙箱以提高安全性".to_string(),
                    );
                }
            },
            Environment::Testing => {
                if self.processor.task_timeout > Duration::from_secs(30) {
                    suggestions
                        .push("测试环境建议使用较短的任务超时时间".to_string());
                }
            },
            Environment::Custom => {
                suggestions
                    .push("自定义环境，请根据实际需求调整配置".to_string());
            },
        }

        suggestions
    }
}

/// 配置验证错误
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    /// 无效的配置值
    InvalidValue { field: String, value: String, reason: String },
    /// 配置冲突
    Conflict { field1: String, field2: String, reason: String },
    /// 缺少必需的配置
    MissingRequired { field: String },
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            ConfigValidationError::InvalidValue { field, value, reason } => {
                write!(
                    f,
                    "配置字段 '{}' 的值 '{}' 无效: {}",
                    field, value, reason
                )
            },
            ConfigValidationError::Conflict { field1, field2, reason } => {
                write!(
                    f,
                    "配置字段 '{}' 和 '{}' 冲突: {}",
                    field1, field2, reason
                )
            },
            ConfigValidationError::MissingRequired { field } => {
                write!(f, "缺少必需的配置字段: {}", field)
            },
        }
    }
}

impl std::error::Error for ConfigValidationError {}

/// 配置构建器
#[derive(Debug, Clone)]
pub struct ForgeConfigBuilder {
    config: ForgeConfig,
}

impl ForgeConfigBuilder {
    /// 创建新的配置构建器
    pub fn new() -> Self {
        Self { config: ForgeConfig::default() }
    }

    /// 从现有配置创建构建器
    pub fn from_config(config: ForgeConfig) -> Self {
        Self { config }
    }

    /// 设置运行环境
    pub fn environment(
        mut self,
        env: Environment,
    ) -> Self {
        self.config.environment = env;
        self
    }

    /// 设置处理器配置
    pub fn processor_config(
        mut self,
        config: ProcessorConfig,
    ) -> Self {
        self.config.processor = config;
        self
    }

    /// 设置性能配置
    pub fn performance_config(
        mut self,
        config: PerformanceConfig,
    ) -> Self {
        self.config.performance = config;
        self
    }

    /// 设置事件配置
    pub fn event_config(
        mut self,
        config: EventConfig,
    ) -> Self {
        self.config.event = config;
        self
    }

    /// 设置历史记录配置
    pub fn history_config(
        mut self,
        config: HistoryConfig,
    ) -> Self {
        self.config.history = config;
        self
    }

    /// 设置扩展配置
    pub fn extension_config(
        mut self,
        config: ExtensionConfig,
    ) -> Self {
        self.config.extension = config;
        self
    }

    /// 设置缓存配置
    pub fn cache_config(
        mut self,
        config: CacheConfig,
    ) -> Self {
        self.config.cache = config;
        self
    }

    /// 设置任务队列大小
    pub fn max_queue_size(
        mut self,
        size: usize,
    ) -> Self {
        self.config.processor.max_queue_size = size;
        self
    }

    /// 设置最大并发任务数
    pub fn max_concurrent_tasks(
        mut self,
        count: usize,
    ) -> Self {
        self.config.processor.max_concurrent_tasks = count;
        self
    }

    /// 设置任务超时时间
    pub fn task_timeout(
        mut self,
        timeout: Duration,
    ) -> Self {
        self.config.processor.task_timeout = timeout;
        self
    }

    /// 设置中间件超时时间
    pub fn middleware_timeout(
        mut self,
        timeout_ms: u64,
    ) -> Self {
        self.config.performance.middleware_timeout_ms = timeout_ms;
        self
    }

    /// 启用/禁用性能监控
    pub fn enable_monitoring(
        mut self,
        enable: bool,
    ) -> Self {
        self.config.performance.enable_monitoring = enable;
        self
    }

    /// 设置历史记录最大条数
    pub fn history_limit(
        mut self,
        limit: usize,
    ) -> Self {
        self.config.history.max_entries = limit;
        self
    }

    /// 构建配置并验证
    pub fn build(self) -> Result<ForgeConfig, ConfigValidationError> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// 构建配置但不验证（用于测试或特殊情况）
    pub fn build_unchecked(self) -> ForgeConfig {
        self.config
    }
}

impl Default for ForgeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 配置工具函数
impl ForgeConfig {
    /// 从 JSON 字符串加载配置
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 将配置序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// 从环境变量加载配置覆盖
    ///
    /// 支持的环境变量格式：
    /// - FORGE_ENVIRONMENT: 运行环境
    /// - FORGE_PROCESSOR_MAX_QUEUE_SIZE: 队列大小
    /// - FORGE_PROCESSOR_MAX_CONCURRENT_TASKS: 并发任务数
    /// - FORGE_PERFORMANCE_ENABLE_MONITORING: 启用监控
    /// - 等等...
    pub fn from_env_override(mut self) -> Self {
        use std::env;

        // 环境类型
        if let Ok(env_str) = env::var("FORGE_ENVIRONMENT") {
            match env_str.to_lowercase().as_str() {
                "development" | "dev" => {
                    self.environment = Environment::Development
                },
                "testing" | "test" => self.environment = Environment::Testing,
                "production" | "prod" => {
                    self.environment = Environment::Production
                },
                "custom" => self.environment = Environment::Custom,
                _ => {},
            }
        }

        // 处理器配置
        if let Ok(size) = env::var("FORGE_PROCESSOR_MAX_QUEUE_SIZE") {
            if let Ok(size) = size.parse::<usize>() {
                self.processor.max_queue_size = size;
            }
        }

        if let Ok(tasks) = env::var("FORGE_PROCESSOR_MAX_CONCURRENT_TASKS") {
            if let Ok(tasks) = tasks.parse::<usize>() {
                self.processor.max_concurrent_tasks = tasks;
            }
        }

        // 性能配置
        if let Ok(enable) = env::var("FORGE_PERFORMANCE_ENABLE_MONITORING") {
            self.performance.enable_monitoring =
                enable.to_lowercase() == "true";
        }

        if let Ok(timeout) = env::var("FORGE_PERFORMANCE_MIDDLEWARE_TIMEOUT_MS")
        {
            if let Ok(timeout) = timeout.parse::<u64>() {
                self.performance.middleware_timeout_ms = timeout;
            }
        }

        self
    }

    /// 合并另一个配置，优先使用 other 的非默认值
    pub fn merge_with(
        mut self,
        other: &ForgeConfig,
    ) -> Self {
        // 这里可以实现更复杂的合并逻辑
        // 目前简单地用 other 覆盖 self
        if other.environment != Environment::Development {
            self.environment = other.environment;
        }

        // 可以添加更细粒度的合并逻辑
        self.processor = other.processor.clone();
        self.performance = other.performance.clone();
        self.event = other.event.clone();
        self.history = other.history.clone();
        self.extension = other.extension.clone();
        self.cache = other.cache.clone();

        self
    }
}

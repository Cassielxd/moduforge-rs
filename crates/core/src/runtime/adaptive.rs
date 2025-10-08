//! 自适应运行时选择器
//!
//! 根据系统资源（CPU核心数、内存大小）自动：
//! 1. 选择最合适的运行时类型（Sync/Async/Actor）
//! 2. 生成优化的运行时配置参数
//! 3. 调整并发数、队列大小等性能参数
//!
//! # 策略说明
//!
//! ## 运行时选择策略
//! - **低配** (<4核 或 <8GB): Sync运行时（开销最小）
//! - **中配** (4-7核 + 8-15GB): Async运行时（平衡性能和开销）
//! - **高配** (≥8核 + ≥16GB): Actor运行时（最大并发能力）
//!
//! ## 配置优化策略
//! - 并发任务数 = CPU线程数 × 75%
//! - 队列大小 = 可用内存GB × 100（限制在500-10000之间）
//! - 超时配置根据资源等级调整
//!
//! # 使用示例
//!
//! ```rust
//! use mf_core::runtime::adaptive::AdaptiveRuntimeSelector;
//! use mf_core::runtime::system_detector::SystemResources;
//!
//! let resources = SystemResources::detect();
//! let runtime_type = AdaptiveRuntimeSelector::select_runtime(&resources);
//! let config = AdaptiveRuntimeSelector::generate_config(&resources);
//! ```

use std::time::Duration;

use crate::config::{
    CacheConfig, EventConfig, ExtensionConfig, ForgeConfig, HistoryConfig,
    PerformanceConfig, ProcessorConfig, RuntimeConfig, RuntimeType,
};

use super::system_detector::{ResourceTier, SystemResources};

/// 自适应运行时选择器
pub struct AdaptiveRuntimeSelector;

impl AdaptiveRuntimeSelector {
    /// 根据系统资源选择最优运行时类型
    ///
    /// # 选择策略
    /// - **Low**: 同步运行时（开销最小，适合低配机器）
    /// - **Medium**: 异步运行时（平衡性能和开销）
    /// - **High**: Actor运行时（最大并发能力）
    ///
    /// # 参数
    /// * `resources` - 系统资源信息
    ///
    /// # 返回值
    /// * `RuntimeType` - 推荐的运行时类型
    ///
    /// # 示例
    /// ```rust
    /// let resources = SystemResources::detect();
    /// let runtime_type = AdaptiveRuntimeSelector::select_runtime(&resources);
    /// println!("推荐运行时: {:?}", runtime_type);
    /// ```
    pub fn select_runtime(resources: &SystemResources) -> RuntimeType {
        match resources.resource_tier() {
            ResourceTier::Low => RuntimeType::Sync,
            ResourceTier::Medium => RuntimeType::Async,
            ResourceTier::High => RuntimeType::Actor,
        }
    }

    /// 根据系统资源生成优化的运行时配置
    ///
    /// 自动调整以下参数：
    /// - 并发任务数（基于CPU线程数）
    /// - 队列大小（基于可用内存）
    /// - 超时配置（基于资源等级）
    /// - 监控采样率（基于资源等级）
    ///
    /// # 参数
    /// * `resources` - 系统资源信息
    ///
    /// # 返回值
    /// * `ForgeConfig` - 优化后的配置
    ///
    /// # 示例
    /// ```rust
    /// let resources = SystemResources::detect();
    /// let config = AdaptiveRuntimeSelector::generate_config(&resources);
    /// println!("并发任务数: {}", config.processor.max_concurrent_tasks);
    /// ```
    pub fn generate_config(resources: &SystemResources) -> ForgeConfig {
        let tier = resources.resource_tier();

        ForgeConfig {
            runtime: RuntimeConfig {
                runtime_type: Self::select_runtime(resources),
            },
            processor: Self::processor_config(resources, tier),
            performance: Self::performance_config(resources, tier),
            event: Self::event_config(resources, tier),
            history: Self::history_config(resources, tier),
            extension: ExtensionConfig::default(),
            cache: Self::cache_config(resources, tier),
            ..Default::default()
        }
    }

    /// 生成处理器配置
    ///
    /// 根据CPU和内存资源优化：
    /// - 并发任务数 = CPU线程数 × 75%（至少2个）
    /// - 队列大小 = 根据可用内存计算
    /// - 超时时间 = 根据资源等级调整
    fn processor_config(
        res: &SystemResources,
        tier: ResourceTier,
    ) -> ProcessorConfig {
        ProcessorConfig {
            // 队列大小：根据内存自适应
            max_queue_size: Self::calc_queue_size(res.available_memory_mb),

            // 并发任务数：CPU线程数的75%（留一些余量，至少2个）
            max_concurrent_tasks: ((res.cpu_threads as f32 * 0.75) as usize)
                .max(2),

            // 任务超时：高配机器超时更短（期望更快响应）
            task_timeout: match tier {
                ResourceTier::High => Duration::from_secs(10),
                ResourceTier::Medium => Duration::from_secs(30),
                ResourceTier::Low => Duration::from_secs(60),
            },

            // 重试次数：低配机器给更多重试机会
            max_retries: match tier {
                ResourceTier::High => 3,
                ResourceTier::Medium => 3,
                ResourceTier::Low => 5,
            },

            retry_delay: Duration::from_secs(1),
            cleanup_timeout: Duration::from_secs(30),
        }
    }

    /// 生成性能监控配置
    ///
    /// 优化策略：
    /// - 高配机器启用详细监控
    /// - 低配机器降低采样率减少开销
    /// - 中间件超时根据CPU性能调整
    fn performance_config(
        _res: &SystemResources,
        tier: ResourceTier,
    ) -> PerformanceConfig {
        PerformanceConfig {
            // 高配机器启用详细监控
            enable_monitoring: tier == ResourceTier::High,
            enable_detailed_logging: tier == ResourceTier::High,

            // 中间件超时：根据CPU性能调整
            middleware_timeout_ms: match tier {
                ResourceTier::High => 300,   // 高配期望更快
                ResourceTier::Medium => 500, // 中配标准超时
                ResourceTier::Low => 1000,   // 低配给更多时间
            },

            // 日志阈值：高配更敏感
            log_threshold_ms: match tier {
                ResourceTier::High => 50,
                ResourceTier::Medium => 100,
                ResourceTier::Low => 200,
            },

            // 任务接收超时
            task_receive_timeout_ms: match tier {
                ResourceTier::High => 3000,
                ResourceTier::Medium => 5000,
                ResourceTier::Low => 10000,
            },

            // 采样率：低配机器降低采样减少开销
            metrics_sampling_rate: match tier {
                ResourceTier::High => 1.0,   // 全采样
                ResourceTier::Medium => 0.5, // 50%采样
                ResourceTier::Low => 0.1,    // 10%采样
            },
        }
    }

    /// 生成事件系统配置
    ///
    /// 优化策略：
    /// - 事件队列大小基于内存
    /// - 并发处理器数基于CPU核心数
    /// - 批处理大小根据资源等级调整
    fn event_config(
        res: &SystemResources,
        tier: ResourceTier,
    ) -> EventConfig {
        EventConfig {
            // 事件队列：队列大小的一半
            max_queue_size: Self::calc_queue_size(res.available_memory_mb)
                / 2,

            // 处理器超时：与中间件超时一致
            handler_timeout: Duration::from_millis(match tier {
                ResourceTier::High => 300,
                ResourceTier::Medium => 500,
                ResourceTier::Low => 1000,
            }),

            // 持久化：低配机器默认不启用
            enable_persistence: tier == ResourceTier::High,

            // 批处理大小：高配用更大批次提升吞吐
            batch_size: match tier {
                ResourceTier::High => 200,
                ResourceTier::Medium => 100,
                ResourceTier::Low => 50,
            },

            // 并发处理器数：核心数的一半（至少1个）
            max_concurrent_handlers: (res.cpu_cores / 2).max(1),
        }
    }

    /// 生成历史记录配置
    ///
    /// 优化策略：
    /// - 历史记录数根据可用内存
    /// - 低配机器启用压缩节省内存
    fn history_config(
        _res: &SystemResources,
        tier: ResourceTier,
    ) -> HistoryConfig {
        HistoryConfig {
            // 历史记录数：根据资源等级
            max_entries: match tier {
                ResourceTier::High => 1000,
                ResourceTier::Medium => 500,
                ResourceTier::Low => 100,
            },

            // 低配机器启用压缩节省内存
            enable_compression: tier == ResourceTier::Low,

            // 持久化间隔：高配更频繁
            persistence_interval: Duration::from_secs(match tier {
                ResourceTier::High => 30,
                ResourceTier::Medium => 60,
                ResourceTier::Low => 120,
            }),
        }
    }

    /// 生成缓存配置
    fn cache_config(
        _res: &SystemResources,
        tier: ResourceTier,
    ) -> CacheConfig {
        CacheConfig {
            // 缓存条目数：根据资源等级
            max_entries: match tier {
                ResourceTier::High => 5000,
                ResourceTier::Medium => 2000,
                ResourceTier::Low => 500,
            },

            // TTL：高配缓存时间更短（数据更新及时）
            entry_ttl: Duration::from_secs(match tier {
                ResourceTier::High => 180,  // 3分钟
                ResourceTier::Medium => 300, // 5分钟
                ResourceTier::Low => 600,   // 10分钟
            }),

            enable_lru: true,

            // 清理间隔
            cleanup_interval: Duration::from_secs(60),
        }
    }

    /// 计算队列大小
    ///
    /// 策略：每GB可用内存 = 100个队列槽位
    /// 限制范围：500 - 10000
    ///
    /// # 参数
    /// * `available_memory_mb` - 可用内存（MB）
    ///
    /// # 返回值
    /// * `usize` - 计算出的队列大小
    fn calc_queue_size(available_memory_mb: u64) -> usize {
        let memory_gb = (available_memory_mb / 1024).max(1);
        (memory_gb as usize * 100).clamp(500, 10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_runtime() {
        // 低配 -> Sync
        let low = SystemResources {
            cpu_cores: 2,
            cpu_threads: 2,
            total_memory_mb: 4096,
            available_memory_mb: 2048,
        };
        assert_eq!(
            AdaptiveRuntimeSelector::select_runtime(&low),
            RuntimeType::Sync
        );

        // 中配 -> Async
        let medium = SystemResources {
            cpu_cores: 4,
            cpu_threads: 8,
            total_memory_mb: 8192,
            available_memory_mb: 4096,
        };
        assert_eq!(
            AdaptiveRuntimeSelector::select_runtime(&medium),
            RuntimeType::Async
        );

        // 高配 -> Actor
        let high = SystemResources {
            cpu_cores: 8,
            cpu_threads: 16,
            total_memory_mb: 16384,
            available_memory_mb: 8192,
        };
        assert_eq!(
            AdaptiveRuntimeSelector::select_runtime(&high),
            RuntimeType::Actor
        );
    }

    #[test]
    fn test_calc_queue_size() {
        // 2GB -> 200 (但限制最小500)
        assert_eq!(AdaptiveRuntimeSelector::calc_queue_size(2048), 500);

        // 4GB -> 400 (但限制最小500)
        assert_eq!(AdaptiveRuntimeSelector::calc_queue_size(4096), 500);

        // 8GB -> 800
        assert_eq!(AdaptiveRuntimeSelector::calc_queue_size(8192), 800);

        // 16GB -> 1600
        assert_eq!(
            AdaptiveRuntimeSelector::calc_queue_size(16384),
            1600
        );

        // 128GB -> 12800 (但限制最大10000)
        assert_eq!(
            AdaptiveRuntimeSelector::calc_queue_size(128 * 1024),
            10000
        );
    }

    #[test]
    fn test_generate_config() {
        let resources = SystemResources {
            cpu_cores: 4,
            cpu_threads: 8,
            total_memory_mb: 8192,
            available_memory_mb: 4096,
        };

        let config = AdaptiveRuntimeSelector::generate_config(&resources);

        // 验证运行时选择
        assert_eq!(config.runtime.runtime_type, RuntimeType::Async);

        // 验证并发数：8 * 0.75 = 6
        assert_eq!(config.processor.max_concurrent_tasks, 6);

        // 验证队列大小：4GB = 400 -> 500(最小限制)
        assert_eq!(config.processor.max_queue_size, 500);

        // 验证中配超时
        assert_eq!(config.performance.middleware_timeout_ms, 500);
    }

    #[test]
    fn test_concurrent_tasks_calculation() {
        // 测试最小值限制
        let low = SystemResources {
            cpu_cores: 1,
            cpu_threads: 1,
            total_memory_mb: 2048,
            available_memory_mb: 1024,
        };
        let config = AdaptiveRuntimeSelector::generate_config(&low);
        assert!(
            config.processor.max_concurrent_tasks >= 2,
            "并发数应该至少为2"
        );

        // 测试正常计算
        let medium = SystemResources {
            cpu_cores: 4,
            cpu_threads: 8,
            total_memory_mb: 8192,
            available_memory_mb: 4096,
        };
        let config = AdaptiveRuntimeSelector::generate_config(&medium);
        assert_eq!(
            config.processor.max_concurrent_tasks, 6,
            "8 * 0.75 = 6"
        );
    }
}

//! 系统资源检测模块
//!
//! 提供自动检测系统硬件配置的功能，包括：
//! - CPU核心数和线程数
//! - 系统内存总量和可用量
//! - 系统资源等级评估
//!
//! # 使用示例
//!
//! ```rust
//! use mf_core::runtime::system_detector::SystemResources;
//!
//! let resources = SystemResources::detect();
//! println!("CPU: {} 核心 / {} 线程", resources.cpu_cores, resources.cpu_threads);
//! println!("内存: {} MB", resources.total_memory_mb);
//! ```

use sysinfo::System;

/// 系统资源信息
#[derive(Debug, Clone)]
pub struct SystemResources {
    /// CPU物理核心数
    pub cpu_cores: usize,
    /// CPU逻辑线程数
    pub cpu_threads: usize,
    /// 系统总内存（MB）
    pub total_memory_mb: u64,
    /// 系统可用内存（MB）
    pub available_memory_mb: u64,
}

impl SystemResources {
    /// 自动检测系统资源
    ///
    /// # 返回值
    /// * `SystemResources` - 包含CPU和内存信息的系统资源结构
    ///
    /// # 示例
    /// ```rust
    /// let resources = SystemResources::detect();
    /// assert!(resources.cpu_cores > 0);
    /// assert!(resources.total_memory_mb > 0);
    /// ```
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            cpu_cores: num_cpus::get_physical(),
            cpu_threads: num_cpus::get(),
            total_memory_mb: sys.total_memory() / 1024 / 1024,
            available_memory_mb: sys.available_memory() / 1024 / 1024,
        }
    }

    /// 判断系统资源等级
    ///
    /// 根据CPU核心数和内存大小评估系统资源等级：
    /// - **High**: 8核以上 + 16GB以上内存
    /// - **Medium**: 4核以上 + 8GB以上内存
    /// - **Low**: 其他配置
    ///
    /// # 返回值
    /// * `ResourceTier` - 系统资源等级枚举
    ///
    /// # 示例
    /// ```rust
    /// let resources = SystemResources::detect();
    /// match resources.resource_tier() {
    ///     ResourceTier::High => println!("高配系统"),
    ///     ResourceTier::Medium => println!("中配系统"),
    ///     ResourceTier::Low => println!("低配系统"),
    /// }
    /// ```
    pub fn resource_tier(&self) -> ResourceTier {
        match (self.cpu_cores, self.total_memory_mb) {
            // 高配：8核以上 + 16GB以上
            (cores, mem) if cores >= 8 && mem >= 16000 => ResourceTier::High,
            // 中配：4核以上 + 8GB以上
            (cores, mem) if cores >= 4 && mem >= 8000 => ResourceTier::Medium,
            // 低配：其他
            _ => ResourceTier::Low,
        }
    }

    /// 获取系统资源等级的描述字符串
    pub fn tier_description(&self) -> &'static str {
        match self.resource_tier() {
            ResourceTier::High => "高性能",
            ResourceTier::Medium => "标准配置",
            ResourceTier::Low => "基础配置",
        }
    }
}

/// 系统资源等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceTier {
    /// 低配机器（<4核 或 <8GB内存）
    Low,
    /// 中配机器（4-7核 + 8-15GB内存）
    Medium,
    /// 高配机器（≥8核 + ≥16GB内存）
    High,
}

impl ResourceTier {
    /// 获取资源等级的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceTier::Low => "Low",
            ResourceTier::Medium => "Medium",
            ResourceTier::High => "High",
        }
    }
}

impl std::fmt::Display for ResourceTier {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_resources_detect() {
        let resources = SystemResources::detect();

        // 基本验证：任何系统都应该有CPU和内存
        assert!(
            resources.cpu_cores > 0,
            "CPU核心数应该大于0"
        );
        assert!(
            resources.cpu_threads > 0,
            "CPU线程数应该大于0"
        );
        assert!(
            resources.total_memory_mb > 0,
            "总内存应该大于0"
        );

        // CPU线程数应该大于等于核心数
        assert!(
            resources.cpu_threads >= resources.cpu_cores,
            "CPU线程数应该大于等于核心数"
        );
    }

    #[test]
    fn test_resource_tier() {
        // 测试低配
        let low_resources = SystemResources {
            cpu_cores: 2,
            cpu_threads: 2,
            total_memory_mb: 4096,
            available_memory_mb: 2048,
        };
        assert_eq!(low_resources.resource_tier(), ResourceTier::Low);

        // 测试中配
        let medium_resources = SystemResources {
            cpu_cores: 4,
            cpu_threads: 8,
            total_memory_mb: 8192,
            available_memory_mb: 4096,
        };
        assert_eq!(
            medium_resources.resource_tier(),
            ResourceTier::Medium
        );

        // 测试高配
        let high_resources = SystemResources {
            cpu_cores: 8,
            cpu_threads: 16,
            total_memory_mb: 16384,
            available_memory_mb: 8192,
        };
        assert_eq!(high_resources.resource_tier(), ResourceTier::High);
    }

    #[test]
    fn test_tier_description() {
        let resources = SystemResources::detect();
        let description = resources.tier_description();
        assert!(
            ["高性能", "标准配置", "基础配置"]
                .contains(&description)
        );
    }
}

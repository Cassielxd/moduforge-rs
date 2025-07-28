//! # ModuForge 依赖注入容器
//! 
//! 这个crate提供了一个灵活且高性能的依赖注入系统，支持：
//! - 自动组件注册和依赖解析
//! - 生命周期管理（单例、瞬态、作用域）
//! - 异步组件支持
//! - 循环依赖检测
//! - 编译时类型安全

pub mod container;
pub mod component;
pub mod lifecycle;
pub mod registry;
pub mod error;
pub mod mutable;
pub mod profile;
pub mod aop;
pub mod aop_proxy;
pub mod config;

// 重新导出核心类型
pub use container::Container;
pub use component::{Component, ComponentInfo, DependencyInfo};
pub use lifecycle::Lifecycle;
pub use error::{ContainerError, ContainerResult};
pub use mutable::{
    MutableComponent, MutexWrapper, RwLockWrapper, 
    AsyncMutexWrapper, AsyncRwLockWrapper
};
pub use profile::{
    ProfileManager, Condition, ProfileCondition, EnvironmentCondition,
    ConditionalRegistration, activate_profile, activate_profiles,
    deactivate_profile, is_profile_active, get_active_profiles,
    load_profiles_from_env, profile, any_profile, all_profiles,
    env_exists, env_equals, and_conditions, or_conditions, not_condition
};
pub use aop::{
    MethodContext, AspectResult, BeforeAspect, AfterAspect, AfterReturningAspect,
    AfterThrowingAspect, AroundAspect, Pointcut, AspectManager, LoggingAspect,
    PerformanceAspect, get_aspect_manager, add_before_aspect, add_after_aspect,
    add_after_returning_aspect, add_after_throwing_aspect, add_around_aspect,
    apply_aspects
};
pub use aop_proxy::{AopProxy, AopProxyWrapper};
pub use config::{
    ConfigSource, ConfigValue, ConfigManager, ConfigChangeListener,
    ConfigComponent, get_config_manager, get_config_string, get_config_i64,
    get_config_f64, get_config_bool, set_config, add_config_source, reload_config
};

// 重新导出宏
pub use mf_derive::{
    Component, Injectable, service, bean, auto_aop,
    BeforeAspect, AfterAspect, AfterReturningAspect, AfterThrowingAspect, AroundAspect
};
// 重新导出ctor用于宏
pub use ctor;

use std::sync::Arc;
use once_cell::sync::Lazy;

// 全局容器实例
static GLOBAL_CONTAINER: Lazy<Arc<Container>> = Lazy::new(|| {
    Arc::new(Container::new())
});

/// 获取全局容器实例
pub fn global_container() -> Arc<Container> {
    GLOBAL_CONTAINER.clone()
}

/// 初始化全局容器并自动扫描加载组件
pub async fn initialize_container() -> ContainerResult<()> {
    let container = global_container();
    container.initialize().await
}


//! AOP (Aspect-Oriented Programming) 切面编程支持
//! 
//! 提供方法拦截、性能监控、日志记录等切面功能

use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
    time::Instant,
};

use async_trait::async_trait;
use once_cell::sync::Lazy;

use crate::{ContainerResult, ContainerError};

/// 全局切面管理器
static ASPECT_MANAGER: Lazy<Arc<RwLock<AspectManager>>> = Lazy::new(|| {
    Arc::new(RwLock::new(AspectManager::new()))
});

/// 方法调用上下文
#[derive(Debug, Clone)]
pub struct MethodContext {
    /// 目标对象类型名
    pub target_type: String,
    /// 方法名
    pub method_name: String,
    /// 参数（简化为字符串表示）
    pub args: Vec<String>,
    /// 调用开始时间
    pub start_time: Instant,
    /// 自定义属性
    pub attributes: HashMap<String, String>,
}

impl MethodContext {
    pub fn new(target_type: &str, method_name: &str) -> Self {
        Self {
            target_type: target_type.to_string(),
            method_name: method_name.to_string(),
            args: Vec::new(),
            start_time: Instant::now(),
            attributes: HashMap::new(),
        }
    }
    
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
    
    pub fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }
    
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }
    
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// 切面执行结果
#[derive(Debug)]
pub enum AspectResult<T> {
    /// 继续执行
    Continue(T),
    /// 返回结果（跳过原方法执行）
    Return(T),
    /// 抛出错误
    Error(ContainerError),
}

/// 前置切面
#[async_trait]
pub trait BeforeAspect: Send + Sync + Debug {
    /// 方法执行前调用
    async fn before(&self, context: &mut MethodContext) -> ContainerResult<()>;
    
    /// 切面名称
    fn name(&self) -> &str;
    
    /// 切面优先级（数字越小优先级越高）
    fn priority(&self) -> i32 {
        0
    }
}

/// 后置切面
#[async_trait]
pub trait AfterAspect: Send + Sync + Debug {
    /// 方法执行后调用（无论成功还是失败）
    async fn after(&self, context: &MethodContext, result: &ContainerResult<Box<dyn Any + Send + Sync>>);
    
    /// 切面名称
    fn name(&self) -> &str;
    
    /// 切面优先级
    fn priority(&self) -> i32 {
        0
    }
}

/// 返回后切面
#[async_trait] 
pub trait AfterReturningAspect: Send + Sync + Debug {
    /// 方法成功返回后调用
    async fn after_returning(&self, context: &MethodContext, result: &Box<dyn Any + Send + Sync>);
    
    /// 切面名称
    fn name(&self) -> &str;
    
    /// 切面优先级
    fn priority(&self) -> i32 {
        0
    }
}

/// 异常后切面
#[async_trait]
pub trait AfterThrowingAspect: Send + Sync + Debug {
    /// 方法抛出异常后调用
    async fn after_throwing(&self, context: &MethodContext, error: &ContainerError);
    
    /// 切面名称
    fn name(&self) -> &str;
    
    /// 切面优先级
    fn priority(&self) -> i32 {
        0
    }
}

/// 环绕切面 (简化版本，不使用泛型以支持 dyn)
#[async_trait]
pub trait AroundAspect: Send + Sync + Debug {
    /// 环绕方法执行前调用
    async fn before_proceed(&self, context: &mut MethodContext) -> ContainerResult<bool>;
    
    /// 环绕方法执行后调用
    async fn after_proceed(&self, context: &MethodContext, result: &ContainerResult<Box<dyn Any + Send + Sync>>);
    
    /// 切面名称
    fn name(&self) -> &str;
    
    /// 切面优先级
    fn priority(&self) -> i32 {
        0
    }
}

/// 切点表达式
#[derive(Debug, Clone)]
pub struct Pointcut {
    /// 类型模式（支持通配符 *）
    pub type_pattern: String,
    /// 方法模式（支持通配符 *）
    pub method_pattern: String,
}

impl Pointcut {
    pub fn new(type_pattern: &str, method_pattern: &str) -> Self {
        Self {
            type_pattern: type_pattern.to_string(),
            method_pattern: method_pattern.to_string(),
        }
    }
    
    /// 匹配类型和方法
    pub fn matches(&self, target_type: &str, method_name: &str) -> bool {
        self.matches_pattern(&self.type_pattern, target_type) &&
        self.matches_pattern(&self.method_pattern, method_name)
    }
    
    fn matches_pattern(&self, pattern: &str, value: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            // 简单的通配符匹配
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.is_empty() {
                return true;
            }
            
            let mut pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                
                if i == 0 {
                    // 第一部分必须从开头匹配
                    if !value.starts_with(part) {
                        return false;
                    }
                    pos = part.len();
                } else if i == parts.len() - 1 {
                    // 最后一部分必须到结尾匹配
                    if !value.ends_with(part) {
                        return false;
                    }
                } else {
                    // 中间部分
                    if let Some(found_pos) = value[pos..].find(part) {
                        pos += found_pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        } else {
            pattern == value
        }
    }
}

/// 切面管理器
#[derive(Debug)]
pub struct AspectManager {
    before_aspects: Vec<(Pointcut, Box<dyn BeforeAspect>)>,
    after_aspects: Vec<(Pointcut, Box<dyn AfterAspect>)>,
    after_returning_aspects: Vec<(Pointcut, Box<dyn AfterReturningAspect>)>,
    after_throwing_aspects: Vec<(Pointcut, Box<dyn AfterThrowingAspect>)>,
    around_aspects: Vec<(Pointcut, Box<dyn AroundAspect>)>,
}

impl AspectManager {
    pub fn new() -> Self {
        Self {
            before_aspects: Vec::new(),
            after_aspects: Vec::new(),
            after_returning_aspects: Vec::new(),
            after_throwing_aspects: Vec::new(),
            around_aspects: Vec::new(),
        }
    }
    
    /// 添加前置切面
    pub fn add_before_aspect(&mut self, pointcut: Pointcut, aspect: Box<dyn BeforeAspect>) {
        self.before_aspects.push((pointcut, aspect));
        self.before_aspects.sort_by_key(|(_, aspect)| aspect.priority());
    }
    
    /// 添加后置切面
    pub fn add_after_aspect(&mut self, pointcut: Pointcut, aspect: Box<dyn AfterAspect>) {
        self.after_aspects.push((pointcut, aspect));
        self.after_aspects.sort_by_key(|(_, aspect)| aspect.priority());
    }
    
    /// 添加返回后切面
    pub fn add_after_returning_aspect(&mut self, pointcut: Pointcut, aspect: Box<dyn AfterReturningAspect>) {
        self.after_returning_aspects.push((pointcut, aspect));
        self.after_returning_aspects.sort_by_key(|(_, aspect)| aspect.priority());
    }
    
    /// 添加异常后切面
    pub fn add_after_throwing_aspect(&mut self, pointcut: Pointcut, aspect: Box<dyn AfterThrowingAspect>) {
        self.after_throwing_aspects.push((pointcut, aspect));
        self.after_throwing_aspects.sort_by_key(|(_, aspect)| aspect.priority());
    }
    
    /// 添加环绕切面
    pub fn add_around_aspect(&mut self, pointcut: Pointcut, aspect: Box<dyn AroundAspect>) {
        self.around_aspects.push((pointcut, aspect));
        self.around_aspects.sort_by_key(|(_, aspect)| aspect.priority());
    }
    
    /// 获取匹配的前置切面
    pub fn get_matching_before_aspects(&self, target_type: &str, method_name: &str) -> Vec<&dyn BeforeAspect> {
        self.before_aspects
            .iter()
            .filter(|(pointcut, _)| pointcut.matches(target_type, method_name))
            .map(|(_, aspect)| aspect.as_ref())
            .collect()
    }
    
    /// 获取匹配的后置切面
    pub fn get_matching_after_aspects(&self, target_type: &str, method_name: &str) -> Vec<&dyn AfterAspect> {
        self.after_aspects
            .iter()
            .filter(|(pointcut, _)| pointcut.matches(target_type, method_name))
            .map(|(_, aspect)| aspect.as_ref())
            .collect()
    }
    
    /// 获取匹配的返回后切面
    pub fn get_matching_after_returning_aspects(&self, target_type: &str, method_name: &str) -> Vec<&dyn AfterReturningAspect> {
        self.after_returning_aspects
            .iter()
            .filter(|(pointcut, _)| pointcut.matches(target_type, method_name))
            .map(|(_, aspect)| aspect.as_ref())
            .collect()
    }
    
    /// 获取匹配的异常后切面
    pub fn get_matching_after_throwing_aspects(&self, target_type: &str, method_name: &str) -> Vec<&dyn AfterThrowingAspect> {
        self.after_throwing_aspects
            .iter()
            .filter(|(pointcut, _)| pointcut.matches(target_type, method_name))
            .map(|(_, aspect)| aspect.as_ref())
            .collect()
    }
    
    /// 获取匹配的环绕切面
    pub fn get_matching_around_aspects(&self, target_type: &str, method_name: &str) -> Vec<&dyn AroundAspect> {
        self.around_aspects
            .iter()
            .filter(|(pointcut, _)| pointcut.matches(target_type, method_name))
            .map(|(_, aspect)| aspect.as_ref())
            .collect()
    }
}

/// 内置切面实现

/// 日志切面
#[derive(Debug)]
pub struct LoggingAspect {
    name: String,
    log_args: bool,
    log_result: bool,
    log_time: bool,
}

impl LoggingAspect {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            log_args: true,
            log_result: true,
            log_time: true,
        }
    }
    
    pub fn with_args(mut self, log_args: bool) -> Self {
        self.log_args = log_args;
        self
    }
    
    pub fn with_result(mut self, log_result: bool) -> Self {
        self.log_result = log_result;
        self
    }
    
    pub fn with_time(mut self, log_time: bool) -> Self {
        self.log_time = log_time;
        self
    }
}

#[async_trait]
impl BeforeAspect for LoggingAspect {
    async fn before(&self, context: &mut MethodContext) -> ContainerResult<()> {
        let args_str = if self.log_args && !context.args.is_empty() {
            format!(" args=[{}]", context.args.join(", "))
        } else {
            String::new()
        };
        
        println!("[{}] -> {}::{}{}", 
                self.name, 
                context.target_type, 
                context.method_name,
                args_str);
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[async_trait]
impl AfterReturningAspect for LoggingAspect {
    async fn after_returning(&self, context: &MethodContext, _result: &Box<dyn Any + Send + Sync>) {
        let time_str = if self.log_time {
            format!(" time={}ms", context.elapsed().as_millis())
        } else {
            String::new()
        };
        
        println!("[{}] <- {}::{} 成功{}", 
                self.name, 
                context.target_type, 
                context.method_name,
                time_str);
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

#[async_trait]
impl AfterThrowingAspect for LoggingAspect {
    async fn after_throwing(&self, context: &MethodContext, error: &ContainerError) {
        let time_str = if self.log_time {
            format!(" time={}ms", context.elapsed().as_millis())
        } else {
            String::new()
        };
        
        println!("[{}] <- {}::{} 错误: {}{}", 
                self.name, 
                context.target_type, 
                context.method_name,
                error,
                time_str);
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 性能监控切面
#[derive(Debug)]
pub struct PerformanceAspect {
    name: String,
    threshold_ms: u64,
}

impl PerformanceAspect {
    pub fn new(name: &str, threshold_ms: u64) -> Self {
        Self {
            name: name.to_string(),
            threshold_ms,
        }
    }
}

#[async_trait]
impl AfterAspect for PerformanceAspect {
    async fn after(&self, context: &MethodContext, _result: &ContainerResult<Box<dyn Any + Send + Sync>>) {
        let elapsed = context.elapsed().as_millis() as u64;
        if elapsed > self.threshold_ms {
            println!("[{}] 慢方法: {}::{} 耗时 {}ms (阈值: {}ms)", 
                    self.name,
                    context.target_type, 
                    context.method_name,
                    elapsed,
                    self.threshold_ms);
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 全局函数
pub fn get_aspect_manager() -> Arc<RwLock<AspectManager>> {
    ASPECT_MANAGER.clone()
}

/// 添加切面的便捷函数
pub fn add_before_aspect(pointcut: Pointcut, aspect: Box<dyn BeforeAspect>) {
    let mut manager = ASPECT_MANAGER.write().unwrap();
    manager.add_before_aspect(pointcut, aspect);
}

pub fn add_after_aspect(pointcut: Pointcut, aspect: Box<dyn AfterAspect>) {
    let mut manager = ASPECT_MANAGER.write().unwrap();
    manager.add_after_aspect(pointcut, aspect);
}

pub fn add_after_returning_aspect(pointcut: Pointcut, aspect: Box<dyn AfterReturningAspect>) {
    let mut manager = ASPECT_MANAGER.write().unwrap();
    manager.add_after_returning_aspect(pointcut, aspect);
}

pub fn add_after_throwing_aspect(pointcut: Pointcut, aspect: Box<dyn AfterThrowingAspect>) {
    let mut manager = ASPECT_MANAGER.write().unwrap();
    manager.add_after_throwing_aspect(pointcut, aspect);
}

pub fn add_around_aspect(pointcut: Pointcut, aspect: Box<dyn AroundAspect>) {
    let mut manager = ASPECT_MANAGER.write().unwrap();
    manager.add_around_aspect(pointcut, aspect);
}

/// 应用切面到方法调用的辅助函数
pub async fn apply_aspects<F, Fut, T>(
    target_type: &str,
    method_name: &str,
    args: Vec<String>,
    method: F,
) -> ContainerResult<T>
where
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = ContainerResult<T>> + Send,
    T: Send + 'static,
{
    let mut context = MethodContext::new(target_type, method_name).with_args(args);
    let manager = ASPECT_MANAGER.read().unwrap();
    
    // 应用前置切面
    let before_aspects = manager.get_matching_before_aspects(target_type, method_name);
    for aspect in before_aspects {
        aspect.before(&mut context).await?;
    }
    
    // 执行方法
    let result = method().await;
    
    // 应用后置切面
    let after_aspects = manager.get_matching_after_aspects(target_type, method_name);
    
    for aspect in after_aspects {
        // 创建一个简化的result用于传递给切面
        let result_any: ContainerResult<Box<dyn Any + Send + Sync>> = match &result {
            Ok(_) => Ok(Box::new(()) as Box<dyn Any + Send + Sync>),
            Err(_) => Err(ContainerError::ComponentCreationFailed {
                name: "AOP".to_string(),
                source: anyhow::anyhow!("Method execution failed"),
            }),
        };
        aspect.after(&context, &result_any).await;
    }
    
    // 应用成功/失败切面
    match &result {
        Ok(_) => {
            let after_returning_aspects = manager.get_matching_after_returning_aspects(target_type, method_name);
            let dummy_result = Box::new(()) as Box<dyn Any + Send + Sync>;
            for aspect in after_returning_aspects {
                aspect.after_returning(&context, &dummy_result).await;
            }
        }
        Err(error) => {
            let after_throwing_aspects = manager.get_matching_after_throwing_aspects(target_type, method_name);
            for aspect in after_throwing_aspects {
                aspect.after_throwing(&context, error).await;
            }
        }
    }
    
    result
}
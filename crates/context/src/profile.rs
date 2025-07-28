//! Profile和条件注册支持
//! 
//! 提供基于环境、条件的组件注册能力

use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use once_cell::sync::Lazy;

/// 全局Profile管理器
static PROFILE_MANAGER: Lazy<Arc<RwLock<ProfileManager>>> = Lazy::new(|| {
    Arc::new(RwLock::new(ProfileManager::new()))
});

/// Profile管理器
#[derive(Debug)]
pub struct ProfileManager {
    /// 当前激活的Profile
    active_profiles: HashSet<String>,
    /// 默认Profile
    default_profiles: HashSet<String>,
}

impl ProfileManager {
    pub fn new() -> Self {
        let mut default_profiles = HashSet::new();
        default_profiles.insert("default".to_string());
        
        Self {
            active_profiles: default_profiles.clone(),
            default_profiles,
        }
    }
    
    /// 激活Profile
    pub fn activate_profile(&mut self, profile: &str) {
        self.active_profiles.insert(profile.to_string());
    }
    
    /// 激活多个Profile
    pub fn activate_profiles(&mut self, profiles: &[&str]) {
        for profile in profiles {
            self.active_profiles.insert(profile.to_string());
        }
    }
    
    /// 停用Profile
    pub fn deactivate_profile(&mut self, profile: &str) {
        self.active_profiles.remove(profile);
    }
    
    /// 检查Profile是否激活
    pub fn is_profile_active(&self, profile: &str) -> bool {
        self.active_profiles.contains(profile)
    }
    
    /// 检查是否有任一Profile激活
    pub fn is_any_profile_active(&self, profiles: &[&str]) -> bool {
        profiles.iter().any(|p| self.is_profile_active(p))
    }
    
    /// 检查是否所有Profile都激活
    pub fn are_all_profiles_active(&self, profiles: &[&str]) -> bool {
        profiles.iter().all(|p| self.is_profile_active(p))
    }
    
    /// 获取所有激活的Profile
    pub fn get_active_profiles(&self) -> Vec<String> {
        self.active_profiles.iter().cloned().collect()
    }
    
    /// 从环境变量加载Profile
    pub fn load_from_env(&mut self) {
        if let Ok(profiles_env) = std::env::var("MODUFORGE_PROFILES") {
            let profiles: Vec<&str> = profiles_env.split(',').map(|s| s.trim()).collect();
            self.activate_profiles(&profiles);
        }
        
        // 检查特定环境变量
        if let Ok(env) = std::env::var("ENVIRONMENT") {
            self.activate_profile(&env);
        }
        
        if let Ok(env) = std::env::var("RUST_ENV") {
            self.activate_profile(&env);
        }
    }
}

/// 条件接口
pub trait Condition: Send + Sync + std::fmt::Debug {
    /// 检查条件是否满足
    fn matches(&self) -> bool;
    
    /// 条件描述
    fn description(&self) -> String;
}

/// Profile条件
#[derive(Debug, Clone)]
pub struct ProfileCondition {
    profiles: Vec<String>,
    match_any: bool, // true: 匹配任一, false: 匹配所有
}

impl ProfileCondition {
    /// 创建匹配任一Profile的条件
    pub fn any_of(profiles: &[&str]) -> Self {
        Self {
            profiles: profiles.iter().map(|s| s.to_string()).collect(),
            match_any: true,
        }
    }
    
    /// 创建匹配所有Profile的条件
    pub fn all_of(profiles: &[&str]) -> Self {
        Self {
            profiles: profiles.iter().map(|s| s.to_string()).collect(),
            match_any: false,
        }
    }
    
    /// 创建单个Profile条件
    pub fn of(profile: &str) -> Self {
        Self {
            profiles: vec![profile.to_string()],
            match_any: true,
        }
    }
}

impl Condition for ProfileCondition {
    fn matches(&self) -> bool {
        let manager = PROFILE_MANAGER.read().unwrap();
        let profile_refs: Vec<&str> = self.profiles.iter().map(|s| s.as_str()).collect();
        
        if self.match_any {
            manager.is_any_profile_active(&profile_refs)
        } else {
            manager.are_all_profiles_active(&profile_refs)
        }
    }
    
    fn description(&self) -> String {
        let op = if self.match_any { "ANY" } else { "ALL" };
        format!("Profile {} of [{}]", op, self.profiles.join(", "))
    }
}

/// 环境变量条件
#[derive(Debug, Clone)]
pub struct EnvironmentCondition {
    var_name: String,
    expected_value: Option<String>,
    exists_only: bool,
}

impl EnvironmentCondition {
    /// 检查环境变量是否存在
    pub fn exists(var_name: &str) -> Self {
        Self {
            var_name: var_name.to_string(),
            expected_value: None,
            exists_only: true,
        }
    }
    
    /// 检查环境变量是否等于指定值
    pub fn equals(var_name: &str, value: &str) -> Self {
        Self {
            var_name: var_name.to_string(),
            expected_value: Some(value.to_string()),
            exists_only: false,
        }
    }
}

impl Condition for EnvironmentCondition {
    fn matches(&self) -> bool {
        match std::env::var(&self.var_name) {
            Ok(value) => {
                if self.exists_only {
                    true
                } else if let Some(expected) = &self.expected_value {
                    value == *expected
                } else {
                    true
                }
            }
            Err(_) => false,
        }
    }
    
    fn description(&self) -> String {
        if self.exists_only {
            format!("Environment variable '{}' exists", self.var_name)
        } else if let Some(expected) = &self.expected_value {
            format!("Environment variable '{}' = '{}'", self.var_name, expected)
        } else {
            format!("Environment variable '{}' is set", self.var_name)
        }
    }
}

/// 复合条件
#[derive(Debug)]
pub struct CompositeCondition {
    conditions: Vec<Box<dyn Condition>>,
    operator: ConditionOperator,
}

#[derive(Debug, Clone)]
pub enum ConditionOperator {
    And,
    Or,
    Not,
}

impl CompositeCondition {
    /// 创建AND条件
    pub fn and(conditions: Vec<Box<dyn Condition>>) -> Self {
        Self {
            conditions,
            operator: ConditionOperator::And,
        }
    }
    
    /// 创建OR条件
    pub fn or(conditions: Vec<Box<dyn Condition>>) -> Self {
        Self {
            conditions,
            operator: ConditionOperator::Or,
        }
    }
    
    /// 创建NOT条件（对第一个条件取反）
    pub fn not(condition: Box<dyn Condition>) -> Self {
        Self {
            conditions: vec![condition],
            operator: ConditionOperator::Not,
        }
    }
}

impl Condition for CompositeCondition {
    fn matches(&self) -> bool {
        match self.operator {
            ConditionOperator::And => {
                self.conditions.iter().all(|c| c.matches())
            }
            ConditionOperator::Or => {
                self.conditions.iter().any(|c| c.matches())
            }
            ConditionOperator::Not => {
                if let Some(condition) = self.conditions.first() {
                    !condition.matches()
                } else {
                    false
                }
            }
        }
    }
    
    fn description(&self) -> String {
        let descriptions: Vec<String> = self.conditions.iter()
            .map(|c| c.description())
            .collect();
            
        match self.operator {
            ConditionOperator::And => format!("({})", descriptions.join(") AND (")),
            ConditionOperator::Or => format!("({})", descriptions.join(") OR (")),
            ConditionOperator::Not => format!("NOT ({})", descriptions.join(", ")),
        }
    }
}

/// 条件注册信息
#[derive(Debug)]
pub struct ConditionalRegistration {
    pub component_name: String,
    pub condition: Box<dyn Condition>,
    pub priority: i32, // 优先级，数字越大优先级越高
}

impl ConditionalRegistration {
    pub fn new(component_name: &str, condition: Box<dyn Condition>) -> Self {
        Self {
            component_name: component_name.to_string(),
            condition,
            priority: 0,
        }
    }
    
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// 全局Profile函数
pub fn activate_profile(profile: &str) {
    let mut manager = PROFILE_MANAGER.write().unwrap();
    manager.activate_profile(profile);
}

pub fn activate_profiles(profiles: &[&str]) {
    let mut manager = PROFILE_MANAGER.write().unwrap();
    manager.activate_profiles(profiles);
}

pub fn deactivate_profile(profile: &str) {
    let mut manager = PROFILE_MANAGER.write().unwrap();
    manager.deactivate_profile(profile);
}

pub fn is_profile_active(profile: &str) -> bool {
    let manager = PROFILE_MANAGER.read().unwrap();
    manager.is_profile_active(profile)
}

pub fn get_active_profiles() -> Vec<String> {
    let manager = PROFILE_MANAGER.read().unwrap();
    manager.get_active_profiles()
}

pub fn load_profiles_from_env() {
    let mut manager = PROFILE_MANAGER.write().unwrap();
    manager.load_from_env();
}

/// 便捷函数创建条件
pub fn profile(name: &str) -> Box<dyn Condition> {
    Box::new(ProfileCondition::of(name))
}

pub fn any_profile(profiles: &[&str]) -> Box<dyn Condition> {
    Box::new(ProfileCondition::any_of(profiles))
}

pub fn all_profiles(profiles: &[&str]) -> Box<dyn Condition> {
    Box::new(ProfileCondition::all_of(profiles))
}

pub fn env_exists(var_name: &str) -> Box<dyn Condition> {
    Box::new(EnvironmentCondition::exists(var_name))
}

pub fn env_equals(var_name: &str, value: &str) -> Box<dyn Condition> {
    Box::new(EnvironmentCondition::equals(var_name, value))
}

pub fn and_conditions(conditions: Vec<Box<dyn Condition>>) -> Box<dyn Condition> {
    Box::new(CompositeCondition::and(conditions))
}

pub fn or_conditions(conditions: Vec<Box<dyn Condition>>) -> Box<dyn Condition> {
    Box::new(CompositeCondition::or(conditions))
}

pub fn not_condition(condition: Box<dyn Condition>) -> Box<dyn Condition> {
    Box::new(CompositeCondition::not(condition))
}
//! 配置管理和环境变量注入
//! 
//! 提供统一的配置管理、环境变量注入、配置热更新等功能

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{ContainerResult, ContainerError, Component, Lifecycle};

/// 全局配置管理器
static CONFIG_MANAGER: Lazy<Arc<RwLock<ConfigManager>>> = Lazy::new(|| {
    Arc::new(RwLock::new(ConfigManager::new()))
});

/// 配置源类型
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// 环境变量
    Environment,
    /// JSON文件
    JsonFile(String),
    /// TOML文件
    TomlFile(String),
    /// YAML文件
    YamlFile(String),
    /// 内存中的键值对
    Memory(HashMap<String, String>),
    /// 远程配置服务
    Remote(String),
}

/// 配置值类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

impl ConfigValue {
    /// 转换为字符串
    pub fn as_string(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            ConfigValue::Integer(i) => Some(i.to_string()),
            ConfigValue::Float(f) => Some(f.to_string()),
            ConfigValue::Boolean(b) => Some(b.to_string()),
            _ => None,
        }
    }
    
    /// 转换为整数
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConfigValue::Integer(i) => Some(*i),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// 转换为浮点数
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Float(f) => Some(*f),
            ConfigValue::Integer(i) => Some(*i as f64),
            ConfigValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// 转换为布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Boolean(b) => Some(*b),
            ConfigValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => None,
            },
            ConfigValue::Integer(i) => Some(*i != 0),
            _ => None,
        }
    }
    
    /// 转换为数组
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
    
    /// 转换为对象
    pub fn as_object(&self) -> Option<&HashMap<String, ConfigValue>> {
        match self {
            ConfigValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

/// 配置管理器
#[derive(Debug)]
pub struct ConfigManager {
    /// 配置源列表（按优先级排序）
    sources: Vec<ConfigSource>,
    /// 缓存的配置值
    cache: HashMap<String, ConfigValue>,
    /// 配置对象缓存
    object_cache: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    /// 监听器
    listeners: Vec<Box<dyn ConfigChangeListener>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            sources: vec![ConfigSource::Environment],
            cache: HashMap::new(),
            object_cache: HashMap::new(),
            listeners: Vec::new(),
        }
    }
    
    /// 添加配置源
    pub fn add_source(&mut self, source: ConfigSource) {
        self.sources.push(source);
        self.reload_all();
    }
    
    /// 获取配置值
    pub fn get(&self, key: &str) -> Option<&ConfigValue> {
        self.cache.get(key)
    }
    
    /// 获取字符串配置
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_string())
    }
    
    /// 获取整数配置
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }
    
    /// 获取浮点数配置
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.get(key).and_then(|v| v.as_f64())
    }
    
    /// 获取布尔配置
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }
    
    /// 获取配置对象
    pub fn get_config<T: DeserializeOwned + Clone + Send + Sync + 'static>(&mut self) -> ContainerResult<T> {
        let type_id = TypeId::of::<T>();
        
        if let Some(cached) = self.object_cache.get(&type_id) {
            if let Some(config) = cached.downcast_ref::<T>() {
                return Ok(config.clone());
            }
        }
        
        // 构建配置对象
        let config_map = self.build_config_map();
        let config_value = ConfigValue::Object(config_map);
        
        // 尝试反序列化
        let json_value = self.config_value_to_serde(&config_value)
            .map_err(|e| ContainerError::ComponentCreationFailed {
                name: std::any::type_name::<T>().to_string(),
                source: e,
            })?;
            
        let config: T = serde_json::from_value(json_value)
            .map_err(|e| ContainerError::ComponentCreationFailed {
                name: std::any::type_name::<T>().to_string(),
                source: anyhow::anyhow!("Config deserialization failed: {}", e),
            })?;
            
        // 缓存结果
        self.object_cache.insert(type_id, Box::new(config.clone()));
        
        Ok(config)
    }
    
    /// 设置配置值
    pub fn set(&mut self, key: &str, value: ConfigValue) {
        let old_value = self.cache.get(key).cloned();
        self.cache.insert(key.to_string(), value.clone());
        
        // 通知监听器
        for listener in &self.listeners {
            listener.on_config_changed(key, old_value.as_ref(), &value);
        }
        
        // 清除对象缓存
        self.object_cache.clear();
    }
    
    /// 重新加载所有配置
    pub fn reload_all(&mut self) {
        self.cache.clear();
        self.object_cache.clear();
        
        let sources_clone = self.sources.clone();
        for source in &sources_clone {
            self.load_from_source(source);
        }
    }
    
    /// 从源加载配置
    fn load_from_source(&mut self, source: &ConfigSource) {
        match source {
            ConfigSource::Environment => {
                self.load_from_environment();
            }
            ConfigSource::JsonFile(path) => {
                if let Err(e) = self.load_from_json_file(path) {
                    eprintln!("Failed to load config from JSON file {}: {}", path, e);
                }
            }
            ConfigSource::TomlFile(path) => {
                if let Err(e) = self.load_from_toml_file(path) {
                    eprintln!("Failed to load config from TOML file {}: {}", path, e);
                }
            }
            ConfigSource::YamlFile(path) => {
                if let Err(e) = self.load_from_yaml_file(path) {
                    eprintln!("Failed to load config from YAML file {}: {}", path, e);
                }
            }
            ConfigSource::Memory(map) => {
                for (key, value) in map {
                    self.cache.insert(key.clone(), ConfigValue::String(value.clone()));
                }
            }
            ConfigSource::Remote(_url) => {
                // TODO: 实现远程配置加载
                eprintln!("Remote config source not implemented yet");
            }
        }
    }
    
    /// 从环境变量加载
    fn load_from_environment(&mut self) {
        for (key, value) in std::env::vars() {
            // 转换环境变量名：APP_DATABASE_HOST -> app.database.host
            let normalized_key = key.to_lowercase().replace('_', ".");
            
            // 尝试解析为不同类型
            let config_value = if let Ok(int_val) = value.parse::<i64>() {
                ConfigValue::Integer(int_val)
            } else if let Ok(float_val) = value.parse::<f64>() {
                ConfigValue::Float(float_val)
            } else if let Ok(bool_val) = value.parse::<bool>() {
                ConfigValue::Boolean(bool_val)
            } else {
                ConfigValue::String(value)
            };
            
            self.cache.insert(normalized_key, config_value);
        }
    }
    
    /// 从JSON文件加载
    fn load_from_json_file(&mut self, path: &str) -> ContainerResult<()> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ContainerError::ComponentCreationFailed {
                name: "ConfigManager".to_string(),
                source: anyhow::anyhow!("Failed to read config file {}: {}", path, e),
            })?;
            
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ContainerError::ComponentCreationFailed {
                name: "ConfigManager".to_string(),
                source: anyhow::anyhow!("Failed to parse JSON config {}: {}", path, e),
            })?;
            
        self.load_from_json_value("", &json_value);
        Ok(())
    }
    
    /// 从TOML文件加载（简化实现）
    fn load_from_toml_file(&mut self, _path: &str) -> ContainerResult<()> {
        // TODO: 实现TOML支持
        Ok(())
    }
    
    /// 从YAML文件加载（简化实现）
    fn load_from_yaml_file(&mut self, _path: &str) -> ContainerResult<()> {
        // TODO: 实现YAML支持
        Ok(())
    }
    
    /// 从JSON值递归加载
    fn load_from_json_value(&mut self, prefix: &str, value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", prefix, key)
                    };
                    self.load_from_json_value(&full_key, val);
                }
            }
            _ => {
                let config_value = self.serde_value_to_config(value);
                self.cache.insert(prefix.to_string(), config_value);
            }
        }
    }
    
    /// 构建配置映射
    fn build_config_map(&self) -> HashMap<String, ConfigValue> {
        let mut map = HashMap::new();
        
        for (key, value) in &self.cache {
            self.insert_nested_key(&mut map, key, value.clone());
        }
        
        map
    }
    
    /// 插入嵌套键
    fn insert_nested_key(&self, map: &mut HashMap<String, ConfigValue>, key: &str, value: ConfigValue) {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 1 {
            map.insert(key.to_string(), value);
            return;
        }
        
        // 递归插入嵌套值
        self.insert_nested_key_recursive(map, &parts, 0, value);
    }
    
    /// 递归插入嵌套键
    fn insert_nested_key_recursive(&self, map: &mut HashMap<String, ConfigValue>, parts: &[&str], index: usize, value: ConfigValue) {
        if index >= parts.len() {
            return;
        }
        
        let part = parts[index];
        
        if index == parts.len() - 1 {
            // 最后一个部分，直接插入值
            map.insert(part.to_string(), value);
        } else {
            // 中间部分，需要创建或获取嵌套对象
            let entry = map.entry(part.to_string()).or_insert_with(|| {
                ConfigValue::Object(HashMap::new())
            });
            
            // 确保entry是Object类型
            if !matches!(entry, ConfigValue::Object(_)) {
                *entry = ConfigValue::Object(HashMap::new());
            }
            
            if let ConfigValue::Object(obj) = entry {
                self.insert_nested_key_recursive(obj, parts, index + 1, value);
            }
        }
    }
    
    /// 转换serde值到配置值
    fn serde_value_to_config(&self, value: &serde_json::Value) -> ConfigValue {
        match value {
            serde_json::Value::Null => ConfigValue::Null,
            serde_json::Value::Bool(b) => ConfigValue::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ConfigValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    ConfigValue::Float(f)
                } else {
                    ConfigValue::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => ConfigValue::String(s.clone()),
            serde_json::Value::Array(arr) => {
                let config_arr = arr.iter()
                    .map(|v| self.serde_value_to_config(v))
                    .collect();
                ConfigValue::Array(config_arr)
            }
            serde_json::Value::Object(obj) => {
                let config_obj = obj.iter()
                    .map(|(k, v)| (k.clone(), self.serde_value_to_config(v)))
                    .collect();
                ConfigValue::Object(config_obj)
            }
        }
    }
    
    /// 转换配置值到serde值
    fn config_value_to_serde(&self, value: &ConfigValue) -> anyhow::Result<serde_json::Value> {
        let result = match value {
            ConfigValue::Null => serde_json::Value::Null,
            ConfigValue::Boolean(b) => serde_json::Value::Bool(*b),
            ConfigValue::Integer(i) => serde_json::Value::Number((*i).into()),
            ConfigValue::Float(f) => {
                serde_json::Number::from_f64(*f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
            ConfigValue::String(s) => serde_json::Value::String(s.clone()),
            ConfigValue::Array(arr) => {
                let json_arr: Result<Vec<_>, _> = arr.iter()
                    .map(|v| self.config_value_to_serde(v))
                    .collect();
                serde_json::Value::Array(json_arr?)
            }
            ConfigValue::Object(obj) => {
                let json_obj: Result<serde_json::Map<String, serde_json::Value>, _> = obj.iter()
                    .map(|(k, v)| self.config_value_to_serde(v).map(|val| (k.clone(), val)))
                    .collect();
                serde_json::Value::Object(json_obj?)
            }
        };
        Ok(result)
    }
    
    /// 添加配置变更监听器
    pub fn add_listener(&mut self, listener: Box<dyn ConfigChangeListener>) {
        self.listeners.push(listener);
    }
}

/// 配置变更监听器
pub trait ConfigChangeListener: Send + Sync + Debug {
    /// 配置变更回调
    fn on_config_changed(&self, key: &str, old_value: Option<&ConfigValue>, new_value: &ConfigValue);
}

/// 配置组件实现
#[derive(Debug)]
pub struct ConfigComponent<T> {
    config: T,
    key_prefix: String,
}

impl<T> ConfigComponent<T>
where
    T: DeserializeOwned + Clone + Send + Sync + Debug + 'static,
{
    pub fn new(key_prefix: &str) -> ContainerResult<Self> {
        let mut manager = CONFIG_MANAGER.write().unwrap();
        let config = manager.get_config::<T>()?;
        
        Ok(Self {
            config,
            key_prefix: key_prefix.to_string(),
        })
    }
    
    /// 获取配置
    pub fn get(&self) -> &T {
        &self.config
    }
    
    /// 重新加载配置
    pub fn reload(&mut self) -> ContainerResult<()> {
        let mut manager = CONFIG_MANAGER.write().unwrap();
        self.config = manager.get_config::<T>()?;
        Ok(())
    }
}

impl<T> Component for ConfigComponent<T>
where
    T: DeserializeOwned + Clone + Send + Sync + Debug + 'static,
{
    fn component_name() -> &'static str {
        "ConfigComponent"
    }
    
    fn lifecycle() -> Lifecycle {
        Lifecycle::Singleton
    }
}

/// 全局函数
pub fn get_config_manager() -> Arc<RwLock<ConfigManager>> {
    CONFIG_MANAGER.clone()
}

pub fn get_config_string(key: &str) -> Option<String> {
    let manager = CONFIG_MANAGER.read().unwrap();
    manager.get_string(key)
}

pub fn get_config_i64(key: &str) -> Option<i64> {
    let manager = CONFIG_MANAGER.read().unwrap();
    manager.get_i64(key)
}

pub fn get_config_f64(key: &str) -> Option<f64> {
    let manager = CONFIG_MANAGER.read().unwrap();
    manager.get_f64(key)
}

pub fn get_config_bool(key: &str) -> Option<bool> {
    let manager = CONFIG_MANAGER.read().unwrap();
    manager.get_bool(key)
}

pub fn set_config(key: &str, value: ConfigValue) {
    let mut manager = CONFIG_MANAGER.write().unwrap();
    manager.set(key, value);
}

pub fn add_config_source(source: ConfigSource) {
    let mut manager = CONFIG_MANAGER.write().unwrap();
    manager.add_source(source);
}

pub fn reload_config() {
    let mut manager = CONFIG_MANAGER.write().unwrap();
    manager.reload_all();
}

/// 环境变量注入宏
#[macro_export]
macro_rules! inject_env {
    ($var:expr) => {
        std::env::var($var).ok()
    };
    ($var:expr, $default:expr) => {
        std::env::var($var).unwrap_or_else(|_| $default.to_string())
    };
}

/// 配置注入宏
#[macro_export]
macro_rules! inject_config {
    ($key:expr) => {
        $crate::config::get_config_string($key)
    };
    ($key:expr, $default:expr) => {
        $crate::config::get_config_string($key).unwrap_or_else(|| $default.to_string())
    };
}
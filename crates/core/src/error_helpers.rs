/// 错误处理辅助工具
///
/// 提供便捷的方法来替换 unwrap() 调用，提供更好的错误信息和上下文
use crate::error::{ForgeError, ForgeResult};

/// 扩展 Option 和 Result 类型以提供更好的错误处理
pub trait UnwrapHelpers<T> {
    /// 替代 unwrap()，提供上下文信息
    fn unwrap_or_forge_error(
        self,
        context: &str,
    ) -> ForgeResult<T>;

    /// 在内部错误时提供默认值
    fn unwrap_or_internal_error(
        self,
        context: &str,
        location: &str,
    ) -> ForgeResult<T>;
}

impl<T> UnwrapHelpers<T> for Option<T> {
    fn unwrap_or_forge_error(
        self,
        context: &str,
    ) -> ForgeResult<T> {
        self.ok_or_else(|| ForgeError::Internal {
            message: format!("意外的 None 值: {context}"),
            location: None,
        })
    }

    fn unwrap_or_internal_error(
        self,
        context: &str,
        location: &str,
    ) -> ForgeResult<T> {
        self.ok_or_else(|| ForgeError::Internal {
            message: format!("意外的 None 值: {context}"),
            location: Some(location.to_string()),
        })
    }
}

impl<T, E> UnwrapHelpers<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn unwrap_or_forge_error(
        self,
        context: &str,
    ) -> ForgeResult<T> {
        self.map_err(|e| ForgeError::Internal {
            message: format!("操作失败: {context}"),
            location: Some(e.to_string()),
        })
    }

    fn unwrap_or_internal_error(
        self,
        context: &str,
        location: &str,
    ) -> ForgeResult<T> {
        self.map_err(|e| ForgeError::Internal {
            message: format!("操作失败: {context}"),
            location: Some(format!("{location}: {e}")),
        })
    }
}

/// 锁操作的辅助函数
pub mod lock_helpers {
    use super::*;
    use std::sync::{RwLock, Mutex};

    /// 安全地获取读锁，提供错误上下文
    pub fn read_lock<'a, T>(
        lock: &'a RwLock<T>,
        context: &str,
    ) -> ForgeResult<std::sync::RwLockReadGuard<'a, T>> {
        lock.read().map_err(|_| ForgeError::Concurrency {
            message: format!("无法获取读锁: {context}"),
            source: None,
        })
    }

    /// 安全地获取写锁，提供错误上下文
    pub fn write_lock<'a, T>(
        lock: &'a RwLock<T>,
        context: &str,
    ) -> ForgeResult<std::sync::RwLockWriteGuard<'a, T>> {
        lock.write().map_err(|_| ForgeError::Concurrency {
            message: format!("无法获取写锁: {context}"),
            source: None,
        })
    }

    /// 安全地获取互斥锁，提供错误上下文
    pub fn mutex_lock<'a, T>(
        lock: &'a Mutex<T>,
        context: &str,
    ) -> ForgeResult<std::sync::MutexGuard<'a, T>> {
        lock.lock().map_err(|_| ForgeError::Concurrency {
            message: format!("无法获取互斥锁: {context}"),
            source: None,
        })
    }
}

/// 集合操作的辅助函数
pub mod collection_helpers {
    use super::*;
    use std::collections::HashMap;

    /// 安全地从 HashMap 获取值
    pub fn get_required<'a, K, V>(
        map: &'a HashMap<K, V>,
        key: &K,
        context: &str,
    ) -> ForgeResult<&'a V>
    where
        K: std::hash::Hash + Eq + std::fmt::Debug,
    {
        map.get(key).ok_or_else(|| ForgeError::Internal {
            message: format!("必需的键不存在: {key:?} ({context})"),
            location: None,
        })
    }

    /// 安全地从 Vec 获取值
    pub fn get_at_index<'a, T>(
        vec: &'a [T],
        index: usize,
        context: &str,
    ) -> ForgeResult<&'a T> {
        vec.get(index).ok_or_else(|| ForgeError::Internal {
            message: format!(
                "索引 {} 超出范围，长度: {} ({})",
                index,
                vec.len(),
                context
            ),
            location: None,
        })
    }
}

/// Schema 相关的辅助函数
pub mod schema_helpers {
    use super::*;
    use mf_model::schema::{Schema, SchemaSpec};
    use std::sync::Arc;

    /// 安全编译 schema
    pub fn compile_schema(
        spec: SchemaSpec,
        context: &str,
    ) -> ForgeResult<Arc<Schema>> {
        Schema::compile(spec).map(Arc::new).map_err(|e| {
            ForgeError::Validation {
                message: format!("Schema 编译失败: {e} ({context})"),
                field: None,
            }
        })
    }
}

/// 状态管理的辅助函数
pub mod state_helpers {
    use super::*;

    /// 安全地获取可变状态
    pub fn get_mut_state<'a, T>(
        option: &'a mut Option<T>,
        context: &str,
    ) -> ForgeResult<&'a mut T> {
        option.as_mut().ok_or_else(|| ForgeError::State {
            message: format!("状态未初始化: {context}"),
            source: None,
        })
    }

    /// 安全地获取不可变状态
    pub fn get_state<'a, T>(
        option: &'a Option<T>,
        context: &str,
    ) -> ForgeResult<&'a T> {
        option.as_ref().ok_or_else(|| ForgeError::State {
            message: format!("状态未初始化: {context}"),
            source: None,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_helpers() {
        let some_val: Option<i32> = Some(42);
        let none_val: Option<i32> = None;

        assert!(some_val.unwrap_or_forge_error("测试值").is_ok());
        assert!(none_val.unwrap_or_forge_error("空值测试").is_err());
    }

    #[test]
    fn test_collection_helpers() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert("key1", "value1");

        assert!(
            collection_helpers::get_required(&map, &"key1", "存在的键").is_ok()
        );
        assert!(
            collection_helpers::get_required(&map, &"key2", "不存在的键")
                .is_err()
        );
    }
}

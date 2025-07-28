use serde::{Deserialize, Serialize};

/// 组件生命周期类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lifecycle {
    /// 单例 - 全局唯一实例
    Singleton,
    /// 瞬态 - 每次请求创建新实例
    Transient,
    /// 作用域 - 在特定作用域内单例
    Scoped,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::Singleton
    }
}

impl std::fmt::Display for Lifecycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Singleton => write!(f, "Singleton"),
            Self::Transient => write!(f, "Transient"),
            Self::Scoped => write!(f, "Scoped"),
        }
    }
}
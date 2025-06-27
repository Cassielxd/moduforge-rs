use std::ops::Deref;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::fmt;

use serde::{Deserialize, Serialize};

/// 一个 Arc 的包装器，它基于指针地址进行 `PartialEq`, `Eq` 和 `Hash`。
///
/// 这使得我们可以将它放入集合中，并确保比较的是引用而不是内容。
#[derive(Clone,Serialize,Deserialize)]
pub struct ArcPtr<T>(pub Arc<T>);


// 1. 实现 PartialEq，核心逻辑在这里
impl<T> PartialEq for ArcPtr<T> {
    /// 比较两个 ArcPtr 是否相等。
    ///
    /// 这个实现总是使用 `Arc::ptr_eq`，即比较指针地址。
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

// 2. 实现 Eq，这是一个标记 trait，表示 `eq` 是一个等价关系
impl<T> Eq for ArcPtr<T> {}

// 3. 实现 Hash，以便可以用于 HashMap 的键
impl<T> Hash for ArcPtr<T> {
    /// 基于 Arc 的指针地址计算哈希值。
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 将指针地址转换为一个整数，然后进行哈希
        (Arc::as_ptr(&self.0) as *const () as usize).hash(state);
    }
}

// 4. (可选但推荐) 实现 Deref，方便直接访问内部数据
impl<T> Deref for ArcPtr<T> {
    type Target = T;

    /// 允许我们通过 `.` 操作符直接调用内部 Arc<T> 指向的数据的方法。
    /// 例如 `arc_ptr_wrapper.some_node_method()`
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 5. (可选但推荐) 实现 Debug，方便打印
impl<T: fmt::Debug> fmt::Debug for ArcPtr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 格式化输出时，同时显示内部数据和指针地址，便于调试
        f.debug_struct("ArcPtr")
         .field("ptr", &Arc::as_ptr(&self.0))
         .field("data", &self.0)
         .finish()
    }
}

// 辅助函数，方便创建
impl<T> ArcPtr<T> {
    pub fn new(inner: T) -> Self {
        Self(Arc::new(inner))
    }
}

use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

pub trait Resource: Any + Send + Sync + 'static {}

impl dyn Resource {
    #[inline(always)]
    fn is<T: Resource>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    #[inline(always)]
    #[allow(clippy::needless_lifetimes)]
    pub fn downcast_arc<'a, T: Resource>(
        self: &'a Arc<Self>
    ) -> Option<&'a Arc<T>> {
        if self.is::<T>() {
            let ptr = self as *const Arc<_> as *const Arc<T>;
            // SAFETY: 这个转换是安全的，因为：
            // 1. 我们通过 `self.is::<T>()` 验证了运行时类型匹配（TypeId 相等）
            // 2. T 实现了 Resource trait，确保 T: Any + Send + Sync + 'static
            // 3. Arc<dyn Resource> 和 Arc<T> 具有相同的内存布局和大小
            //    （Arc 是指针包装器，存储堆上的数据指针和引用计数）
            // 4. 我们只改变了指针的类型标注，不改变底层数据
            // 5. 生命周期 'a 保持不变，确保借用检查器的安全性
            // 6. 返回的是不可变引用，不会破坏内存安全
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }
    #[inline(always)]
    #[allow(clippy::needless_lifetimes)]
    pub fn downcast<'a, T: Resource>(&'a self) -> Option<&'a T> {
        if self.is::<T>() {
            let ptr = self as *const dyn Resource as *const T;
            // SAFETY: 这个转换是安全的，因为：
            // 1. 我们通过 `self.is::<T>()` 验证了运行时类型匹配（TypeId 相等）
            // 2. T 实现了 Resource trait，确保 T: Any + Send + Sync + 'static
            // 3. dyn Resource 和 T 的内存布局兼容
            // 4. 我们只改变了指针的类型标注，不改变底层数据
            // 5. 生命周期 'a 保持不变，确保借用检查器的安全性
            // 6. 返回的是不可变引用，不会破坏内存安全
            Some(unsafe { &*ptr })
        } else {
            None
        }
    }
}

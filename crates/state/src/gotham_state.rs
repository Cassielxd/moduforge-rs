use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

use dashmap::DashMap;

#[derive(Default, Debug)]
pub struct GothamState {
    /// 使用BTreeMap存储不同类型的数据，以TypeId为键
    data: DashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl GothamState {
    /// 将数据存入状态容器中
    ///
    /// # 参数
    /// * `t` - 要存储的数据，必须是'static生命周期
    pub fn put<T: Clone + Send + Sync + 'static>(
        &self,
        t: T,
    ) {
        let type_id = TypeId::of::<T>();
        self.data.insert(type_id, Arc::new(t));
    }

    /// 检查状态容器中是否包含指定类型的数据
    ///
    /// # 参数
    /// * 泛型参数T - 要检查的类型
    ///
    /// # 返回值
    /// * 如果存在返回true，否则返回false
    pub fn has<T: Send + Sync + 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.data.contains_key(&type_id)
    }
    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.try_get::<T>()
    }

    pub fn try_get<T: Clone + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        if let Some(v) = self.data.get(&type_id) {
            let value = v.value().clone();
            Arc::downcast(value).ok()
        } else {
            None
        }
    }

    pub fn try_take<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        match self.data.remove(&type_id) {
            Some((_, v)) => Arc::downcast(v).ok(),
            None => None,
        }
    }
    /// 从状态容器中移除并返回指定类型的数据
    ///
    /// # 参数
    /// * 泛型参数T - 要移除的类型
    ///
    /// # 返回值
    /// * 返回T，如果数据不存在则返回None
    pub fn take<T: Send + Sync + 'static>(&mut self) -> Option<Arc<T>> {
        self.try_take::<T>()
    }
}



use std::{any::{type_name, Any, TypeId}, ops::{Deref, DerefMut}, sync::Arc};

use dashmap::DashMap;

use crate::resource_table::ResourceTable;

/// 全局资源管理器
/// 用于管理编辑器运行时的全局资源和状态
/// - resource_table: 资源表，管理所有注册的资源
/// - gotham_state: Gotham状态，管理特定于Gotham框架的状态
#[derive(Default, Debug)]
pub struct GlobalResourceManager {
    pub resource_table: ResourceTable,
    pub gotham_state: DashMap<TypeId, Box<dyn Any>>,
}

// 实现Send和Sync trait，表明GlobalResourceManager可以在线程间安全传递和共享
unsafe impl Send for GlobalResourceManager {}
unsafe impl Sync for GlobalResourceManager {}

impl GlobalResourceManager {
    /// 创建新的全局资源管理器实例
    pub fn new() -> Self {
        Self {
            resource_table: ResourceTable::default(),
            gotham_state: DashMap::new(),
        }
    }

    /// 清理所有资源
    /// 使用std::mem::take来安全地获取并重置内部状态
    pub fn clear(&mut self) {
        std::mem::take(&mut self.gotham_state);
        std::mem::take(&mut self.resource_table);
    }    
    /// 将数据存入状态容器中
    ///
    /// # 参数
    /// * `t` - 要存储的数据，必须是'static生命周期
    pub fn put<T: Clone+ 'static>(
        &self,
        t: T,
    ) {
        let type_id = TypeId::of::<T>();
        self.gotham_state.insert(type_id, Box::new(t));
    }

    /// 检查状态容器中是否包含指定类型的数据
    ///
    /// # 参数
    /// * 泛型参数T - 要检查的类型
    ///
    /// # 返回值
    /// * 如果存在返回true，否则返回false
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.gotham_state.contains_key(&type_id)
    }
    pub fn get<T: Clone+'static>(&self) -> T {
        self.try_get::<T>().unwrap_or_else(|| missing::<T>())
    }
    pub fn try_get<T:Clone+ 'static>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let data = self.gotham_state.get(&type_id);
        match data {
         Some(v) => {
             let value = v.value().as_ref().downcast_ref::<T>();
             value.cloned()
         },
         None => None,
        }
    }

    pub fn try_take<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        match self.gotham_state.remove(&type_id) {
            Some((_, v)) => {
                let value = v.downcast::<T>();
                match value {
                    Ok(v) => Some(*v),
                    Err(e) => {
                        None
                    }
                }
            }
            None => None,
        }
    }
    /// 从状态容器中移除并返回指定类型的数据
    ///
    /// # 参数
    /// * 泛型参数T - 要移除的类型
    ///
    /// # 返回值
    /// * 返回T，如果数据不存在则panic
    pub fn take<T: 'static>(&mut self) -> T {
        self.try_take().unwrap_or_else(|| missing::<T>())
    }
}
fn missing<T: 'static>() -> ! {
    panic!(
        "required type {} is not present in GothamState container",
        type_name::<T>()
    );
}
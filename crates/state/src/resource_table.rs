use std::sync::Arc;
use std::fmt::{self, Debug};

use dashmap::DashMap;

use crate::resource::Resource;

// 资源ID类型定义
pub type ResourceId = String;

// 资源表结构体，用于管理所有资源
#[derive(Default)]
pub struct ResourceTable {
    // 使用BTreeMap存储资源ID到资源的映射
    index: DashMap<ResourceId, Arc<dyn Resource>>,
}
impl Debug for ResourceTable {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "ResourceTable {{ len: {} }}", self.index.len())
    }
}
impl ResourceTable {
    // 获取资源表中资源的数量
    pub fn len(&self) -> usize {
        self.index.len()
    }

    // 检查资源表是否为空
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    // 添加一个新资源到资源表，返回分配的资源ID
    pub fn add<T: Resource>(
        &self,
        rid: ResourceId,
        resource: T,
    ) {
        self.add_arc(rid, Arc::new(resource));
    }

    // 添加一个Arc包装的资源到资源表
    pub fn add_arc<T: Resource>(
        &self,
        rid: ResourceId,
        resource: Arc<T>,
    ) {
        let resource = resource as Arc<dyn Resource>;
        self.add_arc_dyn(rid, resource);
    }

    // 添加一个动态类型的Arc资源到资源表
    pub fn add_arc_dyn(
        &self,
        rid: ResourceId,
        resource: Arc<dyn Resource>,
    ) {
        self.index.insert(rid, resource);
    }

    // 检查指定ID的资源是否存在
    pub fn has(
        &self,
        rid: ResourceId,
    ) -> bool {
        self.index.contains_key(&rid)
    }

    // 获取指定ID的特定类型资源
    pub fn get<T: Resource>(
        &self,
        rid: ResourceId,
    ) -> Option<Arc<T>> {
        
        self
            .index
            .get(&rid)
            .map(|rc| rc.value().clone())
            .and_then(|rc| rc.downcast_arc::<T>().cloned())
    }

    // 获取指定ID的任意类型资源
    pub fn get_any(
        &self,
        rid: ResourceId,
    ) -> Option<Arc<dyn Resource>> {
        self.index.get(&rid).map(|rc| rc.value().clone())
    }

    // 从资源表中移除并返回指定ID的特定类型资源
    pub fn take<T: Resource>(
        &self,
        rid: ResourceId,
    ) -> Option<Arc<T>> {
        let (_, resource) = self.index.remove(&rid)?;
        resource.downcast_arc::<T>().cloned()
    }

    // 从资源表中移除并返回指定ID的任意类型资源
    pub fn take_any(
        &self,
        rid: ResourceId,
    ) -> Option<Arc<dyn Resource>> {
        self.index.remove(&rid).map(|rc| rc.1)
    }
}

// 资源错误类型定义
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("null or invalid handle")]
    Reference,
    #[error("Bad resource ID")]
    BadResourceId,
    #[error("Resource is unavailable because it is in use by a promise")]
    Unavailable,
    #[error("{0}")]
    Other(String),
}

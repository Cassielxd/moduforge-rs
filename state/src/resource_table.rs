use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::resource::Resource;

// 资源ID类型定义
pub type ResourceId = String;

// 资源表结构体，用于管理所有资源
#[derive(Default, Debug)]
pub struct ResourceTable {
    // 使用BTreeMap存储资源ID到资源的映射
    index: BTreeMap<ResourceId, Arc<dyn Resource>>,
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
        &mut self,
        rid: ResourceId,
        resource: T,
    ) {
        self.add_arc(rid, Arc::new(resource));
    }

    // 添加一个Arc包装的资源到资源表
    pub fn add_arc<T: Resource>(
        &mut self,
        rid: ResourceId,
        resource: Arc<T>,
    ) {
        let resource = resource as Arc<dyn Resource>;
        self.add_arc_dyn(rid, resource);
    }

    // 添加一个动态类型的Arc资源到资源表
    pub fn add_arc_dyn(
        &mut self,
        rid: ResourceId,
        resource: Arc<dyn Resource>,
    ){
        let removed_resource = self.index.insert(rid, resource);
        assert!(removed_resource.is_none());

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
    ) -> Result<Arc<T>, ResourceError> {
        self.index
            .get(&rid)
            .and_then(|rc| rc.downcast_arc::<T>())
            .cloned()
            .ok_or(ResourceError::BadResourceId)
    }

    // 获取指定ID的任意类型资源
    pub fn get_any(
        &self,
        rid: ResourceId,
    ) -> Result<Arc<dyn Resource>, ResourceError> {
        self.index.get(&rid).cloned().ok_or(ResourceError::BadResourceId)
    }

    // 替换指定ID的资源
    pub fn replace<T: Resource>(
        &mut self,
        rid: ResourceId,
        resource: T,
    ) {
        let result =
            self.index.insert(rid, Arc::new(resource) as Arc<dyn Resource>);
        assert!(result.is_some());
    }

    // 从资源表中移除并返回指定ID的特定类型资源
    pub fn take<T: Resource>(
        &mut self,
        rid: ResourceId,
    ) -> Result<Arc<T>, ResourceError> {
        let resource = self.get::<T>(rid.clone())?;
        self.index.remove(&rid);
        Ok(resource)
    }

    // 从资源表中移除并返回指定ID的任意类型资源
    pub fn take_any(
        &mut self,
        rid: ResourceId,
    ) -> Result<Arc<dyn Resource>, ResourceError> {
        self.index.remove(&rid).ok_or(ResourceError::BadResourceId)
    }


    // 获取所有资源的名称
    pub fn names(&self) -> impl Iterator<Item = (ResourceId, Cow<str>)> {
        self.index.iter().map(|(id, resource)| (id.clone(), resource.name()))
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

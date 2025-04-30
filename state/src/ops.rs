use std::ops::{Deref, DerefMut};


use crate::{gotham_state::GothamState, resource_table::ResourceTable};

/// 全局资源管理器
/// 用于管理编辑器运行时的全局资源和状态
/// - resource_table: 资源表，管理所有注册的资源
/// - gotham_state: Gotham状态，管理特定于Gotham框架的状态
#[derive(Default, Debug)]
pub struct GlobalResourceManager {
    pub resource_table: ResourceTable,
    pub gotham_state: GothamState,
}

// 实现Send和Sync trait，表明GlobalResourceManager可以在线程间安全传递和共享
unsafe impl Send for GlobalResourceManager {}
unsafe impl Sync for GlobalResourceManager {}

impl GlobalResourceManager {
    /// 创建新的全局资源管理器实例
    pub fn new() -> Self {
        Self {
            resource_table: ResourceTable::default(),
            gotham_state: GothamState::default(),
        }
    }

    /// 清理所有资源
    /// 使用std::mem::take来安全地获取并重置内部状态
    pub fn clear(&mut self) {
        std::mem::take(&mut self.gotham_state);
        std::mem::take(&mut self.resource_table);
    }    
  
}
impl Deref for GlobalResourceManager {
    type Target = GothamState;

    fn deref(&self) -> &Self::Target {
        &self.gotham_state
    }
}

impl DerefMut for GlobalResourceManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gotham_state
    }
}
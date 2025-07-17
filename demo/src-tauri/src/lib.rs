#[macro_use]
extern crate lazy_static;
// 服务层
pub mod serve;
// 节点层
pub mod nodes;
// 标记层
pub mod marks;
// 插件层
pub mod plugins;
// 扩展层
pub mod exetensions;
// 工具层
pub mod utils;
// 初始化层
pub mod initialize;

// 核心层
pub mod core;

// 控制器层
pub mod controller;

pub mod response;

pub mod commands;

pub mod middleware;
pub mod router;

pub mod error;

pub mod types;

pub type ResponseResult<T> = Result<response::Res<T>, error::AppError>;

use dashmap::{mapref::one::RefMut, DashMap};
use state::TypeMap;

use crate::{core::demo_editor::DemoEditor, types::{EditorTrait}};
static APPLICATION_CONTEXT: TypeMap![Send + Sync] = <TypeMap![Send + Sync]>::new();
/// 全局工具类
pub struct ContextHelper;

impl ContextHelper {
    /// 设置全局变量
    pub fn set<T: Send + Sync + 'static>(v: T) {
        APPLICATION_CONTEXT.set(v);
    }
    /// 设置线程局部变量
    pub fn set_local<T, F>(&self, state_init: F) -> bool
    where
        T: Send + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        APPLICATION_CONTEXT.set_local(state_init)
    }
    /// 获取线程局部变量
    pub fn get_local<T: Send + 'static>(&self) -> &T {
        APPLICATION_CONTEXT.get_local::<T>()
    }
    /// 获取全局变量
    pub fn try_get<T: Send + Sync + 'static>() -> Option<&'static T> {
        APPLICATION_CONTEXT.try_get::<T>()
    }
    /// 获取全局变量
    pub fn get<T: Send + Sync + 'static>() -> &'static T {
        APPLICATION_CONTEXT.get::<T>()
    }
    /// 获取价格编辑器
    pub fn get_editor(name: &str) -> Option<RefMut<'static, String, Box<dyn EditorTrait>>> {
        let map = ContextHelper::get::<DashMap<String, Box<dyn EditorTrait>>>();
        map.get_mut(name)
    }
    /// 设置价格编辑器
    pub fn set_editor(name: &str, editor: Box<dyn EditorTrait>) {
        let map = ContextHelper::get::<DashMap<String, Box<dyn EditorTrait>>>();
        map.insert(name.to_string(), editor);
    }
}

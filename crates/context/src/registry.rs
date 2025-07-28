use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::{
    component::{Component, ComponentFactory, ComponentInfo, ComponentInstance},
    error::{ContainerError, ContainerResult},
    lifecycle::Lifecycle,
};

/// 全局组件注册表
static COMPONENT_REGISTRY: Lazy<ComponentRegistry> = Lazy::new(ComponentRegistry::new);

/// 组件注册表
pub struct ComponentRegistry {
    /// 按名称索引的组件
    components_by_name: DashMap<String, ComponentInstance>,
    /// 按类型ID索引的组件
    components_by_type: DashMap<TypeId, String>,
    /// 组件依赖图
    dependency_graph: RwLock<HashMap<String, Vec<String>>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components_by_name: DashMap::new(),
            components_by_type: DashMap::new(),
            dependency_graph: RwLock::new(HashMap::new()),
        }
    }
    
    /// 注册组件
    pub fn register_component<T: Component + 'static>(
        &self,
        factory: ComponentFactory,
    ) -> ContainerResult<()> {
        self.register_component_with_auto_proxy::<T>(factory, false)
    }
    
    /// 注册组件（支持自动代理配置）
    pub fn register_component_with_auto_proxy<T: Component + 'static>(
        &self,
        factory: ComponentFactory,
        auto_proxy: bool,
    ) -> ContainerResult<()> {
        let type_id = TypeId::of::<T>();
        let component_name = T::component_name().to_string();
        
        // 检查是否已存在
        if self.components_by_name.contains_key(&component_name) {
            return Err(ContainerError::ComponentAlreadyExists {
                name: component_name,
            });
        }
        
        let info = ComponentInfo::new_with_auto_proxy::<T>(auto_proxy);
        let instance = ComponentInstance {
            info,
            instance: None,
            factory,
        };
        
        self.components_by_name.insert(component_name.clone(), instance);
        self.components_by_type.insert(type_id, component_name);
        
        Ok(())
    }
    
    /// 通过名称获取组件
    pub fn get_component_by_name(&self, name: &str) -> Option<dashmap::mapref::one::Ref<String, ComponentInstance>> {
        self.components_by_name.get(name)
    }
    
    /// 通过类型获取组件
    pub fn get_component_by_type<T: 'static>(&self) -> Option<dashmap::mapref::one::Ref<String, ComponentInstance>> {
        let type_id = TypeId::of::<T>();
        if let Some(name_ref) = self.components_by_type.get(&type_id) {
            let name = name_ref.value().clone();
            drop(name_ref);
            self.components_by_name.get(&name)
        } else {
            None
        }
    }
    
    /// 获取所有组件信息
    pub fn get_all_components(&self) -> Vec<ComponentInfo> {
        self.components_by_name
            .iter()
            .map(|entry| entry.value().info.clone())
            .collect()
    }
    
    /// 添加依赖关系
    pub fn add_dependency(&self, component: &str, dependency: &str) {
        let mut graph = self.dependency_graph.write();
        graph.entry(component.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.to_string());
    }
    
    /// 检测循环依赖
    pub fn check_circular_dependencies(&self) -> ContainerResult<()> {
        let graph = self.dependency_graph.read();
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        
        for component in graph.keys() {
            if !visited.contains(component) {
                if let Some(cycle) = self.dfs_check_cycle(
                    component,
                    &graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                ) {
                    return Err(ContainerError::CircularDependency { components: cycle });
                }
            }
        }
        
        Ok(())
    }
    
    fn dfs_check_cycle(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(dependencies) = graph.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    if let Some(cycle) = self.dfs_check_cycle(dep, graph, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    // 找到循环，构建循环路径
                    if let Some(cycle_start) = path.iter().position(|x| x == dep) {
                        let mut cycle = path[cycle_start..].to_vec();
                        cycle.push(dep.to_string());
                        return Some(cycle);
                    }
                }
            }
        }
        
        rec_stack.remove(node);
        path.pop();
        None
    }
}

/// 自动注册组件（由宏调用）
pub fn auto_register_component<T: Component + Default + 'static>() {
    auto_register_component_with_auto_proxy::<T>(false);
}

/// 自动注册组件（支持自动代理配置，由宏调用）
pub fn auto_register_component_with_auto_proxy<T: Component + Default + 'static>(auto_proxy: bool) {
    let factory: ComponentFactory = Box::new(|_resolver| {
        Box::pin(async move {
            let instance = T::default();
            instance.initialize().await?;
            Ok(Arc::new(instance) as Arc<dyn Any + Send + Sync>)
        })
    });
    
    if let Err(error) = COMPONENT_REGISTRY.register_component_with_auto_proxy::<T>(factory, auto_proxy) {
        eprintln!("Failed to register component {}: {}", T::component_name(), error);
    }
}

/// 注册Bean工厂（由宏调用）
pub fn register_bean_factory<F, Fut, T>(
    name: &str,
    factory: F,
    lifecycle: Lifecycle,
) where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ContainerResult<Arc<dyn Any + Send + Sync>>> + Send + 'static,
    T: Any + Send + Sync + 'static,
{
    let factory_fn: ComponentFactory = Box::new(move |_resolver| {
        Box::pin(factory())
    });
    
    // 创建虚拟组件信息
    let info = ComponentInfo {
        name: name.to_string(),
        type_name: std::any::type_name::<T>().to_string(),
        type_id: format!("{:?}", TypeId::of::<T>()),
        lifecycle,
        dependencies: Vec::new(),
        initialized: false,
        auto_proxy: false, // Bean工厂默认不启用自动代理
    };
    
    let instance = ComponentInstance {
        info,
        instance: None,
        factory: factory_fn,
    };
    
    COMPONENT_REGISTRY.components_by_name.insert(name.to_string(), instance);
    COMPONENT_REGISTRY.components_by_type.insert(TypeId::of::<T>(), name.to_string());
}

/// 获取全局注册表
pub fn global_registry() -> &'static ComponentRegistry {
    &COMPONENT_REGISTRY
}
// 不可变状态管理的替代方案
// 避免 ImHashMap 的性能开销同时保持不可变性

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    rc::Rc,
    cell::RefCell,
    time::Instant,
};
use serde::{Serialize, Deserialize};

// ==================== 方案1: Copy-on-Write (COW) 策略 ====================

#[derive(Clone, Debug)]
pub struct CowState {
    // 使用 Arc 包装，只在需要修改时克隆
    fields: Arc<HashMap<String, StateValue>>,
    version: u64,
    // 标记是否为共享状态
    is_shared: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateValue {
    String(String),
    Number(i64),
    Bool(bool),
    Object(HashMap<String, StateValue>),
}

impl CowState {
    pub fn new() -> Self {
        Self {
            fields: Arc::new(HashMap::new()),
            version: 0,
            is_shared: false,
        }
    }

    // 获取值 - 零拷贝
    pub fn get(&self, key: &str) -> Option<&StateValue> {
        self.fields.get(key)
    }

    // 设置值 - 只在需要时克隆
    pub fn set(&mut self, key: String, value: StateValue) -> &mut Self {
        // 如果当前状态被共享，则克隆
        if Arc::strong_count(&self.fields) > 1 {
            self.fields = Arc::new((*self.fields).clone());
        }
        
        // 现在可以安全地修改
        Arc::get_mut(&mut self.fields).unwrap().insert(key, value);
        self.version += 1;
        self
    }

    // 创建快照 - 零拷贝
    pub fn snapshot(&self) -> Self {
        Self {
            fields: Arc::clone(&self.fields),
            version: self.version,
            is_shared: true,
        }
    }
}

// ==================== 方案2: 事件溯源模式 ====================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateEvent {
    Set { key: String, value: StateValue },
    Delete { key: String },
    Batch { events: Vec<StateEvent> },
}

#[derive(Clone, Debug)]
pub struct EventSourcedState {
    // 基础状态快照
    base_snapshot: Arc<HashMap<String, StateValue>>,
    // 自快照以来的事件
    events: Vec<StateEvent>,
    // 快照版本
    snapshot_version: u64,
    // 当前版本
    current_version: u64,
    // 重建阈值
    rebuild_threshold: usize,
}

impl EventSourcedState {
    pub fn new() -> Self {
        Self {
            base_snapshot: Arc::new(HashMap::new()),
            events: Vec::new(),
            snapshot_version: 0,
            current_version: 0,
            rebuild_threshold: 100, // 超过100个事件时重建快照
        }
    }

    pub fn apply_event(&mut self, event: StateEvent) {
        self.events.push(event);
        self.current_version += 1;

        // 如果事件太多，重建快照
        if self.events.len() > self.rebuild_threshold {
            self.create_snapshot();
        }
    }

    pub fn get(&self, key: &str) -> Option<StateValue> {
        // 首先从基础快照获取
        let mut value = self.base_snapshot.get(key).cloned();

        // 然后应用所有事件
        for event in &self.events {
            match event {
                StateEvent::Set { key: event_key, value: event_value } => {
                    if event_key == key {
                        value = Some(event_value.clone());
                    }
                }
                StateEvent::Delete { key: event_key } => {
                    if event_key == key {
                        value = None;
                    }
                }
                StateEvent::Batch { events } => {
                    for sub_event in events {
                        // 递归处理批量事件
                        match sub_event {
                            StateEvent::Set { key: sub_key, value: sub_value } => {
                                if sub_key == key {
                                    value = Some(sub_value.clone());
                                }
                            }
                            StateEvent::Delete { key: sub_key } => {
                                if sub_key == key {
                                    value = None;
                                }
                            }
                            _ => {} // 忽略嵌套的批量事件
                        }
                    }
                }
            }
        }

        value
    }

    pub fn set(&mut self, key: String, value: StateValue) {
        self.apply_event(StateEvent::Set { key, value });
    }

    fn create_snapshot(&mut self) {
        // 重建完整状态
        let mut new_snapshot = (*self.base_snapshot).clone();
        
        for event in &self.events {
            self.apply_event_to_map(&mut new_snapshot, event);
        }

        self.base_snapshot = Arc::new(new_snapshot);
        self.events.clear();
        self.snapshot_version = self.current_version;
    }

    fn apply_event_to_map(&self, map: &mut HashMap<String, StateValue>, event: &StateEvent) {
        match event {
            StateEvent::Set { key, value } => {
                map.insert(key.clone(), value.clone());
            }
            StateEvent::Delete { key } => {
                map.remove(key);
            }
            StateEvent::Batch { events } => {
                for sub_event in events {
                    self.apply_event_to_map(map, sub_event);
                }
            }
        }
    }

    // 创建历史版本 - 只存储事件
    pub fn get_version(&self, version: u64) -> Option<HashMap<String, StateValue>> {
        if version > self.current_version {
            return None;
        }

        let mut state = (*self.base_snapshot).clone();
        let events_to_apply = if version > self.snapshot_version {
            &self.events[..(version - self.snapshot_version) as usize]
        } else {
            // 需要从更早的快照重建（这里简化处理）
            return None;
        };

        for event in events_to_apply {
            self.apply_event_to_map(&mut state, event);
        }

        Some(state)
    }
}

// ==================== 方案3: 分层状态管理 ====================

#[derive(Clone, Debug)]
pub struct LayeredState {
    // 全局状态层（很少变化）
    global_layer: Arc<HashMap<String, StateValue>>,
    // 会话状态层（中等频率变化）
    session_layer: Arc<HashMap<String, StateValue>>,
    // 本地状态层（高频变化）
    local_layer: HashMap<String, StateValue>,
    version: u64,
}

impl LayeredState {
    pub fn new() -> Self {
        Self {
            global_layer: Arc::new(HashMap::new()),
            session_layer: Arc::new(HashMap::new()),
            local_layer: HashMap::new(),
            version: 0,
        }
    }

    pub fn get(&self, key: &str) -> Option<&StateValue> {
        // 从本地层开始查找
        self.local_layer.get(key)
            .or_else(|| self.session_layer.get(key))
            .or_else(|| self.global_layer.get(key))
    }

    pub fn set_local(&mut self, key: String, value: StateValue) {
        self.local_layer.insert(key, value);
        self.version += 1;
    }

    pub fn set_session(&mut self, key: String, value: StateValue) {
        // 只在需要时克隆会话层
        if Arc::strong_count(&self.session_layer) > 1 {
            self.session_layer = Arc::new((*self.session_layer).clone());
        }
        Arc::get_mut(&mut self.session_layer).unwrap().insert(key, value);
        self.version += 1;
    }

    pub fn set_global(&mut self, key: String, value: StateValue) {
        // 只在需要时克隆全局层
        if Arc::strong_count(&self.global_layer) > 1 {
            self.global_layer = Arc::new((*self.global_layer).clone());
        }
        Arc::get_mut(&mut self.global_layer).unwrap().insert(key, value);
        self.version += 1;
    }

    // 创建快照 - 大部分层都是零拷贝
    pub fn snapshot(&self) -> Self {
        Self {
            global_layer: Arc::clone(&self.global_layer),
            session_layer: Arc::clone(&self.session_layer),
            local_layer: self.local_layer.clone(), // 只克隆本地层
            version: self.version,
        }
    }
}

// ==================== 方案4: 延迟克隆策略 ====================

#[derive(Debug)]
pub struct LazyCloneState {
    // 原始状态引用
    original: Option<Arc<HashMap<String, StateValue>>>,
    // 修改缓存
    modifications: HashMap<String, Option<StateValue>>, // None表示删除
    // 版本号
    version: u64,
}

impl Clone for LazyCloneState {
    fn clone(&self) -> Self {
        Self {
            original: self.original.clone(),
            modifications: HashMap::new(), // 新实例从空修改开始
            version: self.version,
        }
    }
}

impl LazyCloneState {
    pub fn new() -> Self {
        Self {
            original: Some(Arc::new(HashMap::new())),
            modifications: HashMap::new(),
            version: 0,
        }
    }

    pub fn get(&self, key: &str) -> Option<&StateValue> {
        // 首先检查修改缓存
        if let Some(modification) = self.modifications.get(key) {
            return modification.as_ref();
        }

        // 然后检查原始状态
        self.original.as_ref().and_then(|orig| orig.get(key))
    }

    pub fn set(&mut self, key: String, value: StateValue) {
        self.modifications.insert(key, Some(value));
        self.version += 1;
    }

    pub fn delete(&mut self, key: String) {
        self.modifications.insert(key, None);
        self.version += 1;
    }

    // 延迟合并 - 只在需要时执行
    pub fn materialize(&self) -> HashMap<String, StateValue> {
        let mut result = self.original
            .as_ref()
            .map(|orig| (**orig).clone())
            .unwrap_or_default();

        // 应用所有修改
        for (key, value) in &self.modifications {
            match value {
                Some(val) => { result.insert(key.clone(), val.clone()); }
                None => { result.remove(key); }
            }
        }

        result
    }

    // 如果修改太多，合并为新的原始状态
    pub fn compact(&mut self) {
        if self.modifications.len() > 50 { // 阈值
            let materialized = self.materialize();
            self.original = Some(Arc::new(materialized));
            self.modifications.clear();
        }
    }
}

// ==================== 方案5: 使用 Arena 分配器 ====================

use std::marker::PhantomData;

pub struct ArenaState<'a> {
    // 使用 Arena 分配器避免分散的堆分配
    arena: &'a Arena,
    // 存储指向 arena 中数据的指针
    fields: HashMap<String, ArenaRef<'a, StateValue>>,
    version: u64,
}

// 简化的 Arena 实现概念
pub struct Arena {
    // 实际实现会更复杂
    _phantom: PhantomData<()>,
}

pub struct ArenaRef<'a, T> {
    // 指向 arena 中数据的引用
    _phantom: PhantomData<&'a T>,
}

// ==================== 性能测试 ====================

pub fn benchmark_alternatives() {
    println!("=== 不可变状态替代方案性能测试 ===\n");

    let iterations = 10000;
    let initial_size = 1000;

    // 测试 COW 策略
    println!("测试 COW 策略:");
    let start = Instant::now();
    benchmark_cow_state(iterations, initial_size);
    println!("  耗时: {:?}\n", start.elapsed());

    // 测试事件溯源
    println!("测试事件溯源:");
    let start = Instant::now();
    benchmark_event_sourced(iterations, initial_size);
    println!("  耗时: {:?}\n", start.elapsed());

    // 测试分层状态
    println!("测试分层状态:");
    let start = Instant::now();
    benchmark_layered_state(iterations, initial_size);
    println!("  耗时: {:?}\n", start.elapsed());

    // 测试延迟克隆
    println!("测试延迟克隆:");
    let start = Instant::now();
    benchmark_lazy_clone(iterations, initial_size);
    println!("  耗时: {:?}\n", start.elapsed());
}

fn benchmark_cow_state(iterations: usize, initial_size: usize) {
    let mut state = CowState::new();
    
    // 初始化
    for i in 0..initial_size {
        state.set(format!("key_{}", i), StateValue::Number(i as i64));
    }

    // 创建多个快照并修改
    let mut snapshots = Vec::new();
    for i in 0..iterations {
        let snapshot = state.snapshot();
        snapshots.push(snapshot);
        
        state.set(format!("new_key_{}", i), StateValue::String(format!("value_{}", i)));
        
        if i % 1000 == 0 {
            println!("  COW 进度: {}/{}", i, iterations);
        }
    }
}

fn benchmark_event_sourced(iterations: usize, initial_size: usize) {
    let mut state = EventSourcedState::new();
    
    // 初始化
    for i in 0..initial_size {
        state.set(format!("key_{}", i), StateValue::Number(i as i64));
    }

    // 执行大量事件
    for i in 0..iterations {
        state.set(format!("key_{}", i % initial_size), StateValue::Number((i * 2) as i64));
        
        if i % 1000 == 0 {
            println!("  事件溯源进度: {}/{}", i, iterations);
        }
    }
}

fn benchmark_layered_state(iterations: usize, initial_size: usize) {
    let mut state = LayeredState::new();
    
    // 初始化全局层
    for i in 0..initial_size / 3 {
        state.set_global(format!("global_key_{}", i), StateValue::Number(i as i64));
    }

    // 执行混合层操作
    for i in 0..iterations {
        match i % 3 {
            0 => state.set_local(format!("local_key_{}", i), StateValue::Number(i as i64)),
            1 => state.set_session(format!("session_key_{}", i), StateValue::Number(i as i64)),
            2 => {
                if i % 100 == 0 { // 较少的全局修改
                    state.set_global(format!("global_key_{}", i), StateValue::Number(i as i64));
                }
            }
            _ => unreachable!(),
        }
        
        if i % 1000 == 0 {
            println!("  分层状态进度: {}/{}", i, iterations);
        }
    }
}

fn benchmark_lazy_clone(iterations: usize, initial_size: usize) {
    let mut state = LazyCloneState::new();
    
    // 通过直接设置原始状态来初始化（在实际实现中）
    for i in 0..initial_size {
        state.set(format!("key_{}", i), StateValue::Number(i as i64));
    }
    
    // 创建克隆并修改
    for i in 0..iterations {
        let mut cloned_state = state.clone();
        cloned_state.set(format!("new_key_{}", i), StateValue::String(format!("value_{}", i)));
        
        // 偶尔合并
        if i % 100 == 0 {
            cloned_state.compact();
        }
        
        if i % 1000 == 0 {
            println!("  延迟克隆进度: {}/{}", i, iterations);
        }
    }
}

// ==================== 使用建议 ====================

pub fn usage_recommendations() {
    println!("=== 不可变状态替代方案使用建议 ===\n");

    println!("1. COW (Copy-on-Write) 策略:");
    println!("   适用场景: 读多写少，偶尔需要快照");
    println!("   优势: 实现简单，内存效率高");
    println!("   劣势: 写时可能有突发性能开销\n");

    println!("2. 事件溯源模式:");
    println!("   适用场景: 需要完整历史记录，审计需求");
    println!("   优势: 历史查询快，存储空间小");
    println!("   劣势: 当前状态查询可能较慢\n");

    println!("3. 分层状态管理:");
    println!("   适用场景: 状态有明确的变化频率层次");
    println!("   优势: 针对性优化，减少不必要的克隆");
    println!("   劣势: 复杂度较高，需要合理设计层次\n");

    println!("4. 延迟克隆策略:");
    println!("   适用场景: 大量状态分叉，但实际修改较少");
    println!("   优势: 延迟开销，减少不必要的克隆");
    println!("   劣势: 内存使用可能不可预测\n");

    println!("5. Arena 分配器:");
    println!("   适用场景: 生命周期明确，批量处理");
    println!("   优势: 内存局域性好，分配效率高");
    println!("   劣势: 生命周期管理复杂\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_state() {
        let mut state = CowState::new();
        state.set("key1".to_string(), StateValue::String("value1".to_string()));
        
        let snapshot = state.snapshot();
        state.set("key2".to_string(), StateValue::String("value2".to_string()));
        
        assert!(snapshot.get("key2").is_none());
        assert!(state.get("key2").is_some());
    }

    #[test]
    fn test_event_sourced() {
        let mut state = EventSourcedState::new();
        state.set("key1".to_string(), StateValue::String("value1".to_string()));
        state.set("key1".to_string(), StateValue::String("value2".to_string()));
        
        assert_eq!(
            state.get("key1").unwrap(),
            StateValue::String("value2".to_string())
        );
    }

    #[test]
    fn test_layered_state() {
        let mut state = LayeredState::new();
        state.set_global("global".to_string(), StateValue::String("global_value".to_string()));
        state.set_local("global".to_string(), StateValue::String("local_value".to_string()));
        
        // 本地值应该覆盖全局值
        match state.get("global").unwrap() {
            StateValue::String(s) => assert_eq!(s, "local_value"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_lazy_clone() {
        let mut state = LazyCloneState::new();
        state.set("key1".to_string(), StateValue::String("value1".to_string()));
        
        let cloned = state.clone();
        assert!(cloned.get("key1").is_some());
    }
}
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use deno_core::OpState;
use mf_state::{State, transaction::Transaction};
use dashmap::DashMap;

/// ModuForge 运行时上下文
/// 存储在 Deno OpState 中，提供对 ModuForge 核心对象的访问
#[derive(Clone)]
pub struct ModuForgeContext {
    /// 当前状态快照
    pub current_state: Arc<State>,

    /// 事务存储映射（事务 ID -> 事务对象）
    pub transactions: Arc<DashMap<u32, Transaction>>,

    /// 事务计数器
    pub transaction_counter: Arc<std::sync::atomic::AtomicU32>,

    /// 插件 ID
    pub plugin_id: String,

    /// 上下文版本（用于缓存失效）
    pub context_version: u64,
}

impl ModuForgeContext {
    /// 创建新的上下文
    pub fn new(state: Arc<State>, plugin_id: String) -> Self {
        Self {
            current_state: state.clone(),
            transactions: Arc::new(DashMap::new()),
            transaction_counter: Arc::new(std::sync::atomic::AtomicU32::new(1)),
            plugin_id,
            context_version: state.version,
        }
    }

    /// 创建新的事务并返回 ID
    pub fn create_transaction(&self) -> u32 {
        let transaction = Transaction::new(&self.current_state);
        let id = self.transaction_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        self.transactions.insert(id, transaction);
        id
    }

    /// 获取事务的可变引用
    pub fn get_transaction_mut(&self, id: u32) -> Option<dashmap::mapref::one::RefMut<u32, Transaction>> {
        self.transactions.get_mut(&id)
    }

    /// 获取事务的不可变引用
    pub fn get_transaction(&self, id: u32) -> Option<dashmap::mapref::one::Ref<u32, Transaction>> {
        self.transactions.get(&id)
    }

    /// 删除事务
    pub fn remove_transaction(&self, id: u32) -> Option<(u32, Transaction)> {
        self.transactions.remove(&id)
    }

    /// 更新状态快照
    pub fn update_state(&mut self, new_state: Arc<State>) {
        self.current_state = new_state.clone();
        self.context_version = new_state.version;
        // 清理所有事务，因为状态已更改
        self.transactions.clear();
    }

    /// 检查上下文是否需要更新
    pub fn needs_update(&self, current_state_version: u64) -> bool {
        self.context_version != current_state_version
    }
}

/// 从 OpState 获取 ModuForgeContext
pub fn get_context_from_opstate(
    op_state: &mut deno_core::OpState,
) -> Result<std::cell::RefMut<ModuForgeContext>, crate::error::DenoError> {
    op_state
        .try_borrow_mut::<ModuForgeContext>()
        .map_err(|e| crate::error::DenoError::Runtime(
            anyhow::anyhow!("Failed to borrow ModuForgeContext: {}", e)
        ))
}

/// 设置 ModuForgeContext 到 OpState
pub fn set_context_to_opstate(
    op_state: Rc<RefCell<OpState>>,
    context: ModuForgeContext,
) {
    op_state.borrow_mut().put(context);
}
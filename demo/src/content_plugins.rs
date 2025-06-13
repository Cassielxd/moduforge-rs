use moduforge_core::extension::Extension;
use moduforge_state::{
    plugin::{Plugin, PluginSpec, PluginTrait, StateField},
    resource::Resource,
    state::{State, StateConfig},
    transaction::Transaction,
    error::StateResult,
};
use async_trait::async_trait;
use std::sync::Arc;
use crate::resources::*;

/// 创建用户管理扩展
/// 负责用户认证、会话管理、用户状态维护
pub fn create_user_management_extension() -> Extension {
    let mut extension = Extension::new();

    // 添加用户管理插件
    let user_plugin = Plugin::new(PluginSpec {
        key: ("user_manager".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(UserStateField)),
        tr: Some(Arc::new(UserPlugin)),
        priority: 10, // 最高优先级
    });

    extension.add_plugin(Arc::new(user_plugin));
    extension
}

/// 创建权限控制扩展
/// 负责用户权限验证、访问控制、操作授权
pub fn create_permission_extension() -> Extension {
    let mut extension = Extension::new();

    // 添加权限验证插件
    let permission_plugin = Plugin::new(PluginSpec {
        key: ("permission".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(PermissionStateField)),
        tr: Some(Arc::new(PermissionPlugin)),
        priority: 20, // 第二优先级
    });

    extension.add_plugin(Arc::new(permission_plugin));
    extension
}

/// 创建协作同步扩展
/// 负责多用户协作、冲突检测、实时同步
pub fn create_collaboration_extension() -> Extension {
    let mut extension = Extension::new();

    // 添加协作插件
    let collaboration_plugin = Plugin::new(PluginSpec {
        key: ("collaboration".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(CollaborationStateField)),
        tr: Some(Arc::new(CollaborationPlugin)),
        priority: 30, // 第三优先级
    });

    extension.add_plugin(Arc::new(collaboration_plugin));
    extension
}

/// 创建版本控制扩展
/// 负责版本管理、历史记录、快照创建
pub fn create_version_control_extension() -> Extension {
    let mut extension = Extension::new();

    // 添加版本控制插件
    let version_plugin = Plugin::new(PluginSpec {
        key: ("version_control".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(VersionControlStateField)),
        tr: Some(Arc::new(VersionControlPlugin)),
        priority: 40, // 第四优先级
    });

    extension.add_plugin(Arc::new(version_plugin));
    extension
}

// ===== 用户管理插件实现 =====

/// 用户管理插件
#[derive(Debug)]
pub struct UserPlugin;

#[async_trait]
impl PluginTrait for UserPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "user_login" => {
                        println!("   🔑 用户管理插件: 处理用户登录");
                        let mut new_tr = Transaction::new(new_state);
                        new_tr.set_meta("generated_by", "user_plugin");
                        new_tr.set_meta("action", "update_user_session");
                        return Ok(Some(new_tr));
                    },
                    "create_document" => {
                        println!("   📄 用户管理插件: 设置文档所有者");
                    },
                    _ => {},
                }
            }
        }
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        _transaction: &Transaction,
        _state: &State,
    ) -> bool {
        // 用户管理插件不过滤事务
        true
    }
}

/// 用户状态字段管理器
#[derive(Debug)]
pub struct UserStateField;

#[async_trait]
impl StateField for UserStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("   🔧 初始化用户管理状态");
        Arc::new(UserState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(user_state) = value.downcast_arc::<UserState>() {
            let mut new_state = (**user_state).clone();

            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "user_login" => {
                        if let Some(username) =
                            tr.get_meta::<String>("username")
                        {
                            new_state.login_user(username.as_str().to_string());
                        }
                    },
                    _ => {},
                }
            }

            Arc::new(new_state)
        } else {
            value
        }
    }
}

// ===== 权限控制插件实现 =====

/// 权限控制插件
#[derive(Debug)]
pub struct PermissionPlugin;

#[async_trait]
impl PluginTrait for PermissionPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "edit_paragraph" | "add_heading" | "add_list" => {
                        println!("   🔒 权限控制插件: 验证编辑权限");
                        let mut new_tr = Transaction::new(new_state);
                        new_tr.set_meta("generated_by", "permission_plugin");
                        new_tr.set_meta("action", "permission_checked");
                        return Ok(Some(new_tr));
                    },
                    _ => {},
                }
            }
        }
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        transaction: &Transaction,
        state: &State,
    ) -> bool {
        // 检查编辑权限
        if let Some(action) = transaction.get_meta::<String>("action") {
            if matches!(
                action.as_str(),
                "edit_paragraph" | "add_heading" | "add_list" | "add_table"
            ) {
                // 获取用户状态来检查权限
                if let Some(user_state) = state.get::<UserState>("user_manager")
                {
                    // 简单的权限检查：只有Editor和Writer可以编辑
                    return user_state.logged_in_users.values().any(|user| {
                        matches!(user.role.as_str(), "Editor" | "Writer")
                    });
                }
                return false;
            }
        }
        true
    }
}

/// 权限状态字段管理器
#[derive(Debug)]
pub struct PermissionStateField;

#[async_trait]
impl StateField for PermissionStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("   🔧 初始化权限控制状态");
        Arc::new(PermissionState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(permission_state) = value.downcast_arc::<PermissionState>()
        {
            let mut new_state = (**permission_state).clone();

            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "permission_checked" => {
                        new_state.last_check = std::time::SystemTime::now();
                        new_state.check_count += 1;
                    },
                    _ => {},
                }
            }

            Arc::new(new_state)
        } else {
            value
        }
    }
}

// ===== 协作同步插件实现 =====

/// 协作同步插件
#[derive(Debug)]
pub struct CollaborationPlugin;

#[async_trait]
impl PluginTrait for CollaborationPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "add_paragraph" | "add_heading" | "add_list" => {
                        println!("   🤝 协作插件: 检测并处理协作编辑");
                        let mut new_tr = Transaction::new(new_state);
                        new_tr.set_meta("generated_by", "collaboration_plugin");
                        new_tr.set_meta("action", "collaboration_synced");
                        return Ok(Some(new_tr));
                    },
                    "resolve_conflict" => {
                        println!("   ⚖️ 协作插件: 处理冲突解决");
                    },
                    _ => {},
                }
            }
        }
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        _transaction: &Transaction,
        _state: &State,
    ) -> bool {
        // 协作插件不过滤事务
        true
    }
}

/// 协作状态字段管理器
#[derive(Debug)]
pub struct CollaborationStateField;

#[async_trait]
impl StateField for CollaborationStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("   🔧 初始化协作同步状态");
        Arc::new(CollaborationState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(collab_state) = value.downcast_arc::<CollaborationState>() {
            let mut new_state = (**collab_state).clone();

            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "collaboration_synced" => {
                        new_state.sync_count += 1;
                        new_state.last_sync = std::time::SystemTime::now();
                    },
                    "resolve_conflict" => {
                        new_state.conflicts_resolved += 1;
                    },
                    _ => {},
                }
            }

            Arc::new(new_state)
        } else {
            value
        }
    }
}

// ===== 版本控制插件实现 =====

/// 版本控制插件
#[derive(Debug)]
pub struct VersionControlPlugin;

#[async_trait]
impl PluginTrait for VersionControlPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "create_snapshot" => {
                        println!("   📸 版本控制插件: 创建版本快照");
                        let mut new_tr = Transaction::new(new_state);
                        new_tr
                            .set_meta("generated_by", "version_control_plugin");
                        new_tr.set_meta("action", "snapshot_created");
                        return Ok(Some(new_tr));
                    },
                    _ => {},
                }
            }
        }
        Ok(None)
    }

    async fn filter_transaction(
        &self,
        _transaction: &Transaction,
        _state: &State,
    ) -> bool {
        // 版本控制插件不过滤事务
        true
    }
}

/// 版本控制状态字段管理器
#[derive(Debug)]
pub struct VersionControlStateField;

#[async_trait]
impl StateField for VersionControlStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("   🔧 初始化版本控制状态");
        Arc::new(VersionState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(version_state) = value.downcast_arc::<VersionState>() {
            let mut new_state = (**version_state).clone();

            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "snapshot_created" => {
                        new_state.create_snapshot("Auto snapshot".to_string());
                    },
                    "create_snapshot" => {
                        if let Some(description) =
                            tr.get_meta::<String>("description")
                        {
                            new_state.create_snapshot(description.to_string());
                        }
                    },
                    _ => {},
                }
            }

            Arc::new(new_state)
        } else {
            value
        }
    }
}

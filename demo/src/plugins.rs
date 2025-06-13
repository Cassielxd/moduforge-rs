use async_trait::async_trait;
use std::sync::Arc;
use moduforge_state::{
    plugin::{PluginTrait, StateField},
    resource::Resource,
    state::{State, StateConfig},
    transaction::Transaction,
    error::StateResult,
};
use crate::resources::*;

/// 用户管理插件
/// 负责管理用户信息、会话状态等
#[derive(Debug)]
pub struct UserPlugin;

#[async_trait]
impl PluginTrait for UserPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 检查是否有用户相关的事务
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "user_login" => {
                        if let Some(username) =
                            tr.get_meta::<String>("username")
                        {
                            if let Some(role) = tr.get_meta::<String>("role") {
                                println!(
                                    "   🔑 用户管理插件: 处理用户登录 - {} ({})",
                                    username.clone(),
                                    role
                                );
                                // 生成用户状态更新事务
                                let mut new_tr = Transaction::new(new_state);
                                new_tr.set_meta("generated_by", "user_plugin");
                                new_tr.set_meta("action", "update_user_status");
                                new_tr.set_meta(
                                    "username",
                                    username.as_ptr().clone(),
                                );
                                new_tr.set_meta("role", role.as_ptr().clone());
                                return Ok(Some(new_tr));
                            }
                        }
                    },
                    "create_document" => {
                        if let Some(title) = tr.get_meta::<String>("title") {
                            println!(
                                "   👤 用户管理插件: 验证文档创建权限 - {}",
                                title
                            );
                        }
                    },
                    "permission_check" => {
                        println!("   👤 用户管理插件: 提供用户角色信息");
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
        _state: &State,
    ) -> bool {
        // 用户管理插件不过滤任何事务
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
        println!("   🔧 初始化用户状态字段");
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

            // 根据事务更新用户状态
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "user_login" => {
                        if let Some(username) =
                            tr.get_meta::<String>("username")
                        {
                            new_state.login_user(username.as_str().to_string());
                        }
                    },
                    "update_user_status" => {
                        if let Some(username) =
                            tr.get_meta::<String>("username")
                        {
                            // 可以在这里更新用户相关信息，比如创建会话等
                            println!(
                                "   🔄 更新用户状态: {}",
                                username.as_str()
                            );
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

/// 权限验证插件
/// 负责验证用户权限、角色检查等
#[derive(Debug)]
pub struct AuthPlugin;

#[async_trait]
impl PluginTrait for AuthPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "user_login" => {
                        if let Some(username) =
                            tr.get_meta::<String>("username")
                        {
                            println!(
                                "   🛡️ 权限验证插件: 验证用户 {} 的登录权限",
                                username
                            );
                        }
                    },
                    "create_document" => {
                        if let Some(title) = tr.get_meta::<String>("title") {
                            println!(
                                "   🔒 权限验证插件: 检查文档创建权限 - {}",
                                title
                            );
                            // 生成权限验证事务
                            let mut new_tr = Transaction::new(new_state);
                            new_tr.set_meta("generated_by", "auth_plugin");
                            new_tr.set_meta(
                                "action",
                                "document_permission_checked",
                            );
                            new_tr.set_meta(
                                "document_title",
                                title.as_ptr().clone(),
                            );
                            return Ok(Some(new_tr));
                        }
                    },
                    "add_heading" | "add_paragraph" | "add_list"
                    | "add_table" => {
                        println!(
                            "   📝 权限验证插件: 检查内容编辑权限 - {}",
                            action
                        );
                        // 验证内容编辑权限
                        let mut new_tr = Transaction::new(new_state);
                        new_tr.set_meta("generated_by", "auth_plugin");
                        new_tr.set_meta("action", "content_permission_checked");
                        new_tr
                            .set_meta("content_type", action.as_ptr().clone());
                        return Ok(Some(new_tr));
                    },
                    "document_edit" => {
                        println!("   🔒 权限验证插件: 检查编辑权限");
                        let mut new_tr = Transaction::new(new_state);
                        new_tr.set_meta("generated_by", "auth_plugin");
                        new_tr.set_meta("action", "permission_validated");
                        return Ok(Some(new_tr));
                    },
                    "permission_check" => {
                        println!("   ✅ 权限验证插件: 执行权限检查");
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
        // 检查是否需要权限验证
        if let Some(action) = transaction.get_meta::<String>("action") {
            match action.as_str() {
                "document_edit" => {
                    // 模拟权限检查
                    if let Some(user_id) =
                        transaction.get_meta::<String>("user_id")
                    {
                        println!(
                            "   🔍 权限验证插件: 验证用户 {} 的编辑权限",
                            user_id
                        );
                        return **user_id == "user_123"; // 简单的权限检查
                    }
                    return false;
                },
                _ => true,
            }
        } else {
            true
        }
    }
}

/// 权限状态字段管理器
#[derive(Debug)]
pub struct AuthStateField;

#[async_trait]
impl StateField for AuthStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("   🔧 初始化权限状态字段");
        Arc::new(AuthState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(auth_state) = value.downcast_arc::<AuthState>() {
            let mut new_state = (**auth_state).clone();

            // 根据事务更新权限状态
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "permission_validated" => {
                        new_state.last_check_time =
                            std::time::SystemTime::now();
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

/// 审计日志插件
/// 负责记录系统操作日志、事件追踪等
#[derive(Debug)]
pub struct AuditPlugin;

#[async_trait]
impl PluginTrait for AuditPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 审计插件记录所有重要操作
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "user_login" => {
                        if let Some(username) =
                            tr.get_meta::<String>("username")
                        {
                            println!(
                                "   📝 审计日志插件: 记录用户登录事件 - {}",
                                username
                            );
                        }
                    },
                    "create_document" => {
                        if let Some(title) = tr.get_meta::<String>("title") {
                            println!(
                                "   📄 审计日志插件: 记录文档创建 - {}",
                                title
                            );
                        }
                    },
                    "add_heading" => {
                        if let Some(text) = tr.get_meta::<String>("text") {
                            if let Some(level) = tr.get_meta::<u32>("level") {
                                println!(
                                    "   📋 审计日志插件: 记录标题添加 - H{}: {}",
                                    level, text
                                );
                            }
                        }
                    },
                    "add_paragraph" => {
                        if let Some(text) = tr.get_meta::<String>("text") {
                            let preview = if text.chars().count() > 30 {
                                text.chars().take(30).collect::<String>()
                            } else {
                                text.as_str().to_string()
                            };
                            println!(
                                "   📝 审计日志插件: 记录段落添加 - {}...",
                                preview
                            );
                        }
                    },
                    "add_list" => {
                        if let Some(count) = tr.get_meta::<u32>("item_count") {
                            println!(
                                "   📋 审计日志插件: 记录列表添加 - {} 项",
                                count
                            );
                        }
                    },
                    "add_table" => {
                        if let Some(rows) = tr.get_meta::<u32>("row_count") {
                            if let Some(cols) = tr.get_meta::<u32>("col_count")
                            {
                                println!(
                                    "   📊 审计日志插件: 记录表格添加 - {}x{}",
                                    cols, rows
                                );
                            }
                        }
                    },
                    "create_snapshot" => {
                        if let Some(desc) = tr.get_meta::<String>("description")
                        {
                            println!(
                                "   📸 审计日志插件: 记录快照创建 - {}",
                                desc
                            );
                        }
                    },
                    "sync_document" => {
                        if let Some(sync_id) = tr.get_meta::<String>("sync_id")
                        {
                            println!(
                                "   🔄 审计日志插件: 记录文档同步 - {}",
                                sync_id
                            );
                        }
                    },
                    "validate_consistency" => {
                        println!("   🔍 审计日志插件: 记录一致性验证");
                    },
                    "document_edit" => {
                        println!("   📋 审计日志插件: 记录文档编辑操作");
                    },
                    "permission_check" => {
                        println!("   🔍 审计日志插件: 记录权限检查结果");
                    },
                    "cache_update" => {
                        println!("   💾 审计日志插件: 记录缓存操作");
                    },
                    _ => {},
                }
            }
        }

        // 生成审计日志事务
        let mut audit_tr = Transaction::new(new_state);
        audit_tr.set_meta("generated_by", "audit_plugin");
        audit_tr.set_meta("action", "audit_logged");
        audit_tr.set_meta("timestamp", std::time::SystemTime::now());

        Ok(Some(audit_tr))
    }

    async fn filter_transaction(
        &self,
        _transaction: &Transaction,
        _state: &State,
    ) -> bool {
        // 审计插件不过滤任何事务
        true
    }
}

/// 审计状态字段管理器
#[derive(Debug)]
pub struct AuditStateField;

#[async_trait]
impl StateField for AuditStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("   🔧 初始化审计状态字段");
        Arc::new(AuditState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(audit_state) = value.downcast_arc::<AuditState>() {
            let mut new_state = (**audit_state).clone();

            // 记录审计事件
            if let Some(action) = tr.get_meta::<String>("action") {
                new_state.log_count += 1;
                new_state.last_action = Some(action.to_string());
            }

            Arc::new(new_state)
        } else {
            value
        }
    }
}

/// 缓存管理插件
/// 负责管理系统缓存、性能优化等
#[derive(Debug)]
pub struct CachePlugin;

#[async_trait]
impl PluginTrait for CachePlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "user_login" => {
                        if let Some(username) =
                            tr.get_meta::<String>("username")
                        {
                            println!(
                                "   💾 缓存管理插件: 缓存用户会话 - {}",
                                username
                            );
                        }
                    },
                    "create_document" => {
                        if let Some(title) = tr.get_meta::<String>("title") {
                            println!(
                                "   📄 缓存管理插件: 缓存文档元数据 - {}",
                                title
                            );
                            // 生成文档缓存事务
                            let mut cache_tr = Transaction::new(new_state);
                            cache_tr.set_meta("generated_by", "cache_plugin");
                            cache_tr.set_meta("action", "document_cached");
                            cache_tr.set_meta(
                                "document_title",
                                title.as_ptr().clone(),
                            );
                            return Ok(Some(cache_tr));
                        }
                    },
                    "add_heading" | "add_paragraph" | "add_list"
                    | "add_table" => {
                        println!(
                            "   🔄 缓存管理插件: 更新内容缓存 - {}",
                            action
                        );
                        // 生成内容缓存更新事务
                        let mut cache_tr = Transaction::new(new_state);
                        cache_tr.set_meta("generated_by", "cache_plugin");
                        cache_tr.set_meta("action", "content_cache_updated");
                        cache_tr
                            .set_meta("content_type", action.as_ptr().clone());
                        return Ok(Some(cache_tr));
                    },
                    "create_snapshot" => {
                        println!("   📸 缓存管理插件: 缓存版本快照");
                    },
                    "sync_document" => {
                        println!("   🔄 缓存管理插件: 同步文档缓存");
                    },
                    "document_edit" => {
                        println!("   🔄 缓存管理插件: 更新文档缓存");
                    },
                    "cache_update" => {
                        println!("   ⚡ 缓存管理插件: 执行缓存操作");

                        // 生成缓存清理事务
                        let mut cache_tr = Transaction::new(new_state);
                        cache_tr.set_meta("generated_by", "cache_plugin");
                        cache_tr.set_meta("action", "cache_cleaned");
                        return Ok(Some(cache_tr));
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
        // 缓存插件不过滤任何事务
        true
    }
}

/// 缓存状态字段管理器
#[derive(Debug)]
pub struct CacheStateField;

#[async_trait]
impl StateField for CacheStateField {
    async fn init(
        &self,
        _config: &StateConfig,
        _instance: Option<&State>,
    ) -> Arc<dyn Resource> {
        println!("   🔧 初始化缓存状态字段");
        Arc::new(CacheState::new())
    }

    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        if let Some(cache_state) = value.downcast_arc::<CacheState>() {
            let mut new_state = (**cache_state).clone();

            // 根据事务更新缓存状态
            if let Some(action) = tr.get_meta::<String>("action") {
                match action.as_str() {
                    "cache_cleaned" => {
                        new_state.cache_hits = 0;
                        new_state.last_cleanup = std::time::SystemTime::now();
                    },
                    _ => {
                        new_state.cache_hits += 1;
                    },
                }
            }

            Arc::new(new_state)
        } else {
            value
        }
    }
}

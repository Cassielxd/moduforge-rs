use async_trait::async_trait;
use std::{sync::Arc, time::SystemTime};
use moduforge_core::{
    middleware::{Middleware, MiddlewareResult},
    error::EditorResult,
};
use moduforge_state::{state::State, transaction::Transaction};

/// 认证中间件
/// 负责验证用户身份和会话状态
#[derive(Debug)]
pub struct AuthenticationMiddleware {
    name: String,
}

impl AuthenticationMiddleware {
    pub fn new() -> Self {
        Self { name: "AuthenticationMiddleware".to_string() }
    }
}

#[async_trait]
impl Middleware for AuthenticationMiddleware {
    fn name(&self) -> String {
        "AuthenticationMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        if let Some(action) = transaction.get_meta::<String>("action") {
            match action.as_str() {
                "user_login" => {
                    println!("🔐 [{}] 验证用户登录凭据", self.name);

                    if let Some(username) =
                        transaction.get_meta::<String>("username")
                    {
                        let username_str = username.as_str().to_string();
                        // 模拟身份验证
                        if username_str.is_empty() {
                            return Err(anyhow::anyhow!("用户名不能为空"));
                        }

                        transaction.set_meta("authenticated", true);
                        transaction.set_meta("auth_time", SystemTime::now());
                        println!(
                            "✅ [{}] 用户 {} 身份验证成功",
                            self.name, username_str
                        );
                    }
                },
                _ => {
                    // 对于其他操作，检查是否已认证
                    if !transaction
                        .get_meta::<bool>("authenticated")
                        .map(|x| **x)
                        .unwrap_or(false)
                    {
                        println!("🔒 [{}] 检查用户认证状态", self.name);
                    }
                },
            }
        }

        Ok(())
    }

    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult> {
        for transaction in transactions {
            if let Some(action) = transaction.get_meta::<String>("action") {
                if action.as_str() == "user_login" {
                    println!("📝 [{}] 更新用户会话信息", self.name);
                }
            }
        }

        Ok(MiddlewareResult::new(Ok(())))
    }
}

/// 权限中间件
/// 负责检查用户操作权限
#[derive(Debug)]
pub struct PermissionMiddleware {
    name: String,
}

impl PermissionMiddleware {
    pub fn new() -> Self {
        Self { name: "PermissionMiddleware".to_string() }
    }

    fn check_edit_permission(
        &self,
        role: &str,
    ) -> bool {
        matches!(role, "Editor" | "Writer")
    }

}

#[async_trait]
impl Middleware for PermissionMiddleware {
    fn name(&self) -> String {
        "PermissionMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        if let Some(action) = transaction.get_meta::<String>("action") {
            match action.as_str() {
                "add_heading" | "add_paragraph" | "add_list" | "add_table"
                | "edit_paragraph" => {
                    println!("🔍 [{}] 检查编辑权限", self.name);

                    if let Some(role) = transaction.get_meta::<String>("role") {
                        if !self.check_edit_permission(&role) {
                            return Err(anyhow::anyhow!(
                                "用户 {} 没有编辑权限",
                                role
                            ));
                        }
                        println!("✅ [{}] 编辑权限验证通过", self.name);
                    } else {
                        // 没有角色信息，拒绝操作
                        return Err(anyhow::anyhow!("缺少用户角色信息"));
                    }
                },
                "create_snapshot" => {
                    println!("🔍 [{}] 检查快照创建权限", self.name);

                    if let Some(role) = transaction.get_meta::<String>("role") {
                        if !self.check_edit_permission(&role) {
                            return Err(anyhow::anyhow!(
                                "用户 {} 没有创建快照权限",
                                role
                            ));
                        }
                    }
                },
                _ => {},
            }
        }

        transaction.set_meta("permission_checked", true);
        Ok(())
    }

    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult> {
        // 记录权限检查结果
        for transaction in transactions {
            if transaction
                .get_meta::<bool>("permission_checked")
                .map(|x| **x)
                .unwrap_or(false)
            {
                println!("📊 [{}] 权限检查完成", self.name);
            }
        }

        Ok(MiddlewareResult::new(Ok(())))
    }
}

/// 协作中间件
/// 负责处理多用户协作和冲突检测
#[derive(Debug)]
pub struct CollaborationMiddleware {
    name: String,
}

impl CollaborationMiddleware {
    pub fn new() -> Self {
        Self { name: "CollaborationMiddleware".to_string() }
    }
}

#[async_trait]
impl Middleware for CollaborationMiddleware {
    fn name(&self) -> String {
        "CollaborationMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        if let Some(action) = transaction.get_meta::<String>("action") {
            match action.as_str() {
                "add_paragraph" | "add_heading" | "add_list"
                | "edit_paragraph" => {
                    println!("🤝 [{}] 检测协作编辑冲突", self.name);

                    // 模拟冲突检测
                    let has_conflict = rand::random::<f32>() < 0.1; // 10% 概率有冲突

                    if has_conflict {
                        println!("⚠️ [{}] 检测到潜在编辑冲突", self.name);
                        transaction.set_meta("has_conflict", true);
                    } else {
                        println!("✅ [{}] 无编辑冲突", self.name);
                        transaction.set_meta("has_conflict", false);
                    }
                },
                "resolve_conflict" => {
                    println!("⚖️ [{}] 处理冲突解决", self.name);
                },
                "sync_document" => {
                    println!("🔄 [{}] 同步协作状态", self.name);
                },
                _ => {},
            }
        }

        transaction.set_meta("collaboration_processed", true);
        Ok(())
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult> {
        let mut needs_sync = false;

        for transaction in transactions {
            if let Some(action) = transaction.get_meta::<String>("action") {
                if matches!(
                    action.as_str(),
                    "add_paragraph" | "add_heading" | "add_list"
                ) {
                    needs_sync = true;
                    break;
                }
            }
        }

        if needs_sync {
            println!("📡 [{}] 触发实时同步", self.name);

            if let Some(state) = state {
                // 生成同步事务
                let mut sync_tr = Transaction::new(&state);
                sync_tr.set_meta("generated_by", "collaboration_middleware");
                sync_tr.set_meta("action", "auto_sync");
                sync_tr.set_meta("sync_time", SystemTime::now());
                sync_tr.commit();

                return Ok(MiddlewareResult::with_transactions(
                    Ok(()),
                    Some(sync_tr),
                ));
            }
        }

        Ok(MiddlewareResult::new(Ok(())))
    }
}

/// 版本控制中间件
/// 负责自动版本管理和快照创建
#[derive(Debug)]
pub struct VersionControlMiddleware {
    name: String,
}

impl VersionControlMiddleware {
    pub fn new() -> Self {
        Self { name: "VersionControlMiddleware".to_string() }
    }
}

#[async_trait]
impl Middleware for VersionControlMiddleware {
    fn name(&self) -> String {
        "VersionControlMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        if let Some(action) = transaction.get_meta::<String>("action") {
            match action.as_str() {
                "create_snapshot" => {
                    println!("📸 [{}] 准备创建版本快照", self.name);
                    transaction.set_meta("snapshot_requested", true);
                },
                "add_table" => {
                    // 重要操作，自动创建快照
                    println!("🔄 [{}] 重要操作，标记需要自动快照", self.name);
                    transaction.set_meta("auto_snapshot_needed", true);
                },
                _ => {},
            }
        }

        Ok(())
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult> {
        let mut needs_snapshot = false;

        for transaction in transactions {
            if transaction
                .get_meta::<bool>("auto_snapshot_needed")
                .map(|x| **x)
                .unwrap_or(false)
            {
                needs_snapshot = true;
                break;
            }
        }

        if needs_snapshot {
            println!("📸 [{}] 创建自动快照", self.name);

            if let Some(state) = state {
                let mut snapshot_tr = Transaction::new(&state);
                snapshot_tr
                    .set_meta("generated_by", "version_control_middleware");
                snapshot_tr.set_meta("action", "auto_snapshot");
                snapshot_tr.set_meta("description", "自动快照 - 重要操作");
                snapshot_tr.commit();

                return Ok(MiddlewareResult::with_transactions(
                    Ok(()),
                    Some(snapshot_tr),
                ));
            }
        }

        Ok(MiddlewareResult::new(Ok(())))
    }
}

/// 审计日志中间件
/// 负责记录所有操作日志
#[derive(Debug)]
pub struct AuditLogMiddleware {
    name: String,
}

impl AuditLogMiddleware {
    pub fn new() -> Self {
        Self { name: "AuditLogMiddleware".to_string() }
    }
}

#[async_trait]
impl Middleware for AuditLogMiddleware {
    fn name(&self) -> String {
        "AuditLogMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> EditorResult<()> {
        if let Some(action) = transaction.get_meta::<String>("action") {
            let user = transaction
                .get_meta::<String>("username")
                .map(|s| s.as_str())
                .unwrap_or("system");

            println!(
                "📝 [{}] 记录操作: {} (用户: {})",
                self.name, action, user
            );

            transaction.set_meta("audit_logged", true);
            transaction.set_meta("audit_time", SystemTime::now());
        }

        Ok(())
    }

    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> EditorResult<MiddlewareResult> {
        let mut operation_count = 0;

        for transaction in transactions {
            if transaction
                .get_meta::<bool>("audit_logged")
                .map(|x| **x)
                .unwrap_or(false)
            {
                operation_count += 1;

                if let Some(action) = transaction.get_meta::<String>("action") {
                    let success = true; // 假设操作成功
                    let status = if success { "SUCCESS" } else { "FAILED" };

                    println!(
                        "📊 [{}] 审计记录: {} - {}",
                        self.name, action, status
                    );
                }
            }
        }

        if operation_count > 0 {
            println!(
                "📈 [{}] 本次处理记录了 {} 个操作",
                self.name, operation_count
            );
        }

        Ok(MiddlewareResult::new(Ok(())))
    }
}

use std::{collections::HashMap, time::SystemTime};
use moduforge_state::resource::Resource;
use im::HashMap as ImHashMap;

/// 用户状态资源
/// 管理当前登录用户、会话信息等
#[derive(Debug, Clone)]
pub struct UserState {
    pub logged_in_users: ImHashMap<String, UserInfo>,
    pub active_sessions: ImHashMap<String, SessionInfo>,
    pub total_users: u64,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub last_login: SystemTime,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
}

impl UserState {
    pub fn new() -> Self {
        Self {
            logged_in_users: ImHashMap::new(),
            active_sessions: ImHashMap::new(),
            total_users: 0,
        }
    }
    
    pub fn login_user(&mut self, user_id: String) {
        let user_info = UserInfo {
            user_id: user_id.clone(),
            username: format!("user_{}", user_id),
            last_login: SystemTime::now(),
            role: "user".to_string(),
        };
        
        self.logged_in_users.insert(user_id.clone(), user_info);
        self.total_users += 1;
        
        println!("   👤 用户 {} 已登录，当前总用户数: {}", user_id, self.total_users);
    }
    
    pub fn logout_user(&mut self, user_id: &str) {
        self.logged_in_users.remove(user_id);
        println!("   👋 用户 {} 已登出", user_id);
    }
    
    pub fn get_user_role(&self, user_id: &str) -> Option<String> {
        self.logged_in_users.get(user_id).map(|user| user.role.clone())
    }
}

impl Resource for UserState {}

/// 权限验证状态资源
/// 管理用户权限、角色验证等
#[derive(Debug, Clone)]
pub struct AuthState {
    pub permissions: ImHashMap<String, Vec<String>>, // user_id -> permissions
    pub roles: ImHashMap<String, String>, // user_id -> role
    pub last_check_time: SystemTime,
    pub permission_cache: ImHashMap<String, bool>, // permission_key -> allowed
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            permissions: ImHashMap::new(),
            roles: ImHashMap::new(),
            last_check_time: SystemTime::now(),
            permission_cache: ImHashMap::new(),
        }
    }
    
    pub fn grant_permission(&mut self, user_id: String, permission: String) {
        let mut user_permissions = self.permissions.get(&user_id).cloned().unwrap_or_default();
        if !user_permissions.contains(&permission) {
            user_permissions.push(permission.clone());
            self.permissions.insert(user_id.clone(), user_permissions);
            println!("   ✅ 用户 {} 获得权限: {}", user_id, permission);
        }
    }
    
    pub fn check_permission(&self, user_id: &str, permission: &str) -> bool {
        if let Some(user_permissions) = self.permissions.get(user_id) {
            user_permissions.contains(&permission.to_string())
        } else {
            false
        }
    }
    
    pub fn set_role(&mut self, user_id: String, role: String) {
        self.roles.insert(user_id.clone(), role.clone());
        println!("   🔐 用户 {} 角色设置为: {}", user_id, role);
    }
}

impl Resource for AuthState {}

/// 审计日志状态资源
/// 管理系统操作日志、事件记录等
#[derive(Debug, Clone)]
pub struct AuditState {
    pub log_entries: Vec<AuditEntry>,
    pub log_count: u64,
    pub last_action: Option<String>,
    pub start_time: SystemTime,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: SystemTime,
    pub action: String,
    pub user_id: Option<String>,
    pub resource: Option<String>,
    pub result: AuditResult,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum AuditResult {
    Success,
    Failure(String),
    Warning(String),
}

impl AuditState {
    pub fn new() -> Self {
        Self {
            log_entries: Vec::new(),
            log_count: 0,
            last_action: None,
            start_time: SystemTime::now(),
        }
    }
    
    pub fn add_entry(&mut self, action: String, user_id: Option<String>, result: AuditResult) {
        let entry = AuditEntry {
            id: self.log_count + 1,
            timestamp: SystemTime::now(),
            action: action.clone(),
            user_id,
            resource: None,
            result,
            details: HashMap::new(),
        };
        
        self.log_entries.push(entry);
        self.log_count += 1;
        self.last_action = Some(action);
        
        println!("   📝 添加审计记录 #{}: {}", self.log_count, self.last_action.as_ref().unwrap());
    }
    
    pub fn get_logs_by_user(&self, user_id: &str) -> Vec<&AuditEntry> {
        self.log_entries.iter()
            .filter(|entry| entry.user_id.as_ref().map_or(false, |id| id == user_id))
            .collect()
    }
    
    pub fn get_recent_logs(&self, count: usize) -> &[AuditEntry] {
        let len = self.log_entries.len();
        if len <= count {
            &self.log_entries
        } else {
            &self.log_entries[len - count..]
        }
    }
}

impl Resource for AuditState {}

/// 缓存管理状态资源
/// 管理系统缓存、性能数据等
#[derive(Debug, Clone)]
pub struct CacheState {
    pub cache_entries: ImHashMap<String, CacheEntry>,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub last_cleanup: SystemTime,
    pub max_entries: usize,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub access_count: u64,
    pub ttl: Option<u64>, // Time to live in seconds
}

impl CacheState {
    pub fn new() -> Self {
        Self {
            cache_entries: ImHashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
            last_cleanup: SystemTime::now(),
            max_entries: 1000,
        }
    }
    
    pub fn put(&mut self, key: String, value: String, ttl: Option<u64>) {
        let entry = CacheEntry {
            key: key.clone(),
            value,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 0,
            ttl,
        };
        
        self.cache_entries.insert(key.clone(), entry);
        println!("   💾 缓存条目已添加: {}", key);
        
        // 如果超过最大条目数，清理最旧的条目
        if self.cache_entries.len() > self.max_entries {
            self.cleanup_old_entries();
        }
    }
    
    pub fn get(&mut self, key: &str) -> Option<String> {
        if let Some(mut entry) = self.cache_entries.get(key).cloned() {
            entry.last_accessed = SystemTime::now();
            entry.access_count += 1;
            self.cache_entries.insert(key.to_string(), entry.clone());
            self.cache_hits += 1;
            println!("   ✅ 缓存命中: {}", key);
            Some(entry.value)
        } else {
            self.cache_misses += 1;
            println!("   ❌ 缓存未命中: {}", key);
            None
        }
    }
    
    pub fn invalidate(&mut self, key: &str) {
        if self.cache_entries.remove(key).is_some() {
            println!("   🗑️ 缓存条目已失效: {}", key);
        }
    }
    
    pub fn clear_all(&mut self) {
        let count = self.cache_entries.len();
        self.cache_entries.clear();
        self.last_cleanup = SystemTime::now();
        println!("   🧹 已清除所有缓存条目，共 {} 个", count);
    }
    
    fn cleanup_old_entries(&mut self) {
        // 简单的清理策略：移除最少访问的条目
        let mut entries: Vec<_> = self.cache_entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        entries.sort_by_key(|(_, entry)| entry.access_count);
        
        let to_remove = entries.len() - self.max_entries + 100; // 额外清理100个
        for (key, _) in entries.iter().take(to_remove) {
            self.cache_entries.remove(key);
        }
        
        self.last_cleanup = SystemTime::now();
        println!("   🧹 缓存清理完成，移除了 {} 个条目", to_remove);
    }
    
    pub fn get_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

impl Resource for CacheState {}

/// 权限状态资源
/// 管理用户权限、角色验证等
#[derive(Debug, Clone)]
pub struct PermissionState {
    pub user_permissions: ImHashMap<String, Vec<String>>, // user_id -> permissions
    pub role_permissions: ImHashMap<String, Vec<String>>, // role -> permissions
    pub last_check: SystemTime,
    pub check_count: u64,
}

impl PermissionState {
    pub fn new() -> Self {
        Self {
            user_permissions: ImHashMap::new(),
            role_permissions: ImHashMap::new(),
            last_check: SystemTime::now(),
            check_count: 0,
        }
    }
    
    pub fn grant_permission(&mut self, user_id: String, permission: String) {
        let mut permissions = self.user_permissions.get(&user_id).cloned().unwrap_or_default();
        if !permissions.contains(&permission) {
            permissions.push(permission.clone());
            self.user_permissions.insert(user_id.clone(), permissions);
            println!("   ✅ 用户 {} 获得权限: {}", user_id, permission);
        }
    }
    
    pub fn check_permission(&self, user_id: &str, permission: &str) -> bool {
        if let Some(permissions) = self.user_permissions.get(user_id) {
            permissions.contains(&permission.to_string())
        } else {
            false
        }
    }
}

impl Resource for PermissionState {}

/// 协作状态资源
/// 管理多用户协作、冲突检测等
#[derive(Debug, Clone)]
pub struct CollaborationState {
    pub active_editors: ImHashMap<String, EditorSession>,
    pub sync_count: u64,
    pub conflicts_resolved: u64,
    pub last_sync: SystemTime,
}

#[derive(Debug, Clone)]
pub struct EditorSession {
    pub user_id: String,
    pub last_activity: SystemTime,
    pub editing_position: Option<String>,
}

impl CollaborationState {
    pub fn new() -> Self {
        Self {
            active_editors: ImHashMap::new(),
            sync_count: 0,
            conflicts_resolved: 0,
            last_sync: SystemTime::now(),
        }
    }
    
    pub fn add_editor(&mut self, user_id: String) {
        let session = EditorSession {
            user_id: user_id.clone(),
            last_activity: SystemTime::now(),
            editing_position: None,
        };
        self.active_editors.insert(user_id.clone(), session);
        println!("   👥 用户 {} 加入协作编辑", user_id);
    }
    
    pub fn remove_editor(&mut self, user_id: &str) {
        self.active_editors.remove(user_id);
        println!("   👋 用户 {} 退出协作编辑", user_id);
    }
}

impl Resource for CollaborationState {}

/// 版本控制状态资源
/// 管理版本快照、历史记录等
#[derive(Debug, Clone)]
pub struct VersionState {
    pub snapshots: Vec<VersionSnapshot>,
    pub current_version: String,
    pub auto_snapshot_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct VersionSnapshot {
    pub id: String,
    pub description: String,
    pub created_at: SystemTime,
    pub creator: String,
    pub hash: String,
}

impl VersionState {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            current_version: "1.0.0".to_string(),
            auto_snapshot_enabled: true,
        }
    }
    
    pub fn create_snapshot(&mut self, description: String) {
        let snapshot = VersionSnapshot {
            id: format!("snap_{}", self.snapshots.len() + 1),
            description: description.clone(),
            created_at: SystemTime::now(),
            creator: "system".to_string(),
            hash: format!("hash_{}", rand::random::<u64>()),
        };
        
        self.snapshots.push(snapshot);
        println!("   📸 创建版本快照: {}", description);
    }
    
    pub fn get_latest_snapshot(&self) -> Option<&VersionSnapshot> {
        self.snapshots.last()
    }
}

impl Resource for VersionState {} 
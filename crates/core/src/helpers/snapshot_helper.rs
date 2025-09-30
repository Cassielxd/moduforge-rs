//! 快照管理辅助模块
//!
//! 提供统一的快照序列化/反序列化逻辑，包括：
//! - 快照加载
//! - 快照验证
//! - 扩展管理器恢复

use crate::{
    debug::debug,
    error::ForgeResult,
    extension_manager::ExtensionManager,
    snapshot::SnapshotManager,
};

/// 快照管理辅助器
pub struct SnapshotHelper;

impl SnapshotHelper {
    /// 从文件加载快照
    ///
    /// # 参数
    /// * `snapshot_path` - 快照文件路径
    ///
    /// # 返回值
    /// * `ForgeResult<crate::snapshot::CoreSnapshot>` - 快照对象或错误
    pub fn load_snapshot(
        snapshot_path: &str,
    ) -> ForgeResult<crate::snapshot::CoreSnapshot> {
        debug!("正在从快照加载: {}", snapshot_path);
        SnapshotManager::load_from_file(snapshot_path)
    }

    /// 验证快照
    ///
    /// # 参数
    /// * `snapshot` - 快照对象的引用
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub fn validate_snapshot(snapshot: &crate::snapshot::CoreSnapshot) -> ForgeResult<()> {
        debug!("正在验证快照");
        SnapshotManager::validate_snapshot(snapshot)
    }

    /// 从快照恢复扩展管理器
    ///
    /// # 参数
    /// * `snapshot` - 快照对象的引用
    ///
    /// # 返回值
    /// * `ForgeResult<ExtensionManager>` - 扩展管理器或错误
    pub fn restore_extension_manager(
        snapshot: &crate::snapshot::CoreSnapshot,
    ) -> ForgeResult<ExtensionManager> {
        debug!("正在从快照恢复扩展管理器");
        SnapshotManager::restore_extension_manager(snapshot)
    }

    /// 完整的快照加载流程（加载 + 验证 + 恢复扩展管理器）
    ///
    /// # 参数
    /// * `snapshot_path` - 快照文件路径
    ///
    /// # 返回值
    /// * `ForgeResult<(crate::snapshot::CoreSnapshot, ExtensionManager)>` - 快照和扩展管理器或错误
    pub fn load_and_restore(
        snapshot_path: &str,
    ) -> ForgeResult<(crate::snapshot::CoreSnapshot, ExtensionManager)> {
        // 加载快照
        let snapshot = Self::load_snapshot(snapshot_path)?;

        // 验证快照
        Self::validate_snapshot(&snapshot)?;

        // 恢复扩展管理器
        let extension_manager = Self::restore_extension_manager(&snapshot)?;

        debug!("快照加载和恢复完成");
        Ok((snapshot, extension_manager))
    }

    /// 保存快照到文件
    ///
    /// # 参数
    /// * `snapshot_path` - 快照文件路径
    /// * `snapshot` - 快照对象的引用
    ///
    /// # 返回值
    /// * `ForgeResult<()>` - 成功或错误
    pub fn save_snapshot(
        snapshot_path: &str,
        snapshot: &crate::snapshot::CoreSnapshot,
    ) -> ForgeResult<()> {
        debug!("正在保存快照到: {}", snapshot_path);
        SnapshotManager::save_to_file(snapshot, snapshot_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_helper_load_nonexistent() {
        let result = SnapshotHelper::load_snapshot("nonexistent.bin");
        assert!(result.is_err());
    }
}
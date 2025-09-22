use std::sync::Arc;

use mf_macro::{
    mf_meta, mf_plugin, mf_plugin_config, mf_plugin_metadata,
    mf_plugin_with_config,
};
use mf_state::{Transaction, State, error::StateResult};
use mf_state::plugin::{PluginMetadata, PluginConfig};

// 定义一些示例事务处理函数
async fn logging_append_transaction(
    _trs: &[Arc<Transaction>],
    _old_state: &Arc<State>,
    _new_state: &Arc<State>,
) -> StateResult<Option<Transaction>> {
    println!("📝 记录事务处理过程");
    Ok(None)
}

async fn security_filter_transaction(
    tr: &Transaction,
    _state: &State,
) -> bool {
    // 实际的安全检查逻辑示例

    // 检查事务大小限制
    if tr.steps.len() > 100 {
        println!("🚫 事务被拒绝: 步骤数量过多 ({})", tr.steps.len());
        return false;
    }

    // 检查事务元数据
    if let Some(source) = tr.get_meta::<String>("source") {
        if source == "untrusted" {
            println!("🚫 事务被拒绝: 来源不可信");
            return false;
        }
    }

    // 检查是否是系统保留的事务类型
    if tr.get_meta::<bool>("system_reserved").unwrap_or(false) {
        println!("🚫 事务被拒绝: 系统保留事务类型");
        return false;
    }

    println!("🔒 安全检查通过");
    true
}

async fn validation_append_transaction(
    trs: &[Transaction],
    _old_state: &State,
    _new_state: &State,
) -> StateResult<Option<Transaction>> {
    println!("✅ 验证 {} 个事务", trs.len());
    Ok(None)
}

async fn audit_filter_transaction(
    tr: &Transaction,
    state: &State,
) -> bool {
    println!("📋 审计事务: {:?}", tr.id);

    // 实际的审计过滤逻辑

    // 检查状态中是否有审计配置
    if let Some(_audit_config) = state.get_field("audit_config") {
        // 检查事务是否需要特殊审计
        if tr.get_meta::<bool>("requires_audit").unwrap_or(false) {
            println!("⚠️  事务需要手动审计批准");
            // 在实际环境中，这里可能需要等待审批或记录到审计队列
            return false; // 暂时拒绝，直到获得审批
        }
    }

    // 检查事务频率限制
    if let Some(user_id) = tr.get_meta::<String>("user_id") {
        // 简化的频率检查逻辑
        if user_id == "high_frequency_user" {
            println!("🚫 用户操作过于频繁，事务被限制");
            return false;
        }
    }

    // 检查是否是敏感操作
    let steps = &tr.steps;
    for step in steps {
        if step.name().contains("delete") || step.name().contains("Drop") {
            println!("⚠️  检测到敏感操作: {}", step.name());
            // 敏感操作需要额外验证
            if !tr.get_meta::<bool>("admin_approved").unwrap_or(false) {
                println!("🚫 敏感操作缺少管理员批准");
                return false;
            }
        }
    }

    println!("✅ 审计检查通过");
    true
}

// 1. 基础插件 - 使用默认配置和元数据
mf_plugin!(simple_plugin, docs = "简单的示例插件，展示基础功能");

// 2. 带元数据的插件
mf_plugin!(
    logging_plugin,
    metadata = mf_plugin_metadata!(
        "logging_plugin",
        version = "1.0.0",
        description = "日志记录插件，跟踪所有事务",
        author = "ModuForge Team",
        tags = ["logging", "monitoring"]
    ),
    append_transaction = logging_append_transaction,
    docs = "日志记录插件，用于跟踪和记录所有事务操作"
);

// 3. 带配置的插件
mf_plugin!(
    security_plugin,
    config = mf_plugin_config!(
        enabled = true,
        priority = 100,
        settings = {
            "strict_mode" => true,
            "max_attempts" => 3,
            "timeout_seconds" => 30
        }
    ),
    filter_transaction = security_filter_transaction,
    docs = "安全插件，提供事务安全检查和过滤功能"
);

// 4. 完整功能插件
mf_plugin!(
    validation_plugin,
    metadata = mf_meta!(
        version = "2.1.0",
        description = "事务验证插件，确保数据完整性",
        author = "ModuForge Team",
        dependencies = ["logging_plugin"],
        conflicts = ["legacy_validator"],
        state_fields = ["validation_state"],
        tags = ["validation", "integrity", "data"]
    ),
    config = mf_plugin_config!(
        enabled = true,
        priority = 50,
        settings = {
            "validation_level" => "strict",
            "auto_repair" => false,
            "batch_size" => 100
        }
    ),
    append_transaction = validation_append_transaction,
    filter_transaction = audit_filter_transaction,
    docs = "全功能验证插件，提供数据完整性检查和修复功能"
);

// 5. 可配置插件示例
mf_plugin_with_config!(
    dynamic_plugin,
    config = {
        name: String,
        enabled: bool,
        log_level: u32
    },
    init_fn = |name: String, enabled: bool, log_level: u32| {
        use mf_state::plugin::{PluginSpec, PluginTrait};
        use std::sync::Arc;
        use async_trait::async_trait;

        // 创建动态元数据
        let metadata = mf_plugin_metadata!(
            &name,
            version = "1.0.0",
            description = "动态配置插件",
            author = "ModuForge Team"
        );

        let config = mf_plugin_config!(
            enabled = enabled,
            priority = log_level as i32
        );

        #[derive(Debug)]
        struct DynamicPluginImpl {
            metadata: PluginMetadata,
            config: PluginConfig,
            log_level: u32,
        }

        #[async_trait]
        impl PluginTrait for DynamicPluginImpl {
            fn metadata(&self) -> PluginMetadata {
                self.metadata.clone()
            }

            fn config(&self) -> PluginConfig {
                self.config.clone()
            }

            async fn append_transaction(
                &self,
                trs: &[Arc<Transaction>],
                _old_state: &Arc<State>,
                _new_state: &Arc<State>,
            ) -> StateResult<Option<Transaction>> {
                if self.log_level > 0 {
                    println!("🔧 [{}] 处理 {} 个事务", self.metadata.name, trs.len());
                }
                Ok(None)
            }

            async fn filter_transaction(
                &self,
                tr: &Transaction,
                _state: &State,
            ) -> bool {
                if self.log_level > 1 {
                    println!("🔧 [{}] 检查事务过滤条件", self.metadata.name);
                }

                // 根据日志级别进行不同的过滤策略
                match self.log_level {
                    0 => {
                        // 级别0: 只允许基本操作
                        let allowed_operations = ["read", "query", "get"];
                        let steps = &tr.steps;
                        for step in steps {
                            if !allowed_operations.iter().any(|op| step.name().contains(op)) {
                                if self.log_level > 0 {
                                    println!("🔧 [{}] 🚫 级别0限制: 不允许 {}", self.metadata.name, step.name());
                                }
                                return false;
                            }
                        }
                    },
                    1 => {
                        // 级别1: 允许读写，禁止删除
                        let steps = &tr.steps;
                        for step in steps {
                            if step.name().contains("delete") || step.name().contains("Drop") {
                                if self.log_level > 0 {
                                    println!("🔧 [{}] 🚫 级别1限制: 不允许删除操作 {}", self.metadata.name, step.name());
                                }
                                return false;
                            }
                        }
                    },
                    2 => {
                        // 级别2: 允许大部分操作，但限制批量操作
                        if tr.steps.len() > 50 {
                            println!("🔧 [{}] 🚫 级别2限制: 批量操作过大 ({})", self.metadata.name, tr.steps.len());
                            return false;
                        }
                    },
                    _ => {
                        // 级别3+: 允许所有操作
                        if self.log_level > 2 {
                            println!("🔧 [{}] ✅ 级别{}+: 允许所有操作", self.metadata.name, self.log_level);
                        }
                    }
                }

                if self.log_level > 1 {
                    println!("🔧 [{}] ✅ 事务过滤通过", self.metadata.name);
                }
                true
            }
        }

        PluginSpec {
            state_field: None,
            tr: Arc::new(DynamicPluginImpl {
                metadata,
                config,
                log_level,
            }),
        }
    },
    docs = "可动态配置名称、启用状态和日志级别的插件"
);

fn main() {
    println!("=== ModuForge 插件宏示例 ===\n");

    // 1. 基础插件演示
    println!("1. 基础插件:");
    let plugin = simple_plugin::new();
    let metadata = plugin.get_metadata();
    println!("   - 名称: {}", metadata.name);
    println!("   - 版本: {}", metadata.version);
    println!("   - 描述: {}", metadata.description);
    println!();

    // 2. 日志插件演示
    println!("2. 日志插件:");
    let plugin = logging_plugin::new();
    let metadata = plugin.get_metadata();
    println!("   - 名称: {}", metadata.name);
    println!("   - 版本: {}", metadata.version);
    println!("   - 描述: {}", metadata.description);
    println!("   - 作者: {}", metadata.author);
    println!("   - 标签: {:?}", metadata.tags);
    println!();

    // 3. 安全插件演示
    println!("3. 安全插件:");
    let plugin = security_plugin::new();
    let config = plugin.get_config();
    println!("   - 启用状态: {}", config.enabled);
    println!("   - 优先级: {}", config.priority);
    println!("   - 配置项:");
    for (key, value) in &config.settings {
        println!("     * {}: {}", key, value);
    }
    println!();

    // 4. 验证插件演示
    println!("4. 完整验证插件:");
    let plugin = validation_plugin::new();
    let metadata = plugin.get_metadata();
    let config = plugin.get_config();

    println!("   元数据:");
    println!("     - 名称: {}", metadata.name);
    println!("     - 版本: {}", metadata.version);
    println!("     - 描述: {}", metadata.description);
    println!("     - 依赖: {:?}", metadata.dependencies);
    println!("     - 冲突: {:?}", metadata.conflicts);
    println!("     - 状态字段: {:?}", metadata.state_fields);

    println!("   配置:");
    println!("     - 启用: {}", config.enabled);
    println!("     - 优先级: {}", config.priority);
    println!("     - 设置: {:?}", config.settings);
    println!();

    // 5. 动态配置插件演示
    println!("5. 动态配置插件:");
    let plugin1 = dynamic_plugin::new("高级日志器".to_string(), true, 2);
    let plugin2 = dynamic_plugin::new("简单监控器".to_string(), true, 0);

    println!("   插件1: {}", plugin1.get_metadata().name);
    println!("   插件2: {}", plugin2.get_metadata().name);
    println!();

    // 6. 插件规范演示
    println!("6. 插件规范和架构:");
    let spec1 = simple_plugin::spec();
    let spec2 = logging_plugin::spec();

    println!("   - 简单插件有状态字段: {}", spec1.state_field.is_some());
    println!("   - 日志插件有状态字段: {}", spec2.state_field.is_some());
    println!("   - 插件特征对象类型安全: ✓");
    println!();

    println!("=== 类型安全和编译时验证 ===");
    println!("✅ 所有插件都实现了 PluginTrait");
    println!("✅ 元数据和配置在编译时验证");
    println!("✅ 支持条件功能（事务处理、过滤等）");
    println!("✅ 类型安全的插件规范创建");
    println!("✅ 零成本抽象 - 运行时无开销");

    println!("\n=== 示例完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_plugins_creation() {
        // 测试所有插件都能正确创建
        let _p1 = simple_plugin::new();
        let _p2 = logging_plugin::new();
        let _p3 = security_plugin::new();
        let _p4 = validation_plugin::new();
        let _p5 = dynamic_plugin::new("test".to_string(), true, 1);

        println!("所有插件创建成功!");
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = validation_plugin::new();
        let metadata = plugin.get_metadata();

        assert_eq!(metadata.name, "validation_plugin");
        assert_eq!(metadata.version, "2.1.0");
        assert_eq!(metadata.description, "事务验证插件，确保数据完整性");
        assert_eq!(metadata.dependencies, vec!["logging_plugin"]);
        assert_eq!(metadata.conflicts, vec!["legacy_validator"]);
        assert_eq!(metadata.tags, vec!["validation", "integrity", "data"]);
    }

    #[test]
    fn test_plugin_config() {
        let plugin = security_plugin::new();
        let config = plugin.get_config();

        assert!(config.enabled);
        assert_eq!(config.priority, 100);
        assert_eq!(
            config.settings.get("strict_mode").unwrap(),
            &serde_json::json!(true)
        );
        assert_eq!(
            config.settings.get("max_attempts").unwrap(),
            &serde_json::json!(3)
        );
    }

    #[test]
    fn test_dynamic_plugin() {
        let plugin = dynamic_plugin::new("动态测试插件".to_string(), true, 5);

        let metadata = plugin.get_metadata();
        assert_eq!(metadata.name, "动态测试插件");

        let config = plugin.get_config();
        assert!(config.enabled);
        assert_eq!(config.priority, 5);
    }

    #[test]
    fn test_plugin_specs() {
        let spec1 = simple_plugin::spec();
        let spec2 = validation_plugin::spec();

        // 简单插件没有状态字段
        assert!(spec1.state_field.is_none());

        // 验证插件没有状态字段（在这个示例中）
        assert!(spec2.state_field.is_none());

        // 都有有效的 trait 实现
        assert_eq!(spec1.tr.metadata().name, "simple_plugin");
        assert_eq!(spec2.tr.metadata().name, "validation_plugin");
    }
}

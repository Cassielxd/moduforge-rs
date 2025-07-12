use async_trait::async_trait;
use std::{
    sync::Arc,
    time::{Instant, SystemTime},
};
use mf_core::{
    middleware::{Middleware},
    error::ForgeResult,
};
use mf_state::{state::State, transaction::Transaction};
use anyhow;

/// 日志记录中间件
/// 记录所有事务的处理过程和结果
#[derive(Debug)]
pub struct LoggingMiddleware {
    name: String,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self { name: "LoggingMiddleware".to_string() }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    fn name(&self) -> String {
        "LoggingMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        let action = transaction
            .get_meta::<String>("action")
            .unwrap_or("unknown".to_string());

        println!(
            "🔍 [{}] 事务处理开始 - ID: {}, 动作: {}",
            self.name, transaction.id, action
        );

        // 记录处理开始时间
        transaction.set_meta("middleware_start_time", SystemTime::now());

        Ok(())
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        for transaction in transactions {
            let action = transaction
                .get_meta::<String>("action").unwrap_or("unknown".to_string());

            let start_time =
                transaction.get_meta::<SystemTime>("middleware_start_time");
            let duration_info = if let Some(start_time) = start_time {
                if let Ok(duration) = start_time.elapsed() {
                    format!(" (耗时: {:?})", duration)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            println!(
                "✅ [{}] 事务处理完成 - ID: {}, 动作: {}{}",
                self.name, transaction.id, action, duration_info
            );
        }

        if let Some(state) = state {
            println!(
                "📊 [{}] 当前状态版本: {}, 插件数量: {}",
                self.name,
                state.version,
                state.plugins().len()
            );
        }

        Ok(None)
    }
}

/// 性能监控中间件
/// 监控事务处理性能和系统资源使用情况
#[derive(Debug)]
pub struct MetricsMiddleware {
    name: String,
    transaction_count: std::sync::atomic::AtomicU64,
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {
            name: "MetricsMiddleware".to_string(),
            transaction_count: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl Middleware for MetricsMiddleware {
    fn name(&self) -> String {
        "MetricsMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        // 记录性能监控开始时间
        transaction.set_meta("metrics_start_time", SystemTime::now());

        let count = self
            .transaction_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        println!(
            "📈 [{}] 开始性能监控 - 事务 #{}, 步骤数: {}",
            self.name,
            count,
            transaction.steps.len()
        );

        Ok(())
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        for transaction in transactions {
            if let Some(start_time) =
                transaction.get_meta::<SystemTime>("metrics_start_time")
            {
                let duration = start_time.elapsed();
                let steps_count = transaction.steps.len();

                println!("⚡ [{}] 性能报告:", self.name);
                println!("   - 处理时间: {:?}", duration);
                println!("   - 步骤数量: {}", steps_count);

                if let Some(state) = &state {
                    println!("   - 状态版本: {}", state.version);
                    println!(
                        "   - 字段实例数: {}",
                        state.fields_instances.len()
                    );
                }


                if steps_count > 10 {
                    println!(
                        "⚠️  [{}] 复杂度警告: 事务步骤数量较多 ({})",
                        self.name, steps_count
                    );
                }
            }
        }

        Ok(None)
    }
}

/// 验证中间件
/// 验证事务的合法性和完整性
#[derive(Debug)]
pub struct ValidationMiddleware {
    name: String,
}

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self { name: "ValidationMiddleware".to_string() }
    }

    fn validate_transaction_basic(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        // 只进行基本验证，不检查在execute中设置的元数据
        if transaction.id == 0 {
            return Err("事务ID无效".to_string());
        }

        Ok(())
    }

    fn validate_transaction_post(
        &self,
        transaction: &Transaction,
    ) -> Result<(), String> {
        // 在after_dispatch中进行详细验证（此时元数据已设置）

        // 验证必需的元数据
        if transaction.get_meta::<String>("action").is_none() {
            return Err("缺少动作元数据".to_string());
        }

        // 验证特定动作的参数
        if let Some(action) = transaction.get_meta::<String>("action") {
            match action.as_str() {
                "user_login" => {
                    if transaction.get_meta::<String>("username").is_none() {
                        return Err("用户登录需要username参数".to_string());
                    }
                },
                "document_edit" => {
                    if transaction.get_meta::<String>("user_id").is_none() {
                        return Err("文档编辑需要user_id参数".to_string());
                    }
                    if transaction.get_meta::<String>("document_id").is_none() {
                        return Err("文档编辑需要document_id参数".to_string());
                    }
                },
                "permission_check" => {
                    if transaction.get_meta::<String>("user_id").is_none() {
                        return Err("权限检查需要user_id参数".to_string());
                    }
                    if transaction.get_meta::<String>("resource").is_none() {
                        return Err("权限检查需要resource参数".to_string());
                    }
                },
                _ => {
                    // 其他动作的验证
                },
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Middleware for ValidationMiddleware {
    fn name(&self) -> String {
        "ValidationMiddleware".to_string()
    }
    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        println!("🔒 [{}] 开始事务验证", self.name);

        // 执行基本验证
        if let Err(error) = self.validate_transaction_basic(transaction) {
            println!("❌ [{}] 验证失败: {}", self.name, error);
            return Err(anyhow::anyhow!("验证失败: {}", error));
        }

        println!("✅ [{}] 事务验证通过", self.name);

        // 添加验证标记
        transaction.set_meta("validated", true);
        transaction.set_meta("validation_time", SystemTime::now());

        Ok(())
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        // 后置验证 - 检查事务和状态
        for transaction in transactions {
            if let Err(error) = self.validate_transaction_post(transaction) {
                println!("❌ [{}] 后置验证失败: {}", self.name, error);
                // 继续处理而不是失败，只记录警告
            }
        }

        if let Some(state) = &state {
            println!("🔍 [{}] 执行状态验证", self.name);

            // 验证状态一致性
            if state.version == 0 {
                println!("⚠️  [{}] 警告: 状态版本为0", self.name);
            }

            // 验证插件状态
            let plugin_count = state.plugins().len();
            let field_count = state.fields_instances.len();

            if plugin_count != field_count {
                println!(
                    "⚠️  [{}] 警告: 插件数量({})与字段实例数量({})不匹配",
                    self.name, plugin_count, field_count
                );
            }

            println!("✅ [{}] 后置验证完成", self.name);
        }

        // 检查是否需要生成额外的验证事务
        let validation_needed = transactions.iter().any(|tr| {
            if let Some(action) = tr.get_meta::<String>("action") {
                matches!(action.as_str(), "document_edit" | "permission_check")
            } else {
                false
            }
        });

        if validation_needed {
            if let Some(state) = state {
                // 生成额外的验证事务
                let mut validation_tr = Transaction::new(&state);
                validation_tr.set_meta("generated_by", "validation_middleware");
                validation_tr.set_meta("action", "post_validation");
                validation_tr.set_meta("timestamp", SystemTime::now());
                validation_tr.commit();

                println!(
                    "📋 [{}] 生成后置验证事务: {}",
                    self.name, validation_tr.id
                );

                return Ok(Some(validation_tr));
            }
        }

        Ok(None)
    }
}

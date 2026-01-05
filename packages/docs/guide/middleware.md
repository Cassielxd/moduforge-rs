# 中间件系统

ModuForge-RS 提供了强大的中间件系统，允许在事务处理前后注入自定义逻辑。本指南展示了如何使用中间件系统来实现日志记录、性能监控、权限验证、数据转换等功能。

## 核心概念

### MiddlewareGeneric 特征

所有中间件都必须实现 `MiddlewareGeneric` 特征：

```rust
use mf_model::traits::{DataContainer, SchemaDefinition};
use mf_state::{state::StateGeneric, transaction::TransactionGeneric};
use mf_core::error::ForgeResult;

#[async_trait::async_trait]
pub trait MiddlewareGeneric<C, S>: Send + Sync
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    /// 返回中间件的名称
    fn name(&self) -> String;

    /// 在事务到达核心分发之前处理事务
    async fn before_dispatch(
        &self,
        transaction: &mut TransactionGeneric<C, S>,
    ) -> ForgeResult<()> {
        Ok(())
    }

    /// 在核心分发之后处理结果
    async fn after_dispatch(
        &self,
        state: Option<Arc<StateGeneric<C, S>>>,
        transactions: &[Arc<TransactionGeneric<C, S>>],
    ) -> ForgeResult<Option<TransactionGeneric<C, S>>> {
        Ok(None)
    }
}
```

### 中间件栈

中间件通过 `MiddlewareStackGeneric` 组织成栈式结构：

```rust
pub struct MiddlewareStackGeneric<C, S>
where
    C: DataContainer + 'static,
    S: SchemaDefinition<Container = C> + 'static,
{
    pub middlewares: Vec<Arc<dyn MiddlewareGeneric<C, S>>>,
}

impl<C, S> MiddlewareStackGeneric<C, S> {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add<M>(&mut self, middleware: M)
    where
        M: MiddlewareGeneric<C, S> + 'static,
    {
        self.middlewares.push(Arc::new(middleware));
    }
}
```

## 实际中间件实现

### 1. 日志中间件

记录每个事务的执行过程：

```rust
use mf_core::{Middleware, ForgeResult};
use mf_state::{State, Transaction};
use std::sync::Arc;
use tracing::{info, debug};

#[derive(Debug)]
pub struct LoggingMiddleware {
    name: String,
    log_level: LogLevel,
}

#[derive(Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        match self.log_level {
            LogLevel::Debug => {
                debug!(
                    "🔍 [{}] 事务处理开始 - ID: {}, Type: {:?}",
                    self.name,
                    transaction.id,
                    transaction.steps
                );
            }
            LogLevel::Info => {
                info!(
                    "📋 [{}] 处理事务: {}",
                    self.name,
                    transaction.id
                );
            }
            LogLevel::Warn => {
                // 只记录警告级别的事务
            }
        }
        Ok(())
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Arc<Transaction>],
    ) -> ForgeResult<Option<Transaction>> {
        info!(
            "✅ [{}] 事务处理完成 - 共 {} 个事务",
            self.name,
            transactions.len()
        );

        if let Some(state) = state {
            debug!("📊 状态版本: {}", state.version);
        }

        Ok(None)
    }
}
```

### 2. 性能监控中间件

监控事务执行性能并记录指标：

```rust
use std::time::Instant;
use mf_core::metrics;

pub struct PerformanceMiddleware {
    name: String,
    threshold_ms: u64, // 性能警告阈值
}

#[async_trait::async_trait]
impl Middleware for PerformanceMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        // 在事务中存储开始时间
        transaction.metadata.insert(
            "perf_start".to_string(),
            serde_json::json!(Instant::now()),
        );
        Ok(())
    }

    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Arc<Transaction>],
    ) -> ForgeResult<Option<Transaction>> {
        for transaction in transactions {
            if let Some(start) = transaction.metadata.get("perf_start") {
                let duration = start.elapsed();

                // 记录性能指标
                metrics::middleware_execution_duration(
                    duration,
                    "transaction",
                    &self.name,
                );

                // 如果超过阈值，记录警告
                if duration.as_millis() > self.threshold_ms {
                    warn!(
                        "⚠️ 事务 {} 执行时间过长: {}ms",
                        transaction.id,
                        duration.as_millis()
                    );
                }
            }
        }
        Ok(None)
    }
}
```

### 3. 权限验证中间件

在事务执行前验证用户权限：

```rust
pub struct AuthorizationMiddleware {
    name: String,
    permission_checker: Arc<dyn PermissionChecker>,
}

#[async_trait::async_trait]
pub trait PermissionChecker: Send + Sync {
    async fn check_permission(
        &self,
        user_id: &str,
        action: &str,
        resource: &str,
    ) -> bool;
}

#[async_trait::async_trait]
impl Middleware for AuthorizationMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        // 从事务元数据中获取用户信息
        let user_id = transaction.metadata
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("缺少用户身份信息"))?;

        // 检查每个步骤的权限
        for step in &transaction.steps {
            let action = step.action.as_str();
            let resource = step.target_id.as_ref()
                .map(|id| id.as_str())
                .unwrap_or("*");

            if !self.permission_checker
                .check_permission(user_id, action, resource)
                .await
            {
                return Err(anyhow::anyhow!(
                    "权限不足: 用户 {} 无法执行 {} 操作",
                    user_id,
                    action
                ));
            }
        }

        Ok(())
    }
}
```

### 4. 数据验证中间件

验证事务数据的完整性和有效性：

```rust
pub struct ValidationMiddleware {
    name: String,
    validators: HashMap<String, Box<dyn Validator>>,
}

#[async_trait::async_trait]
pub trait Validator: Send + Sync {
    async fn validate(&self, data: &serde_json::Value) -> ForgeResult<()>;
}

#[async_trait::async_trait]
impl Middleware for ValidationMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        for step in &transaction.steps {
            // 根据操作类型选择验证器
            if let Some(validator) = self.validators.get(&step.action) {
                // 验证步骤数据
                validator.validate(&step.params).await?;
            }
        }
        Ok(())
    }
}

// 具体的验证器实现
pub struct PriceValidator {
    min_value: f64,
    max_value: f64,
}

#[async_trait::async_trait]
impl Validator for PriceValidator {
    async fn validate(&self, data: &serde_json::Value) -> ForgeResult<()> {
        if let Some(price) = data.get("price").and_then(|v| v.as_f64()) {
            if price < self.min_value || price > self.max_value {
                return Err(anyhow::anyhow!(
                    "价格超出有效范围: {} (允许范围: {}-{})",
                    price,
                    self.min_value,
                    self.max_value
                ));
            }
        }
        Ok(())
    }
}
```

### 5. 缓存中间件

实现事务结果缓存：

```rust
use lru::LruCache;
use std::sync::Mutex;

pub struct CacheMiddleware {
    name: String,
    cache: Arc<Mutex<LruCache<String, Arc<State>>>>,
}

impl CacheMiddleware {
    pub fn new(name: String, capacity: usize) -> Self {
        Self {
            name,
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }

    fn generate_cache_key(&self, transaction: &Transaction) -> String {
        // 根据事务内容生成缓存键
        format!("{:?}", transaction.steps)
    }
}

#[async_trait::async_trait]
impl Middleware for CacheMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn before_dispatch(
        &self,
        transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        // 检查是否有缓存的结果
        let cache_key = self.generate_cache_key(transaction);

        if let Ok(mut cache) = self.cache.lock() {
            if let Some(cached_state) = cache.get(&cache_key) {
                info!("🎯 缓存命中: {}", cache_key);
                // 可以通过元数据标记使用了缓存
                transaction.metadata.insert(
                    "cache_hit".to_string(),
                    serde_json::json!(true),
                );
            }
        }

        Ok(())
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Arc<Transaction>],
    ) -> ForgeResult<Option<Transaction>> {
        // 缓存新的结果
        if let Some(state) = state {
            for transaction in transactions {
                let cache_key = self.generate_cache_key(transaction);

                if let Ok(mut cache) = self.cache.lock() {
                    cache.put(cache_key.clone(), state.clone());
                    debug!("📦 缓存更新: {}", cache_key);
                }
            }
        }

        Ok(None)
    }
}
```

## Price-RS 项目中的中间件配置

### Bootstrap 中间件提供器

Price-RS 使用提供器模式配置中间件：

```rust
use price_common::bootstrap::{
    BootstrapResult, MiddlewareProvider, ProjectProfile, ProjectPhase
};
use mf_core::middleware::MiddlewareStack;

/// 默认的中间件提供器
/// 根据项目阶段配置不同的中间件栈
pub struct DefaultMiddlewareProvider;

#[async_trait]
impl MiddlewareProvider for DefaultMiddlewareProvider {
    async fn provide(
        &self,
        profile: &ProjectProfile,
    ) -> BootstrapResult<MiddlewareStack> {
        let mut stack = MiddlewareStack::new();

        // 根据项目阶段添加不同的中间件
        match profile.phase {
            ProjectPhase::Budget => {
                // 预算项目的中间件配置

                // 1. 添加日志中间件
                stack.add(LoggingMiddleware {
                    name: "BudgetLogger".to_string(),
                    log_level: LogLevel::Info,
                });

                // 2. 添加性能监控
                stack.add(PerformanceMiddleware {
                    name: "BudgetPerformance".to_string(),
                    threshold_ms: 1000,
                });

                // 3. 添加价格验证
                let mut validators = HashMap::new();
                validators.insert(
                    "update_price".to_string(),
                    Box::new(PriceValidator {
                        min_value: 0.0,
                        max_value: 999999999.99,
                    }) as Box<dyn Validator>,
                );

                stack.add(ValidationMiddleware {
                    name: "BudgetValidation".to_string(),
                    validators,
                });

                // 4. 添加费用汇总中间件（后置处理器）
                stack.add(CostAggregationMiddleware::new());
            }

            ProjectPhase::Settlement => {
                // 结算项目的中间件配置
                stack.add(SettlementAuthMiddleware::new());
                stack.add(AuditLogMiddleware::new());
            }

            _ => {
                // 其他阶段使用基础中间件
                stack.add(LoggingMiddleware {
                    name: "DefaultLogger".to_string(),
                    log_level: LogLevel::Warn,
                });
            }
        }

        Ok(stack)
    }

    fn name(&self) -> &'static str {
        "default_middleware_provider"
    }
}
```

### 费用汇总中间件示例

Price-RS 特有的业务中间件：

```rust
/// 费用汇总中间件
/// 在后置处理中自动计算和更新费用汇总
pub struct CostAggregationMiddleware {
    name: String,
}

impl CostAggregationMiddleware {
    pub fn new() -> Self {
        Self {
            name: "CostAggregation".to_string(),
        }
    }

    async fn aggregate_costs(
        &self,
        state: &State,
        node_id: &str,
    ) -> ForgeResult<f64> {
        let node = state.doc.get_node(node_id)
            .ok_or_else(|| anyhow::anyhow!("节点不存在"))?;

        let mut total = 0.0;

        // 递归计算子节点费用
        for child_id in node.children() {
            let child = state.doc.get_node(child_id).unwrap();

            match child.r#type.as_str() {
                "FbNode" | "QdNode" => {
                    // 分部或清单节点，递归计算
                    total += self.aggregate_costs(state, child_id).await?;
                }
                "UnitProjectNode" => {
                    // 单位工程节点，直接获取价格
                    if let Some(price) = child.attrs.get("total_price")
                        .and_then(|v| v.as_f64()) {
                        total += price;
                    }
                }
                _ => {}
            }
        }

        Ok(total)
    }
}

#[async_trait::async_trait]
impl Middleware for CostAggregationMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn after_dispatch(
        &self,
        state: Option<Arc<State>>,
        transactions: &[Arc<Transaction>],
    ) -> ForgeResult<Option<Transaction>> {
        if let Some(state) = state {
            // 检查是否有价格更新操作
            let needs_aggregation = transactions.iter().any(|t| {
                t.steps.iter().any(|s| {
                    s.action == "update_price" ||
                    s.action == "insert" ||
                    s.action == "delete"
                })
            });

            if needs_aggregation {
                info!("💰 触发费用汇总计算");

                // 获取项目根节点
                if let Some(root) = state.doc.root() {
                    let total = self.aggregate_costs(&state, &root.id).await?;

                    // 创建更新汇总的事务
                    let mut update_transaction = Transaction::new();
                    update_transaction.update_node(
                        root.id.clone(),
                        HashMap::from([
                            ("total_cost".to_string(), json!(total)),
                            ("aggregated_at".to_string(), json!(chrono::Utc::now())),
                        ]),
                    );

                    info!("💰 费用汇总完成: {:.2}", total);

                    // 返回需要额外执行的事务
                    return Ok(Some(update_transaction));
                }
            }
        }

        Ok(None)
    }
}
```

## 中间件执行流程

### 执行辅助器

ModuForge-RS 提供了中间件执行辅助器来管理执行流程：

```rust
use mf_core::helpers::middleware_helper::MiddlewareHelper;
use mf_core::config::ForgeConfig;
use std::time::Duration;

pub struct MiddlewareHelper;

impl MiddlewareHelper {
    /// 执行前置中间件链
    pub async fn run_before_middleware(
        transaction: &mut Transaction,
        middleware_stack: &MiddlewareStack,
        config: &ForgeConfig,
    ) -> ForgeResult<()> {
        let timeout = Duration::from_millis(
            config.performance.middleware_timeout_ms,
        );

        for middleware in &middleware_stack.middlewares {
            let start_time = Instant::now();

            // 使用超时保护
            match tokio::time::timeout(
                timeout,
                middleware.before_dispatch(transaction),
            )
            .await
            {
                Ok(Ok(())) => {
                    // 记录性能指标
                    metrics::middleware_execution_duration(
                        start_time.elapsed(),
                        "before",
                        middleware.name().as_str(),
                    );
                }
                Ok(Err(e)) => {
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行失败: {e}"
                    )));
                }
                Err(_) => {
                    return Err(error_utils::middleware_error(format!(
                        "前置中间件执行超时（{}ms）",
                        config.performance.middleware_timeout_ms
                    )));
                }
            }
        }

        Ok(())
    }

    /// 执行后置中间件链
    pub async fn run_after_middleware(
        state: &mut Option<Arc<State>>,
        transactions: &mut [Arc<Transaction>],
        middleware_stack: &MiddlewareStack,
        config: &ForgeConfig,
    ) -> ForgeResult<()> {
        // 类似的后置处理逻辑...
    }
}
```

## 最佳实践

### 1. 中间件顺序

中间件的执行顺序很重要：

```rust
let mut stack = MiddlewareStack::new();

// 1. 认证/授权（最先执行）
stack.add(AuthenticationMiddleware::new());
stack.add(AuthorizationMiddleware::new());

// 2. 验证
stack.add(ValidationMiddleware::new());

// 3. 日志（在验证后，避免记录无效请求）
stack.add(LoggingMiddleware::new());

// 4. 性能监控
stack.add(PerformanceMiddleware::new());

// 5. 业务逻辑
stack.add(BusinessLogicMiddleware::new());

// 6. 缓存（最后执行，确保只缓存有效结果）
stack.add(CacheMiddleware::new());
```

### 2. 错误处理

中间件应该优雅地处理错误：

```rust
async fn before_dispatch(
    &self,
    transaction: &mut Transaction,
) -> ForgeResult<()> {
    // 使用 ? 操作符传播错误
    let result = self.validate_transaction(transaction)?;

    // 或者转换错误
    self.check_permission(transaction)
        .map_err(|e| anyhow::anyhow!("权限验证失败: {}", e))?;

    // 记录但不中断的错误
    if let Err(e) = self.optional_check(transaction) {
        warn!("可选检查失败（继续执行）: {}", e);
    }

    Ok(())
}
```

### 3. 性能优化

避免在中间件中执行耗时操作：

```rust
pub struct OptimizedMiddleware {
    // 使用缓存避免重复计算
    cache: Arc<DashMap<String, CachedResult>>,
    // 使用连接池避免频繁建立连接
    db_pool: Arc<DbPool>,
}

impl OptimizedMiddleware {
    async fn expensive_operation(&self, key: &str) -> ForgeResult<Value> {
        // 先检查缓存
        if let Some(cached) = self.cache.get(key) {
            if !cached.is_expired() {
                return Ok(cached.value.clone());
            }
        }

        // 使用连接池而不是创建新连接
        let conn = self.db_pool.get().await?;
        let value = conn.query(key).await?;

        // 更新缓存
        self.cache.insert(key.to_string(), CachedResult::new(value.clone()));

        Ok(value)
    }
}
```

### 4. 测试中间件

为中间件编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mf_test::create_test_transaction;

    #[tokio::test]
    async fn test_validation_middleware() {
        let middleware = ValidationMiddleware::new();
        let mut transaction = create_test_transaction();

        // 测试有效数据
        transaction.metadata.insert(
            "price".to_string(),
            json!(100.0),
        );

        let result = middleware.before_dispatch(&mut transaction).await;
        assert!(result.is_ok());

        // 测试无效数据
        transaction.metadata.insert(
            "price".to_string(),
            json!(-100.0),
        );

        let result = middleware.before_dispatch(&mut transaction).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_middleware_stack_order() {
        let mut stack = MiddlewareStack::new();

        // 添加多个中间件
        stack.add(FirstMiddleware::new());
        stack.add(SecondMiddleware::new());
        stack.add(ThirdMiddleware::new());

        // 验证执行顺序
        let mut transaction = create_test_transaction();

        for middleware in &stack.middlewares {
            middleware.before_dispatch(&mut transaction).await.unwrap();
        }

        // 检查元数据以验证执行顺序
        assert_eq!(
            transaction.metadata.get("execution_order").unwrap(),
            &json!(["first", "second", "third"])
        );
    }
}
```

## 配置示例

### 在运行时配置中间件

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeOptions};
use price_budget::bootstrap::DefaultMiddlewareProvider;

async fn setup_runtime() -> ForgeResult<ForgeRuntime> {
    // 创建中间件提供器
    let middleware_provider = DefaultMiddlewareProvider;

    // 获取项目配置
    let profile = ProjectProfile {
        phase: ProjectPhase::Budget,
        // 其他配置...
    };

    // 创建中间件栈
    let middleware_stack = middleware_provider.provide(&profile).await?;

    // 配置运行时选项
    let options = RuntimeOptions {
        middleware_stack: Some(middleware_stack),
        // 其他选项...
    };

    // 构建运行时
    let runtime = ForgeRuntimeBuilder::new()
        .with_config(config)
        .with_options(options)
        .build()
        .await?;

    Ok(runtime)
}
```

## 总结

ModuForge-RS 的中间件系统提供了：

- **灵活的扩展点**：在事务处理前后注入自定义逻辑
- **类型安全**：泛型设计支持不同的数据容器和模式定义
- **性能保护**：内置超时机制和性能监控
- **栈式组织**：有序执行，易于管理
- **业务集成**：Price-RS 展示了如何集成复杂业务逻辑

通过中间件，您可以轻松实现日志记录、性能监控、权限验证、数据验证、缓存、费用汇总等功能，而无需修改核心业务逻辑。
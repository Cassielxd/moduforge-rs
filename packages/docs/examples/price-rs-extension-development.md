# Price-RS 扩展开发实战

本文档详细介绍 Price-RS 项目中的扩展开发模式，展示如何使用 ModuForge-RS 的扩展系统构建复杂的业务功能。

## 扩展系统架构

### 扩展组织结构

Price-RS 的每个扩展都遵循统一的目录结构：

```
exts/
├── [extension-name]/
│   ├── [extension-name]/           # 实现包
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # 扩展入口
│   │       ├── nodes/              # 节点定义
│   │       │   ├── mod.rs
│   │       │   └── node_definitions.rs
│   │       ├── plugins/            # 插件实现
│   │       │   ├── mod.rs
│   │       │   └── plugin_*.rs
│   │       ├── command/            # 命令定义
│   │       │   ├── mod.rs
│   │       │   ├── insert.rs
│   │       │   ├── update.rs
│   │       │   └── delete.rs
│   │       └── router/             # API 路由（可选）
│   │           ├── mod.rs
│   │           └── handlers.rs
│   │
│   └── [extension-name]-interface/ # 接口包
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs              # 公共接口定义
```

## 完整扩展示例：标准换算扩展 (bzhs)

### 1. 接口定义

首先定义扩展的公共接口：

```rust
// exts/bzhs/extension-bzhs-interface/src/lib.rs

use serde::{Deserialize, Serialize};
use moduforge_core::ForgeResult;

/// 标准换算服务接口
#[async_trait]
pub trait StandardConversionService: Send + Sync {
    /// 获取换算标准
    async fn get_standard(&self, code: &str) -> ForgeResult<ConversionStandard>;

    /// 执行换算
    async fn convert(
        &self,
        value: f64,
        from_unit: &str,
        to_unit: &str,
        standard: &str
    ) -> ForgeResult<f64>;

    /// 批量换算
    async fn batch_convert(
        &self,
        items: Vec<ConversionRequest>
    ) -> ForgeResult<Vec<ConversionResult>>;
}

/// 换算标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionStandard {
    pub code: String,
    pub name: String,
    pub category: String,
    pub rules: Vec<ConversionRule>,
}

/// 换算规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRule {
    pub from_unit: String,
    pub to_unit: String,
    pub factor: f64,
    pub formula: Option<String>,
}

/// 换算请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRequest {
    pub id: String,
    pub value: f64,
    pub from_unit: String,
    pub to_unit: String,
    pub standard: String,
}

/// 换算结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub id: String,
    pub original_value: f64,
    pub converted_value: f64,
    pub from_unit: String,
    pub to_unit: String,
    pub factor_used: f64,
}
```

### 2. 节点定义

定义标准换算相关的节点：

```rust
// exts/bzhs/extension-bzhs/src/nodes/node_definitions.rs

use mf_derive::Node;
use serde::{Deserialize, Serialize};

/// 标准换算容器节点
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "bzhs_container"]
#[desc = "标准换算容器"]
#[content = "(bzhs_group|bzhs_item)+"]
pub struct BzhsContainerNode {
    #[attr]
    pub project_id: String,              // 项目ID

    #[attr]
    pub standard_code: String,           // 标准代码

    #[attr]
    pub standard_name: String,           // 标准名称

    #[attr]
    pub version: String,                 // 版本号

    #[attr]
    pub effective_date: Option<String>,  // 生效日期
}

/// 标准换算组节点
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "bzhs_group"]
#[desc = "标准换算组"]
#[content = "bzhs_item+"]
pub struct BzhsGroupNode {
    #[attr]
    pub group_code: String,              // 组代码

    #[attr]
    pub group_name: String,              // 组名称

    #[attr]
    pub category: String,                // 类别

    #[attr]
    pub description: Option<String>,     // 描述
}

/// 标准换算项节点
#[derive(Node, Debug, Clone, Serialize, Deserialize)]
#[node_type = "bzhs_item"]
#[desc = "标准换算项"]
pub struct BzhsItemNode {
    #[attr]
    pub item_code: String,               // 项目代码

    #[attr]
    pub item_name: String,               // 项目名称

    #[attr]
    pub source_unit: String,             // 源单位

    #[attr]
    pub target_unit: String,             // 目标单位

    #[attr]
    pub conversion_factor: f64,          // 换算系数

    #[attr]
    pub formula: Option<String>,         // 换算公式

    #[attr]
    pub precision: i32,                  // 精度

    #[attr(default = true)]
    pub is_active: bool,                 // 是否启用

    #[attr]
    pub notes: Option<String>,           // 备注
}
```

### 3. 插件实现

实现标准换算的核心逻辑：

```rust
// exts/bzhs/extension-bzhs/src/plugins/plugin_bzhs.rs

use mf_state::{Plugin, Transaction, State};
use mf_core::ForgeResult;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct BzhsPlugin {
    // 缓存换算规则
    conversion_cache: HashMap<String, ConversionRule>,
    // 配置
    config: BzhsConfig,
}

impl BzhsPlugin {
    pub fn new() -> Self {
        Self {
            conversion_cache: HashMap::new(),
            config: BzhsConfig::default(),
        }
    }

    /// 执行换算
    fn perform_conversion(
        &self,
        value: f64,
        item: &BzhsItemNode
    ) -> ForgeResult<f64> {
        // 检查是否有公式
        if let Some(formula) = &item.formula {
            self.evaluate_formula(value, formula)
        } else {
            // 使用换算系数
            Ok(value * item.conversion_factor)
        }
    }

    /// 计算公式
    fn evaluate_formula(&self, value: f64, formula: &str) -> ForgeResult<f64> {
        // 使用表达式引擎计算
        let context = HashMap::from([("x", value)]);

        zen_expression::evaluate(formula, context)
            .map_err(|e| ForgeError::Custom(format!("公式计算失败: {}", e)))
    }

    /// 验证换算合理性
    fn validate_conversion(
        &self,
        original: f64,
        converted: f64,
        factor: f64
    ) -> bool {
        // 验证换算结果的合理性
        let expected = original * factor;
        let tolerance = self.config.tolerance;

        (converted - expected).abs() / expected < tolerance
    }
}

#[async_trait]
impl Plugin for BzhsPlugin {
    fn name(&self) -> &str {
        "bzhs_plugin"
    }

    async fn init(&mut self, state: &State) -> ForgeResult<()> {
        // 加载换算规则到缓存
        let items = state.doc.find_nodes_by_type("bzhs_item");

        for node_id in items {
            if let Some(node) = state.doc.get_node(&node_id) {
                if let Ok(item) = node.deserialize::<BzhsItemNode>() {
                    let key = format!("{}:{}", item.source_unit, item.target_unit);
                    self.conversion_cache.insert(key, ConversionRule {
                        factor: item.conversion_factor,
                        formula: item.formula.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    async fn append_transaction(
        &mut self,
        tr: &Transaction,
        old_state: &State,
        new_state: &State,
    ) -> Option<Transaction> {
        // 查找需要换算的节点
        let affected_nodes = self.find_affected_nodes(tr, new_state);

        if affected_nodes.is_empty() {
            return None;
        }

        // 创建更新事务
        let mut update_tr = new_state.tr();

        for (node_id, conversion_item) in affected_nodes {
            // 获取原始值
            if let Some(node) = new_state.doc.get_node(&node_id) {
                if let Some(value) = node.get_attr("value").and_then(|v| v.as_f64()) {
                    // 执行换算
                    let converted = self.perform_conversion(value, &conversion_item)
                        .unwrap_or(value);

                    // 更新节点
                    update_tr.set_node_attribute(
                        &node_id,
                        "converted_value",
                        converted.into()
                    );

                    // 添加换算记录
                    update_tr.add_metadata(&format!("conversion_{}", node_id), json!({
                        "original": value,
                        "converted": converted,
                        "factor": conversion_item.conversion_factor,
                        "units": format!("{} -> {}",
                            conversion_item.source_unit,
                            conversion_item.target_unit)
                    }));
                }
            }
        }

        Some(update_tr)
    }

    async fn validate_state(&self, state: &State) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // 验证所有换算项
        let items = state.doc.find_nodes_by_type("bzhs_item");

        for node_id in items {
            if let Some(node) = state.doc.get_node(&node_id) {
                if let Ok(item) = node.deserialize::<BzhsItemNode>() {
                    // 验证换算系数
                    if item.conversion_factor <= 0.0 {
                        errors.push(ValidationError {
                            node_id: node_id.clone(),
                            field: "conversion_factor".to_string(),
                            message: "换算系数必须大于0".to_string(),
                        });
                    }

                    // 验证单位
                    if item.source_unit == item.target_unit {
                        errors.push(ValidationError {
                            node_id: node_id.clone(),
                            field: "units".to_string(),
                            message: "源单位和目标单位不能相同".to_string(),
                        });
                    }
                }
            }
        }

        errors
    }
}
```

### 4. 命令实现

实现标准换算的命令：

```rust
// exts/bzhs/extension-bzhs/src/command/batch_convert.rs

use mf_state::{Command, Transaction, State};
use mf_core::ForgeResult;
use async_trait::async_trait;

/// 批量换算命令
pub struct BatchConvertCommand {
    pub conversions: Vec<ConversionRequest>,
    pub standard: String,
    pub auto_create_nodes: bool,
}

#[async_trait]
impl Command for BatchConvertCommand {
    fn name(&self) -> &str {
        "batch_convert"
    }

    async fn execute(&self, tr: &mut Transaction) -> ForgeResult<()> {
        // 获取换算标准
        let standard_node = self.find_standard_node(tr, &self.standard)?;

        // 执行批量换算
        for request in &self.conversions {
            // 查找换算规则
            let rule = self.find_conversion_rule(
                &standard_node,
                &request.from_unit,
                &request.to_unit
            )?;

            // 执行换算
            let converted_value = if let Some(formula) = &rule.formula {
                self.evaluate_formula(request.value, formula)?
            } else {
                request.value * rule.factor
            };

            // 创建或更新节点
            if self.auto_create_nodes {
                let node_id = format!("conversion_{}", request.id);

                tr.insert_node(Node::new("conversion_result")
                    .with_attr("request_id", request.id.clone())
                    .with_attr("original_value", request.value)
                    .with_attr("converted_value", converted_value)
                    .with_attr("from_unit", request.from_unit.clone())
                    .with_attr("to_unit", request.to_unit.clone())
                    .with_attr("factor", rule.factor)
                    .with_attr("timestamp", chrono::Utc::now().to_rfc3339())
                );
            }

            // 触发事件
            tr.dispatch_event("conversion_completed", json!({
                "id": request.id,
                "result": converted_value
            }));
        }

        Ok(())
    }

    async fn validate(&self, state: &State) -> ForgeResult<()> {
        // 验证标准是否存在
        let standards = state.doc.find_nodes_by_attr("standard_code", &self.standard);

        if standards.is_empty() {
            return Err(ForgeError::Custom(
                format!("换算标准不存在: {}", self.standard)
            ));
        }

        // 验证请求数据
        for request in &self.conversions {
            if request.value < 0.0 {
                return Err(ForgeError::InvalidValue(
                    format!("换算值不能为负数: {}", request.id)
                ));
            }

            if request.from_unit.is_empty() || request.to_unit.is_empty() {
                return Err(ForgeError::InvalidValue(
                    format!("单位不能为空: {}", request.id)
                ));
            }
        }

        Ok(())
    }

    fn is_idempotent(&self) -> bool {
        // 批量换算是幂等的
        true
    }
}
```

### 5. 服务实现

实现标准换算服务：

```rust
// exts/bzhs/extension-bzhs/src/service.rs

use extension_bzhs_interface::{StandardConversionService, ConversionRequest, ConversionResult};
use mf_core::{ForgeRuntime, ForgeResult};
use std::sync::Arc;

pub struct BzhsService {
    runtime: Arc<ForgeRuntime>,
    cache: Arc<ConversionCache>,
}

impl BzhsService {
    pub fn new(runtime: Arc<ForgeRuntime>) -> Self {
        Self {
            runtime,
            cache: Arc::new(ConversionCache::new()),
        }
    }

    /// 加载换算规则
    async fn load_rules(&self, standard: &str) -> ForgeResult<Vec<ConversionRule>> {
        // 从运行时获取状态
        let state = self.runtime.get_state().await?;

        // 查找标准节点
        let standard_nodes = state.doc.find_nodes_by_attr("standard_code", standard);

        if let Some(standard_id) = standard_nodes.first() {
            // 获取所有换算项
            let items = state.doc.get_children(standard_id, "bzhs_item");

            let mut rules = Vec::new();
            for item_id in items {
                if let Some(node) = state.doc.get_node(&item_id) {
                    if let Ok(item) = node.deserialize::<BzhsItemNode>() {
                        rules.push(ConversionRule {
                            from_unit: item.source_unit,
                            to_unit: item.target_unit,
                            factor: item.conversion_factor,
                            formula: item.formula,
                        });
                    }
                }
            }

            Ok(rules)
        } else {
            Err(ForgeError::NotFound(format!("标准不存在: {}", standard)))
        }
    }
}

#[async_trait]
impl StandardConversionService for BzhsService {
    async fn get_standard(&self, code: &str) -> ForgeResult<ConversionStandard> {
        // 检查缓存
        if let Some(standard) = self.cache.get_standard(code).await {
            return Ok(standard);
        }

        // 从运行时加载
        let rules = self.load_rules(code).await?;

        let standard = ConversionStandard {
            code: code.to_string(),
            name: format!("{} 换算标准", code),
            category: "工程造价".to_string(),
            rules,
        };

        // 缓存结果
        self.cache.set_standard(code, standard.clone()).await;

        Ok(standard)
    }

    async fn convert(
        &self,
        value: f64,
        from_unit: &str,
        to_unit: &str,
        standard: &str
    ) -> ForgeResult<f64> {
        let std = self.get_standard(standard).await?;

        // 查找匹配的规则
        for rule in &std.rules {
            if rule.from_unit == from_unit && rule.to_unit == to_unit {
                if let Some(formula) = &rule.formula {
                    // 使用公式计算
                    return self.evaluate_formula(value, formula);
                } else {
                    // 使用系数
                    return Ok(value * rule.factor);
                }
            }
        }

        // 尝试反向换算
        for rule in &std.rules {
            if rule.from_unit == to_unit && rule.to_unit == from_unit {
                return Ok(value / rule.factor);
            }
        }

        Err(ForgeError::NotFound(
            format!("未找到换算规则: {} -> {}", from_unit, to_unit)
        ))
    }

    async fn batch_convert(
        &self,
        items: Vec<ConversionRequest>
    ) -> ForgeResult<Vec<ConversionResult>> {
        let mut results = Vec::new();

        // 按标准分组以优化性能
        let mut grouped: HashMap<String, Vec<&ConversionRequest>> = HashMap::new();
        for item in &items {
            grouped.entry(item.standard.clone())
                .or_insert_with(Vec::new)
                .push(item);
        }

        // 批量处理每个标准
        for (standard, requests) in grouped {
            let std = self.get_standard(&standard).await?;

            for request in requests {
                let converted = self.convert(
                    request.value,
                    &request.from_unit,
                    &request.to_unit,
                    &standard
                ).await?;

                results.push(ConversionResult {
                    id: request.id.clone(),
                    original_value: request.value,
                    converted_value: converted,
                    from_unit: request.from_unit.clone(),
                    to_unit: request.to_unit.clone(),
                    factor_used: converted / request.value,
                });
            }
        }

        Ok(results)
    }
}
```

### 6. API 路由

提供 REST API 接口：

```rust
// exts/bzhs/extension-bzhs/src/router/mod.rs

use axum::{Router, Json, extract::{Path, State as AxumState}};
use extension_bzhs_interface::{ConversionRequest, ConversionResult};
use crate::service::BzhsService;

pub fn bzhs_routes() -> Router<AppState> {
    Router::new()
        .route("/standards", get(list_standards))
        .route("/standards/:code", get(get_standard))
        .route("/convert", post(convert_single))
        .route("/batch-convert", post(batch_convert))
}

/// 单个换算
async fn convert_single(
    AxumState(state): AxumState<AppState>,
    Json(request): Json<ConversionRequest>,
) -> Result<Json<ConversionResult>, ApiError> {
    let service = state.get_service::<BzhsService>()?;

    let result = service.convert(
        request.value,
        &request.from_unit,
        &request.to_unit,
        &request.standard
    ).await?;

    Ok(Json(ConversionResult {
        id: request.id,
        original_value: request.value,
        converted_value: result,
        from_unit: request.from_unit,
        to_unit: request.to_unit,
        factor_used: result / request.value,
    }))
}

/// 批量换算
async fn batch_convert(
    AxumState(state): AxumState<AppState>,
    Json(requests): Json<Vec<ConversionRequest>>,
) -> Result<Json<Vec<ConversionResult>>, ApiError> {
    let service = state.get_service::<BzhsService>()?;
    let results = service.batch_convert(requests).await?;
    Ok(Json(results))
}
```

### 7. 扩展注册

将扩展注册到 ModuForge-RS：

```rust
// exts/bzhs/extension-bzhs/src/lib.rs

use mf_macro::{mf_extension, mf_op};
use mf_core::{Extension, ForgeResult};

pub mod nodes;
pub mod plugins;
pub mod command;
pub mod service;
pub mod router;

use crate::plugins::plugin_bzhs::BzhsPlugin;
use crate::nodes::node_definitions::{
    BzhsContainerNode,
    BzhsGroupNode,
    BzhsItemNode
};

// 定义扩展操作
mf_op!(setup, |manager| {
    // 注册服务
    let service = Arc::new(BzhsService::new(manager.runtime()));
    manager.register_service("bzhs", service);

    // 注册命令处理器
    manager.register_command_handler(
        "batch_convert",
        BatchConvertCommandHandler::new()
    );

    Ok(())
});

mf_op!(cleanup, |manager| {
    // 清理资源
    manager.unregister_service("bzhs");
    Ok(())
});

// 节点转换
fn transform_nodes(node: &mut Node) -> ForgeResult<()> {
    // 自动设置默认值
    match node.node_type() {
        "bzhs_item" => {
            if !node.has_attr("precision") {
                node.set_attr("precision", 2);
            }
            if !node.has_attr("is_active") {
                node.set_attr("is_active", true);
            }
        }
        _ => {}
    }
    Ok(())
}

// 注册扩展
mf_extension!(
    bzhs,
    version = "1.0.0",
    ops = [setup, cleanup],
    plugins = [BzhsPlugin::new()],
    node_transform = transform_nodes,
    nodes = [
        BzhsContainerNode::node_definition(),
        BzhsGroupNode::node_definition(),
        BzhsItemNode::node_definition()
    ],
    dependencies = ["base-schema"],
    docs = "标准换算扩展，提供工程造价中的各种单位换算功能"
);
```

## 扩展间的协作

### 1. 扩展依赖

```rust
// 在 djgc 扩展中使用 bzhs 扩展
use extension_bzhs_interface::StandardConversionService;

pub struct DjgcCalculator {
    conversion_service: Arc<dyn StandardConversionService>,
}

impl DjgcCalculator {
    pub async fn calculate_with_conversion(
        &self,
        item: &DjgcItem,
        target_unit: &str
    ) -> ForgeResult<f64> {
        // 计算原始价格
        let price = self.calculate_price(item)?;

        // 如果需要单位换算
        if item.unit != target_unit {
            self.conversion_service.convert(
                price,
                &item.unit,
                target_unit,
                "GB50500-2013"  // 使用国标
            ).await
        } else {
            Ok(price)
        }
    }
}
```

### 2. 扩展通信

```rust
// 通过事件系统进行扩展间通信
impl Plugin for RcjPlugin {
    async fn on_event(&mut self, event: &Event, state: &State) -> ForgeResult<()> {
        match event.name() {
            "djgc_calculated" => {
                // 单价构成计算完成，更新人材机汇总
                self.update_rcj_summary(event.data(), state).await?;
            }
            "bzhs_converted" => {
                // 标准换算完成，重新计算
                self.recalculate(event.data(), state).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 测试扩展

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mf_core::test_utils::*;

    #[tokio::test]
    async fn test_bzhs_extension() {
        // 创建测试运行时
        let runtime = create_test_runtime()
            .with_extension(bzhs::extension())
            .build()
            .await
            .unwrap();

        // 创建测试状态
        let mut state = create_test_state();

        // 添加换算规则
        state.apply_transaction(|tr| {
            tr.insert_node(BzhsItemNode {
                item_code: "TEST001".to_string(),
                item_name: "测试换算".to_string(),
                source_unit: "m".to_string(),
                target_unit: "cm".to_string(),
                conversion_factor: 100.0,
                formula: None,
                precision: 2,
                is_active: true,
                notes: None,
            }.into());
        }).await.unwrap();

        // 执行换算命令
        let command = BatchConvertCommand {
            conversions: vec![ConversionRequest {
                id: "1".to_string(),
                value: 1.5,
                from_unit: "m".to_string(),
                to_unit: "cm".to_string(),
                standard: "TEST".to_string(),
            }],
            standard: "TEST".to_string(),
            auto_create_nodes: true,
        };

        runtime.execute_command(command).await.unwrap();

        // 验证结果
        let result_node = state.doc.get_node("conversion_1").unwrap();
        assert_eq!(result_node.get_attr("converted_value"), 150.0);
    }
}
```

## 最佳实践总结

1. **接口分离**：通过独立的 interface 包定义公共接口
2. **插件化设计**：业务逻辑封装在插件中，易于测试和维护
3. **命令模式**：所有操作通过命令执行，支持撤销和重做
4. **服务抽象**：通过服务接口提供功能，便于扩展间协作
5. **事件驱动**：使用事件系统实现松耦合的扩展通信
6. **缓存优化**：合理使用缓存提升性能
7. **批量处理**：支持批量操作减少开销
8. **完整测试**：每个扩展都有独立的测试套件
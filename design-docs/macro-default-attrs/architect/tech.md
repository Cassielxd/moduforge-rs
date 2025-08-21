# ModuForge-RS Default 属性扩展 - 技术架构规划

## 技术栈概述

### 核心 ModuForge-RS 模块选择

基于现有 `moduforge-macros-derive` 库的深度分析，本扩展将专注于以下核心组件：

#### 主要依赖模块
- **mf-derive (moduforge-macros-derive)**：核心宏扩展平台
  - 现有的 AttributeParser、FieldAnalyzer 基础设施
  - 成熟的错误处理和编译时验证框架
  - 完善的 Node/Mark 代码生成器架构

#### 外部技术栈
- **syn 2.0**：语法解析和 AST 操作
- **quote 1.0**：代码生成和 TokenStream 构建
- **proc-macro2 1.0**：过程宏核心功能
- **serde_json 1.0**：JSON 默认值解析和验证
- **thiserror 1.0**：结构化错误处理

### 技术选型理由

#### 1. 编译时架构选择
**选择**：基于现有 proc-macro 基础设施进行扩展
**理由**：
- 零运行时开销：所有默认值处理在编译期完成
- 类型安全保证：编译期验证确保类型一致性
- 向后兼容：完全基于现有架构，无破坏性变更

#### 2. 默认值存储策略
**选择**：编译时字符串解析 + 类型验证
**理由**：
- 灵活性：支持复杂的 JSON 表达式
- 性能：编译时解析，运行时直接使用
- 类型安全：编译期确保默认值与字段类型匹配

#### 3. 验证架构设计
**选择**：分层验证 + 早期失败策略
**理由**：
- 语法验证 → 类型验证 → 语义验证的清晰层次
- 快速反馈：第一个错误出现时立即停止
- 友好体验：精确的错误定位和修复建议

## 架构模式设计

### 1. 扩展点注入模式
遵循**开闭原则**，通过扩展现有组件而非修改核心逻辑：

```rust
// 现有 FieldConfig 的扩展
#[derive(Debug, Clone)]
pub struct FieldConfig {
    // 现有字段保持不变
    pub name: String,
    pub type_name: String,
    pub is_optional: bool,
    pub is_attr: bool,
    pub field: Field,
    
    // 新增默认值支持（保持向后兼容）
    pub default_value: Option<DefaultValue>,
}

// 新增默认值表示
#[derive(Debug, Clone)]
pub struct DefaultValue {
    pub raw_value: String,
    pub value_type: DefaultValueType,
    pub is_json: bool,
}
```

### 2. 类型验证策略模式
实现**策略模式**支持不同类型的验证逻辑：

```rust
// 验证策略接口
trait DefaultValueValidator {
    fn validate(&self, default_value: &str, field_type: &Type) -> MacroResult<()>;
    fn supports_type(&self, field_type: &Type) -> bool;
}

// 具体验证策略
struct SimpleTypeValidator;   // String, i32, bool 等
struct JsonTypeValidator;     // serde_json::Value
struct OptionTypeValidator;   // Option<T>
```

### 3. 渐进式代码生成模式
遵循**里氏替换原则**，生成的代码完全兼容现有接口：

```rust
// 现有方法保持不变
impl MyNode {
    pub fn to_node(&self) -> mf_core::node::Node { /* 现有逻辑 */ }
}

// 新增默认值支持的方法
impl MyNode {
    pub fn new() -> Self { /* 使用所有默认值 */ }
    pub fn with_defaults() -> Self { /* 灵活的默认值构造 */ }
}
```

## 编译时验证系统

### 验证流水线架构

```mermaid
graph TD
    A[解析 #[attr] 属性] --> B[提取 default 参数]
    B --> C[语法验证]
    C --> D{是否为 JSON?}
    D -->|是| E[JSON 语法验证]
    D -->|否| F[简单值验证]
    E --> G[JSON 类型验证]
    F --> H[基础类型验证]
    G --> I[类型一致性检查]
    H --> I
    I --> J{验证通过?}
    J -->|否| K[生成友好错误]
    J -->|是| L[生成代码]
```

### 分层验证设计

#### 第一层：语法验证
```rust
// 验证 default 属性的基础语法
impl AttributeParser {
    fn validate_default_attribute_syntax(attr: &Attribute) -> MacroResult<String> {
        // 1. 检查属性格式：#[attr(default="value")]
        // 2. 提取默认值字符串
        // 3. 验证字符串格式正确性
    }
}
```

#### 第二层：类型验证
```rust
// 验证默认值与字段类型的兼容性
impl DefaultValueValidator {
    fn validate_type_compatibility(
        default_value: &str,
        field_type: &Type,
        is_json: bool
    ) -> MacroResult<()> {
        // 1. 识别字段的基础类型
        // 2. 根据类型选择合适的验证策略
        // 3. 执行类型特定的验证逻辑
    }
}
```

#### 第三层：语义验证
```rust
// 验证默认值的业务逻辑合理性
impl SemanticValidator {
    fn validate_semantic_correctness(config: &FieldConfig) -> MacroResult<()> {
        // 1. JSON 默认值必须对应 serde_json::Value 类型
        // 2. Option<T> 类型可以使用 "null" 作为默认值
        // 3. 数值类型的范围合理性检查
    }
}
```

## 代码生成增强策略

### 1. 现有代码生成器的扩展

遵循**单一职责原则**，每个生成器只负责特定的代码生成任务：

```rust
// 扩展现有的 NodeGenerator
impl NodeGenerator {
    // 现有方法保持不变
    pub fn generate_to_node_method(&self) -> MacroResult<TokenStream2> {
        // 增强：支持默认值的 to_node 实现
    }
    
    // 新增默认值支持的方法
    pub fn generate_new_method(&self) -> MacroResult<TokenStream2> {
        // 生成使用所有默认值的构造函数
    }
    
    pub fn generate_with_defaults_method(&self) -> MacroResult<TokenStream2> {
        // 生成灵活的默认值构造函数
    }
}
```

### 2. 智能代码优化

基于字段类型和默认值，生成最优的代码：

```rust
// 简单类型的优化生成
#[attr(default = "hello")]
name: String,

// 生成的代码
node.set_attr("name", serde_json::Value::String(
    self.name.unwrap_or_else(|| "hello".to_string())
));

// JSON 类型的优化生成
#[attr(default = r#"{"key": "value"}"#)]
config: serde_json::Value,

// 生成的代码
node.set_attr("config", 
    self.config.unwrap_or_else(|| serde_json::json!({"key": "value"}))
);
```

## 性能优化技术

### 编译时性能要求

| 性能指标 | 目标值 | 实现策略 |
|---------|-------|---------|
| 类型验证时间 | < 1ms/字段 | 缓存类型信息，避免重复解析 |
| JSON 解析时间 | < 2ms/字段 | 使用 serde_json 的快速解析 |
| 代码生成时间 | < 5ms/结构体 | 预编译模板，减少运行时构建 |

### 优化实现策略

#### 1. 类型信息缓存
```rust
// 使用 once_cell 实现类型信息缓存
static TYPE_CACHE: Lazy<HashMap<String, TypeInfo>> = Lazy::new(|| {
    // 预构建常用类型的信息
});
```

#### 2. 验证器预排序
```rust
// 按使用频率排序验证器，优先检查常见类型
static VALIDATORS: Lazy<Vec<Box<dyn DefaultValueValidator>>> = Lazy::new(|| {
    vec![
        Box::new(StringValidator),    // 最常用
        Box::new(NumericValidator),   // 第二常用
        Box::new(BooleanValidator),   // 第三常用
        Box::new(JsonValidator),      // 特殊类型
    ]
});
```

#### 3. 增量代码生成
```rust
// 只为有默认值的字段生成额外代码
impl CodeGenerator {
    fn generate_optimized_code(&self) -> MacroResult<TokenStream2> {
        let fields_with_defaults: Vec<_> = self.config.attr_fields
            .iter()
            .filter(|f| f.default_value.is_some())
            .collect();
            
        if fields_with_defaults.is_empty() {
            // 无默认值字段时，使用现有代码生成逻辑
            return self.generate_standard_code();
        }
        
        // 有默认值字段时，生成增强代码
        self.generate_enhanced_code(fields_with_defaults)
    }
}
```

## 错误处理系统

### 友好错误消息设计

遵循**接口隔离原则**，为不同类型的错误提供专门的处理逻辑：

```rust
// 扩展现有的 MacroError
#[derive(Error, Debug)]
pub enum MacroError {
    // 现有错误类型保持不变
    MissingAttribute { /* ... */ },
    InvalidAttributeValue { /* ... */ },
    
    // 新增默认值相关错误
    #[error("默认值类型不匹配: 字段 '{field_name}' 类型为 '{field_type}'，但默认值 '{default_value}' 类型为 '{actual_type}'")]
    DefaultValueTypeMismatch {
        field_name: String,
        field_type: String,
        default_value: String,
        actual_type: String,
        span: Option<Span>,
    },
    
    #[error("JSON 默认值格式错误: {reason}")]
    InvalidJsonDefaultValue {
        reason: String,
        value: String,
        span: Option<Span>,
    },
}
```

### 错误恢复策略

实现**容错设计**，即使部分字段验证失败也能继续处理：

```rust
impl DefaultValueProcessor {
    fn process_all_fields_with_recovery(
        &self, 
        fields: &[FieldConfig]
    ) -> (Vec<ProcessedField>, Vec<MacroError>) {
        let mut processed = Vec::new();
        let mut errors = Vec::new();
        
        for field in fields {
            match self.process_field(field) {
                Ok(processed_field) => processed.push(processed_field),
                Err(error) => {
                    errors.push(error);
                    // 继续处理其他字段
                }
            }
        }
        
        (processed, errors)
    }
}
```

## 扩展性架构

### 插件化验证器系统

遵循**依赖倒置原则**，支持自定义验证器的注册：

```rust
// 验证器注册表
pub struct ValidatorRegistry {
    validators: Vec<Box<dyn DefaultValueValidator>>,
}

impl ValidatorRegistry {
    pub fn register<V: DefaultValueValidator + 'static>(&mut self, validator: V) {
        self.validators.push(Box::new(validator));
        // 按优先级重新排序
        self.validators.sort_by_key(|v| -v.priority());
    }
    
    pub fn validate(&self, default_value: &str, field_type: &Type) -> MacroResult<()> {
        for validator in &self.validators {
            if validator.supports_type(field_type) {
                return validator.validate(default_value, field_type);
            }
        }
        
        Err(MacroError::UnsupportedFieldType { /* ... */ })
    }
}
```

### 代码生成模板系统

支持自定义代码生成模板：

```rust
// 模板接口
trait CodeTemplate {
    fn generate(&self, context: &GenerationContext) -> MacroResult<TokenStream2>;
    fn supports_pattern(&self, pattern: &str) -> bool;
}

// 内置模板
struct SimpleDefaultTemplate;   // 简单类型默认值
struct JsonDefaultTemplate;     // JSON 默认值
struct OptionDefaultTemplate;   // Option 类型默认值

// 模板注册和使用
impl TemplateRegistry {
    fn select_template(&self, field: &FieldConfig) -> Option<&dyn CodeTemplate> {
        for template in &self.templates {
            if template.supports_pattern(&field.pattern()) {
                return Some(template.as_ref());
            }
        }
        None
    }
}
```

## 集成策略

### 与现有 ModuForge-RS 组件的无缝集成

#### 1. mf-core 集成
- **Node 创建**：增强的 `to_node()` 方法完全兼容现有 Node 接口
- **属性系统**：默认值直接设置到 Node 的 attributes 中
- **类型系统**：默认值转换遵循现有的类型转换规则

#### 2. mf-model 集成
- **Schema 兼容**：默认值信息集成到 NodeSpec 和 AttributeSpec 中
- **序列化支持**：默认值支持所有现有的序列化格式
- **验证规则**：默认值验证集成到现有的 Schema 验证流程

#### 3. mf-state 集成
- **事务支持**：带默认值的 Node 创建支持事务操作
- **状态管理**：默认值不影响状态的不可变性原则
- **插件兼容**：现有插件可以正常处理带默认值的节点

### 版本兼容策略

#### 向后兼容保证
- 现有的 `#[attr]` 语法完全保持不变
- 生成的代码接口完全兼容现有调用方式
- 新增的方法作为可选扩展，不影响现有功能

#### 渐进式迁移路径
```rust
// 阶段1：现有代码继续工作
#[derive(Node)]
#[node_type = "paragraph"]
struct Paragraph {
    #[attr]
    content: String,  // 现有方式
}

// 阶段2：选择性添加默认值
#[derive(Node)]
#[node_type = "paragraph"]
struct Paragraph {
    #[attr(default = "默认内容")]
    content: String,  // 新方式

    #[attr]
    author: Option<String>,  // 混合使用
}

// 阶段3：全面使用新功能
#[derive(Node)]
#[node_type = "paragraph"]
struct Paragraph {
    #[attr(default = "默认内容")]
    content: String,

    #[attr(default = "16")]
    font_size: i32,

    #[attr(default = "true")]
    visible: bool,
}
```

## 质量保证

### 测试策略

#### 1. 单元测试覆盖
- **验证器测试**：每种类型验证器的完整测试覆盖
- **代码生成测试**：生成代码的正确性和性能测试
- **错误处理测试**：各种错误场景的测试覆盖

#### 2. 集成测试
- **端到端测试**：从属性解析到代码生成的完整流程测试
- **兼容性测试**：与现有 ModuForge-RS 组件的集成测试
- **性能基准测试**：编译时间和内存使用的基准测试

#### 3. 模糊测试
- **输入验证**：各种边界情况和异常输入的处理测试
- **类型组合**：复杂类型组合的验证测试
- **并发安全**：多线程编译环境下的安全性测试

### 代码质量标准

- **测试覆盖率**：≥ 95%
- **文档覆盖率**：≥ 90%
- **性能要求**：编译时间增加 < 10%
- **内存使用**：峰值内存增加 < 20MB

---

*此技术架构规划为 ModuForge-RS Default 属性扩展项目提供了全面的技术实现指导，确保项目能够在保持高质量的同时实现预期目标。*
# ModuForge-RS 宏扩展技术栈规划

## ModuForge-RS 模块选择

### 核心依赖模块
- **mf-derive (moduforge-macros-derive)**: 主要扩展目标，现有的派生宏实现
- **mf-core**: 提供 Node 和 Mark 结构体定义及其构造方法
- **mf-model**: 提供 NodeSpec 和 AttributeSpec 类型定义
- **serde_json**: 用于 JSON 格式默认值解析和验证

### 技术架构层次
```
用户结构体 (User Struct with #[attr(default="value")])
    ↓ (应用扩展宏)
mf-derive 扩展派生宏 (Enhanced Procedural Macros)
    ↓ (编译时验证 + 代码生成)
mf-core API (Node::create, Mark::new)
    ↓ (使用类型)
mf-model 类型定义 (NodeSpec, AttributeSpec)
    ↓ (默认值处理)
serde_json::Value (默认值存储)
```

## 技术栈决策

### 1. 宏系统扩展策略

#### 基于现有派生宏扩展 - 主选方案
**选择理由**：
- **向后兼容**：保持现有 API 完全不变
- **渐进式改进**：在现有架构基础上增加新功能
- **类型安全**：利用现有的类型检查和验证机制
- **开发效率**：复用现有代码和测试

**扩展实现**：
```rust
// 扩展现有的 derive 宏，添加 default 参数支持
#[proc_macro_derive(Node, attributes(node_type, marks, content, attr))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // 扩展现有的处理流程支持 default 属性
    node::derive_impl::process_derive_node_with_defaults(input)
}

// 新增 default 属性支持：#[attr(default="value")]
```

### 2. 默认值处理技术

#### 编译时默认值解析和验证
**核心组件**：
- **默认值解析器**: 解析 `default="value"` 参数
- **类型推断引擎**: 从字面量推断目标类型
- **兼容性验证器**: 验证默认值与字段类型的兼容性
- **JSON 验证器**: 对 JSON 格式默认值的专用验证

**选择理由**：
- **编译时安全**：所有验证在编译时完成，运行时零风险
- **类型一致性**：强制保证默认值与字段类型匹配
- **JSON 支持**：特殊处理 JSON 格式，强制要求 serde_json::Value 类型
- **友好错误**：提供明确的编译错误信息和修复建议

### 3. 扩展属性解析技术

#### 增强的属性解析器
```rust
// 扩展现有的 FieldConfig 结构
#[derive(Debug, Clone)]
pub struct FieldConfig {
    pub name: String,
    pub type_name: String,
    pub is_optional: bool,
    pub is_attr: bool,
    pub field: Field,
    // 新增：默认值支持
    pub default_value: Option<DefaultValue>,
}

// 新增：默认值类型定义
#[derive(Debug, Clone)]
pub enum DefaultValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(String),  // JSON 格式的字符串
}
```

### 4. 默认值类型转换系统

#### 编译时类型验证和转换
**支持的默认值类型**：
- `String` 类型字段 ← `default="string_value"`
- `i32`, `i64`, `f32`, `f64` 类型字段 ← `default="42"`
- `bool` 类型字段 ← `default="true"`
- `serde_json::Value` 类型字段 ← `default="{\"key\":\"value\"}"`
- `Option<T>` 类型字段 ← 任意上述类型的默认值

**编译时验证策略**：
```rust
/// 编译时类型兼容性验证器
pub struct DefaultValueValidator;

impl DefaultValueValidator {
    pub fn validate_compatibility(
        field_type: &Type, 
        default_value: &DefaultValue
    ) -> MacroResult<()> {
        match (field_type, default_value) {
            // 验证 JSON 默认值只能用于 serde_json::Value 类型
            (Type::Path(path), DefaultValue::Json(_)) => {
                if !is_serde_json_value_type(path) {
                    return Err(MacroError::type_mismatch(
                        "JSON 格式默认值只能用于 serde_json::Value 类型字段"
                    ));
                }
            }
            // 其他类型验证...
        }
        Ok(())
    }
}
```

## 架构模式设计

### 1. 单一职责原则 (SRP) 应用

#### 模块职责划分（扩展现有架构）
```rust
// 现有模块保持不变
src/
├── parser/
│   ├── attribute_parser.rs     // 扩展：支持 default 属性解析
│   ├── field_analyzer.rs       // 扩展：增加默认值分析
│   ├── validation.rs          // 扩展：添加默认值验证
│   └── default_value_parser.rs // 新增：默认值解析器
├── generator/
│   ├── node_generator.rs      // 扩展：支持默认值代码生成
│   ├── mark_generator.rs      // 扩展：支持默认值代码生成
│   └── default_value_generator.rs // 新增：默认值代码生成器
├── converter/
│   ├── type_converter.rs      // 扩展：支持默认值转换
│   ├── builtin_converters.rs  // 扩展：增加默认值转换器
│   └── json_converter.rs      // 新增：JSON 默认值专用转换器
└── common/
    ├── error.rs               // 扩展：添加默认值相关错误类型
    └── default_value_types.rs // 新增：默认值类型定义
```

**新增模块职责**：
- `default_value_parser`: 专门解析 `default="value"` 属性
- `default_value_generator`: 专门生成默认值处理代码
- `json_converter`: 专门处理 JSON 格式默认值转换和验证
- `default_value_types`: 定义默认值相关的所有类型

### 2. 接口隔离原则 (ISP) 应用

#### 默认值专用接口设计
```rust
/// 默认值解析器接口 - 只处理默认值解析
pub trait DefaultValueParser {
    fn parse_default_value(&self, attr: &Attribute) -> MacroResult<Option<DefaultValue>>;
    fn supports_attribute(&self, attr: &Attribute) -> bool;
}

/// 默认值验证器接口 - 只处理类型兼容性验证
pub trait DefaultValueValidator {
    fn validate_type_compatibility(
        &self, 
        field_type: &Type, 
        default_value: &DefaultValue
    ) -> MacroResult<()>;
}

/// JSON 默认值处理器接口 - 专门处理 JSON 格式
pub trait JsonDefaultValueHandler {
    fn validate_json_syntax(&self, json_str: &str) -> MacroResult<()>;
    fn ensure_serde_json_value_type(&self, field_type: &Type) -> MacroResult<()>;
}

/// 默认值代码生成器接口 - 只生成默认值处理代码
pub trait DefaultValueCodeGenerator {
    fn generate_default_value_assignment(
        &self, 
        field: &FieldConfig
    ) -> MacroResult<TokenStream2>;
}
```

### 3. 开闭原则 (OCP) 应用

#### 可扩展的默认值处理系统
```rust
/// 默认值处理器 trait - 对新类型扩展开放
pub trait DefaultValueHandler: Send + Sync {
    fn handle_default_value(
        &self, 
        field_type: &Type, 
        default_value: &DefaultValue
    ) -> MacroResult<TokenStream2>;
    
    fn supports_type(&self, field_type: &Type) -> bool;
    fn priority(&self) -> i32;  // 处理器优先级
}

/// 默认值处理器注册表 - 支持注册新的处理器
pub struct DefaultValueHandlerRegistry {
    handlers: Vec<Box<dyn DefaultValueHandler>>,
}

impl DefaultValueHandlerRegistry {
    pub fn register<T: DefaultValueHandler + 'static>(&mut self, handler: T) {
        self.handlers.push(Box::new(handler));
        // 按优先级排序
        self.handlers.sort_by_key(|h| -h.priority());
    }
    
    pub fn find_handler(&self, field_type: &Type) -> Option<&dyn DefaultValueHandler> {
        self.handlers.iter()
            .find(|h| h.supports_type(field_type))
            .map(|h| h.as_ref())
    }
}
```

### 4. 里氏替换原则 (LSP) 应用

#### 一致的默认值处理契约
```rust
/// 所有默认值处理器必须遵循相同的契约
pub trait DefaultValueProcessor {
    /// 处理默认值，必须返回合法的 TokenStream 或明确的错误
    fn process(
        &self, 
        field: &FieldConfig, 
        default_value: &DefaultValue
    ) -> MacroResult<TokenStream2>;
    
    /// 验证默认值，所有处理器都必须实现一致的验证逻辑
    fn validate(
        &self, 
        field_type: &Type, 
        default_value: &DefaultValue
    ) -> MacroResult<()>;
}

// 所有具体实现都必须保证可替换性
impl DefaultValueProcessor for StringDefaultProcessor {
    // 实现必须满足 LSP 契约
}

impl DefaultValueProcessor for JsonDefaultProcessor {
    // 实现必须满足 LSP 契约
}
```

## 性能要求

### 编译时性能目标
- **默认值验证时间**: < 50ms (对于包含 20 个默认值字段的结构体)
- **JSON 解析验证**: < 10ms (对于 1KB 的 JSON 默认值)
- **类型兼容性检查**: < 5ms (每个字段)
- **总编译时间增加**: < 10% (相比不使用默认值功能)

### 生成代码性能目标
- **默认值处理开销**: 运行时零额外开销
- **内存使用**: 生成的默认值不增加运行时内存占用
- **二进制大小**: 默认值功能不显著增加编译后二进制大小

### 性能优化策略

#### 1. 默认值验证优化
```rust
// 使用 once_cell 缓存默认值验证结果
use once_cell::sync::Lazy;

static DEFAULT_VALUE_CACHE: Lazy<HashMap<(String, String), bool>> = 
    Lazy::new(HashMap::new);

// 缓存默认值类型兼容性检查结果
fn is_compatible_cached(
    field_type: &str, 
    default_value_type: &str
) -> bool {
    let key = (field_type.to_string(), default_value_type.to_string());
    *DEFAULT_VALUE_CACHE.entry(key)
        .or_insert_with(|| validate_type_compatibility(field_type, default_value_type))
}
```

#### 2. 默认值代码生成优化
```rust
// 生成高效的默认值处理代码
fn generate_optimized_default_handling() -> TokenStream2 {
    quote! {
        // 编译时常量折叠，运行时零开销
        const DEFAULT_VALUES: &[(&str, serde_json::Value)] = &[
            #(#default_value_pairs),*
        ];
        
        // 直接使用预计算的默认值，避免运行时解析
        #(#field_assignments)*
    }
}
```

## 扩展性考虑

### 1. 默认值类型扩展
```rust
/// 支持自定义默认值类型处理器
pub trait CustomDefaultValueHandler: Send + Sync {
    fn handle_custom_type(
        &self, 
        field_type: &Type, 
        default_value: &str
    ) -> MacroResult<TokenStream2>;
    
    fn supports_type(&self, field_type: &Type) -> bool;
    fn type_name(&self) -> &'static str;
}

// 支持注册自定义默认值处理器
pub struct DefaultValueExtensionRegistry {
    handlers: HashMap<String, Box<dyn CustomDefaultValueHandler>>,
}
```

### 2. 默认值格式扩展
```rust
/// 允许扩展新的默认值格式
pub trait DefaultValueFormat {
    fn parse_format(&self, input: &str) -> MacroResult<DefaultValue>;
    fn format_name(&self) -> &'static str;
    fn validate_syntax(&self, input: &str) -> MacroResult<()>;
}

// 支持 YAML、TOML 等其他格式
pub struct YamlDefaultValueFormat;
pub struct TomlDefaultValueFormat;
```

### 3. 未来功能扩展预留

#### 计划中的默认值扩展点
1. **复杂类型默认值**: 支持自定义 struct 和 enum 的默认值
2. **条件默认值**: `#[attr(default_if="condition")]` 用于条件默认值
3. **动态默认值**: 支持运行时计算的默认值
4. **默认值继承**: 从父类或 trait 继承默认值

#### 架构预留接口
```rust
// 为未来默认值扩展预留的 trait
pub trait DefaultValueExtension {
    type Config;
    type Context;
    
    fn extend_default_value(
        &self, 
        config: Self::Config, 
        context: Self::Context
    ) -> MacroResult<DefaultValue>;
}
```

## 技术风险评估

### 高风险项
1. **默认值类型推断复杂性**: JSON 默认值的编译时验证可能很复杂
   - **缓解策略**: 使用简化的验证规则，只验证 JSON 语法正确性
2. **向后兼容性破坏**: 新增功能可能影响现有代码
   - **缓解策略**: 严格的回归测试，全面的兼容性检查

### 中风险项
1. **JSON 默认值验证性能**: 大量 JSON 默认值可能影响编译性能
   - **缓解策略**: 实现缓存机制，优化 JSON 解析性能
2. **错误信息复杂性**: 默认值错误可能产生复杂的错误信息
   - **缓解策略**: 设计精简、清晰的错误消息模板

### 技术债务控制
- **默认值测试覆盖率**: 保持 95% 以上的测试覆盖率
- **类型安全性测试**: 建立全面的类型兼容性测试套件
- **性能基准监控**: 监控默认值处理对编译性能的影响
- **文档完整性**: 所有新增功能都必须有完整的中文文档

通过这个技术架构扩展设计，moduforge-macros-derive 将在保持现有架构优势的基础上，新增默认值支持功能，为 ModuForge-RS 生态提供更强大、更安全的代码生成能力。
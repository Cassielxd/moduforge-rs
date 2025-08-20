# 技术栈规划 - ModuForge-RS 宏模块扩展

## ModuForge-RS 模块选择

### 核心依赖模块
- **mf-core**: 提供 `Node` 和 `Mark` 结构体定义及其构造方法
- **mf-model**: 提供 `NodeSpec` 和 `MarkSpec` 类型定义
- **mf-derive**: 主要扩展目标，实现派生宏功能

### 技术架构层次
```
用户结构体 (User Struct)
    ↓ (应用宏)
mf-derive 派生宏 (Procedural Macros)  
    ↓ (生成代码调用)
mf-core API (Node::create, Mark::new)
    ↓ (使用类型)
mf-model 类型定义 (NodeSpec, MarkSpec)
```

## 技术栈决策

### 1. 宏系统技术选型

#### 派生宏 (Derive Macros) - 主选方案
**选择理由**：
- **类型安全**：能够访问结构体的完整类型信息
- **IDE 支持**：现代 IDE 对派生宏有良好支持
- **维护性**：代码生成逻辑集中，易于维护
- **扩展性**：可以轻松添加新的派生目标

**技术实现**：
```rust
// 在 mf-derive/src/lib.rs 中实现
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Node, attributes(node_type, marks, content, attr))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // 实现 Node 生成逻辑
}

#[proc_macro_derive(Mark, attributes(mark_type, attr))]
pub fn derive_mark(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // 实现 Mark 生成逻辑
}
```

### 2. 代码生成技术

#### proc-macro2 + syn + quote 技术栈
**核心组件**：
- **syn**: 解析 Rust 代码为 AST
- **quote**: 生成 Rust 代码的模板引擎
- **proc-macro2**: 提供更好的测试和开发体验

**选择理由**：
- **成熟生态**：Rust 生态标准的宏开发工具链
- **类型安全**：编译时验证生成的代码正确性
- **性能优化**：零运行时开销，纯编译时处理
- **错误处理**：提供详细的编译时错误信息

### 3. 属性解析技术

#### 自定义属性解析器
```rust
// 属性解析结构
#[derive(Debug)]
pub struct NodeAttributes {
    pub node_type: Option<String>,      // #[node_type = "GCXM"]
    pub marks: Option<String>,          // #[marks = "color"]  
    pub content: Option<String>,        // #[content = "DCXM"]
}

#[derive(Debug)]  
pub struct FieldAttributes {
    pub is_attr: bool,                  // #[attr]
    pub custom_name: Option<String>,    // #[attr(name = "custom")]
}
```

### 4. 类型转换系统

#### 智能类型映射
**基本类型转换**：
- `String` → `serde_json::Value::String`
- `i32`, `i64`, `f32`, `f64` → `serde_json::Value::Number`
- `bool` → `serde_json::Value::Bool`
- `Option<T>` → `serde_json::Value::Null` 或转换后的 `T`

**实现策略**：
```rust
/// 类型转换 trait，支持自定义转换逻辑
pub trait ToJsonValue {
    fn to_json_value(&self) -> serde_json::Value;
}

// 为常用类型提供默认实现
impl ToJsonValue for String {
    fn to_json_value(&self) -> serde_json::Value {
        serde_json::Value::String(self.clone())
    }
}
```

## 架构模式设计

### 1. 单一职责原则 (SRP) 应用

#### 模块职责划分
```rust
// mf-derive/src/lib.rs - 宏入口点
pub mod node_derive;      // Node 派生宏实现
pub mod mark_derive;      // Mark 派生宏实现
pub mod attr_parser;      // 属性解析器
pub mod code_generator;   // 代码生成器
pub mod type_converter;   // 类型转换器
pub mod validator;        // 编译时验证器
```

**职责边界**：
- `node_derive`: 仅负责 Node 相关的宏逻辑
- `mark_derive`: 仅负责 Mark 相关的宏逻辑  
- `attr_parser`: 专门处理属性解析
- `code_generator`: 专门负责代码生成
- `type_converter`: 专门处理类型转换
- `validator`: 专门进行编译时验证

### 2. 接口隔离原则 (ISP) 应用

#### 精简接口设计
```rust
/// Node 生成器接口 - 只暴露 Node 相关方法
pub trait NodeGenerator {
    fn generate_to_node_method(&self, input: &DeriveInput) -> Result<TokenStream2>;
}

/// Mark 生成器接口 - 只暴露 Mark 相关方法  
pub trait MarkGenerator {
    fn generate_to_mark_method(&self, input: &DeriveInput) -> Result<TokenStream2>;
}

/// 属性验证器接口 - 只暴露验证方法
pub trait AttributeValidator {
    fn validate_node_attributes(&self, attrs: &NodeAttributes) -> Result<()>;
    fn validate_mark_attributes(&self, attrs: &MarkAttributes) -> Result<()>;
}
```

### 3. 开闭原则 (OCP) 应用

#### 可扩展的转换器系统
```rust
/// 类型转换器 trait - 对扩展开放
pub trait TypeConverter: Send + Sync {
    fn convert_type(&self, rust_type: &Type) -> Option<TokenStream2>;
    fn supports_type(&self, rust_type: &Type) -> bool;
}

/// 转换器注册表 - 支持注册新的转换器
pub struct ConverterRegistry {
    converters: Vec<Box<dyn TypeConverter>>,
}

impl ConverterRegistry {
    pub fn register<T: TypeConverter + 'static>(&mut self, converter: T) {
        self.converters.push(Box::new(converter));
    }
}
```

### 4. 里氏替换原则 (LSP) 应用

#### 一致的接口契约
```rust
/// 所有代码生成器必须遵循相同的契约
pub trait CodeGenerator {
    type Input;
    type Output;
    type Error;
    
    /// 生成代码，必须返回合法的 TokenStream 或明确的错误
    fn generate(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

// 具体实现必须保证可替换性
impl CodeGenerator for NodeCodeGenerator {
    // 实现必须满足 LSP 契约
}
```

## 性能要求

### 编译时性能目标
- **宏展开时间**: < 100ms (对于包含 50 个字段的结构体)
- **内存使用**: < 10MB (在宏展开过程中的峰值内存)
- **增量编译**: 支持增量编译，只重新处理变更的结构体

### 生成代码性能目标
- **方法调用开销**: 与手写代码相同的性能
- **内存分配**: 最小化不必要的内存分配
- **二进制大小**: 生成的代码不显著增加二进制大小

### 性能优化策略

#### 1. 编译时优化
```rust
// 使用 once_cell 缓存重复计算
use once_cell::sync::Lazy;

static TYPE_CACHE: Lazy<HashMap<String, TokenStream2>> = Lazy::new(HashMap::new);

// 避免重复的类型分析
fn get_cached_type_conversion(rust_type: &str) -> Option<&TokenStream2> {
    TYPE_CACHE.get(rust_type)
}
```

#### 2. 生成代码优化
```rust
// 生成高效的属性构建代码
fn generate_optimized_attrs_building() -> TokenStream2 {
    quote! {
        // 预分配 HashMap 容量，减少重新分配
        let mut attrs = std::collections::HashMap::with_capacity(#field_count);
        #(#field_insertions)*
        attrs
    }
}
```

## 扩展性考虑

### 1. 插件化属性处理
```rust
/// 支持自定义属性处理器
pub trait AttributeProcessor: Send + Sync {
    fn process_attribute(&self, attr: &Attribute) -> Option<AttributeResult>;
    fn attribute_name(&self) -> &'static str;
}

// 支持注册自定义处理器
pub struct MacroExtensionRegistry {
    processors: HashMap<String, Box<dyn AttributeProcessor>>,
}
```

### 2. 自定义代码生成钩子
```rust
/// 允许用户自定义代码生成逻辑
pub trait CodeGenHook {
    fn pre_generate(&self, input: &DeriveInput) -> Result<()>;
    fn post_generate(&self, output: &mut TokenStream2) -> Result<()>;
}
```

### 3. 未来功能扩展预留

#### 计划中的扩展点
1. **批量操作支持**: `#[derive(NodeBatch)]` 用于批量创建节点
2. **模板系统集成**: 与 mf-template 集成生成模板化节点
3. **验证规则扩展**: 支持自定义的编译时验证规则
4. **序列化优化**: 与 mf-file 集成优化序列化性能

#### 架构预留接口
```rust
// 为未来扩展预留的 trait
pub trait FutureExtension {
    type Config;
    fn apply_extension(&self, config: Self::Config) -> Result<TokenStream2>;
}
```

## 技术风险评估

### 高风险项
1. **编译时间影响**: 复杂宏可能显著增加编译时间
   - **缓解策略**: 实现增量编译缓存，优化宏逻辑
2. **IDE 支持兼容性**: 生成的代码可能不被所有 IDE 正确识别
   - **缓解策略**: 生成标准化代码，添加必要的类型注解

### 中风险项
1. **错误信息质量**: 宏错误可能难以理解
   - **缓解策略**: 实现详细的错误消息和修复建议
2. **版本兼容性**: 依赖的 proc-macro 生态版本更新
   - **缓解策略**: 固定版本范围，定期升级测试

### 技术债务控制
- **代码覆盖率**: 保持 90% 以上的测试覆盖率
- **文档完整性**: 所有公开 API 必须有完整文档
- **性能基准**: 建立性能基准测试，监控回归

通过这个技术架构设计，mf-derive 将成为一个高性能、可扩展、符合设计原则的宏系统，为 ModuForge-RS 生态提供强大的代码生成能力。
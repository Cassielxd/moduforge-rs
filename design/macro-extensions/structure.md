# ModuForge-RS 宏扩展项目结构设计

## 目录结构

### moduforge-macros-derive 扩展结构
```
crates/derive/
├── Cargo.toml                      # 依赖配置和元数据
├── README.md                       # 使用说明文档
├── examples/                       # 使用示例
│   ├── basic_node.rs              # 基础 Node 派生示例
│   ├── complex_node.rs            # 复杂 Node 配置示例
│   ├── basic_mark.rs              # 基础 Mark 派生示例
│   └── custom_attributes.rs       # 自定义属性示例
├── tests/                         # 集成测试
│   ├── node_derive_tests.rs       # Node 派生宏测试
│   ├── mark_derive_tests.rs       # Mark 派生宏测试
│   ├── error_cases.rs             # 错误场景测试
│   └── integration.rs             # 集成测试
├── benches/                       # 性能基准测试
│   ├── compilation_time.rs        # 编译时间基准
│   └── generated_code.rs          # 生成代码性能
└── src/
    ├── lib.rs                     # 主入口点，导出派生宏
    ├── common/                    # 通用模块
    │   ├── mod.rs
    │   ├── error.rs               # 统一错误类型定义
    │   ├── utils.rs               # 通用工具函数
    │   └── constants.rs           # 常量定义
    ├── parser/                    # 属性解析模块
    │   ├── mod.rs
    │   ├── attribute_parser.rs    # 属性解析器
    │   ├── field_analyzer.rs      # 字段分析器
    │   └── validation.rs          # 解析验证逻辑
    ├── generator/                 # 代码生成模块
    │   ├── mod.rs
    │   ├── node_generator.rs      # Node 代码生成器
    │   ├── mark_generator.rs      # Mark 代码生成器
    │   └── helper_generator.rs    # 辅助方法生成器
    ├── converter/                 # 类型转换模块
    │   ├── mod.rs
    │   ├── type_converter.rs      # 类型转换核心逻辑
    │   ├── builtin_converters.rs  # 内置类型转换器
    │   └── converter_registry.rs  # 转换器注册表
    ├── node/                      # Node 派生宏实现
    │   ├── mod.rs
    │   ├── derive_impl.rs         # Node 派生宏主要实现
    │   ├── attribute_handler.rs   # Node 属性处理
    │   └── code_gen.rs            # Node 代码生成逻辑
    └── mark/                      # Mark 派生宏实现
        ├── mod.rs
        ├── derive_impl.rs         # Mark 派生宏主要实现
        ├── attribute_handler.rs   # Mark 属性处理
        └── code_gen.rs            # Mark 代码生成逻辑
```

## 模块划分

### 1. 核心入口模块 (lib.rs)

**单一职责**: 仅负责导出派生宏和公共接口

```rust
//! ModuForge-RS 宏扩展模块
//!
//! 为 ModuForge-RS 框架提供声明式的 Node 和 Mark 定义宏，
//! 自动生成与 mf-core 兼容的实例构造代码。

// 重新导出核心派生宏
pub use node::derive_node;
pub use mark::derive_mark;

// 重新导出公共类型
pub use common::error::MacroError;
pub use converter::type_converter::TypeConverter;

// 宏入口点定义
use proc_macro::TokenStream;

/// Node 派生宏
///
/// 为结构体自动生成 `to_node()` 方法，返回 `mf_core::node::Node` 实例
///
/// # 属性支持
/// - `node_type`: 指定节点类型字符串
/// - `marks`: 指定支持的标记类型
/// - `content`: 指定内容约束表达式
/// 
/// # 字段属性
/// - `#[attr]`: 标记字段作为节点属性
///
/// # 示例
/// ```rust
/// use mf_derive::Node;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Node, Serialize, Deserialize)]
/// #[node_type = "project", content = "*"]
/// pub struct Project {
///     #[attr]
///     name: String,
///     #[attr]
///     description: Option<String>,
/// }
/// ```
#[proc_macro_derive(Node, attributes(node_type, marks, content, attr))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    node::derive_node(input)
}

/// Mark 派生宏
///
/// 为结构体自动生成 `to_mark()` 方法，返回 `mf_core::mark::Mark` 实例
///
/// # 属性支持
/// - `mark_type`: 指定标记类型字符串
///
/// # 字段属性
/// - `#[attr]`: 标记字段作为标记属性
///
/// # 示例
/// ```rust
/// use mf_derive::Mark;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Mark, Serialize, Deserialize)]
/// #[mark_type = "emphasis"]
/// pub struct Emphasis {
///     #[attr]
///     level: i32,
/// }
/// ```
#[proc_macro_derive(Mark, attributes(mark_type, attr))]
pub fn derive_mark(input: TokenStream) -> TokenStream {
    mark::derive_mark(input)
}
```

### 2. 通用模块 (common/)

#### error.rs - 统一错误处理
**单一职责**: 定义所有宏处理过程中的错误类型

```rust
use proc_macro2::Span;
use syn::Error as SynError;

/// 宏处理过程中的错误类型
#[derive(Debug)]
pub enum MacroError {
    /// 属性解析错误
    AttributeParseError {
        message: String,
        span: Span,
    },
    /// 字段分析错误
    FieldAnalysisError {
        field_name: String,
        message: String,
        span: Span,
    },
    /// 类型转换错误
    TypeConversionError {
        rust_type: String,
        message: String,
        span: Span,
    },
    /// 代码生成错误
    CodeGenerationError {
        message: String,
        span: Span,
    },
    /// 验证错误
    ValidationError {
        rule: String,
        message: String,
        span: Span,
    },
}

impl MacroError {
    /// 转换为 syn::Error 以便编译器显示
    pub fn to_syn_error(&self) -> SynError {
        match self {
            MacroError::AttributeParseError { message, span } => {
                SynError::new(*span, format!("属性解析错误: {}", message))
            }
            MacroError::FieldAnalysisError { field_name, message, span } => {
                SynError::new(*span, format!("字段 '{}' 分析错误: {}", field_name, message))
            }
            MacroError::TypeConversionError { rust_type, message, span } => {
                SynError::new(*span, format!("类型 '{}' 转换错误: {}", rust_type, message))
            }
            MacroError::CodeGenerationError { message, span } => {
                SynError::new(*span, format!("代码生成错误: {}", message))
            }
            MacroError::ValidationError { rule, message, span } => {
                SynError::new(*span, format!("验证规则 '{}' 失败: {}", rule, message))
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, MacroError>;
```

### 3. 属性解析模块 (parser/)

#### attribute_parser.rs - 属性解析器
**单一职责**: 解析宏属性并转换为结构化数据

```rust
use syn::{Attribute, Lit, Meta, NestedMeta};
use crate::common::{MacroError, Result};

/// Node 宏的属性配置
#[derive(Debug, Default)]
pub struct NodeAttributes {
    /// 节点类型标识符
    pub node_type: Option<String>,
    /// 支持的标记类型表达式
    pub marks: Option<String>,
    /// 内容约束表达式
    pub content: Option<String>,
}

/// Mark 宏的属性配置
#[derive(Debug, Default)]
pub struct MarkAttributes {
    /// 标记类型标识符
    pub mark_type: Option<String>,
}

/// 字段级别的属性配置
#[derive(Debug, Default)]
pub struct FieldAttributes {
    /// 是否作为属性字段
    pub is_attr: bool,
    /// 自定义属性名称
    pub custom_name: Option<String>,
}

/// 属性解析器
pub struct AttributeParser;

impl AttributeParser {
    /// 解析 Node 宏的属性
    pub fn parse_node_attributes(attrs: &[Attribute]) -> Result<NodeAttributes> {
        let mut node_attrs = NodeAttributes::default();
        
        for attr in attrs {
            match attr.path.get_ident().map(|i| i.to_string()).as_deref() {
                Some("node_type") => {
                    node_attrs.node_type = Some(Self::parse_string_value(attr)?);
                }
                Some("marks") => {
                    node_attrs.marks = Some(Self::parse_string_value(attr)?);
                }
                Some("content") => {
                    node_attrs.content = Some(Self::parse_string_value(attr)?);
                }
                _ => {} // 忽略其他属性
            }
        }
        
        // 验证必需属性
        if node_attrs.node_type.is_none() {
            return Err(MacroError::AttributeParseError {
                message: "node_type 属性是必需的".to_string(),
                span: proc_macro2::Span::call_site(),
            });
        }
        
        Ok(node_attrs)
    }
    
    /// 解析 Mark 宏的属性
    pub fn parse_mark_attributes(attrs: &[Attribute]) -> Result<MarkAttributes> {
        let mut mark_attrs = MarkAttributes::default();
        
        for attr in attrs {
            if let Some(ident) = attr.path.get_ident() {
                if ident == "mark_type" {
                    mark_attrs.mark_type = Some(Self::parse_string_value(attr)?);
                }
            }
        }
        
        // 验证必需属性
        if mark_attrs.mark_type.is_none() {
            return Err(MacroError::AttributeParseError {
                message: "mark_type 属性是必需的".to_string(),
                span: proc_macro2::Span::call_site(),
            });
        }
        
        Ok(mark_attrs)
    }
    
    /// 解析字段属性
    pub fn parse_field_attributes(attrs: &[Attribute]) -> Result<FieldAttributes> {
        let mut field_attrs = FieldAttributes::default();
        
        for attr in attrs {
            if let Some(ident) = attr.path.get_ident() {
                if ident == "attr" {
                    field_attrs.is_attr = true;
                    // 可以扩展支持 #[attr(name = "custom")] 形式
                }
            }
        }
        
        Ok(field_attrs)
    }
    
    /// 解析字符串值属性
    fn parse_string_value(attr: &Attribute) -> Result<String> {
        match attr.parse_meta() {
            Ok(Meta::NameValue(meta)) => {
                if let Lit::Str(lit_str) = meta.lit {
                    Ok(lit_str.value())
                } else {
                    Err(MacroError::AttributeParseError {
                        message: "期望字符串值".to_string(),
                        span: attr.span(),
                    })
                }
            }
            _ => Err(MacroError::AttributeParseError {
                message: "无效的属性格式".to_string(),
                span: attr.span(),
            })
        }
    }
}
```

### 4. 代码生成模块 (generator/)

#### node_generator.rs - Node 代码生成器
**单一职责**: 生成 Node 相关的方法代码

```rust
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Field};

use crate::common::{MacroError, Result};
use crate::parser::{NodeAttributes, FieldAttributes};
use crate::converter::TypeConverter;

/// Node 代码生成器
pub struct NodeGenerator {
    /// 类型转换器
    type_converter: Box<dyn TypeConverter>,
}

impl NodeGenerator {
    pub fn new(type_converter: Box<dyn TypeConverter>) -> Self {
        Self { type_converter }
    }
    
    /// 生成 to_node 方法
    pub fn generate_to_node_method(
        &self,
        input: &DeriveInput,
        attrs: &NodeAttributes,
        fields: &[(&Field, FieldAttributes)],
    ) -> Result<TokenStream2> {
        let struct_name = &input.ident;
        let node_type = attrs.node_type.as_ref().unwrap();
        
        // 生成属性构建代码
        let attr_building_code = self.generate_attr_building_code(fields)?;
        
        // 生成 NodeSpec 构建代码
        let spec_building_code = self.generate_spec_building_code(attrs)?;
        
        Ok(quote! {
            impl #struct_name {
                /// 将当前结构体转换为 mf_core::node::Node 实例
                ///
                /// # 返回值
                /// 返回配置完成的 Node 实例，可直接用于 ModuForge-RS 框架
                ///
                /// # 示例
                /// ```rust
                /// let instance = MyStruct { name: "test".to_string() };
                /// let node = instance.to_node();
                /// ```
                pub fn to_node(&self) -> mf_core::node::Node {
                    // 构建属性 HashMap
                    #attr_building_code
                    
                    // 构建 NodeSpec
                    #spec_building_code
                    
                    // 创建并配置 Node
                    let mut node = mf_core::node::Node::create(#node_type, spec);
                    
                    // 设置属性
                    if !attrs.is_empty() {
                        node.set_attrs(attrs);
                    }
                    
                    node
                }
            }
        })
    }
    
    /// 生成属性构建代码
    fn generate_attr_building_code(
        &self,
        fields: &[(&Field, FieldAttributes)],
    ) -> Result<TokenStream2> {
        let attr_fields: Vec<_> = fields.iter()
            .filter(|(_, field_attrs)| field_attrs.is_attr)
            .collect();
        
        if attr_fields.is_empty() {
            return Ok(quote! {
                let attrs = std::collections::HashMap::new();
            });
        }
        
        let field_count = attr_fields.len();
        let field_conversions: Result<Vec<_>> = attr_fields.iter()
            .map(|(field, field_attrs)| {
                let field_name = field.ident.as_ref().unwrap();
                let attr_name = field_attrs.custom_name.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or_else(|| field_name.to_string().as_str());
                
                let conversion = self.type_converter.convert_field_to_json_value(field)?;
                
                Ok(quote! {
                    attrs.insert(
                        #attr_name.to_string(),
                        mf_model::schema::AttributeSpec { 
                            default: Some(#conversion) 
                        }
                    );
                })
            })
            .collect();
        
        let field_conversions = field_conversions?;
        
        Ok(quote! {
            let mut attrs = std::collections::HashMap::with_capacity(#field_count);
            #(#field_conversions)*
        })
    }
    
    /// 生成 NodeSpec 构建代码
    fn generate_spec_building_code(&self, attrs: &NodeAttributes) -> Result<TokenStream2> {
        let content = attrs.content.as_deref();
        let marks = attrs.marks.as_deref();
        
        Ok(quote! {
            let spec = mf_model::node_type::NodeSpec {
                content: #content.map(|s| s.to_string()),
                marks: #marks.map(|s| s.to_string()),
                group: None,
                desc: None,
                attrs: if attrs.is_empty() { None } else { Some(attrs) },
            };
        })
    }
}
```

### 5. 类型转换模块 (converter/)

#### type_converter.rs - 类型转换核心逻辑
**单一职责**: 处理 Rust 类型到 JSON 值的转换

```rust
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Field, Type, TypePath};

use crate::common::{MacroError, Result};

/// 类型转换器 trait
/// 
/// 符合开闭原则：对扩展开放，对修改关闭
pub trait TypeConverter {
    /// 将字段转换为 JSON 值的代码
    fn convert_field_to_json_value(&self, field: &Field) -> Result<TokenStream2>;
    
    /// 检查是否支持某个类型
    fn supports_type(&self, field_type: &Type) -> bool;
}

/// 内置类型转换器
pub struct BuiltinTypeConverter;

impl TypeConverter for BuiltinTypeConverter {
    fn convert_field_to_json_value(&self, field: &Field) -> Result<TokenStream2> {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        
        match field_type {
            Type::Path(type_path) if self.is_string_type(type_path) => {
                Ok(quote! {
                    serde_json::Value::String(self.#field_name.clone())
                })
            }
            Type::Path(type_path) if self.is_integer_type(type_path) => {
                Ok(quote! {
                    serde_json::Value::Number(
                        serde_json::Number::from(self.#field_name)
                    )
                })
            }
            Type::Path(type_path) if self.is_float_type(type_path) => {
                Ok(quote! {
                    serde_json::Number::from_f64(self.#field_name as f64)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                })
            }
            Type::Path(type_path) if self.is_bool_type(type_path) => {
                Ok(quote! {
                    serde_json::Value::Bool(self.#field_name)
                })
            }
            Type::Path(type_path) if self.is_option_type(type_path) => {
                // 递归处理 Option<T> 类型
                self.convert_option_type(field_name, type_path)
            }
            _ => Err(MacroError::TypeConversionError {
                rust_type: quote!(#field_type).to_string(),
                message: "不支持的类型转换".to_string(),
                span: field.span(),
            })
        }
    }
    
    fn supports_type(&self, field_type: &Type) -> bool {
        match field_type {
            Type::Path(type_path) => {
                self.is_string_type(type_path) ||
                self.is_integer_type(type_path) ||
                self.is_float_type(type_path) ||
                self.is_bool_type(type_path) ||
                self.is_option_type(type_path)
            }
            _ => false,
        }
    }
}

impl BuiltinTypeConverter {
    /// 检查是否为字符串类型
    fn is_string_type(&self, type_path: &TypePath) -> bool {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "String"
        } else {
            false
        }
    }
    
    /// 检查是否为整数类型
    fn is_integer_type(&self, type_path: &TypePath) -> bool {
        if let Some(segment) = type_path.path.segments.last() {
            matches!(segment.ident.to_string().as_str(), 
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            )
        } else {
            false
        }
    }
    
    /// 检查是否为浮点类型
    fn is_float_type(&self, type_path: &TypePath) -> bool {
        if let Some(segment) = type_path.path.segments.last() {
            matches!(segment.ident.to_string().as_str(), "f32" | "f64")
        } else {
            false
        }
    }
    
    /// 检查是否为布尔类型
    fn is_bool_type(&self, type_path: &TypePath) -> bool {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "bool"
        } else {
            false
        }
    }
    
    /// 检查是否为 Option 类型
    fn is_option_type(&self, type_path: &TypePath) -> bool {
        if let Some(segment) = type_path.path.segments.last() {
            segment.ident == "Option"
        } else {
            false
        }
    }
    
    /// 转换 Option 类型
    fn convert_option_type(
        &self, 
        field_name: &syn::Ident, 
        _type_path: &TypePath
    ) -> Result<TokenStream2> {
        Ok(quote! {
            match &self.#field_name {
                Some(value) => {
                    // 根据内部类型递归转换
                    // 这里简化处理，实际需要根据 T 的具体类型来转换
                    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
                }
                None => serde_json::Value::Null,
            }
        })
    }
}
```

## 依赖关系

### 内部模块依赖图
```
lib.rs (宏入口)
    ↓
node/ + mark/ (派生实现)
    ↓
generator/ (代码生成)
    ↓
parser/ + converter/ (解析和转换)
    ↓
common/ (通用功能)
```

### 外部依赖关系

#### Cargo.toml 配置
```toml
[package]
name = "mf-derive"
version = "0.1.0"
edition = "2021"
description = "ModuForge-RS 宏扩展模块，提供 Node 和 Mark 的派生宏"
license = "MIT OR Apache-2.0"

[lib]
proc-macro = true

[dependencies]
# 宏开发核心依赖
proc-macro2 = "1.0"
syn = { version = "2.0", features = ["full"] }
quote = "1.0"

# ModuForge-RS 依赖
mf-core = { path = "../core" }
mf-model = { path = "../model" }

# 序列化支持
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 工具依赖
once_cell = "1.19"

[dev-dependencies]
# 测试依赖
trybuild = "1.0"
tokio = { version = "1.0", features = ["full"] }

[features]
default = []
# 调试功能，启用详细的宏展开日志
debug = []
```

### 依赖关系约束

#### 版本兼容性
- **mf-core**: 必须使用相同工作空间版本，确保 API 兼容
- **mf-model**: 必须使用相同工作空间版本，确保类型定义一致
- **proc-macro2/syn/quote**: 使用稳定版本，避免宏生态破坏性更新

#### 循环依赖预防
- **mf-derive → mf-core**: 单向依赖，mf-core 不依赖 mf-derive
- **模块内部**: 严格按照层次化依赖，防止循环引用

## 配置管理

### 编译时配置

#### 功能开关
```rust
// src/lib.rs 中的条件编译
#[cfg(feature = "debug")]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        eprintln!("[mf-derive] {}", format!($($arg)*));
    };
}

#[cfg(not(feature = "debug"))]
macro_rules! debug_print {
    ($($arg:tt)*) => {};
}
```

#### 环境变量支持
```rust
// 在编译时读取环境变量进行配置
const ENABLE_OPTIMIZATION: bool = option_env!("MF_DERIVE_OPTIMIZE")
    .map(|s| s == "1")
    .unwrap_or(true);
```

### 运行时配置

#### 转换器配置
```rust
/// 全局转换器注册表
pub static CONVERTER_REGISTRY: once_cell::sync::Lazy<
    std::sync::Mutex<ConverterRegistry>
> = once_cell::sync::Lazy::new(|| {
    let mut registry = ConverterRegistry::new();
    
    // 注册内置转换器
    registry.register(BuiltinTypeConverter);
    
    std::sync::Mutex::new(registry)
});

/// 允许用户注册自定义转换器
pub fn register_type_converter<T: TypeConverter + 'static>(converter: T) {
    if let Ok(mut registry) = CONVERTER_REGISTRY.lock() {
        registry.register(converter);
    }
}
```

## 开发规范

### 代码规范

#### 文件头模板
```rust
//! 模块功能简介
//!
//! 详细说明模块的职责和使用方式
//!
//! # 示例
//! ```rust
//! // 使用示例代码
//! ```

use std::collections::HashMap;
// 标准库导入

use proc_macro2::TokenStream as TokenStream2;
// 第三方库导入

use crate::common::Result;
// 内部模块导入
```

#### 错误处理规范
```rust
/// 所有可能出错的函数必须返回 Result<T, MacroError>
pub fn risky_operation() -> Result<String> {
    // 实现逻辑
    Ok("success".to_string())
}

/// 错误传播使用 ? 操作符
pub fn caller() -> Result<()> {
    let result = risky_operation()?;
    // 处理结果
    Ok(())
}
```

#### 文档规范
```rust
/// 函数功能的简短描述
///
/// 更详细的功能说明，包括使用场景和注意事项。
///
/// # 参数
/// - `input`: 参数说明
/// - `config`: 配置参数说明
///
/// # 返回值
/// 返回值的详细说明
///
/// # 错误
/// 可能出现的错误情况说明
///
/// # 示例
/// ```rust
/// let result = function_name(input, config)?;
/// assert_eq!(result.len(), 5);
/// ```
pub fn function_name(input: &str, config: Config) -> Result<Vec<String>> {
    // 实现
}
```

### 测试规范

#### 测试文件结构
```rust
// tests/node_derive_tests.rs
use mf_derive::Node;
use serde::{Serialize, Deserialize};

/// 基础功能测试
mod basic_functionality {
    use super::*;
    
    #[test]
    fn test_simple_node_derivation() {
        // 测试实现
    }
}

/// 错误场景测试
mod error_cases {
    use super::*;
    
    #[test]
    fn test_missing_node_type_attribute() {
        // 测试错误场景
    }
}
```

#### 基准测试规范
```rust
// benches/compilation_time.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_node_derivation(c: &mut Criterion) {
    c.bench_function("derive node with 10 fields", |b| {
        b.iter(|| {
            // 基准测试逻辑
        })
    });
}

criterion_group!(benches, benchmark_node_derivation);
criterion_main!(benches);
```

### 提交规范

#### Git Commit 消息格式
```
类型(范围): 简短描述

详细描述变更的内容和原因。

符合的设计原则：
- SRP: 模块职责单一
- ISP: 接口精简专用
- OCP: 支持扩展
- LSP: 保证替换性

BREAKING CHANGE: 如果有破坏性变更的说明
```

#### 类型标识符
- `feat`: 新功能
- `fix`: 错误修复
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 重构
- `test`: 测试相关
- `perf`: 性能优化

通过这个项目结构设计，mf-derive 将成为一个职责清晰、易于维护、符合设计原则的高质量宏系统，为 ModuForge-RS 生态提供强大的开发工具支持。
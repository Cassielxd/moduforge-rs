# ModuForge-RS 宏模块扩展 - 模块化设计方案

## 1. 设计概述

### 1.1 设计理念
基于现有 `mf-derive` 模块，通过添加新的派生宏功能，为 ModuForge-RS 框架提供声明式的节点和标记定义能力。设计严格遵循 SOLID 原则，确保代码的可维护性和扩展性。

### 1.2 设计目标
- **功能完整性**: 支持 `#[derive(Node)]` 和 `#[derive(Mark)]` 派生宏
- **类型安全性**: 编译时验证和类型检查
- **性能优化**: 最小化编译时和运行时开销
- **代码质量**: 高内聚、低耦合的模块化设计

### 1.3 设计约束
- **现有架构**: 仅在 `mf-derive` 模块中扩展，不改变整体项目结构
- **API 兼容**: 完全兼容 `mf-core` 现有 API
- **依赖限制**: 最小化新增外部依赖

## 2. 系统架构

### 2.1 整体架构图
```
┌─────────────────────────────────────────────────────────────────────┐
│                          用户代码层                                  │
│  #[derive(Node)] struct MyNode { #[attr] field: String }           │
└─────────────────────┬───────────────────────────────────────────────┘
                      │ 编译时宏展开
                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      mf-derive 宏处理层                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐    │
│  │   Node 派生宏    │  │   Mark 派生宏    │  │   通用工具宏     │    │
│  │   derive_node    │  │   derive_mark    │  │   utilities     │    │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘    │
│              │                    │                    │           │
│              ▼                    ▼                    ▼           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐    │
│  │  属性解析器      │  │  代码生成器      │  │  类型转换器      │    │
│  │  AttributeParser │  │  CodeGenerator   │  │  TypeConverter   │    │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘    │
└─────────────────────┬───────────────────┬───────────────────┬─────┘
                      │                   │                   │
                      ▼                   ▼                   ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         生成代码层                                   │
│  impl MyNode { pub fn to_node(&self) -> mf_core::node::Node {...} } │
└─────────────────────┬───────────────────────────────────────────────┘
                      │ 调用
                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        mf-core API层                                │
│  Node::create(), Mark::new(), node.set_attrs(), mark.set_name()    │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 模块层次结构
```
crates/derive/src/
├── lib.rs                    # 宏入口点和公共接口
├── common/                   # 通用模块 (遵循DRY原则)
│   ├── mod.rs
│   ├── error.rs             # 统一错误处理
│   ├── utils.rs             # 通用工具函数
│   └── constants.rs         # 常量定义
├── parser/                   # 解析模块 (单一职责原则)
│   ├── mod.rs
│   ├── attribute_parser.rs  # 宏属性解析
│   ├── field_analyzer.rs    # 字段分析
│   └── validation.rs        # 验证逻辑
├── generator/                # 生成模块 (开闭原则)
│   ├── mod.rs
│   ├── node_generator.rs    # Node 代码生成
│   ├── mark_generator.rs    # Mark 代码生成
│   └── helper_generator.rs  # 辅助代码生成
├── converter/                # 转换模块 (接口隔离原则)
│   ├── mod.rs
│   ├── type_converter.rs    # 类型转换核心
│   ├── builtin_converters.rs # 内置转换器
│   └── converter_registry.rs # 转换器注册
├── node/                     # Node 宏实现 (里氏替换原则)
│   ├── mod.rs
│   ├── derive_impl.rs       # 派生宏实现
│   ├── attribute_handler.rs # 属性处理
│   └── code_gen.rs          # 代码生成逻辑
└── mark/                     # Mark 宏实现 (里氏替换原则)
    ├── mod.rs
    ├── derive_impl.rs       # 派生宏实现
    ├── attribute_handler.rs # 属性处理
    └── code_gen.rs          # 代码生成逻辑
```

## 3. 核心模块设计

### 3.1 入口模块 (lib.rs)

#### 3.1.1 模块职责
- **单一职责**: 仅负责宏注册和公共接口导出
- **接口隔离**: 只暴露必要的公共 API

#### 3.1.2 接口设计
```rust
//! ModuForge-RS 宏扩展
//! 
//! 提供 #[derive(Node)] 和 #[derive(Mark)] 派生宏

use proc_macro::TokenStream;

/// Node 派生宏入口点
#[proc_macro_derive(Node, attributes(node_type, marks, content, attr))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    node::derive_impl::process_derive_node(input)
}

/// Mark 派生宏入口点
#[proc_macro_derive(Mark, attributes(mark_type, attr))]  
pub fn derive_mark(input: TokenStream) -> TokenStream {
    mark::derive_impl::process_derive_mark(input)
}

// 重新导出公共类型
pub use common::error::MacroError;
pub use parser::attribute_parser::AttributeConfig;
pub use converter::type_converter::{TypeConverter, ConversionResult};
```

### 3.2 通用模块 (common/)

#### 3.2.1 错误处理设计 (error.rs)
```rust
use thiserror::Error;
use syn::Error as SynError;

/// 宏处理过程中的错误类型
#[derive(Error, Debug)]
pub enum MacroError {
    #[error("缺少必需的宏属性: {attribute}")]
    MissingAttribute { attribute: String },
    
    #[error("无效的属性值 '{value}' 用于属性 '{attribute}'")]
    InvalidAttributeValue { attribute: String, value: String },
    
    #[error("不支持的字段类型 '{field_type}' 在字段 '{field_name}' 中")]
    UnsupportedFieldType { field_name: String, field_type: String },
    
    #[error("属性解析错误: {message}")]
    ParseError { message: String },
    
    #[error("代码生成错误: {message}")]
    GenerationError { message: String },
    
    #[error("语法错误: {source}")]
    SyntaxError {
        #[from]
        source: SynError,
    },
}

impl MacroError {
    /// 转换为编译时错误
    pub fn to_compile_error(&self) -> proc_macro2::TokenStream {
        let message = self.to_string();
        quote::quote! {
            compile_error!(#message);
        }
    }
    
    /// 创建带有具体位置的错误
    pub fn spanned<T: quote::ToTokens>(message: &str, spanned: T) -> Self {
        let span = spanned.into_token_stream().span();
        Self::ParseError {
            message: format!("{} at {}", message, span.start()),
        }
    }
}

/// 宏处理结果类型
pub type MacroResult<T> = Result<T, MacroError>;
```

#### 3.2.2 工具函数设计 (utils.rs)
```rust
use quote::quote;
use syn::{Ident, Type};
use proc_macro2::TokenStream;

/// 生成导入语句
pub fn generate_imports() -> TokenStream {
    quote! {
        use mf_core::node::Node as CoreNode;
        use mf_core::mark::Mark as CoreMark;
        use serde_json::Value as JsonValue;
    }
}

/// 检查类型是否为 Option<T>
pub fn is_option_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            type_path.path.segments
                .last()
                .map(|seg| seg.ident == "Option")
                .unwrap_or(false)
        }
        _ => false,
    }
}

/// 提取 Option<T> 中的内部类型 T
pub fn extract_option_inner_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last()?;
            if last_segment.ident != "Option" {
                return None;
            }
            
            match &last_segment.arguments {
                syn::PathArguments::AngleBracketed(args) => {
                    args.args.first().and_then(|arg| {
                        match arg {
                            syn::GenericArgument::Type(ty) => Some(ty),
                            _ => None,
                        }
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// 生成字段到 JsonValue 的转换代码
pub fn generate_field_conversion(field_name: &Ident, field_type: &Type) -> TokenStream {
    if is_option_type(field_type) {
        quote! {
            self.#field_name.as_ref().map(|v| serde_json::to_value(v).unwrap_or(JsonValue::Null))
                .unwrap_or(JsonValue::Null)
        }
    } else {
        quote! {
            serde_json::to_value(&self.#field_name).unwrap_or(JsonValue::Null)
        }
    }
}

/// 生成友好的编译错误消息
pub fn create_compile_error(message: &str) -> TokenStream {
    quote! {
        compile_error!(#message);
    }
}
```

### 3.3 解析模块 (parser/)

#### 3.3.1 属性解析器设计 (attribute_parser.rs)
```rust
use syn::{DeriveInput, Attribute, Meta, NestedMeta, Lit};
use crate::common::error::{MacroError, MacroResult};

/// Node 属性配置
#[derive(Debug, Clone, Default)]
pub struct NodeConfig {
    pub node_type: Option<String>,
    pub marks: Option<Vec<String>>,
    pub content: Option<String>,
    pub attr_fields: Vec<FieldConfig>,
}

/// Mark 属性配置
#[derive(Debug, Clone, Default)]
pub struct MarkConfig {
    pub mark_type: Option<String>,
    pub attr_fields: Vec<FieldConfig>,
}

/// 字段配置
#[derive(Debug, Clone)]
pub struct FieldConfig {
    pub name: String,
    pub type_name: String,
    pub is_optional: bool,
    pub is_attr: bool,
}

/// 属性解析器
pub struct AttributeParser;

impl AttributeParser {
    /// 解析 Node 相关属性
    pub fn parse_node_attributes(input: &DeriveInput) -> MacroResult<NodeConfig> {
        let mut config = NodeConfig::default();
        
        // 解析结构体级别的属性
        for attr in &input.attrs {
            if attr.path.is_ident("node_type") {
                config.node_type = Some(Self::parse_string_attribute(attr)?);
            } else if attr.path.is_ident("marks") {
                let marks_str = Self::parse_string_attribute(attr)?;
                config.marks = Some(
                    marks_str.split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                );
            } else if attr.path.is_ident("content") {
                config.content = Some(Self::parse_string_attribute(attr)?);
            }
        }
        
        // 验证必需属性
        if config.node_type.is_none() {
            return Err(MacroError::MissingAttribute {
                attribute: "node_type".to_string(),
            });
        }
        
        // 解析字段属性
        config.attr_fields = Self::parse_field_attributes(input)?;
        
        Ok(config)
    }
    
    /// 解析 Mark 相关属性
    pub fn parse_mark_attributes(input: &DeriveInput) -> MacroResult<MarkConfig> {
        let mut config = MarkConfig::default();
        
        // 解析结构体级别的属性
        for attr in &input.attrs {
            if attr.path.is_ident("mark_type") {
                config.mark_type = Some(Self::parse_string_attribute(attr)?);
            }
        }
        
        // 验证必需属性
        if config.mark_type.is_none() {
            return Err(MacroError::MissingAttribute {
                attribute: "mark_type".to_string(),
            });
        }
        
        // 解析字段属性
        config.attr_fields = Self::parse_field_attributes(input)?;
        
        Ok(config)
    }
    
    /// 解析字符串属性值
    fn parse_string_attribute(attr: &Attribute) -> MacroResult<String> {
        match attr.parse_meta()? {
            Meta::NameValue(meta) => {
                match meta.lit {
                    Lit::Str(lit_str) => Ok(lit_str.value()),
                    _ => Err(MacroError::InvalidAttributeValue {
                        attribute: attr.path.get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_default(),
                        value: "非字符串值".to_string(),
                    }),
                }
            }
            _ => Err(MacroError::ParseError {
                message: "属性格式错误，期望 key = \"value\"".to_string(),
            }),
        }
    }
    
    /// 解析字段属性
    fn parse_field_attributes(input: &DeriveInput) -> MacroResult<Vec<FieldConfig>> {
        let mut fields = Vec::new();
        
        match &input.data {
            syn::Data::Struct(data_struct) => {
                match &data_struct.fields {
                    syn::Fields::Named(named_fields) => {
                        for field in &named_fields.named {
                            let field_name = field.ident.as_ref()
                                .ok_or_else(|| MacroError::ParseError {
                                    message: "字段缺少名称".to_string(),
                                })?;
                            
                            let is_attr = field.attrs.iter()
                                .any(|attr| attr.path.is_ident("attr"));
                            
                            if is_attr {
                                let type_name = quote::quote!(#(&field.ty)).to_string();
                                let is_optional = crate::common::utils::is_option_type(&field.ty);
                                
                                fields.push(FieldConfig {
                                    name: field_name.to_string(),
                                    type_name,
                                    is_optional,
                                    is_attr,
                                });
                            }
                        }
                    }
                    _ => {
                        return Err(MacroError::ParseError {
                            message: "只支持具名字段的结构体".to_string(),
                        });
                    }
                }
            }
            _ => {
                return Err(MacroError::ParseError {
                    message: "只支持结构体类型".to_string(),
                });
            }
        }
        
        Ok(fields)
    }
}
```

#### 3.3.2 验证模块设计 (validation.rs)
```rust
use syn::Type;
use crate::common::error::{MacroError, MacroResult};
use crate::parser::attribute_parser::{NodeConfig, MarkConfig, FieldConfig};

/// 编译时验证器
pub struct Validator;

impl Validator {
    /// 验证 Node 配置
    pub fn validate_node_config(config: &NodeConfig) -> MacroResult<()> {
        // 验证节点类型
        Self::validate_identifier(&config.node_type.as_ref().unwrap(), "node_type")?;
        
        // 验证标记类型
        if let Some(marks) = &config.marks {
            for mark in marks {
                Self::validate_identifier(mark, "marks")?;
            }
        }
        
        // 验证字段类型
        for field in &config.attr_fields {
            Self::validate_field_type(field)?;
        }
        
        Ok(())
    }
    
    /// 验证 Mark 配置
    pub fn validate_mark_config(config: &MarkConfig) -> MacroResult<()> {
        // 验证标记类型
        Self::validate_identifier(&config.mark_type.as_ref().unwrap(), "mark_type")?;
        
        // 验证字段类型
        for field in &config.attr_fields {
            Self::validate_field_type(field)?;
        }
        
        Ok(())
    }
    
    /// 验证标识符格式
    fn validate_identifier(identifier: &str, context: &str) -> MacroResult<()> {
        if identifier.is_empty() {
            return Err(MacroError::InvalidAttributeValue {
                attribute: context.to_string(),
                value: "空字符串".to_string(),
            });
        }
        
        if !identifier.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(MacroError::InvalidAttributeValue {
                attribute: context.to_string(),
                value: format!("'{}'包含无效字符", identifier),
            });
        }
        
        Ok(())
    }
    
    /// 验证字段类型兼容性
    fn validate_field_type(field: &FieldConfig) -> MacroResult<()> {
        // 检查是否为支持的基本类型
        let supported_types = [
            "String", "str", "i32", "i64", "u32", "u64", 
            "f32", "f64", "bool", "usize", "isize"
        ];
        
        let type_str = &field.type_name;
        let is_supported = supported_types.iter()
            .any(|&t| type_str.contains(t));
        
        if !is_supported && !field.is_optional {
            return Err(MacroError::UnsupportedFieldType {
                field_name: field.name.clone(),
                field_type: type_str.clone(),
            });
        }
        
        Ok(())
    }
}
```

### 3.4 代码生成模块 (generator/)

#### 3.4.1 Node 代码生成器设计 (node_generator.rs)
```rust
use quote::{quote, ToTokens};
use proc_macro2::TokenStream;
use syn::{DeriveInput, Ident};

use crate::common::error::{MacroError, MacroResult};
use crate::common::utils;
use crate::parser::attribute_parser::NodeConfig;

/// Node 代码生成器
pub struct NodeGenerator;

impl NodeGenerator {
    /// 生成 to_node() 方法
    pub fn generate_to_node_method(
        input: &DeriveInput,
        config: &NodeConfig,
    ) -> MacroResult<TokenStream> {
        let struct_name = &input.ident;
        let node_type = config.node_type.as_ref().unwrap();
        
        // 生成属性设置代码
        let attr_setters = Self::generate_attr_setters(config)?;
        
        // 生成标记设置代码  
        let marks_setter = Self::generate_marks_setter(config)?;
        
        // 生成内容设置代码
        let content_setter = Self::generate_content_setter(config)?;
        
        // 生成导入语句
        let imports = utils::generate_imports();
        
        let expanded = quote! {
            #imports
            
            impl #struct_name {
                /// 将结构体转换为 mf_core::node::Node 实例
                /// 
                /// 此方法由 #[derive(Node)] 宏自动生成，
                /// 根据结构体的字段和宏属性配置创建相应的 Node 实例。
                /// 
                /// # 返回值
                /// 
                /// 返回配置好的 `mf_core::node::Node` 实例
                /// 
                /// # 示例
                /// 
                /// ```rust
                /// let instance = MyStruct { field: "value".to_string() };
                /// let node = instance.to_node();
                /// ```
                pub fn to_node(&self) -> CoreNode {
                    // 创建 NodeSpec
                    let mut node_spec = mf_model::node_type::NodeSpec::default();
                    node_spec.name = #node_type.to_string();
                    
                    // 创建基础 Node
                    let mut node = CoreNode::create("", node_spec);
                    
                    // 设置节点名称 (使用结构体类型名)
                    node.set_name(stringify!(#struct_name));
                    
                    #attr_setters
                    #marks_setter
                    #content_setter
                    
                    node
                }
            }
        };
        
        Ok(expanded)
    }
    
    /// 生成属性设置代码
    fn generate_attr_setters(config: &NodeConfig) -> MacroResult<TokenStream> {
        let mut setters = Vec::new();
        
        for field in &config.attr_fields {
            let field_name = syn::parse_str::<Ident>(&field.name)
                .map_err(|_| MacroError::ParseError {
                    message: format!("无效的字段名: {}", field.name),
                })?;
            
            let field_type = syn::parse_str::<syn::Type>(&field.type_name)
                .map_err(|_| MacroError::ParseError {
                    message: format!("无效的类型: {}", field.type_name),
                })?;
            
            let conversion = utils::generate_field_conversion(&field_name, &field_type);
            
            setters.push(quote! {
                node.set_attr(stringify!(#field_name), Some(#conversion));
            });
        }
        
        Ok(quote! { #(#setters)* })
    }
    
    /// 生成标记设置代码
    fn generate_marks_setter(config: &NodeConfig) -> MacroResult<TokenStream> {
        if let Some(marks) = &config.marks {
            let marks_str = marks.join(",");
            Ok(quote! {
                node.set_marks(#marks_str.to_string());
            })
        } else {
            Ok(quote! {})
        }
    }
    
    /// 生成内容设置代码
    fn generate_content_setter(config: &NodeConfig) -> MacroResult<TokenStream> {
        if let Some(content) = &config.content {
            Ok(quote! {
                node.set_content(#content);
            })
        } else {
            Ok(quote! {})
        }
    }
}
```

#### 3.4.2 Mark 代码生成器设计 (mark_generator.rs)
```rust
use quote::{quote, ToTokens};
use proc_macro2::TokenStream;
use syn::{DeriveInput, Ident};

use crate::common::error::{MacroError, MacroResult};
use crate::common::utils;
use crate::parser::attribute_parser::MarkConfig;

/// Mark 代码生成器
pub struct MarkGenerator;

impl MarkGenerator {
    /// 生成 to_mark() 方法
    pub fn generate_to_mark_method(
        input: &DeriveInput,
        config: &MarkConfig,
    ) -> MacroResult<TokenStream> {
        let struct_name = &input.ident;
        let mark_type = config.mark_type.as_ref().unwrap();
        
        // 生成属性设置代码
        let attr_setters = Self::generate_attr_setters(config)?;
        
        // 生成导入语句
        let imports = utils::generate_imports();
        
        let expanded = quote! {
            #imports
            
            impl #struct_name {
                /// 将结构体转换为 mf_core::mark::Mark 实例
                /// 
                /// 此方法由 #[derive(Mark)] 宏自动生成，
                /// 根据结构体的字段和宏属性配置创建相应的 Mark 实例。
                /// 
                /// # 返回值
                /// 
                /// 返回配置好的 `mf_core::mark::Mark` 实例
                /// 
                /// # 示例
                /// 
                /// ```rust
                /// let instance = MyMark { field: "value".to_string() };
                /// let mark = instance.to_mark();
                /// ```
                pub fn to_mark(&self) -> CoreMark {
                    // 创建 MarkSpec
                    let mut mark_spec = mf_model::mark_type::MarkSpec::default();
                    mark_spec.name = #mark_type.to_string();
                    
                    // 创建基础 Mark
                    let mut mark = CoreMark::new(stringify!(#struct_name), mark_spec);
                    
                    #attr_setters
                    
                    mark
                }
            }
        };
        
        Ok(expanded)
    }
    
    /// 生成属性设置代码
    fn generate_attr_setters(config: &MarkConfig) -> MacroResult<TokenStream> {
        let mut setters = Vec::new();
        
        for field in &config.attr_fields {
            let field_name = syn::parse_str::<Ident>(&field.name)
                .map_err(|_| MacroError::ParseError {
                    message: format!("无效的字段名: {}", field.name),
                })?;
            
            let field_type = syn::parse_str::<syn::Type>(&field.type_name)
                .map_err(|_| MacroError::ParseError {
                    message: format!("无效的类型: {}", field.type_name),
                })?;
            
            let conversion = utils::generate_field_conversion(&field_name, &field_type);
            
            setters.push(quote! {
                mark.set_attr(stringify!(#field_name), Some(#conversion));
            });
        }
        
        Ok(quote! { #(#setters)* })
    }
}
```

### 3.5 派生宏实现模块

#### 3.5.1 Node 派生宏实现 (node/derive_impl.rs)
```rust
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use crate::common::error::MacroError;
use crate::parser::attribute_parser::AttributeParser;
use crate::parser::validation::Validator;
use crate::generator::node_generator::NodeGenerator;

/// 处理 #[derive(Node)] 派生宏
pub fn process_derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // 错误处理包装器
    match process_node_derive_internal(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// 内部处理逻辑
fn process_node_derive_internal(input: &DeriveInput) -> Result<proc_macro2::TokenStream, MacroError> {
    // 1. 解析属性配置
    let config = AttributeParser::parse_node_attributes(input)?;
    
    // 2. 验证配置正确性
    Validator::validate_node_config(&config)?;
    
    // 3. 生成代码
    let generated = NodeGenerator::generate_to_node_method(input, &config)?;
    
    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    
    #[test]
    fn test_basic_node_derive() {
        let input = quote! {
            #[derive(Node)]
            #[node_type = "test_node"]
            pub struct TestNode {
                #[attr]
                name: String,
                #[attr]
                value: Option<i32>,
            }
        };
        
        let result = process_derive_node(input.into());
        assert!(!result.to_string().contains("compile_error"));
    }
    
    #[test]
    fn test_missing_node_type_error() {
        let input = quote! {
            #[derive(Node)]
            pub struct TestNode {
                #[attr]
                name: String,
            }
        };
        
        let result = process_derive_node(input.into());
        assert!(result.to_string().contains("compile_error"));
        assert!(result.to_string().contains("node_type"));
    }
}
```

#### 3.5.2 Mark 派生宏实现 (mark/derive_impl.rs)
```rust
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use crate::common::error::MacroError;
use crate::parser::attribute_parser::AttributeParser;
use crate::parser::validation::Validator;
use crate::generator::mark_generator::MarkGenerator;

/// 处理 #[derive(Mark)] 派生宏
pub fn process_derive_mark(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // 错误处理包装器
    match process_mark_derive_internal(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// 内部处理逻辑
fn process_mark_derive_internal(input: &DeriveInput) -> Result<proc_macro2::TokenStream, MacroError> {
    // 1. 解析属性配置
    let config = AttributeParser::parse_mark_attributes(input)?;
    
    // 2. 验证配置正确性
    Validator::validate_mark_config(&config)?;
    
    // 3. 生成代码
    let generated = MarkGenerator::generate_to_mark_method(input, &config)?;
    
    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    
    #[test]
    fn test_basic_mark_derive() {
        let input = quote! {
            #[derive(Mark)]
            #[mark_type = "test_mark"]
            pub struct TestMark {
                #[attr]
                name: String,
            }
        };
        
        let result = process_derive_mark(input.into());
        assert!(!result.to_string().contains("compile_error"));
    }
    
    #[test]
    fn test_missing_mark_type_error() {
        let input = quote! {
            #[derive(Mark)]
            pub struct TestMark {
                #[attr]
                name: String,
            }
        };
        
        let result = process_derive_mark(input.into());
        assert!(result.to_string().contains("compile_error"));
        assert!(result.to_string().contains("mark_type"));
    }
}
```

## 4. 接口设计

### 4.1 公共接口
```rust
// 派生宏接口
#[proc_macro_derive(Node, attributes(node_type, marks, content, attr))]
pub fn derive_node(input: TokenStream) -> TokenStream;

#[proc_macro_derive(Mark, attributes(mark_type, attr))]
pub fn derive_mark(input: TokenStream) -> TokenStream;

// 配置类型接口
pub struct NodeConfig {
    pub node_type: Option<String>,
    pub marks: Option<Vec<String>>,
    pub content: Option<String>,
    pub attr_fields: Vec<FieldConfig>,
}

pub struct MarkConfig {
    pub mark_type: Option<String>,
    pub attr_fields: Vec<FieldConfig>,
}

// 错误处理接口
pub enum MacroError {
    MissingAttribute { attribute: String },
    InvalidAttributeValue { attribute: String, value: String },
    UnsupportedFieldType { field_name: String, field_type: String },
    // ...其他错误类型
}
```

### 4.2 内部接口
```rust
// 解析器接口
trait AttributeParser {
    type Config;
    fn parse(input: &DeriveInput) -> MacroResult<Self::Config>;
}

// 验证器接口
trait Validator<T> {
    fn validate(config: &T) -> MacroResult<()>;
}

// 代码生成器接口
trait CodeGenerator<T> {
    fn generate(input: &DeriveInput, config: &T) -> MacroResult<TokenStream>;
}
```

## 5. 数据流设计

### 5.1 处理流程图
```
用户代码 (结构体 + 宏属性)
    │
    ▼
TokenStream 输入
    │
    ▼
syn 解析为 DeriveInput
    │
    ▼
属性解析器 (AttributeParser)
    │
    ▼
配置对象 (NodeConfig/MarkConfig)
    │
    ▼
验证器 (Validator)
    │
    ▼ (验证通过)
代码生成器 (CodeGenerator)
    │
    ▼
生成的 TokenStream
    │
    ▼
编译器处理
    │
    ▼
最终的 Rust 代码
```

### 5.2 错误处理流程
```
解析/验证/生成过程
    │
    ▼ (发生错误)
MacroError 
    │
    ▼
错误转换为 compile_error! 宏
    │
    ▼
编译时错误信息
    │
    ▼
用户收到友好的错误提示
```

## 6. 性能优化设计

### 6.1 编译时性能优化
1. **缓存机制**: 对重复的类型分析结果进行缓存
2. **懒加载**: 只在需要时进行复杂的类型推导
3. **批量处理**: 对多个字段的处理进行批量优化
4. **内存管理**: 及时释放临时数据结构

### 6.2 生成代码优化
1. **零拷贝**: 生成的代码尽量避免不必要的数据拷贝
2. **内联优化**: 为小函数添加 `#[inline]` 提示
3. **分支预测**: 优化条件判断的顺序
4. **内存布局**: 考虑数据结构的内存对齐

## 7. 测试策略

### 7.1 单元测试
- **解析器测试**: 各种属性配置的解析正确性
- **验证器测试**: 错误配置的检测能力
- **生成器测试**: 生成代码的正确性

### 7.2 集成测试
- **端到端测试**: 完整的宏处理流程
- **兼容性测试**: 与 mf-core API 的集成
- **性能测试**: 编译时和运行时性能

### 7.3 错误场景测试
- **编译失败测试**: 使用 `trybuild` 测试预期的编译错误
- **边界条件测试**: 极端输入情况的处理
- **回归测试**: 确保修改不会破坏现有功能

## 8. 扩展性设计

### 8.1 插件系统
```rust
/// 宏处理插件接口
trait MacroPlugin {
    fn name(&self) -> &str;
    fn process_attributes(&self, attrs: &mut Vec<Attribute>);
    fn generate_additional_code(&self, config: &dyn Any) -> TokenStream;
}

/// 插件注册器
pub struct PluginRegistry {
    plugins: Vec<Box<dyn MacroPlugin>>,
}

impl PluginRegistry {
    pub fn register<P: MacroPlugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }
}
```

### 8.2 自定义转换器
```rust
/// 类型转换器特征
pub trait TypeConverter {
    fn can_convert(&self, type_info: &TypeInfo) -> bool;
    fn convert(&self, value: &TokenStream) -> MacroResult<TokenStream>;
}

/// 转换器注册表
pub struct ConverterRegistry {
    converters: Vec<Box<dyn TypeConverter>>,
}
```

### 8.3 配置扩展
```rust
/// 可扩展的配置系统
pub trait ExtendableConfig {
    type Extension;
    
    fn add_extension(&mut self, ext: Self::Extension);
    fn get_extension<T: 'static>(&self) -> Option<&T>;
}
```

这份设计文档为 ModuForge-RS 宏模块扩展提供了完整的模块化设计方案，严格遵循 SOLID 原则，确保代码的可维护性、可扩展性和可测试性。
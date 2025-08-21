# ModuForge-RS Default 属性扩展 - 模块化设计架构

## 1. 设计概述

### 1.1 设计原则遵循

本设计严格遵循核心设计原则，确保扩展的高质量和可维护性：

#### 🎯 单一职责原则（SRP）应用
- **模块职责明确**：每个新增模块只负责一个特定的功能领域
- **DefaultValueProcessor**：专门处理默认值解析和验证
- **ValidationPipeline**：专门处理分层验证逻辑
- **CodeGeneratorEnhancer**：专门处理代码生成增强

#### 🔗 接口隔离原则（ISP）应用
- **最小化接口**：为不同职责提供专门的 trait 接口
- **DefaultValueValidator**：只包含验证相关方法
- **CodeGenerator**：只包含代码生成相关方法
- **TypeAnalyzer**：只包含类型分析相关方法

#### 🔓 开闭原则（OCP）应用
- **扩展而非修改**：现有代码保持不变，通过扩展实现新功能
- **插件化验证器**：支持新的类型验证器无缝添加
- **模板化代码生成**：支持自定义代码生成模板

#### 🔄 里氏替换原则（LSP）应用
- **接口兼容性**：新增的 FieldConfig 完全兼容现有使用
- **行为一致性**：扩展的代码生成器保持与原有生成器相同的接口契约
- **类型安全**：所有默认值处理保持类型安全保证

### 1.2 架构设计目标

- **零破坏性变更**：现有代码完全不受影响
- **渐进式增强**：新功能作为可选扩展提供
- **高性能**：编译时优化，运行时零开销
- **高可维护性**：清晰的模块边界和职责分离
- **高扩展性**：插件化架构支持未来功能扩展

## 2. 系统架构设计

### 2.1 整体架构图

```mermaid
graph TB
    subgraph "用户代码层"
        A[#[derive(Node)]] --> B[#[attr(default="value")]]
        C[#[derive(Mark)]] --> D[#[attr(default="value")]]
    end
    
    subgraph "解析层 (遵循SRP)"
        E[AttributeParser] --> F[DefaultValueParser 🆕]
        F --> G[ValidationPipeline 🆕]
    end
    
    subgraph "验证层 (遵循ISP)"
        G --> H[StringValidator]
        G --> I[NumericValidator]
        G --> J[JsonValidator]
        G --> K[OptionValidator]
    end
    
    subgraph "生成层 (遵循OCP)"
        L[NodeGenerator Enhanced] --> M[DefaultValueCodeGen 🆕]
        N[MarkGenerator Enhanced] --> M
    end
    
    subgraph "输出层"
        M --> O[生成的 Rust 代码]
        O --> P[to_node() 方法增强]
        O --> Q[new() 构造函数]
        O --> R[with_defaults() 方法]
    end
    
    B --> E
    D --> E
    G --> L
    G --> N
    
    style F fill:#e1f5fe
    style G fill:#f3e5f5
    style M fill:#e8f5e8
```

### 2.2 模块层次设计

#### 2.2.1 核心模块扩展

```rust
// crates/derive/src/parser/default_value.rs 🆕
// 职责：默认值解析和表示
pub mod default_value {
    /// 默认值表示 - 遵循单一职责原则
    #[derive(Debug, Clone, PartialEq)]
    pub struct DefaultValue {
        /// 原始字符串值
        pub raw_value: String,
        /// 解析后的值类型
        pub value_type: DefaultValueType,
        /// 是否为 JSON 格式
        pub is_json: bool,
        /// 源码位置信息（用于错误报告）
        pub span: Option<Span>,
    }
    
    /// 默认值类型枚举 - 遵循接口隔离原则
    #[derive(Debug, Clone, PartialEq)]
    pub enum DefaultValueType {
        String(String),
        Integer(i64),
        Float(f64),
        Boolean(bool),
        Json(serde_json::Value),
        Null,
    }
    
    /// 默认值解析器 - 单一职责：解析默认值字符串
    pub struct DefaultValueParser;
    
    impl DefaultValueParser {
        /// 解析默认值字符串为结构化表示
        /// 
        /// # 设计原则体现
        /// - **单一职责**: 只负责字符串解析，不处理验证
        /// - **开闭原则**: 通过类型匹配支持新的默认值类型
        pub fn parse(raw_value: &str, span: Option<Span>) -> MacroResult<DefaultValue> {
            // 实现解析逻辑
        }
        
        /// 检测是否为 JSON 格式
        fn is_json_format(value: &str) -> bool {
            // JSON 格式检测逻辑
        }
    }
}
```

#### 2.2.2 验证器系统设计

```rust
// crates/derive/src/parser/validation.rs 扩展
// 职责：类型验证和约束检查

/// 默认值验证器接口 - 遵循接口隔离原则
pub trait DefaultValueValidator {
    /// 验证默认值与字段类型的兼容性
    fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()>;
    
    /// 检查是否支持指定的字段类型
    fn supports_type(&self, field_type: &Type) -> bool;
    
    /// 验证器优先级（用于排序）
    fn priority(&self) -> i32;
    
    /// 获取验证器名称（用于错误报告）
    fn name(&self) -> &'static str;
}

/// 验证器注册表 - 遵循依赖倒置原则
pub struct ValidatorRegistry {
    validators: Vec<Box<dyn DefaultValueValidator>>,
}

impl ValidatorRegistry {
    /// 创建预配置的验证器注册表
    /// 
    /// # 设计原则体现
    /// - **开闭原则**: 支持动态添加新的验证器
    /// - **依赖倒置**: 依赖抽象接口而非具体实现
    pub fn new() -> Self {
        let mut registry = Self {
            validators: Vec::new(),
        };
        
        // 注册内置验证器（按优先级排序）
        registry.register(Box::new(StringValidator));
        registry.register(Box::new(NumericValidator));
        registry.register(Box::new(BooleanValidator));
        registry.register(Box::new(JsonValidator));
        registry.register(Box::new(OptionValidator));
        
        registry
    }
    
    /// 注册新的验证器
    pub fn register(&mut self, validator: Box<dyn DefaultValueValidator>) {
        self.validators.push(validator);
        // 按优先级重新排序
        self.validators.sort_by_key(|v| -v.priority());
    }
    
    /// 验证默认值
    pub fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
        for validator in &self.validators {
            if validator.supports_type(field_type) {
                return validator.validate(default_value, field_type);
            }
        }
        
        Err(MacroError::unsupported_field_type(
            &extract_type_name(field_type),
            field_type.span()
        ))
    }
}

/// 字符串类型验证器 - 遵循单一职责原则
pub struct StringValidator;

impl DefaultValueValidator for StringValidator {
    fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
        if !self.supports_type(field_type) {
            return Err(MacroError::validation_error(
                "StringValidator 不支持此类型",
                default_value.span.unwrap_or_else(Span::call_site)
            ));
        }
        
        match &default_value.value_type {
            DefaultValueType::String(_) => Ok(()),
            _ => Err(MacroError::default_value_type_mismatch(
                &extract_type_name(field_type),
                &format!("{:?}", default_value.value_type),
                "String",
                default_value.span.unwrap_or_else(Span::call_site)
            ))
        }
    }
    
    fn supports_type(&self, field_type: &Type) -> bool {
        matches!(extract_type_name(field_type).as_str(), "String" | "str" | "&str")
    }
    
    fn priority(&self) -> i32 { 100 } // 高优先级，最常用
    
    fn name(&self) -> &'static str { "StringValidator" }
}

/// 数值类型验证器 - 遵循单一职责原则
pub struct NumericValidator;

impl DefaultValueValidator for NumericValidator {
    fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
        let type_name = extract_type_name(field_type);
        
        match &default_value.value_type {
            DefaultValueType::Integer(value) => {
                self.validate_integer_range(*value, &type_name, default_value.span)
            },
            DefaultValueType::Float(value) => {
                self.validate_float_range(*value, &type_name, default_value.span)
            },
            _ => Err(MacroError::default_value_type_mismatch(
                &type_name,
                &format!("{:?}", default_value.value_type),
                "numeric",
                default_value.span.unwrap_or_else(Span::call_site)
            ))
        }
    }
    
    fn supports_type(&self, field_type: &Type) -> bool {
        const NUMERIC_TYPES: &[&str] = &[
            "i8", "i16", "i32", "i64", "i128", "isize",
            "u8", "u16", "u32", "u64", "u128", "usize",
            "f32", "f64"
        ];
        
        let type_name = extract_type_name(field_type);
        NUMERIC_TYPES.contains(&type_name.as_str())
    }
    
    fn priority(&self) -> i32 { 90 }
    
    fn name(&self) -> &'static str { "NumericValidator" }
}

impl NumericValidator {
    /// 验证整数值是否在类型范围内
    fn validate_integer_range(&self, value: i64, type_name: &str, span: Option<Span>) -> MacroResult<()> {
        let in_range = match type_name {
            "i8" => value >= i8::MIN as i64 && value <= i8::MAX as i64,
            "i16" => value >= i16::MIN as i64 && value <= i16::MAX as i64,
            "i32" => value >= i32::MIN as i64 && value <= i32::MAX as i64,
            "i64" => true, // i64 范围最大
            "u8" => value >= 0 && value <= u8::MAX as i64,
            "u16" => value >= 0 && value <= u16::MAX as i64,
            "u32" => value >= 0 && value <= u32::MAX as i64,
            "u64" => value >= 0, // 正数检查
            _ => true,
        };
        
        if in_range {
            Ok(())
        } else {
            Err(MacroError::validation_error(
                &format!("默认值 {} 超出类型 {} 的取值范围", value, type_name),
                span.unwrap_or_else(Span::call_site)
            ))
        }
    }
    
    /// 验证浮点数值是否在类型范围内
    fn validate_float_range(&self, value: f64, type_name: &str, span: Option<Span>) -> MacroResult<()> {
        let in_range = match type_name {
            "f32" => value.is_finite() && value >= f32::MIN as f64 && value <= f32::MAX as f64,
            "f64" => value.is_finite(),
            _ => false,
        };
        
        if in_range {
            Ok(())
        } else {
            Err(MacroError::validation_error(
                &format!("默认值 {} 不适用于类型 {}", value, type_name),
                span.unwrap_or_else(Span::call_site)
            ))
        }
    }
}

/// JSON 类型验证器 - 遵循单一职责原则
pub struct JsonValidator;

impl DefaultValueValidator for JsonValidator {
    fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
        // 只有 serde_json::Value 类型才能使用 JSON 默认值
        if !self.supports_type(field_type) {
            return Err(MacroError::json_value_type_required(
                &extract_type_name(field_type),
                default_value.span.unwrap_or_else(Span::call_site)
            ));
        }
        
        match &default_value.value_type {
            DefaultValueType::Json(_) => Ok(()),
            _ => Err(MacroError::invalid_json_default_value(
                "JSON 默认值必须是有效的 JSON 格式",
                &default_value.raw_value,
                default_value.span.unwrap_or_else(Span::call_site)
            ))
        }
    }
    
    fn supports_type(&self, field_type: &Type) -> bool {
        const JSON_TYPES: &[&str] = &[
            "serde_json::Value",
            "Value", // 假设已 use serde_json::Value
            "JsonValue"
        ];
        
        let type_name = extract_type_name(field_type);
        JSON_TYPES.contains(&type_name.as_str())
    }
    
    fn priority(&self) -> i32 { 70 }
    
    fn name(&self) -> &'static str { "JsonValidator" }
}

/// Option 类型验证器 - 遵循单一职责原则
pub struct OptionValidator;

impl DefaultValueValidator for OptionValidator {
    fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
        if !self.supports_type(field_type) {
            return Err(MacroError::validation_error(
                "OptionValidator 只支持 Option<T> 类型",
                default_value.span.unwrap_or_else(Span::call_site)
            ));
        }
        
        // 提取 Option 的内部类型
        if let Some(inner_type) = extract_option_inner_type(field_type) {
            // 如果默认值是 "null"，直接通过验证
            if matches!(&default_value.value_type, DefaultValueType::Null) {
                return Ok(());
            }
            
            // 否则，验证默认值与内部类型的兼容性
            let registry = ValidatorRegistry::new();
            registry.validate(default_value, &inner_type)
        } else {
            Err(MacroError::validation_error(
                "无法提取 Option 的内部类型",
                default_value.span.unwrap_or_else(Span::call_site)
            ))
        }
    }
    
    fn supports_type(&self, field_type: &Type) -> bool {
        is_option_type(field_type)
    }
    
    fn priority(&self) -> i32 { 80 }
    
    fn name(&self) -> &'static str { "OptionValidator" }
}
```

### 2.3 代码生成器设计

#### 2.3.1 增强现有生成器

```rust
// crates/derive/src/generator/node_generator.rs 扩展
// 职责：Node 代码生成增强

use crate::parser::default_value::{DefaultValue, DefaultValueType};

impl NodeGenerator {
    /// 生成增强的 to_node 方法 - 遵循里氏替换原则
    /// 
    /// 生成的方法完全兼容现有接口，但支持默认值处理
    pub fn generate_to_node_method(&self) -> MacroResult<TokenStream2> {
        let struct_name = &self.input.ident;
        let node_type = self.config.node_type.as_ref()
            .ok_or_else(|| MacroError::missing_attribute("node_type", None))?;
        
        // 分离有默认值和无默认值的字段
        let (fields_with_defaults, fields_without_defaults): (Vec<_>, Vec<_>) = 
            self.config.attr_fields
                .iter()
                .partition(|f| f.default_value.is_some());
        
        // 生成导入语句
        let imports = self.generate_imports();
        
        // 生成 NodeSpec 创建代码
        let spec_code = self.generate_node_spec_creation()?;
        
        // 生成字段设置代码
        let field_setters = self.generate_enhanced_field_setters(
            &fields_with_defaults,
            &fields_without_defaults
        )?;
        
        Ok(quote! {
            /// 将当前实例转换为 ModuForge Node
            /// 
            /// 支持默认值的智能处理：当字段值为空或未设置时，自动使用声明的默认值。
            /// 
            /// # 设计原则体现
            /// - **里氏替换**: 完全兼容现有 to_node 方法接口
            /// - **开闭原则**: 支持默认值而不修改现有逻辑
            pub fn to_node(&self) -> mf_core::node::Node {
                #imports
                #spec_code
                
                // 创建节点实例
                let mut node = mf_core::node::Node::create(#node_type, spec);
                
                // 设置字段属性（增强版：支持默认值）
                #field_setters
                
                node
            }
        })
    }
    
    /// 生成增强的字段设置代码
    /// 
    /// # 设计原则体现
    /// - **单一职责**: 专门负责字段设置代码生成
    /// - **开闭原则**: 通过模式匹配支持新的默认值类型
    fn generate_enhanced_field_setters(
        &self,
        fields_with_defaults: &[&FieldConfig],
        fields_without_defaults: &[&FieldConfig]
    ) -> MacroResult<TokenStream2> {
        let mut setters = Vec::new();
        
        // 处理有默认值的字段
        for field_config in fields_with_defaults {
            let setter = self.generate_field_setter_with_default(field_config)?;
            setters.push(setter);
        }
        
        // 处理无默认值的字段（保持现有逻辑）
        for field_config in fields_without_defaults {
            let setter = self.generate_standard_field_setter(field_config)?;
            setters.push(setter);
        }
        
        Ok(quote! {
            #(#setters)*
        })
    }
    
    /// 生成带默认值的字段设置代码
    fn generate_field_setter_with_default(&self, field_config: &FieldConfig) -> MacroResult<TokenStream2> {
        let field_name = syn::parse_str::<Ident>(&field_config.name)?;
        let attr_name = &field_config.name;
        
        let default_value = field_config.default_value.as_ref()
            .ok_or_else(|| MacroError::validation_error("字段缺少默认值", Span::call_site()))?;
        
        let default_expr = self.generate_default_value_expression(default_value, field_config)?;
        
        if field_config.is_optional {
            // Option 类型字段的处理
            Ok(quote! {
                node.set_attr(#attr_name, match &self.#field_name {
                    Some(value) => serde_json::to_value(value).unwrap(),
                    None => #default_expr,
                });
            })
        } else {
            // 非 Option 类型字段的处理
            let empty_check = self.generate_empty_check(&field_config.type_name, &field_name);
            
            Ok(quote! {
                node.set_attr(#attr_name, 
                    if #empty_check {
                        #default_expr
                    } else {
                        serde_json::to_value(&self.#field_name).unwrap()
                    }
                );
            })
        }
    }
    
    /// 生成默认值表达式
    /// 
    /// # 设计原则体现
    /// - **开闭原则**: 通过模式匹配支持新的默认值类型
    /// - **单一职责**: 专门负责默认值表达式生成
    fn generate_default_value_expression(
        &self,
        default_value: &DefaultValue,
        field_config: &FieldConfig
    ) -> MacroResult<TokenStream2> {
        match &default_value.value_type {
            DefaultValueType::String(s) => {
                Ok(quote! { serde_json::Value::String(#s.to_string()) })
            },
            DefaultValueType::Integer(i) => {
                Ok(quote! { serde_json::Value::Number(serde_json::Number::from(#i)) })
            },
            DefaultValueType::Float(f) => {
                Ok(quote! { 
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(#f).unwrap()
                    ) 
                })
            },
            DefaultValueType::Boolean(b) => {
                Ok(quote! { serde_json::Value::Bool(#b) })
            },
            DefaultValueType::Json(json_value) => {
                let json_str = serde_json::to_string(json_value)
                    .map_err(|e| MacroError::generation_error(&format!("JSON 序列化失败: {}", e)))?;
                Ok(quote! { 
                    serde_json::from_str(#json_str).unwrap()
                })
            },
            DefaultValueType::Null => {
                Ok(quote! { serde_json::Value::Null })
            }
        }
    }
    
    /// 生成空值检查表达式
    fn generate_empty_check(&self, type_name: &str, field_name: &Ident) -> TokenStream2 {
        match type_name {
            "String" => quote! { self.#field_name.is_empty() },
            "i32" | "i64" | "u32" | "u64" | "f32" | "f64" => quote! { self.#field_name == 0 },
            "bool" => quote! { false }, // 布尔值不检查空值，始终使用实际值
            _ => quote! { false }, // 其他类型默认不检查
        }
    }
    
    /// 生成构造函数 - 遵循开闭原则
    /// 
    /// 只有当结构体包含默认值字段时才生成构造函数
    pub fn generate_constructor_methods(&self) -> MacroResult<TokenStream2> {
        let has_defaults = self.config.attr_fields
            .iter()
            .any(|f| f.default_value.is_some());
        
        if !has_defaults {
            // 没有默认值字段时，不生成构造函数
            return Ok(quote! {});
        }
        
        let struct_name = &self.input.ident;
        let new_method = self.generate_new_method()?;
        let with_defaults_method = self.generate_with_defaults_method()?;
        
        Ok(quote! {
            impl #struct_name {
                #new_method
                #with_defaults_method
            }
        })
    }
    
    /// 生成 new() 方法
    fn generate_new_method(&self) -> MacroResult<TokenStream2> {
        let field_initializers = self.generate_field_initializers()?;
        
        Ok(quote! {
            /// 使用所有默认值创建新实例
            /// 
            /// # 设计原则体现
            /// - **里氏替换**: 返回类型与 Default::default() 兼容
            /// - **单一职责**: 专门负责默认值实例创建
            pub fn new() -> Self {
                Self {
                    #field_initializers
                }
            }
        })
    }
    
    /// 生成字段初始化代码
    fn generate_field_initializers(&self) -> MacroResult<TokenStream2> {
        let mut initializers = Vec::new();
        
        for field_config in &self.config.attr_fields {
            let field_name = syn::parse_str::<Ident>(&field_config.name)?;
            
            let initializer = if let Some(default_value) = &field_config.default_value {
                self.generate_field_initializer_with_default(default_value, field_config)?
            } else {
                // 无默认值字段使用 Default::default()
                quote! { Default::default() }
            };
            
            initializers.push(quote! {
                #field_name: #initializer
            });
        }
        
        Ok(quote! {
            #(#initializers),*
        })
    }
    
    /// 生成带默认值的字段初始化器
    fn generate_field_initializer_with_default(
        &self,
        default_value: &DefaultValue,
        field_config: &FieldConfig
    ) -> MacroResult<TokenStream2> {
        match &default_value.value_type {
            DefaultValueType::String(s) => Ok(quote! { #s.to_string() }),
            DefaultValueType::Integer(i) => {
                // 根据字段类型生成适当的转换
                let type_name = &field_config.type_name;
                match type_name.as_str() {
                    "i8" => Ok(quote! { #i as i8 }),
                    "i16" => Ok(quote! { #i as i16 }),
                    "i32" => Ok(quote! { #i as i32 }),
                    "i64" => Ok(quote! { #i }),
                    "u8" => Ok(quote! { #i as u8 }),
                    "u16" => Ok(quote! { #i as u16 }),
                    "u32" => Ok(quote! { #i as u32 }),
                    "u64" => Ok(quote! { #i as u64 }),
                    _ => Ok(quote! { #i }),
                }
            },
            DefaultValueType::Float(f) => Ok(quote! { #f }),
            DefaultValueType::Boolean(b) => Ok(quote! { #b }),
            DefaultValueType::Json(json_value) => {
                let json_str = serde_json::to_string(json_value)
                    .map_err(|e| MacroError::generation_error(&format!("JSON 序列化失败: {}", e)))?;
                Ok(quote! { serde_json::from_str(#json_str).unwrap() })
            },
            DefaultValueType::Null => {
                if field_config.is_optional {
                    Ok(quote! { None })
                } else {
                    Err(MacroError::validation_error(
                        "null 默认值只能用于 Option 类型字段",
                        default_value.span.unwrap_or_else(Span::call_site)
                    ))
                }
            }
        }
    }
    
    /// 生成 with_defaults() 方法（构建器模式）
    fn generate_with_defaults_method(&self) -> MacroResult<TokenStream2> {
        // 这里可以实现更复杂的构建器模式
        // 暂时返回空实现，作为未来扩展点
        Ok(quote! {
            /// 创建带有默认值的构建器
            /// 
            /// # 设计原则体现
            /// - **开闭原则**: 预留扩展点支持构建器模式
            pub fn with_defaults() -> Self {
                Self::new()
            }
        })
    }
}
```

### 2.4 错误处理系统设计

#### 2.4.1 错误类型扩展

```rust
// crates/derive/src/common/error.rs 扩展
// 职责：错误类型和友好错误消息

impl MacroError {
    // === 新增默认值相关错误构造方法 ===
    
    /// 创建默认值类型不匹配错误
    /// 
    /// # 设计原则体现
    /// - **单一职责**: 专门处理类型不匹配错误
    /// - **接口隔离**: 提供专门的错误构造接口
    pub fn default_value_type_mismatch(
        field_type: &str,
        actual_type: &str,
        expected_type: &str,
        span: Span,
    ) -> Self {
        Self::DefaultValueTypeMismatch {
            field_type: field_type.to_string(),
            actual_type: actual_type.to_string(),
            expected_type: expected_type.to_string(),
            span: Some(span),
        }
    }
    
    /// 创建 JSON 默认值格式错误
    pub fn invalid_json_default_value(
        reason: &str,
        value: &str,
        span: Span,
    ) -> Self {
        Self::InvalidJsonDefaultValue {
            reason: reason.to_string(),
            value: value.to_string(),
            span: Some(span),
        }
    }
    
    /// 创建 JSON 类型约束错误
    pub fn json_value_type_required(
        actual_type: &str,
        span: Span,
    ) -> Self {
        Self::JsonValueTypeRequired {
            actual_type: actual_type.to_string(),
            span: Some(span),
        }
    }
    
    /// 生成友好的错误消息和修复建议
    /// 
    /// # 设计原则体现
    /// - **单一职责**: 专门负责错误消息格式化
    /// - **开闭原则**: 支持新的错误类型扩展
    pub fn to_friendly_message(&self) -> String {
        match self {
            Self::DefaultValueTypeMismatch { 
                field_type, 
                actual_type, 
                expected_type, 
                .. 
            } => {
                format!(
                    "默认值类型不匹配\n\n\
                    字段类型: {}\n\
                    实际默认值类型: {}\n\
                    期望类型: {}\n\n\
                    修复建议:\n\
                    - 检查默认值格式是否正确\n\
                    - 确保默认值与字段类型匹配\n\
                    - 参考文档了解支持的默认值格式\n\n\
                    示例:\n\
                    #[attr(default = \"正确的默认值\")]",
                    field_type, actual_type, expected_type
                )
            },
            
            Self::InvalidJsonDefaultValue { reason, value, .. } => {
                format!(
                    "JSON 默认值格式错误\n\n\
                    错误原因: {}\n\
                    问题值: {}\n\n\
                    修复建议:\n\
                    - 检查 JSON 语法是否正确\n\
                    - 确保所有字符串都用双引号包围\n\
                    - 验证 JSON 格式的有效性\n\n\
                    示例:\n\
                    #[attr(default = r#\"{{\"key\": \"value\"}}\"#)]",
                    reason, value
                )
            },
            
            Self::JsonValueTypeRequired { actual_type, .. } => {
                format!(
                    "JSON 默认值类型约束错误\n\n\
                    当前字段类型: {}\n\n\
                    说明:\n\
                    JSON 格式的默认值只能用于 serde_json::Value 类型的字段。\n\n\
                    解决方案:\n\
                    1. 将字段类型改为 serde_json::Value\n\
                    2. 或者使用简单字符串作为默认值\n\n\
                    示例:\n\
                    // 正确用法\n\
                    #[attr(default = r#\"{{\"config\": true}}\"#)]\n\
                    config: serde_json::Value,\n\n\
                    // 或者\n\
                    #[attr(default = \"simple_value\")]\n\
                    config: String,",
                    actual_type
                )
            },
            
            _ => {
                // 委托给现有的错误处理逻辑
                format!("{}", self)
            }
        }
    }
}

/// 新增的默认值相关错误类型
#[derive(Error, Debug)]
pub enum MacroError {
    // === 现有错误类型保持不变 ===
    // ...
    
    // === 新增默认值相关错误 ===
    
    /// 默认值类型不匹配错误
    #[error("默认值类型不匹配: 字段类型 '{field_type}'，实际类型 '{actual_type}'，期望类型 '{expected_type}'")]
    DefaultValueTypeMismatch {
        field_type: String,
        actual_type: String,
        expected_type: String,
        span: Option<Span>,
    },
    
    /// JSON 默认值格式错误
    #[error("JSON 默认值格式错误: {reason}")]
    InvalidJsonDefaultValue {
        reason: String,
        value: String,
        span: Option<Span>,
    },
    
    /// JSON 类型约束错误
    #[error("JSON 默认值只能用于 serde_json::Value 类型字段，当前类型: {actual_type}")]
    JsonValueTypeRequired {
        actual_type: String,
        span: Option<Span>,
    },
    
    /// 默认值解析错误
    #[error("默认值解析失败: {reason}")]
    DefaultValueParseError {
        reason: String,
        value: String,
        span: Option<Span>,
    },
}
```

## 3. 数据模型设计

### 3.1 核心数据结构

#### 3.1.1 FieldConfig 扩展设计

```rust
// crates/derive/src/parser/attribute_parser.rs 扩展
// 职责：字段配置的结构化表示

/// 字段配置 - 遵循开闭原则的扩展设计
#[derive(Debug, Clone)]
pub struct FieldConfig {
    // === 现有字段保持完全不变 ===
    /// 字段名称
    pub name: String,
    /// 字段类型名称
    pub type_name: String,
    /// 是否为可选类型 (Option<T>)
    pub is_optional: bool,
    /// 是否标记为属性字段
    pub is_attr: bool,
    /// 原始字段定义
    pub field: Field,
    
    // === 新增字段（保持向后兼容）===
    /// 默认值配置（None 表示无默认值，保持现有行为）
    /// 
    /// # 设计原则体现
    /// - **开闭原则**: 通过 Option 类型实现无破坏性扩展
    /// - **里氏替换**: 现有代码可以忽略此字段继续工作
    pub default_value: Option<DefaultValue>,
}

impl FieldConfig {
    /// 创建新的字段配置（保持现有构造函数不变）
    /// 
    /// # 设计原则体现
    /// - **里氏替换**: 与现有构造函数完全兼容
    pub fn new(name: String, type_name: String, is_optional: bool, is_attr: bool, field: Field) -> Self {
        Self {
            name,
            type_name,
            is_optional,
            is_attr,
            field,
            default_value: None, // 默认无默认值，保持现有行为
        }
    }
    
    /// 设置默认值（新增方法，支持链式调用）
    /// 
    /// # 设计原则体现
    /// - **开闭原则**: 通过新增方法扩展功能
    /// - **单一职责**: 专门负责默认值设置
    pub fn with_default_value(mut self, default_value: DefaultValue) -> Self {
        self.default_value = Some(default_value);
        self
    }
    
    /// 检查字段是否有默认值
    pub fn has_default_value(&self) -> bool {
        self.default_value.is_some()
    }
    
    /// 获取默认值引用
    pub fn get_default_value(&self) -> Option<&DefaultValue> {
        self.default_value.as_ref()
    }
}
```

### 3.2 类型系统设计

#### 3.2.1 类型分析器

```rust
// crates/derive/src/common/utils.rs 扩展
// 职责：类型分析和识别

/// 类型分析器 - 遵循单一职责原则
pub struct TypeAnalyzer;

impl TypeAnalyzer {
    /// 提取类型名称
    /// 
    /// # 设计原则体现
    /// - **单一职责**: 专门负责类型名称提取
    /// - **开闭原则**: 支持新的类型模式扩展
    pub fn extract_type_name(ty: &Type) -> String {
        match ty {
            Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    segment.ident.to_string()
                } else {
                    "Unknown".to_string()
                }
            },
            Type::Reference(type_ref) => {
                format!("&{}", Self::extract_type_name(&type_ref.elem))
            },
            _ => "Unknown".to_string(),
        }
    }
    
    /// 检查是否为 Option 类型
    pub fn is_option_type(ty: &Type) -> bool {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == "Option"
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// 提取 Option 的内部类型
    /// 
    /// # 设计原则体现
    /// - **单一职责**: 专门处理 Option 类型解析
    pub fn extract_option_inner_type(ty: &Type) -> Option<Type> {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Option" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_type)) = args.args.first() {
                            return Some(inner_type.clone());
                        }
                    }
                }
            }
        }
        None
    }
    
    /// 检查类型是否为基本数值类型
    pub fn is_numeric_type(type_name: &str) -> bool {
        matches!(type_name, 
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
            "f32" | "f64"
        )
    }
    
    /// 检查类型是否为字符串类型
    pub fn is_string_type(type_name: &str) -> bool {
        matches!(type_name, "String" | "str" | "&str")
    }
    
    /// 检查类型是否为 JSON 值类型
    pub fn is_json_value_type(type_name: &str) -> bool {
        matches!(type_name,
            "serde_json::Value" | "Value" | "JsonValue"
        )
    }
}
```

## 4. 性能优化设计

### 4.1 编译时性能优化

#### 4.1.1 缓存策略

```rust
// crates/derive/src/common/utils.rs 扩展
// 职责：性能优化和缓存管理

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// 类型信息缓存 - 遵循单一职责原则
/// 
/// 缓存常用类型的解析结果，避免重复计算
static TYPE_INFO_CACHE: Lazy<HashMap<String, TypeInfo>> = Lazy::new(|| {
    let mut cache = HashMap::new();
    
    // 预缓存常用类型信息
    cache.insert("String".to_string(), TypeInfo::string_type());
    cache.insert("i32".to_string(), TypeInfo::i32_type());
    cache.insert("i64".to_string(), TypeInfo::i64_type());
    cache.insert("f64".to_string(), TypeInfo::f64_type());
    cache.insert("bool".to_string(), TypeInfo::bool_type());
    cache.insert("serde_json::Value".to_string(), TypeInfo::json_value_type());
    
    cache
});

/// 类型信息结构体
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub is_numeric: bool,
    pub is_string: bool,
    pub is_json_value: bool,
    pub is_option: bool,
    pub inner_type: Option<String>,
}

impl TypeInfo {
    /// 创建字符串类型信息
    pub fn string_type() -> Self {
        Self {
            name: "String".to_string(),
            is_numeric: false,
            is_string: true,
            is_json_value: false,
            is_option: false,
            inner_type: None,
        }
    }
    
    /// 创建数值类型信息
    pub fn numeric_type(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_numeric: true,
            is_string: false,
            is_json_value: false,
            is_option: false,
            inner_type: None,
        }
    }
    
    /// 快速类型信息查找
    pub fn get_or_analyze(ty: &Type) -> TypeInfo {
        let type_name = TypeAnalyzer::extract_type_name(ty);
        
        if let Some(cached) = TYPE_INFO_CACHE.get(&type_name) {
            cached.clone()
        } else {
            // 实时分析未缓存的类型
            Self::analyze_type(ty)
        }
    }
    
    /// 分析类型信息
    fn analyze_type(ty: &Type) -> TypeInfo {
        let type_name = TypeAnalyzer::extract_type_name(ty);
        
        TypeInfo {
            name: type_name.clone(),
            is_numeric: TypeAnalyzer::is_numeric_type(&type_name),
            is_string: TypeAnalyzer::is_string_type(&type_name),
            is_json_value: TypeAnalyzer::is_json_value_type(&type_name),
            is_option: TypeAnalyzer::is_option_type(ty),
            inner_type: TypeAnalyzer::extract_option_inner_type(ty)
                .map(|t| TypeAnalyzer::extract_type_name(&t)),
        }
    }
}
```

#### 4.1.2 验证器优化

```rust
// crates/derive/src/parser/validation.rs 扩展
// 职责：高性能验证逻辑

/// 高性能验证管道 - 遵循单一职责原则
pub struct OptimizedValidationPipeline {
    validators: Vec<Box<dyn DefaultValueValidator>>,
    type_validator_map: HashMap<String, usize>, // 类型到验证器的快速映射
}

impl OptimizedValidationPipeline {
    /// 创建优化的验证管道
    /// 
    /// # 设计原则体现
    /// - **单一职责**: 专门负责验证流程优化
    /// - **依赖倒置**: 依赖验证器抽象接口
    pub fn new() -> Self {
        let validators: Vec<Box<dyn DefaultValueValidator>> = vec![
            Box::new(StringValidator),
            Box::new(NumericValidator),
            Box::new(BooleanValidator),
            Box::new(JsonValidator),
            Box::new(OptionValidator),
        ];
        
        // 构建类型到验证器的快速映射
        let mut type_validator_map = HashMap::new();
        for (index, validator) in validators.iter().enumerate() {
            // 这里可以预计算每个验证器支持的类型
            // 为简化示例，使用运行时检查
        }
        
        Self {
            validators,
            type_validator_map,
        }
    }
    
    /// 快速验证默认值
    /// 
    /// 使用缓存和预排序提高验证性能
    pub fn validate_fast(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
        let type_info = TypeInfo::get_or_analyze(field_type);
        
        // 快速路径：使用类型信息直接选择验证器
        for validator in &self.validators {
            if self.validator_supports_type_fast(validator.as_ref(), &type_info) {
                return validator.validate(default_value, field_type);
            }
        }
        
        Err(MacroError::unsupported_field_type(
            &type_info.name,
            field_type.span()
        ))
    }
    
    /// 快速类型支持检查
    fn validator_supports_type_fast(&self, validator: &dyn DefaultValueValidator, type_info: &TypeInfo) -> bool {
        // 基于缓存的类型信息进行快速检查
        match validator.name() {
            "StringValidator" => type_info.is_string,
            "NumericValidator" => type_info.is_numeric,
            "BooleanValidator" => type_info.name == "bool",
            "JsonValidator" => type_info.is_json_value,
            "OptionValidator" => type_info.is_option,
            _ => false, // 兜底：使用原始方法
        }
    }
}
```

## 5. 扩展性设计

### 5.1 插件化架构

#### 5.1.1 验证器插件系统

```rust
// crates/derive/src/parser/validation.rs 扩展
// 职责：可扩展的验证器系统

/// 验证器插件注册表 - 遵循开闭原则
pub struct ValidatorPluginRegistry {
    validators: Vec<Box<dyn DefaultValueValidator>>,
    type_mappings: HashMap<String, Vec<usize>>, // 类型到验证器索引的映射
}

impl ValidatorPluginRegistry {
    /// 创建新的插件注册表
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
            type_mappings: HashMap::new(),
        }
    }
    
    /// 注册验证器插件
    /// 
    /// # 设计原则体现
    /// - **开闭原则**: 支持新验证器的动态注册
    /// - **依赖倒置**: 依赖抽象接口而非具体实现
    pub fn register_validator<V: DefaultValueValidator + 'static>(&mut self, validator: V) {
        let index = self.validators.len();
        let validator_box = Box::new(validator);
        
        // 预分析该验证器支持的类型（优化性能）
        self.analyze_and_cache_supported_types(&*validator_box, index);
        
        self.validators.push(validator_box);
        
        // 按优先级重新排序
        self.sort_validators_by_priority();
    }
    
    /// 批量注册内置验证器
    pub fn register_builtin_validators(&mut self) {
        self.register_validator(StringValidator);
        self.register_validator(NumericValidator);
        self.register_validator(BooleanValidator);
        self.register_validator(JsonValidator);
        self.register_validator(OptionValidator);
    }
    
    /// 分析并缓存验证器支持的类型
    fn analyze_and_cache_supported_types(&mut self, validator: &dyn DefaultValueValidator, index: usize) {
        // 这里可以预分析常用类型
        let common_types = vec![
            "String", "i32", "i64", "f64", "bool", "serde_json::Value",
            "Option<String>", "Option<i32>", "Option<bool>"
        ];
        
        for type_name in common_types {
            if let Ok(parsed_type) = syn::parse_str::<Type>(type_name) {
                if validator.supports_type(&parsed_type) {
                    self.type_mappings
                        .entry(type_name.to_string())
                        .or_insert_with(Vec::new)
                        .push(index);
                }
            }
        }
    }
    
    /// 按优先级排序验证器
    fn sort_validators_by_priority(&mut self) {
        self.validators.sort_by_key(|v| -v.priority());
        
        // 重新构建类型映射（因为索引可能变化）
        self.rebuild_type_mappings();
    }
    
    /// 重新构建类型映射
    fn rebuild_type_mappings(&mut self) {
        self.type_mappings.clear();
        
        for (index, validator) in self.validators.iter().enumerate() {
            self.analyze_and_cache_supported_types(validator.as_ref(), index);
        }
    }
}

/// 自定义验证器示例
/// 
/// 展示如何实现新的验证器
pub struct CustomDateValidator;

impl DefaultValueValidator for CustomDateValidator {
    fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
        // 自定义日期验证逻辑
        if !self.supports_type(field_type) {
            return Err(MacroError::validation_error(
                "CustomDateValidator 不支持此类型",
                default_value.span.unwrap_or_else(Span::call_site)
            ));
        }
        
        // 验证日期格式
        match &default_value.value_type {
            DefaultValueType::String(date_str) => {
                if self.is_valid_date_format(date_str) {
                    Ok(())
                } else {
                    Err(MacroError::validation_error(
                        &format!("无效的日期格式: {}", date_str),
                        default_value.span.unwrap_or_else(Span::call_site)
                    ))
                }
            },
            _ => Err(MacroError::validation_error(
                "日期验证器只支持字符串默认值",
                default_value.span.unwrap_or_else(Span::call_site)
            ))
        }
    }
    
    fn supports_type(&self, field_type: &Type) -> bool {
        let type_name = TypeAnalyzer::extract_type_name(field_type);
        matches!(type_name.as_str(), "DateTime" | "NaiveDate" | "chrono::DateTime")
    }
    
    fn priority(&self) -> i32 { 60 }
    
    fn name(&self) -> &'static str { "CustomDateValidator" }
}

impl CustomDateValidator {
    /// 验证日期格式
    fn is_valid_date_format(&self, date_str: &str) -> bool {
        // 简单的日期格式验证示例
        // 实际实现可能需要更复杂的逻辑
        date_str.len() == 10 && date_str.chars().nth(4) == Some('-') && date_str.chars().nth(7) == Some('-')
    }
}
```

#### 5.1.2 代码生成模板系统

```rust
// crates/derive/src/generator/templates.rs 🆕
// 职责：可扩展的代码生成模板系统

/// 代码生成模板接口 - 遵循接口隔离原则
pub trait CodeGenerationTemplate {
    /// 生成代码
    fn generate(&self, context: &GenerationContext) -> MacroResult<TokenStream2>;
    
    /// 检查是否支持指定的模式
    fn supports_pattern(&self, pattern: &str) -> bool;
    
    /// 模板名称
    fn name(&self) -> &'static str;
    
    /// 模板优先级
    fn priority(&self) -> i32;
}

/// 代码生成上下文
#[derive(Debug)]
pub struct GenerationContext {
    pub field_config: FieldConfig,
    pub default_value: DefaultValue,
    pub struct_name: Ident,
    pub generation_mode: GenerationMode,
}

/// 代码生成模式
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationMode {
    ToNodeMethod,
    ConstructorMethod,
    FieldInitializer,
    DefaultValueExpression,
}

/// 模板注册表 - 遵循开闭原则
pub struct TemplateRegistry {
    templates: Vec<Box<dyn CodeGenerationTemplate>>,
    pattern_mappings: HashMap<String, Vec<usize>>,
}

impl TemplateRegistry {
    /// 创建新的模板注册表
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
            pattern_mappings: HashMap::new(),
        }
    }
    
    /// 注册代码生成模板
    /// 
    /// # 设计原则体现
    /// - **开闭原则**: 支持新模板的动态添加
    /// - **单一职责**: 专门负责模板管理
    pub fn register_template<T: CodeGenerationTemplate + 'static>(&mut self, template: T) {
        let index = self.templates.len();
        let template_box = Box::new(template);
        
        // 预分析模板支持的模式
        self.analyze_template_patterns(&*template_box, index);
        
        self.templates.push(template_box);
        
        // 按优先级排序
        self.sort_templates_by_priority();
    }
    
    /// 选择合适的模板
    pub fn select_template(&self, pattern: &str) -> Option<&dyn CodeGenerationTemplate> {
        if let Some(indices) = self.pattern_mappings.get(pattern) {
            if let Some(&index) = indices.first() {
                return Some(self.templates[index].as_ref());
            }
        }
        
        // 兜底：遍历所有模板
        for template in &self.templates {
            if template.supports_pattern(pattern) {
                return Some(template.as_ref());
            }
        }
        
        None
    }
    
    /// 分析模板模式
    fn analyze_template_patterns(&mut self, template: &dyn CodeGenerationTemplate, index: usize) {
        // 预定义的常见模式
        let common_patterns = vec![
            "String", "i32", "i64", "f64", "bool",
            "Option<String>", "Option<i32>",
            "serde_json::Value", "JSON"
        ];
        
        for pattern in common_patterns {
            if template.supports_pattern(pattern) {
                self.pattern_mappings
                    .entry(pattern.to_string())
                    .or_insert_with(Vec::new)
                    .push(index);
            }
        }
    }
    
    /// 按优先级排序模板
    fn sort_templates_by_priority(&mut self) {
        self.templates.sort_by_key(|t| -t.priority());
        self.rebuild_pattern_mappings();
    }
    
    /// 重新构建模式映射
    fn rebuild_pattern_mappings(&mut self) {
        self.pattern_mappings.clear();
        for (index, template) in self.templates.iter().enumerate() {
            self.analyze_template_patterns(template.as_ref(), index);
        }
    }
}

/// 简单默认值模板
pub struct SimpleDefaultTemplate;

impl CodeGenerationTemplate for SimpleDefaultTemplate {
    fn generate(&self, context: &GenerationContext) -> MacroResult<TokenStream2> {
        match context.generation_mode {
            GenerationMode::DefaultValueExpression => {
                self.generate_simple_default_expression(context)
            },
            GenerationMode::FieldInitializer => {
                self.generate_field_initializer(context)
            },
            _ => Err(MacroError::generation_error("不支持的生成模式"))
        }
    }
    
    fn supports_pattern(&self, pattern: &str) -> bool {
        matches!(pattern, "String" | "i32" | "i64" | "f64" | "bool")
    }
    
    fn name(&self) -> &'static str { "SimpleDefaultTemplate" }
    
    fn priority(&self) -> i32 { 100 }
}

impl SimpleDefaultTemplate {
    /// 生成简单默认值表达式
    fn generate_simple_default_expression(&self, context: &GenerationContext) -> MacroResult<TokenStream2> {
        match &context.default_value.value_type {
            DefaultValueType::String(s) => Ok(quote! { serde_json::Value::String(#s.to_string()) }),
            DefaultValueType::Integer(i) => Ok(quote! { serde_json::Value::Number(serde_json::Number::from(#i)) }),
            DefaultValueType::Float(f) => Ok(quote! { serde_json::Value::Number(serde_json::Number::from_f64(#f).unwrap()) }),
            DefaultValueType::Boolean(b) => Ok(quote! { serde_json::Value::Bool(#b) }),
            _ => Err(MacroError::generation_error("SimpleDefaultTemplate 不支持此默认值类型"))
        }
    }
    
    /// 生成字段初始化器
    fn generate_field_initializer(&self, context: &GenerationContext) -> MacroResult<TokenStream2> {
        match &context.default_value.value_type {
            DefaultValueType::String(s) => Ok(quote! { #s.to_string() }),
            DefaultValueType::Integer(i) => Ok(quote! { #i }),
            DefaultValueType::Float(f) => Ok(quote! { #f }),
            DefaultValueType::Boolean(b) => Ok(quote! { #b }),
            _ => Err(MacroError::generation_error("SimpleDefaultTemplate 不支持此默认值类型"))
        }
    }
}

/// JSON 默认值模板
pub struct JsonDefaultTemplate;

impl CodeGenerationTemplate for JsonDefaultTemplate {
    fn generate(&self, context: &GenerationContext) -> MacroResult<TokenStream2> {
        match &context.default_value.value_type {
            DefaultValueType::Json(json_value) => {
                let json_str = serde_json::to_string(json_value)
                    .map_err(|e| MacroError::generation_error(&format!("JSON 序列化失败: {}", e)))?;
                
                match context.generation_mode {
                    GenerationMode::DefaultValueExpression => {
                        Ok(quote! { serde_json::from_str(#json_str).unwrap() })
                    },
                    GenerationMode::FieldInitializer => {
                        Ok(quote! { serde_json::from_str(#json_str).unwrap() })
                    },
                    _ => Err(MacroError::generation_error("不支持的生成模式"))
                }
            },
            _ => Err(MacroError::generation_error("JsonDefaultTemplate 只支持 JSON 默认值"))
        }
    }
    
    fn supports_pattern(&self, pattern: &str) -> bool {
        matches!(pattern, "serde_json::Value" | "Value" | "JSON")
    }
    
    fn name(&self) -> &'static str { "JsonDefaultTemplate" }
    
    fn priority(&self) -> i32 { 90 }
}
```

## 6. 测试策略设计

### 6.1 分层测试架构

#### 6.1.1 单元测试设计

```rust
// crates/derive/tests/default_value_tests.rs 🆕
// 职责：默认值功能的全面测试覆盖

#[cfg(test)]
mod default_value_parsing_tests {
    use super::*;
    use crate::parser::default_value::*;
    
    /// 测试基本类型默认值解析
    #[test]
    fn test_parse_string_default() {
        let result = DefaultValueParser::parse("hello world", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert_eq!(default_value.raw_value, "hello world");
        assert!(matches!(default_value.value_type, DefaultValueType::String(_)));
        assert!(!default_value.is_json);
    }
    
    #[test]
    fn test_parse_integer_default() {
        let result = DefaultValueParser::parse("42", None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(matches!(default_value.value_type, DefaultValueType::Integer(42)));
    }
    
    #[test]
    fn test_parse_json_default() {
        let json_str = r#"{"key": "value", "number": 123}"#;
        let result = DefaultValueParser::parse(json_str, None);
        assert!(result.is_ok());
        
        let default_value = result.unwrap();
        assert!(default_value.is_json);
        assert!(matches!(default_value.value_type, DefaultValueType::Json(_)));
    }
    
    #[test]
    fn test_parse_invalid_json() {
        let invalid_json = r#"{"invalid": json"#;
        let result = DefaultValueParser::parse(invalid_json, None);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    /// 测试字符串验证器
    #[test]
    fn test_string_validator() {
        let validator = StringValidator;
        let field_type: Type = syn::parse_str("String").unwrap();
        let default_value = DefaultValue {
            raw_value: "test".to_string(),
            value_type: DefaultValueType::String("test".to_string()),
            is_json: false,
            span: None,
        };
        
        let result = validator.validate(&default_value, &field_type);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_string_validator_type_mismatch() {
        let validator = StringValidator;
        let field_type: Type = syn::parse_str("i32").unwrap();
        let default_value = DefaultValue {
            raw_value: "test".to_string(),
            value_type: DefaultValueType::String("test".to_string()),
            is_json: false,
            span: None,
        };
        
        let result = validator.validate(&default_value, &field_type);
        assert!(result.is_err());
    }
    
    /// 测试数值验证器范围检查
    #[test]
    fn test_numeric_validator_range() {
        let validator = NumericValidator;
        let field_type: Type = syn::parse_str("i8").unwrap();
        
        // 有效范围
        let valid_value = DefaultValue {
            raw_value: "100".to_string(),
            value_type: DefaultValueType::Integer(100),
            is_json: false,
            span: None,
        };
        assert!(validator.validate(&valid_value, &field_type).is_ok());
        
        // 超出范围
        let invalid_value = DefaultValue {
            raw_value: "1000".to_string(),
            value_type: DefaultValueType::Integer(1000), // 超出 i8 范围
            is_json: false,
            span: None,
        };
        assert!(validator.validate(&invalid_value, &field_type).is_err());
    }
    
    /// 测试 JSON 验证器类型约束
    #[test]
    fn test_json_validator_type_constraint() {
        let validator = JsonValidator;
        
        // 正确类型
        let json_field: Type = syn::parse_str("serde_json::Value").unwrap();
        let json_value = DefaultValue {
            raw_value: r#"{"key": "value"}"#.to_string(),
            value_type: DefaultValueType::Json(serde_json::json!({"key": "value"})),
            is_json: true,
            span: None,
        };
        assert!(validator.validate(&json_value, &json_field).is_ok());
        
        // 错误类型
        let string_field: Type = syn::parse_str("String").unwrap();
        assert!(validator.validate(&json_value, &string_field).is_err());
    }
    
    /// 测试 Option 验证器
    #[test]
    fn test_option_validator() {
        let validator = OptionValidator;
        let option_string_type: Type = syn::parse_str("Option<String>").unwrap();
        
        // null 默认值
        let null_value = DefaultValue {
            raw_value: "null".to_string(),
            value_type: DefaultValueType::Null,
            is_json: false,
            span: None,
        };
        assert!(validator.validate(&null_value, &option_string_type).is_ok());
        
        // 内部类型默认值
        let string_value = DefaultValue {
            raw_value: "test".to_string(),
            value_type: DefaultValueType::String("test".to_string()),
            is_json: false,
            span: None,
        };
        assert!(validator.validate(&string_value, &option_string_type).is_ok());
    }
}

#[cfg(test)]
mod code_generation_tests {
    use super::*;
    
    /// 测试简单默认值模板
    #[test]
    fn test_simple_default_template() {
        let template = SimpleDefaultTemplate;
        let context = GenerationContext {
            field_config: FieldConfig::new(
                "test_field".to_string(),
                "String".to_string(),
                false,
                true,
                syn::parse_str("test_field: String").unwrap()
            ),
            default_value: DefaultValue {
                raw_value: "test".to_string(),
                value_type: DefaultValueType::String("test".to_string()),
                is_json: false,
                span: None,
            },
            struct_name: syn::parse_str("TestStruct").unwrap(),
            generation_mode: GenerationMode::DefaultValueExpression,
        };
        
        let result = template.generate(&context);
        assert!(result.is_ok());
        
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("serde_json::Value::String"));
        assert!(code.contains("test"));
    }
    
    /// 测试 JSON 默认值模板
    #[test]
    fn test_json_default_template() {
        let template = JsonDefaultTemplate;
        let context = GenerationContext {
            field_config: FieldConfig::new(
                "config".to_string(),
                "serde_json::Value".to_string(),
                false,
                true,
                syn::parse_str("config: serde_json::Value").unwrap()
            ),
            default_value: DefaultValue {
                raw_value: r#"{"key": "value"}"#.to_string(),
                value_type: DefaultValueType::Json(serde_json::json!({"key": "value"})),
                is_json: true,
                span: None,
            },
            struct_name: syn::parse_str("ConfigStruct").unwrap(),
            generation_mode: GenerationMode::DefaultValueExpression,
        };
        
        let result = template.generate(&context);
        assert!(result.is_ok());
        
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("serde_json::from_str"));
    }
}
```

#### 6.1.2 集成测试设计

```rust
// crates/derive/tests/integration_tests.rs 扩展
// 职责：端到端集成测试

#[cfg(test)]
mod default_value_integration_tests {
    use syn::parse_quote;
    use crate::*;
    
    /// 测试完整的 Node 派生与默认值
    #[test]
    fn test_complete_node_with_defaults() {
        let input = parse_quote! {
            #[derive(Node)]
            #[node_type = "test_paragraph"]
            pub struct TestParagraph {
                #[attr(default = "默认内容")]
                content: String,
                
                #[attr(default = "16")]
                font_size: i32,
                
                #[attr(default = "true")]
                visible: bool,
                
                #[attr(default = r#"{"theme": "light"}"#)]
                config: serde_json::Value,
                
                #[attr]
                author: Option<String>,
            }
        };
        
        // 执行完整的代码生成流程
        let result = process_derive_node_with_recovery(input);
        assert!(result.is_ok());
        
        let generated = result.unwrap();
        let code = generated.to_string();
        
        // 验证生成的方法存在
        assert!(code.contains("pub fn to_node"));
        assert!(code.contains("pub fn new"));
        
        // 验证默认值被正确使用
        assert!(code.contains("默认内容"));
        assert!(code.contains("16"));
        assert!(code.contains("true"));
        assert!(code.contains("theme"));
        assert!(code.contains("light"));
    }
    
    /// 测试 Mark 派生与默认值
    #[test]
    fn test_complete_mark_with_defaults() {
        let input = parse_quote! {
            #[derive(Mark)]
            #[mark_type = "emphasis"]
            pub struct EmphasisMark {
                #[attr(default = "normal")]
                style: String,
                
                #[attr(default = "1.0")]
                weight: f64,
                
                #[attr(default = "false")]
                italic: bool,
            }
        };
        
        let result = process_derive_mark_with_recovery(input);
        assert!(result.is_ok());
        
        let generated = result.unwrap();
        let code = generated.to_string();
        
        // 验证 Mark 特定的方法
        assert!(code.contains("pub fn to_mark"));
        assert!(code.contains("pub fn new"));
        
        // 验证默认值
        assert!(code.contains("normal"));
        assert!(code.contains("1.0"));
        assert!(code.contains("false"));
    }
    
    /// 测试混合字段（有些有默认值，有些没有）
    #[test]
    fn test_mixed_default_and_regular_fields() {
        let input = parse_quote! {
            #[derive(Node)]
            #[node_type = "mixed_node"]
            pub struct MixedNode {
                #[attr(default = "default_value")]
                with_default: String,
                
                #[attr]
                without_default: String,
                
                #[attr(default = "null")]
                optional_with_default: Option<String>,
                
                #[attr]
                optional_without_default: Option<i32>,
            }
        };
        
        let result = process_derive_node_with_recovery(input);
        assert!(result.is_ok());
        
        let generated = result.unwrap();
        let code = generated.to_string();
        
        // 验证混合处理逻辑
        assert!(code.contains("default_value"));
        assert!(code.contains("with_default"));
        assert!(code.contains("without_default"));
        assert!(code.contains("optional_with_default"));
        assert!(code.contains("optional_without_default"));
    }
    
    /// 测试向后兼容性
    #[test]
    fn test_backward_compatibility() {
        let existing_input = parse_quote! {
            #[derive(Node)]
            #[node_type = "legacy_node"]
            pub struct LegacyNode {
                #[attr]
                content: String,
                
                #[attr]
                author: Option<String>,
            }
        };
        
        let result = process_derive_node_with_recovery(existing_input);
        assert!(result.is_ok());
        
        let generated = result.unwrap();
        let code = generated.to_string();
        
        // 验证现有行为保持不变
        assert!(code.contains("pub fn to_node"));
        assert!(!code.contains("pub fn new")); // 无默认值时不生成 new 方法
        assert!(!code.contains("default")); // 不包含默认值相关代码
    }
    
    /// 测试错误情况处理
    #[test]
    fn test_error_handling() {
        // 类型不匹配错误
        let type_mismatch_input = parse_quote! {
            #[derive(Node)]
            #[node_type = "error_node"]
            pub struct ErrorNode {
                #[attr(default = "not_a_number")]
                number_field: i32,
            }
        };
        
        let result = process_derive_node_with_recovery(type_mismatch_input);
        assert!(result.is_err());
        
        // JSON 类型约束错误
        let json_constraint_input = parse_quote! {
            #[derive(Node)]
            #[node_type = "json_error_node"]
            pub struct JsonErrorNode {
                #[attr(default = r#"{"key": "value"}"#)]
                not_json_field: String,
            }
        };
        
        let result = process_derive_node_with_recovery(json_constraint_input);
        assert!(result.is_err());
    }
}
```

## 7. 质量保证设计

### 7.1 代码质量标准

#### 7.1.1 中文注释标准

```rust
//! 模块级文档要求
//!
//! 每个模块都必须包含详细的中文文档，说明：
//! 1. 模块的职责和功能
//! 2. 设计原则的体现方式
//! 3. 与其他模块的关系
//! 4. 使用示例和最佳实践
//!
//! # 设计原则体现
//! 
//! - **单一职责**: 说明本模块如何遵循单一职责原则
//! - **开闭原则**: 说明本模块如何支持扩展而不修改
//! - **里氏替换**: 说明本模块如何保证接口兼容性
//! - **接口隔离**: 说明本模块如何提供最小化接口
//! - **依赖倒置**: 说明本模块如何依赖抽象而非具体实现

/// 函数/类型级文档要求
/// 
/// 每个公共函数和类型都必须包含：
/// 
/// # 功能说明
/// 
/// 详细描述函数或类型的功能和用途
/// 
/// # 参数说明
/// 
/// * `param1` - 参数1的详细说明
/// * `param2` - 参数2的详细说明
/// 
/// # 返回值
/// 
/// 返回值的详细说明，包括可能的错误情况
/// 
/// # 错误处理
/// 
/// 列出所有可能的错误情况和处理方式
/// 
/// # 使用示例
/// 
/// ```rust
/// // 提供实际的使用示例代码
/// let result = function_name(param1, param2)?;
/// assert_eq!(result, expected_value);
/// ```
/// 
/// # 设计原则体现
/// 
/// - **具体原则**: 说明此函数如何体现相关设计原则
/// 
/// # 性能考虑
/// 
/// 说明性能特点和注意事项
/// 
/// # 线程安全
/// 
/// 说明线程安全性（如适用）
pub fn example_function() {
    // 实现逻辑的中文注释
    // 每个关键步骤都要有解释
}

/// 内部函数的注释要求
/// 
/// 即使是私有函数也需要适当的中文注释
fn internal_function() {
    // 复杂逻辑需要分步骤注释
    
    // 步骤1：准备数据
    let data = prepare_data();
    
    // 步骤2：验证输入
    if !validate_input(&data) {
        // 处理验证失败情况
        return;
    }
    
    // 步骤3：执行核心逻辑
    let result = process_data(data);
    
    // 步骤4：清理资源
    cleanup_resources();
}
```

#### 7.1.2 测试质量标准

```rust
// 测试函数的命名和文档标准
#[cfg(test)]
mod test_module_name {
    use super::*;
    
    /// 测试函数的标准格式
    /// 
    /// 测试函数名应该清晰描述测试场景：test_功能_场景_预期结果
    /// 
    /// # 测试目标
    /// 
    /// 明确说明此测试要验证什么功能或行为
    /// 
    /// # 测试场景
    /// 
    /// 描述测试的具体场景和输入条件
    /// 
    /// # 预期结果
    /// 
    /// 明确说明期望的测试结果
    #[test]
    fn test_default_value_parsing_success() {
        // 准备测试数据
        let input = "test_value";
        let expected = DefaultValueType::String("test_value".to_string());
        
        // 执行测试操作
        let result = DefaultValueParser::parse(input, None);
        
        // 验证结果
        assert!(result.is_ok(), "解析应该成功");
        let parsed = result.unwrap();
        assert_eq!(parsed.value_type, expected, "解析结果应该匹配预期");
        assert!(!parsed.is_json, "简单字符串不应该被识别为 JSON");
    }
    
    /// 测试错误情况的标准格式
    #[test]
    fn test_default_value_validation_type_mismatch_error() {
        // 准备错误场景的测试数据
        let string_value = DefaultValue {
            raw_value: "not_a_number".to_string(),
            value_type: DefaultValueType::String("not_a_number".to_string()),
            is_json: false,
            span: None,
        };
        let integer_field: Type = syn::parse_str("i32").unwrap();
        
        // 执行应该失败的操作
        let validator = NumericValidator;
        let result = validator.validate(&string_value, &integer_field);
        
        // 验证错误情况
        assert!(result.is_err(), "验证应该失败");
        
        let error = result.unwrap_err();
        match error {
            MacroError::DefaultValueTypeMismatch { .. } => {
                // 验证错误类型正确
            },
            _ => panic!("错误类型不正确: {:?}", error),
        }
    }
    
    /// 性能测试的标准格式
    #[test]
    fn test_validation_performance() {
        use std::time::Instant;
        
        // 准备大量测试数据
        let test_cases = prepare_performance_test_data(1000);
        
        // 开始计时
        let start = Instant::now();
        
        // 执行性能测试
        for case in test_cases {
            let _ = validate_test_case(case);
        }
        
        // 检查性能指标
        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 100, 
            "1000个验证案例应该在100ms内完成，实际用时: {:?}", 
            duration
        );
    }
}
```

## 8. 部署和维护策略

### 8.1 版本管理策略

#### 8.1.1 版本号规则

```toml
# Cargo.toml 版本配置
[package]
name = "moduforge-macros-derive"
version = "0.3.0"  # 从现有版本升级 MINOR 版本
edition = "2021"

# 版本号含义:
# MAJOR.MINOR.PATCH
# - MAJOR: 破坏性 API 变更（避免）
# - MINOR: 新增功能（默认值支持）
# - PATCH: Bug 修复和性能优化

# 功能标志管理
[features]
default = ["default-values"]
default-values = []  # 默认值支持功能
experimental = []    # 实验性功能
```

#### 8.1.2 发布检查清单

```markdown
# 发布前检查清单

## 代码质量检查
- [ ] 所有测试通过 (`cargo test`)
- [ ] 代码格式检查通过 (`cargo fmt --check`)
- [ ] Clippy 检查通过 (`cargo clippy -- -D warnings`)
- [ ] 文档生成成功 (`cargo doc`)
- [ ] 基准测试结果在预期范围内

## 功能验证
- [ ] 所有 P0 需求 100% 实现
- [ ] 向后兼容性验证通过
- [ ] 性能指标满足要求
- [ ] 错误消息友好度检查

## 文档完整性
- [ ] API 文档覆盖率 ≥ 90%
- [ ] 更新日志 (CHANGELOG.md) 更新
- [ ] 迁移指南完整
- [ ] 示例代码验证

## 集成测试
- [ ] 与现有 ModuForge-RS 组件集成测试
- [ ] 真实项目兼容性测试
- [ ] 不同 Rust 版本兼容性测试
```

### 8.2 监控和维护

#### 8.2.1 性能监控

```rust
// crates/derive/src/common/metrics.rs 🆕
// 职责：性能监控和指标收集

#[cfg(feature = "metrics")]
pub mod metrics {
    use std::time::{Duration, Instant};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    
    /// 性能指标收集器
    /// 
    /// # 设计原则体现
    /// - **单一职责**: 专门负责性能指标收集
    /// - **开闭原则**: 支持新的指标类型扩展
    pub struct PerformanceMetrics {
        parse_time: AtomicU64,
        validation_time: AtomicU64,
        generation_time: AtomicU64,
        total_operations: AtomicU64,
    }
    
    impl PerformanceMetrics {
        pub fn new() -> Self {
            Self {
                parse_time: AtomicU64::new(0),
                validation_time: AtomicU64::new(0),
                generation_time: AtomicU64::new(0),
                total_operations: AtomicU64::new(0),
            }
        }
        
        /// 记录解析时间
        pub fn record_parse_time(&self, duration: Duration) {
            self.parse_time.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        }
        
        /// 记录验证时间
        pub fn record_validation_time(&self, duration: Duration) {
            self.validation_time.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        }
        
        /// 记录代码生成时间
        pub fn record_generation_time(&self, duration: Duration) {
            self.generation_time.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
        }
        
        /// 获取平均解析时间
        pub fn average_parse_time(&self) -> Duration {
            let total_time = self.parse_time.load(Ordering::Relaxed);
            let total_ops = self.total_operations.load(Ordering::Relaxed);
            
            if total_ops > 0 {
                Duration::from_nanos(total_time / total_ops)
            } else {
                Duration::from_nanos(0)
            }
        }
        
        /// 生成性能报告
        pub fn generate_report(&self) -> String {
            format!(
                "性能指标报告:\n\
                 - 总操作数: {}\n\
                 - 平均解析时间: {:?}\n\
                 - 平均验证时间: {:?}\n\
                 - 平均生成时间: {:?}",
                self.total_operations.load(Ordering::Relaxed),
                self.average_parse_time(),
                self.average_validation_time(),
                self.average_generation_time()
            )
        }
        
        fn average_validation_time(&self) -> Duration {
            // 类似 average_parse_time 的实现
            Duration::from_nanos(0) // 简化实现
        }
        
        fn average_generation_time(&self) -> Duration {
            // 类似 average_parse_time 的实现
            Duration::from_nanos(0) // 简化实现
        }
    }
    
    /// 全局性能指标实例
    static GLOBAL_METRICS: once_cell::sync::Lazy<PerformanceMetrics> = 
        once_cell::sync::Lazy::new(|| PerformanceMetrics::new());
    
    /// 获取全局性能指标
    pub fn global_metrics() -> &'static PerformanceMetrics {
        &GLOBAL_METRICS
    }
    
    /// 性能测量工具
    pub struct PerformanceMeasurement {
        start_time: Instant,
        metric_type: MetricType,
    }
    
    pub enum MetricType {
        Parse,
        Validation,
        Generation,
    }
    
    impl PerformanceMeasurement {
        pub fn start(metric_type: MetricType) -> Self {
            Self {
                start_time: Instant::now(),
                metric_type,
            }
        }
    }
    
    impl Drop for PerformanceMeasurement {
        fn drop(&mut self) {
            let duration = self.start_time.elapsed();
            let metrics = global_metrics();
            
            match self.metric_type {
                MetricType::Parse => metrics.record_parse_time(duration),
                MetricType::Validation => metrics.record_validation_time(duration),
                MetricType::Generation => metrics.record_generation_time(duration),
            }
        }
    }
    
    /// 性能测量宏
    macro_rules! measure_performance {
        ($type:expr, $block:block) => {
            {
                let _measurement = PerformanceMeasurement::start($type);
                $block
            }
        };
    }
    
    pub(crate) use measure_performance;
}
```

#### 8.2.2 错误追踪和诊断

```rust
// crates/derive/src/common/diagnostics.rs 🆕
// 职责：错误追踪和诊断信息收集

/// 诊断信息收集器
/// 
/// # 设计原则体现
/// - **单一职责**: 专门负责诊断信息收集和分析
/// - **开闭原则**: 支持新的诊断信息类型
pub struct DiagnosticsCollector {
    errors: Vec<DiagnosticEntry>,
    warnings: Vec<DiagnosticEntry>,
    performance_issues: Vec<PerformanceIssue>,
}

/// 诊断条目
#[derive(Debug, Clone)]
pub struct DiagnosticEntry {
    pub timestamp: std::time::Instant,
    pub error_type: String,
    pub message: String,
    pub context: DiagnosticContext,
    pub severity: Severity,
}

/// 诊断上下文
#[derive(Debug, Clone)]
pub struct DiagnosticContext {
    pub struct_name: Option<String>,
    pub field_name: Option<String>,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
}

/// 严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 性能问题记录
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub operation: String,
    pub duration: std::time::Duration,
    pub threshold: std::time::Duration,
    pub context: DiagnosticContext,
}

impl DiagnosticsCollector {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            performance_issues: Vec::new(),
        }
    }
    
    /// 记录错误
    pub fn record_error(&mut self, error: MacroError, context: DiagnosticContext) {
        let entry = DiagnosticEntry {
            timestamp: std::time::Instant::now(),
            error_type: error.error_type(),
            message: error.to_string(),
            context,
            severity: Severity::Error,
        };
        
        self.errors.push(entry);
    }
    
    /// 记录性能问题
    pub fn record_performance_issue(&mut self, issue: PerformanceIssue) {
        self.performance_issues.push(issue);
    }
    
    /// 生成诊断报告
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== 诊断报告 ===\n\n");
        
        // 错误统计
        report.push_str(&format!("错误总数: {}\n", self.errors.len()));
        report.push_str(&format!("警告总数: {}\n", self.warnings.len()));
        report.push_str(&format!("性能问题: {}\n\n", self.performance_issues.len()));
        
        // 详细错误信息
        if !self.errors.is_empty() {
            report.push_str("=== 错误详情 ===\n");
            for error in &self.errors {
                report.push_str(&format!(
                    "- 类型: {}\n  消息: {}\n  上下文: {:?}\n\n",
                    error.error_type, error.message, error.context
                ));
            }
        }
        
        // 性能问题详情
        if !self.performance_issues.is_empty() {
            report.push_str("=== 性能问题 ===\n");
            for issue in &self.performance_issues {
                report.push_str(&format!(
                    "- 操作: {}\n  用时: {:?}\n  阈值: {:?}\n  上下文: {:?}\n\n",
                    issue.operation, issue.duration, issue.threshold, issue.context
                ));
            }
        }
        
        report
    }
}

impl MacroError {
    /// 获取错误类型字符串
    pub fn error_type(&self) -> String {
        match self {
            Self::DefaultValueTypeMismatch { .. } => "DefaultValueTypeMismatch".to_string(),
            Self::InvalidJsonDefaultValue { .. } => "InvalidJsonDefaultValue".to_string(),
            Self::JsonValueTypeRequired { .. } => "JsonValueTypeRequired".to_string(),
            Self::DefaultValueParseError { .. } => "DefaultValueParseError".to_string(),
            _ => "UnknownError".to_string(),
        }
    }
}
```

---

*此模块化设计架构文档为 ModuForge-RS Default 属性扩展项目提供了完整的技术实现蓝图，严格遵循核心设计原则，确保项目能够实现高质量、高性能、高可维护性的目标。*
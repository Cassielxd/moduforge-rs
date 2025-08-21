# ModuForge-RS Default 属性扩展 - 精确的开发任务清单

## 1. 任务概述

### 1.1 项目背景
基于现有 `crates/derive` 中的 `moduforge-macros-derive` 库，添加默认值属性支持，实现声明式的字段默认值功能。项目严格遵循核心设计原则，确保向后兼容性和高质量实现。

### 1.2 开发约束
- **兼容性约束**：现有代码 100% 向后兼容，无破坏性变更
- **性能约束**：编译时间增加 < 10%，内存使用增加 < 20MB
- **质量约束**：测试覆盖率 ≥ 95%，文档覆盖率 ≥ 90%
- **架构约束**：严格遵循单一职责、开闭原则、里氏替换、接口隔离原则

### 1.3 交付标准
- 所有代码必须包含详细的中文注释和文档
- 每个功能模块都需要完整的单元测试和集成测试
- 代码生成必须通过性能基准测试
- 错误消息必须友好且包含具体的修复建议

## 2. 开发任务分解

### 阶段一：基础架构搭建（第1-2周）

#### 任务 TASK-001: 默认值数据结构设计
**优先级**: P0 (必须完成)
**预估工时**: 1.5天
**依赖关系**: 无
**负责模块**: `parser/default_value.rs`

**任务描述**:
创建默认值的核心数据结构和基础解析逻辑，为整个功能提供数据模型基础。

**具体工作项**:
1. **创建 `default_value.rs` 模块**:
   ```rust
   // 文件位置: crates/derive/src/parser/default_value.rs
   
   //! 默认值处理器模块
   //! 
   //! 负责解析、验证和处理 #[attr(default="value")] 属性中的默认值。
   //! 严格遵循单一职责原则，专门处理默认值相关的所有逻辑。
   //! 
   //! # 设计原则体现
   //! - **单一职责**: 专门负责默认值相关的数据结构和基础逻辑
   //! - **开闭原则**: 通过枚举和 trait 支持新的默认值类型扩展
   ```

2. **定义 `DefaultValue` 结构体**:
   - 原始字符串值存储
   - 解析后的类型化值存储
   - JSON 格式标识
   - 源码位置信息（用于错误报告）

3. **定义 `DefaultValueType` 枚举**:
   - String、Integer、Float、Boolean 基本类型
   - Json 复杂类型
   - Null 空值类型

4. **实现 `DefaultValueParser`**:
   - 字符串解析为类型化值
   - JSON 格式检测和解析
   - 错误处理和位置追踪

**验收标准**:
- [ ] `DefaultValue` 结构体完整定义，包含所有必要字段
- [ ] `DefaultValueType` 枚举支持所有规划的基本类型
- [ ] `DefaultValueParser::parse` 方法正确解析所有支持的类型
- [ ] JSON 格式能被正确识别和解析
- [ ] 错误情况能被正确捕获并包含位置信息
- [ ] 所有代码包含详细中文注释和文档

**技术要求**:
- 使用 `serde_json` 进行 JSON 解析和验证
- 使用 `syn::Span` 追踪源码位置
- 实现 `Debug`、`Clone`、`PartialEq` 等必要 trait
- 错误处理使用项目统一的 `MacroResult<T>` 类型

**测试要求**:
- 基本类型解析测试（String、i32、f64、bool）
- JSON 格式解析测试（简单对象、复杂嵌套、数组）
- 错误情况测试（无效 JSON、不支持的格式）
- 边界情况测试（空字符串、特殊字符、Unicode）

---

#### 任务 TASK-002: FieldConfig 结构扩展
**优先级**: P0 (必须完成)
**预估工时**: 1天
**依赖关系**: TASK-001
**负责模块**: `parser/attribute_parser.rs`

**任务描述**:
扩展现有的 `FieldConfig` 结构，添加默认值支持，同时保持完全的向后兼容性。

**具体工作项**:
1. **扩展 `FieldConfig` 结构体**:
   ```rust
   #[derive(Debug, Clone)]
   pub struct FieldConfig {
       // === 现有字段保持完全不变 ===
       pub name: String,
       pub type_name: String,
       pub is_optional: bool,
       pub is_attr: bool,
       pub field: Field,
       
       // === 新增字段（保持向后兼容）===
       /// 默认值配置（None 表示无默认值，保持现有行为）
       /// 
       /// # 设计原则体现
       /// - **开闭原则**: 通过 Option 类型实现无破坏性扩展
       /// - **里氏替换**: 现有代码可以忽略此字段继续工作
       pub default_value: Option<DefaultValue>,
   }
   ```

2. **保持现有构造函数不变**:
   - `FieldConfig::new()` 方法签名和行为完全不变
   - 新字段默认为 `None`，保持现有行为

3. **添加新的便利方法**:
   - `with_default_value()` - 链式设置默认值
   - `has_default_value()` - 检查是否有默认值
   - `get_default_value()` - 获取默认值引用

4. **更新模块导入**:
   - 在 `parser/mod.rs` 中导出新模块
   - 更新相关的 `use` 语句

**验收标准**:
- [ ] `FieldConfig` 结构体成功扩展，包含 `default_value` 字段
- [ ] 现有构造函数 `new()` 保持完全不变
- [ ] 新增的便利方法功能正确
- [ ] 所有现有测试继续通过，无回归问题
- [ ] 新字段的默认值为 `None`，保持向后兼容
- [ ] 代码包含详细的中文注释说明设计原则体现

**技术要求**:
- 使用 `Option<DefaultValue>` 实现可选的默认值
- 保持所有现有方法的签名和行为不变
- 新增方法使用 builder 模式支持链式调用
- 遵循项目的命名约定和代码风格

**测试要求**:
- 现有代码兼容性测试
- 新增功能的单元测试
- 链式调用功能测试
- 边界情况测试（None 值处理）

---

#### 任务 TASK-003: 属性解析器增强
**优先级**: P0 (必须完成)
**预估工时**: 2天
**依赖关系**: TASK-002
**负责模块**: `parser/attribute_parser.rs`

**任务描述**:
增强现有的 `AttributeParser`，添加 `default` 参数的解析功能，支持 `#[attr(default="value")]` 语法。

**具体工作项**:
1. **扩展 `AttributeParser` 实现**:
   ```rust
   impl AttributeParser {
       // === 现有方法保持不变 ===
       
       // === 新增方法 ===
       /// 解析字段的 default 参数
       /// 
       /// 从 #[attr(default="value")] 中提取并验证默认值
       /// 
       /// # 设计原则体现
       /// - **单一职责**: 专门负责 default 参数解析
       /// - **开闭原则**: 通过新增方法扩展功能而不修改现有逻辑
       fn parse_default_parameter(attr: &Attribute) -> MacroResult<Option<String>> {
           // 实现 default 参数提取逻辑
       }
       
       /// 增强的字段属性解析
       /// 
       /// 在现有解析基础上添加默认值支持
       /// 
       /// # 设计原则体现
       /// - **里氏替换**: 返回的 FieldConfig 完全兼容现有使用
       /// - **开闭原则**: 扩展现有功能而不修改核心逻辑
       pub fn parse_field_attributes_enhanced(input: &DeriveInput) -> MacroResult<Vec<FieldConfig>> {
           // 调用现有解析逻辑
           // 扩展默认值解析
       }
   }
   ```

2. **实现默认值参数提取**:
   - 解析 `default="value"` 语法
   - 支持字符串字面量和原始字符串
   - 提取参数值并进行基本验证

3. **集成到现有解析流程**:
   - 在现有字段解析后添加默认值解析
   - 使用 `DefaultValueParser` 解析提取的值
   - 创建带默认值的 `FieldConfig`

4. **错误处理增强**:
   - 添加默认值相关的错误类型
   - 提供精确的错误位置信息
   - 友好的错误消息和修复建议

**验收标准**:
- [ ] 成功解析 `#[attr(default="value")]` 语法
- [ ] 支持各种字符串格式（普通字符串、原始字符串）
- [ ] 正确集成到现有解析流程，不影响现有功能
- [ ] 错误情况能被正确处理并报告
- [ ] 生成的 `FieldConfig` 包含正确的默认值信息
- [ ] 所有现有解析测试继续通过

**技术要求**:
- 使用 `syn` 库解析属性语法
- 支持 `r#"raw string"#` 格式（用于 JSON）
- 错误处理使用统一的 `MacroError` 类型
- 遵循现有的解析模式和错误处理风格

**测试要求**:
- 各种默认值语法解析测试
- 混合使用新旧语法的兼容性测试
- 错误情况的解析测试
- 复杂属性组合的解析测试

---

#### 任务 TASK-004: 验证器系统基础架构
**优先级**: P0 (必须完成)
**预估工时**: 2天
**依赖关系**: TASK-001
**负责模块**: `parser/validation.rs`

**任务描述**:
设计和实现可扩展的验证器系统，支持不同类型的默认值验证。

**具体工作项**:
1. **定义验证器接口**:
   ```rust
   /// 默认值验证器接口 - 遵循接口隔离原则
   /// 
   /// # 设计原则体现
   /// - **接口隔离**: 提供最小化、专用的验证接口
   /// - **依赖倒置**: 高层模块依赖此抽象接口
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
   ```

2. **实现验证器注册表**:
   ```rust
   /// 验证器注册表 - 遵循依赖倒置原则
   /// 
   /// # 设计原则体现
   /// - **开闭原则**: 支持动态添加新的验证器
   /// - **依赖倒置**: 依赖抽象接口而非具体实现
   pub struct ValidatorRegistry {
       validators: Vec<Box<dyn DefaultValueValidator>>,
   }
   ```

3. **实现基础验证器**:
   - `StringValidator` - 字符串类型验证
   - `NumericValidator` - 数值类型验证和范围检查
   - `BooleanValidator` - 布尔类型验证
   - `JsonValidator` - JSON 格式和类型约束验证
   - `OptionValidator` - Option 类型验证

4. **实现验证流水线**:
   - 分层验证（语法 → 类型 → 语义）
   - 错误收集和报告
   - 性能优化（缓存、预排序）

**验收标准**:
- [ ] `DefaultValueValidator` trait 设计合理，接口最小化
- [ ] `ValidatorRegistry` 支持验证器的注册和查找
- [ ] 所有基础验证器实现正确
- [ ] 验证流水线能够处理复杂的验证场景
- [ ] 错误信息详细且包含修复建议
- [ ] 验证性能满足 < 1ms/字段的要求

**技术要求**:
- 验证器使用 trait object 实现多态
- 支持验证器的优先级排序
- 使用 `syn::Type` 进行类型分析
- 实现高效的类型匹配算法

**测试要求**:
- 每个验证器的完整功能测试
- 验证器注册表的管理功能测试
- 复杂验证场景的集成测试
- 性能基准测试

---

### 阶段二：核心功能实现（第3-4周）

#### 任务 TASK-005: 类型验证器实现
**优先级**: P0 (必须完成)
**预估工时**: 3天
**依赖关系**: TASK-004
**负责模块**: `parser/validation.rs`

**任务描述**:
实现所有规划的类型验证器，确保默认值与字段类型的严格匹配。

**具体工作项**:
1. **StringValidator 实现**:
   ```rust
   /// 字符串类型验证器 - 遵循单一职责原则
   /// 
   /// # 支持类型
   /// - String, &str, str
   /// 
   /// # 设计原则体现
   /// - **单一职责**: 专门负责字符串类型验证
   /// - **开闭原则**: 通过 trait 实现支持扩展
   pub struct StringValidator;
   
   impl DefaultValueValidator for StringValidator {
       fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
           // 验证字段类型是否为字符串类型
           // 验证默认值是否为字符串格式
           // 特殊处理 Unicode 字符串
       }
       
       fn supports_type(&self, field_type: &Type) -> bool {
           // 识别 String, &str, str 类型
       }
   }
   ```

2. **NumericValidator 实现**:
   ```rust
   /// 数值类型验证器 - 遵循单一职责原则
   /// 
   /// # 支持类型
   /// - 整数: i8, i16, i32, i64, i128, isize
   /// - 无符号整数: u8, u16, u32, u64, u128, usize  
   /// - 浮点数: f32, f64
   /// 
   /// # 验证规则
   /// - 数值格式正确性
   /// - 类型范围检查
   /// - 溢出检测
   pub struct NumericValidator;
   
   impl NumericValidator {
       /// 验证整数值是否在类型范围内
       fn validate_integer_range(&self, value: i64, type_name: &str, span: Option<Span>) -> MacroResult<()> {
           // 检查各种整数类型的取值范围
           // 提供详细的范围错误信息
       }
       
       /// 验证浮点数值是否在类型范围内
       fn validate_float_range(&self, value: f64, type_name: &str, span: Option<Span>) -> MacroResult<()> {
           // 检查浮点数的有效性（finite, 范围）
           // 特殊值处理（NaN, Infinity）
       }
   }
   ```

3. **JsonValidator 实现**:
   ```rust
   /// JSON 类型验证器 - 遵循单一职责原则
   /// 
   /// # 类型约束
   /// - 只支持 serde_json::Value 类型字段
   /// - 验证 JSON 语法正确性
   /// - 验证 JSON 复杂度限制
   pub struct JsonValidator;
   
   impl DefaultValueValidator for JsonValidator {
       fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
           // 检查字段类型是否为 serde_json::Value
           // 验证 JSON 语法和格式
           // 检查 JSON 复杂度（嵌套深度、大小）
       }
   }
   ```

4. **OptionValidator 实现**:
   ```rust
   /// Option 类型验证器 - 遵循单一职责原则
   /// 
   /// # 验证规则
   /// - "null" 字符串 → None 值
   /// - 其他值按内部类型 T 验证
   /// - 支持嵌套 Option 类型
   pub struct OptionValidator;
   
   impl DefaultValueValidator for OptionValidator {
       fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
           // 提取 Option<T> 的内部类型 T
           // "null" 特殊值处理
           // 递归验证内部类型
       }
   }
   ```

5. **类型分析工具实现**:
   ```rust
   /// 类型分析器 - 遵循单一职责原则
   pub struct TypeAnalyzer;
   
   impl TypeAnalyzer {
       /// 提取类型名称
       pub fn extract_type_name(ty: &Type) -> String;
       
       /// 检查是否为 Option 类型
       pub fn is_option_type(ty: &Type) -> bool;
       
       /// 提取 Option 的内部类型
       pub fn extract_option_inner_type(ty: &Type) -> Option<Type>;
       
       /// 检查类型类别
       pub fn is_numeric_type(type_name: &str) -> bool;
       pub fn is_string_type(type_name: &str) -> bool;
       pub fn is_json_value_type(type_name: &str) -> bool;
   }
   ```

**验收标准**:
- [ ] 所有验证器正确实现 `DefaultValueValidator` trait
- [ ] 字符串验证器支持所有字符串类型变体
- [ ] 数值验证器精确检查所有数值类型的范围
- [ ] JSON 验证器严格执行类型约束
- [ ] Option 验证器正确处理嵌套类型
- [ ] 类型分析器准确识别各种复杂类型
- [ ] 所有错误消息友好且包含修复建议

**技术要求**:
- 使用 `syn::Type` 进行准确的类型分析
- 支持类型别名和完全限定路径
- 优化性能，避免重复的类型解析
- 实现全面的错误处理和报告

**测试要求**:
- 每种类型的边界值测试
- 类型别名和路径变体测试
- 复杂嵌套类型测试
- 错误消息质量测试

---

#### 任务 TASK-006: 错误处理系统扩展
**优先级**: P0 (必须完成)
**预估工时**: 1.5天
**依赖关系**: TASK-005
**负责模块**: `common/error.rs`

**任务描述**:
扩展现有的错误处理系统，添加默认值相关的错误类型和友好错误消息。

**具体工作项**:
1. **扩展 MacroError 枚举**:
   ```rust
   #[derive(Error, Debug)]
   pub enum MacroError {
       // === 现有错误类型保持不变 ===
       MissingAttribute { /* ... */ },
       InvalidAttributeValue { /* ... */ },
       UnsupportedFieldType { /* ... */ },
       
       // === 新增默认值相关错误 ===
       
       /// 默认值类型不匹配错误
       /// 
       /// 当默认值类型与字段类型不兼容时触发
       #[error("默认值类型不匹配: 字段 '{field_name}' 类型为 '{field_type}'，但默认值 '{default_value}' 类型为 '{actual_type}'")]
       DefaultValueTypeMismatch {
           field_name: String,
           field_type: String,
           default_value: String,
           actual_type: String,
           span: Option<Span>,
       },
       
       /// JSON 默认值格式错误
       #[error("JSON 默认值格式错误: {reason}")]
       InvalidJsonDefaultValue {
           reason: String,
           value: String,
           field_name: String,
           span: Option<Span>,
       },
       
       /// JSON 类型约束错误
       #[error("JSON 默认值只能用于 serde_json::Value 类型字段")]
       JsonValueTypeRequired {
           field_name: String,
           actual_type: String,
           span: Option<Span>,
       },
       
       /// 默认值解析错误
       #[error("默认值解析失败: {reason}")]
       DefaultValueParseError {
           reason: String,
           value: String,
           field_name: String,
           span: Option<Span>,
       },
   }
   ```

2. **实现友好错误消息生成**:
   ```rust
   impl MacroError {
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
                       - 参考文档了解支持的默认值格式",
                       field_type, actual_type, expected_type
                   )
               },
               // 其他错误类型的友好消息
           }
       }
       
       /// 为默认值错误提供专门的修复建议
       pub fn default_value_suggestion(&self) -> String {
           // 根据错误类型提供具体的修复建议
       }
   }
   ```

3. **实现便利构造方法**:
   ```rust
   impl MacroError {
       /// 创建默认值类型不匹配错误
       pub fn default_value_type_mismatch<T: Spanned>(
           field_name: &str,
           field_type: &str,
           default_value: &str,
           expected_type: &str,
           spanned: &T,
       ) -> Self;
       
       /// 创建 JSON 格式错误
       pub fn invalid_json_default<T: Spanned>(
           reason: &str,
           value: &str,
           field_name: &str,
           spanned: &T,
       ) -> Self;
       
       /// 创建 JSON 类型约束错误
       pub fn json_value_type_required<T: Spanned>(
           field_name: &str,
           actual_type: &str,
           spanned: &T,
       ) -> Self;
   }
   ```

4. **实现错误恢复策略**:
   ```rust
   /// 错误恢复处理器
   /// 
   /// # 设计原则体现
   /// - **单一职责**: 专门负责错误恢复逻辑
   pub struct ErrorRecoveryHandler;
   
   impl ErrorRecoveryHandler {
       /// 处理所有字段的默认值，收集错误但不中断处理
       pub fn process_all_fields_with_recovery(
           fields: &[FieldConfig]
       ) -> (Vec<ProcessedField>, Vec<MacroError>) {
           // 实现容错处理逻辑
       }
   }
   ```

**验收标准**:
- [ ] 所有新增错误类型定义完整且语义明确
- [ ] 错误消息友好，包含具体的修复建议
- [ ] 便利构造方法简化错误创建过程
- [ ] 错误恢复策略能处理多个错误场景
- [ ] 错误位置信息精确到字符级别
- [ ] 支持中文本地化的错误消息

**技术要求**:
- 使用 `thiserror` 维持与现有错误系统的一致性
- 确保所有错误都包含 `Span` 信息
- 错误消息模板化，便于维护和国际化
- 性能优化，避免不必要的字符串分配

**测试要求**:
- 每种错误类型的生成和格式化测试
- 错误消息质量的用户体验测试
- 错误恢复逻辑的边界情况测试
- 本地化支持测试

---

#### 任务 TASK-007: 代码生成器增强
**优先级**: P0 (必须完成)
**预估工时**: 3天
**依赖关系**: TASK-006
**负责模块**: `generator/node_generator.rs`, `generator/mark_generator.rs`

**任务描述**:
增强现有的代码生成器，支持默认值的智能处理和新方法生成。

**具体工作项**:
1. **增强 NodeGenerator**:
   ```rust
   impl NodeGenerator {
       /// 生成增强的 to_node 方法 - 遵循里氏替换原则
       /// 
       /// 生成的方法完全兼容现有接口，但支持默认值处理
       /// 
       /// # 设计原则体现
       /// - **里氏替换**: 完全兼容现有 to_node 方法接口
       /// - **开闭原则**: 支持默认值而不修改现有逻辑
       pub fn generate_to_node_method(&self) -> MacroResult<TokenStream2> {
           // 分离有默认值和无默认值的字段
           // 生成字段设置代码
           // 保持现有方法签名和行为
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
       ) -> MacroResult<TokenStream2>;
       
       /// 生成带默认值的字段设置代码
       fn generate_field_setter_with_default(&self, field_config: &FieldConfig) -> MacroResult<TokenStream2>;
       
       /// 生成默认值表达式
       /// 
       /// # 设计原则体现
       /// - **开闭原则**: 通过模式匹配支持新的默认值类型
       /// - **单一职责**: 专门负责默认值表达式生成
       fn generate_default_value_expression(
           &self,
           default_value: &DefaultValue,
           field_config: &FieldConfig
       ) -> MacroResult<TokenStream2>;
   }
   ```

2. **实现构造函数生成**:
   ```rust
   impl NodeGenerator {
       /// 生成构造函数方法 - 遵循开闭原则
       /// 
       /// 只有当结构体包含默认值字段时才生成构造函数
       /// 
       /// # 设计原则体现
       /// - **开闭原则**: 新增功能不影响现有代码
       /// - **单一职责**: 专门负责构造函数生成
       pub fn generate_constructor_methods(&self) -> MacroResult<TokenStream2> {
           // 检查是否有默认值字段
           // 生成 new() 方法
           // 生成 with_defaults() 方法
       }
       
       /// 生成 new() 方法
       fn generate_new_method(&self) -> MacroResult<TokenStream2>;
       
       /// 生成字段初始化代码
       fn generate_field_initializers(&self) -> MacroResult<TokenStream2>;
       
       /// 生成带默认值的字段初始化器
       fn generate_field_initializer_with_default(
           &self,
           default_value: &DefaultValue,
           field_config: &FieldConfig
       ) -> MacroResult<TokenStream2>;
   }
   ```

3. **增强 MarkGenerator**:
   ```rust
   impl MarkGenerator {
       /// 生成增强的 to_mark 方法
       /// 
       /// 与 NodeGenerator 类似的增强逻辑，适配 Mark 类型
       pub fn generate_to_mark_method(&self) -> MacroResult<TokenStream2>;
       
       /// 生成 Mark 构造函数
       pub fn generate_mark_constructor_methods(&self) -> MacroResult<TokenStream2>;
   }
   ```

4. **智能代码优化**:
   ```rust
   /// 代码生成优化器
   /// 
   /// # 设计原则体现
   /// - **单一职责**: 专门负责代码生成优化
   pub struct CodeGenerationOptimizer;
   
   impl CodeGenerationOptimizer {
       /// 优化字段设置代码
       /// 
       /// 基于字段类型和默认值，生成最优的代码
       pub fn optimize_field_setter(
           field_config: &FieldConfig,
           default_value: &DefaultValue
       ) -> MacroResult<TokenStream2> {
           // 根据类型选择最优的代码生成策略
           // 避免不必要的运行时检查
           // 内联常量值
       }
       
       /// 生成空值检查表达式
       pub fn generate_empty_check(type_name: &str, field_name: &Ident) -> TokenStream2;
   }
   ```

**验收标准**:
- [ ] 生成的 `to_node/to_mark` 方法完全兼容现有接口
- [ ] 默认值在适当时机被正确应用
- [ ] 构造函数只在有默认值时生成
- [ ] 生成的代码类型安全，无运行时错误
- [ ] 代码优化，避免不必要的运行时开销
- [ ] 生成的代码易于阅读和调试

**技术要求**:
- 使用 `quote!` 宏生成类型安全的代码
- 支持各种复杂的默认值表达式
- 优化生成代码的性能
- 保持生成代码的可读性

**测试要求**:
- 各种默认值类型的代码生成测试
- 混合字段的代码生成测试
- 生成代码的编译和运行测试
- 性能基准测试

---

### 阶段三：高级功能和优化（第5-6周）

#### 任务 TASK-008: 性能优化实现
**优先级**: P1 (重要完成)
**预估工时**: 2天
**依赖关系**: TASK-007
**负责模块**: `common/utils.rs`, `parser/validation.rs`

**任务描述**:
实现编译时性能优化，确保默认值功能不会显著影响编译速度。

**具体工作项**:
1. **类型信息缓存系统**:
   ```rust
   /// 类型信息缓存 - 遵循单一职责原则
   /// 
   /// 缓存常用类型的解析结果，避免重复计算
   /// 
   /// # 设计原则体现
   /// - **单一职责**: 专门负责类型信息缓存
   /// - **开闭原则**: 支持新的类型信息扩展
   pub struct TypeInfoCache {
       cache: HashMap<String, TypeInfo>,
   }
   
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
   
   impl TypeInfoCache {
       /// 获取或分析类型信息
       pub fn get_or_analyze(&mut self, ty: &Type) -> TypeInfo;
       
       /// 预缓存常用类型
       pub fn preload_common_types(&mut self);
   }
   ```

2. **验证器性能优化**:
   ```rust
   /// 优化的验证管道 - 遵循单一职责原则
   pub struct OptimizedValidationPipeline {
       validators: Vec<Box<dyn DefaultValueValidator>>,
       type_validator_map: HashMap<String, usize>, // 类型到验证器的快速映射
   }
   
   impl OptimizedValidationPipeline {
       /// 快速验证默认值
       /// 
       /// 使用缓存和预排序提高验证性能
       /// 
       /// # 性能目标
       /// - 单个字段验证 < 1ms
       /// - JSON 验证 < 2ms
       pub fn validate_fast(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()>;
       
       /// 批量验证优化
       pub fn validate_batch(&self, validations: &[(DefaultValue, Type)]) -> Vec<MacroResult<()>>;
   }
   ```

3. **代码生成缓存**:
   ```rust
   /// 代码生成缓存系统
   /// 
   /// # 设计原则体现
   /// - **单一职责**: 专门负责生成代码的缓存
   pub struct CodeGenerationCache {
       expression_cache: HashMap<String, TokenStream2>,
       template_cache: HashMap<String, CompiledTemplate>,
   }
   
   impl CodeGenerationCache {
       /// 缓存常用的默认值表达式
       pub fn cache_common_expressions(&mut self);
       
       /// 获取或生成默认值表达式
       pub fn get_or_generate_expression(
           &mut self,
           default_value: &DefaultValue,
           field_config: &FieldConfig
       ) -> MacroResult<TokenStream2>;
   }
   ```

4. **性能监控系统**:
   ```rust
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
       /// 记录各阶段的处理时间
       pub fn record_parse_time(&self, duration: Duration);
       pub fn record_validation_time(&self, duration: Duration);
       pub fn record_generation_time(&self, duration: Duration);
       
       /// 生成性能报告
       pub fn generate_report(&self) -> String;
   }
   ```

**验收标准**:
- [ ] 编译时间增加 < 10%（基准测试验证）
- [ ] 内存使用峰值增加 < 20MB
- [ ] 单个字段验证时间 < 1ms
- [ ] JSON 验证时间 < 2ms
- [ ] 缓存命中率 > 80%（常用类型）
- [ ] 性能监控数据准确可靠

**技术要求**:
- 使用 `once_cell` 实现全局缓存
- 原子操作确保线程安全
- 内存使用优化，避免泄漏
- 基准测试验证性能改进

**测试要求**:
- 性能基准测试套件
- 内存使用分析测试
- 缓存效果验证测试
- 并发安全性测试

---

#### 任务 TASK-009: 可扩展性架构实现
**优先级**: P1 (重要完成)
**预估工时**: 2.5天
**依赖关系**: TASK-008
**负责模块**: `parser/validation.rs`, `generator/templates.rs`

**任务描述**:
实现插件化的验证器和代码生成模板系统，支持未来功能扩展。

**具体工作项**:
1. **验证器插件系统**:
   ```rust
   /// 验证器插件注册表 - 遵循开闭原则
   /// 
   /// # 设计原则体现
   /// - **开闭原则**: 支持新验证器的动态注册
   /// - **依赖倒置**: 依赖抽象接口而非具体实现
   pub struct ValidatorPluginRegistry {
       validators: Vec<Box<dyn DefaultValueValidator>>,
       type_mappings: HashMap<String, Vec<usize>>,
   }
   
   impl ValidatorPluginRegistry {
       /// 注册验证器插件
       pub fn register_validator<V: DefaultValueValidator + 'static>(&mut self, validator: V);
       
       /// 批量注册内置验证器
       pub fn register_builtin_validators(&mut self);
       
       /// 快速查找验证器
       pub fn find_validator(&self, field_type: &Type) -> Option<&dyn DefaultValueValidator>;
   }
   ```

2. **自定义验证器示例**:
   ```rust
   /// 自定义日期验证器示例
   /// 
   /// 展示如何实现新的验证器
   /// 
   /// # 设计原则体现
   /// - **开闭原则**: 通过实现 trait 扩展验证功能
   /// - **单一职责**: 专门负责日期类型验证
   pub struct CustomDateValidator;
   
   impl DefaultValueValidator for CustomDateValidator {
       fn validate(&self, default_value: &DefaultValue, field_type: &Type) -> MacroResult<()> {
           // 自定义日期验证逻辑
           // 验证日期格式（ISO 8601, RFC 3339 等）
           // 检查日期有效性
       }
       
       fn supports_type(&self, field_type: &Type) -> bool {
           // 支持 DateTime, NaiveDate, chrono::DateTime 等类型
       }
       
       fn priority(&self) -> i32 { 60 }
       fn name(&self) -> &'static str { "CustomDateValidator" }
   }
   ```

3. **代码生成模板系统**:
   ```rust
   /// 代码生成模板接口 - 遵循接口隔离原则
   /// 
   /// # 设计原则体现
   /// - **接口隔离**: 提供最小化的模板接口
   /// - **开闭原则**: 支持自定义模板扩展
   pub trait CodeGenerationTemplate {
       /// 生成代码
       fn generate(&self, context: &GenerationContext) -> MacroResult<TokenStream2>;
       
       /// 检查是否支持指定的模式
       fn supports_pattern(&self, pattern: &str) -> bool;
       
       /// 模板名称和优先级
       fn name(&self) -> &'static str;
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
   ```

4. **模板注册表实现**:
   ```rust
   /// 模板注册表 - 遵循开闭原则
   pub struct TemplateRegistry {
       templates: Vec<Box<dyn CodeGenerationTemplate>>,
       pattern_mappings: HashMap<String, Vec<usize>>,
   }
   
   impl TemplateRegistry {
       /// 注册代码生成模板
       pub fn register_template<T: CodeGenerationTemplate + 'static>(&mut self, template: T);
       
       /// 选择合适的模板
       pub fn select_template(&self, pattern: &str) -> Option<&dyn CodeGenerationTemplate>;
       
       /// 批量注册内置模板
       pub fn register_builtin_templates(&mut self);
   }
   ```

5. **内置模板实现**:
   ```rust
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
   }
   
   /// JSON 默认值模板
   pub struct JsonDefaultTemplate;
   
   /// Option 默认值模板
   pub struct OptionDefaultTemplate;
   ```

**验收标准**:
- [ ] 验证器插件系统支持动态注册
- [ ] 自定义验证器能够正确集成
- [ ] 模板系统支持多种生成模式
- [ ] 模板注册表支持模式匹配和优先级
- [ ] 内置模板覆盖所有支持的类型
- [ ] 扩展示例完整且可运行

**技术要求**:
- 使用 trait object 实现插件多态
- 支持插件的优先级排序
- 模板系统支持模式匹配
- 提供完整的扩展文档和示例

**测试要求**:
- 插件注册和查找功能测试
- 自定义验证器集成测试
- 模板系统的生成功能测试
- 扩展性示例的完整测试

---

### 阶段四：测试和文档（第7-8周）

#### 任务 TASK-010: 全面测试套件实现
**优先级**: P0 (必须完成)
**预估工时**: 3天
**依赖关系**: TASK-009
**负责模块**: `tests/`

**任务描述**:
实现全面的测试套件，确保功能正确性和质量标准。

**具体工作项**:
1. **单元测试实现**:
   ```rust
   // tests/default_value_tests.rs
   
   #[cfg(test)]
   mod default_value_parsing_tests {
       /// 测试基本类型默认值解析
       /// 
       /// # 测试目标
       /// 验证 DefaultValueParser 能够正确解析各种基本类型的默认值
       /// 
       /// # 测试场景
       /// - 字符串解析
       /// - 数值解析（整数、浮点数）
       /// - 布尔值解析
       /// - JSON 格式解析
       /// 
       /// # 预期结果
       /// 所有支持的类型都能被正确解析为相应的 DefaultValueType
       #[test]
       fn test_parse_all_basic_types() {
           // 字符串解析测试
           let string_result = DefaultValueParser::parse("hello world", None);
           assert!(string_result.is_ok());
           assert!(matches!(string_result.unwrap().value_type, DefaultValueType::String(_)));
           
           // 整数解析测试
           let int_result = DefaultValueParser::parse("42", None);
           assert!(int_result.is_ok());
           assert!(matches!(int_result.unwrap().value_type, DefaultValueType::Integer(42)));
           
           // JSON 解析测试
           let json_result = DefaultValueParser::parse(r#"{"key": "value"}"#, None);
           assert!(json_result.is_ok());
           assert!(json_result.unwrap().is_json);
       }
       
       /// 测试边界情况和错误处理
       #[test]
       fn test_parsing_edge_cases() {
           // 空字符串
           let empty_result = DefaultValueParser::parse("", None);
           assert!(empty_result.is_ok());
           
           // Unicode 字符串
           let unicode_result = DefaultValueParser::parse("你好世界 🦀", None);
           assert!(unicode_result.is_ok());
           
           // 无效 JSON
           let invalid_json = DefaultValueParser::parse(r#"{"invalid": json"#, None);
           assert!(invalid_json.is_err());
           
           // 极大数值
           let large_number = DefaultValueParser::parse("999999999999999999999", None);
           assert!(large_number.is_ok());
       }
   }
   
   #[cfg(test)]
   mod validation_comprehensive_tests {
       /// 测试所有验证器的功能
       #[test]
       fn test_all_validators_comprehensive() {
           let registry = ValidatorRegistry::new();
           
           // 测试字符串验证器
           test_string_validator_scenarios(&registry);
           
           // 测试数值验证器
           test_numeric_validator_scenarios(&registry);
           
           // 测试 JSON 验证器
           test_json_validator_scenarios(&registry);
           
           // 测试 Option 验证器
           test_option_validator_scenarios(&registry);
       }
       
       fn test_string_validator_scenarios(registry: &ValidatorRegistry) {
           // 正确的字符串类型匹配
           let string_field: Type = syn::parse_str("String").unwrap();
           let string_value = create_string_default_value("test");
           assert!(registry.validate(&string_value, &string_field).is_ok());
           
           // 类型不匹配
           let int_field: Type = syn::parse_str("i32").unwrap();
           assert!(registry.validate(&string_value, &int_field).is_err());
       }
       
       fn test_numeric_validator_scenarios(registry: &ValidatorRegistry) {
           // 各种数值类型的范围测试
           test_integer_ranges(registry);
           test_float_ranges(registry);
       }
       
       fn test_integer_ranges(registry: &ValidatorRegistry) {
           // i8 范围测试
           let i8_field: Type = syn::parse_str("i8").unwrap();
           
           // 有效范围
           let valid_i8 = create_integer_default_value(100);
           assert!(registry.validate(&valid_i8, &i8_field).is_ok());
           
           // 超出范围
           let invalid_i8 = create_integer_default_value(1000);
           assert!(registry.validate(&invalid_i8, &i8_field).is_err());
           
           // 重复测试其他整数类型...
       }
   }
   ```

2. **集成测试实现**:
   ```rust
   // tests/integration_tests.rs
   
   #[cfg(test)]
   mod default_value_integration_tests {
       /// 测试完整的 Node 派生与默认值
       /// 
       /// # 测试目标
       /// 验证从宏属性解析到代码生成的完整流程
       #[test]
       fn test_complete_node_generation_pipeline() {
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
                   
                   #[attr(default = r#"{"theme": "light", "size": 12}"#)]
                   config: serde_json::Value,
                   
                   #[attr(default = "null")]
                   author: Option<String>,
                   
                   #[attr]
                   without_default: Option<String>,
               }
           };
           
           // 执行完整的代码生成流程
           let result = process_derive_node_with_recovery(input);
           assert!(result.is_ok(), "代码生成应该成功");
           
           let generated = result.unwrap();
           let code = generated.to_string();
           
           // 验证生成的方法存在
           assert!(code.contains("pub fn to_node"), "应该生成 to_node 方法");
           assert!(code.contains("pub fn new"), "应该生成 new 方法");
           
           // 验证默认值被正确使用
           assert!(code.contains("默认内容"), "应该包含字符串默认值");
           assert!(code.contains("16"), "应该包含数值默认值");
           assert!(code.contains("true"), "应该包含布尔默认值");
           assert!(code.contains("theme"), "应该包含 JSON 默认值");
           assert!(code.contains("light"), "应该包含 JSON 内容");
           
           // 验证生成代码的结构
           verify_generated_code_structure(&code);
       }
       
       fn verify_generated_code_structure(code: &str) {
           // 验证 to_node 方法结构
           assert!(code.contains("serde_json::Value::String"), "字符串应该转换为 JSON");
           assert!(code.contains("serde_json::Value::Number"), "数值应该转换为 JSON");
           assert!(code.contains("serde_json::Value::Bool"), "布尔值应该转换为 JSON");
           
           // 验证构造函数结构
           assert!(code.contains("Self {"), "构造函数应该有正确的结构");
           
           // 验证错误处理
           assert!(!code.contains("unwrap()"), "生成的代码不应该包含 unwrap");
       }
       
       /// 测试向后兼容性
       #[test]
       fn test_backward_compatibility_comprehensive() {
           // 测试现有代码完全不受影响
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
           assert!(result.is_ok(), "现有代码应该继续工作");
           
           let generated = result.unwrap();
           let code = generated.to_string();
           
           // 验证现有行为保持不变
           assert!(code.contains("pub fn to_node"), "应该生成 to_node 方法");
           assert!(!code.contains("pub fn new"), "无默认值时不应该生成 new 方法");
           assert!(!code.contains("default"), "不应该包含默认值相关代码");
           
           // 验证方法签名保持一致
           verify_method_signatures_unchanged(&code);
       }
   }
   ```

3. **性能测试实现**:
   ```rust
   // tests/performance_tests.rs
   
   #[cfg(test)]
   mod performance_benchmarks {
       use std::time::Instant;
       
       /// 编译时性能基准测试
       #[test]
       fn test_compilation_performance() {
           let test_cases = generate_performance_test_cases(100);
           
           // 基准测试：无默认值的情况
           let start_baseline = Instant::now();
           for case in &test_cases {
               let _ = process_node_without_defaults(case);
           }
           let baseline_duration = start_baseline.elapsed();
           
           // 测试：有默认值的情况
           let start_with_defaults = Instant::now();
           for case in &test_cases {
               let _ = process_node_with_defaults(case);
           }
           let with_defaults_duration = start_with_defaults.elapsed();
           
           // 验证性能要求
           let performance_overhead = with_defaults_duration.as_millis() as f64 / baseline_duration.as_millis() as f64;
           assert!(performance_overhead < 1.1, "性能开销应该小于 10%: 实际 {:.1}%", (performance_overhead - 1.0) * 100.0);
           
           println!("基准时间: {:?}", baseline_duration);
           println!("带默认值时间: {:?}", with_defaults_duration);
           println!("性能开销: {:.1}%", (performance_overhead - 1.0) * 100.0);
       }
       
       /// 内存使用测试
       #[test]
       fn test_memory_usage() {
           let initial_memory = get_memory_usage();
           
           // 处理大量带默认值的结构体
           let large_test_cases = generate_large_test_cases(1000);
           for case in large_test_cases {
               let _ = process_derive_node_with_recovery(case);
           }
           
           let final_memory = get_memory_usage();
           let memory_increase = final_memory - initial_memory;
           
           assert!(memory_increase < 20 * 1024 * 1024, "内存增加应该小于 20MB: 实际 {}MB", memory_increase / 1024 / 1024);
           
           println!("初始内存: {}MB", initial_memory / 1024 / 1024);
           println!("最终内存: {}MB", final_memory / 1024 / 1024);
           println!("内存增加: {}MB", memory_increase / 1024 / 1024);
       }
       
       /// 验证器性能测试
       #[test]
       fn test_validator_performance() {
           let registry = ValidatorRegistry::new();
           let test_validations = generate_validation_test_cases(10000);
           
           let start = Instant::now();
           for (default_value, field_type) in test_validations {
               let _ = registry.validate(&default_value, &field_type);
           }
           let duration = start.elapsed();
           
           let average_time = duration.as_micros() / 10000;
           assert!(average_time < 1000, "平均验证时间应该小于 1ms: 实际 {}μs", average_time);
           
           println!("10000 次验证总时间: {:?}", duration);
           println!("平均验证时间: {}μs", average_time);
       }
   }
   ```

4. **错误处理测试**:
   ```rust
   // tests/error_handling_tests.rs
   
   #[cfg(test)]
   mod error_message_quality_tests {
       /// 测试错误消息的友好性和有用性
       #[test]
       fn test_friendly_error_messages() {
           // 类型不匹配错误
           test_type_mismatch_error_message();
           
           // JSON 类型约束错误
           test_json_constraint_error_message();
           
           // 范围错误
           test_numeric_range_error_message();
       }
       
       fn test_type_mismatch_error_message() {
           let input = parse_quote! {
               #[derive(Node)]
               #[node_type = "error_node"]
               pub struct ErrorNode {
                   #[attr(default = "not_a_number")]
                   number_field: i32,
               }
           };
           
           let result = process_derive_node_with_recovery(input);
           assert!(result.is_err());
           
           let error = result.unwrap_err();
           let message = error.to_friendly_message();
           
           // 验证错误消息质量
           assert!(message.contains("类型不匹配"), "应该说明错误类型");
           assert!(message.contains("number_field"), "应该包含字段名");
           assert!(message.contains("i32"), "应该包含期望类型");
           assert!(message.contains("修复建议"), "应该包含修复建议");
           assert!(message.contains("示例"), "应该包含示例");
           
           println!("类型不匹配错误消息:\n{}", message);
       }
       
       fn test_json_constraint_error_message() {
           let input = parse_quote! {
               #[derive(Node)]
               #[node_type = "json_error_node"]
               pub struct JsonErrorNode {
                   #[attr(default = r#"{"key": "value"}"#)]
                   not_json_field: String,
               }
           };
           
           let result = process_derive_node_with_recovery(input);
           assert!(result.is_err());
           
           let error = result.unwrap_err();
           let message = error.to_friendly_message();
           
           // 验证 JSON 约束错误消息
           assert!(message.contains("JSON 默认值"), "应该说明 JSON 约束");
           assert!(message.contains("serde_json::Value"), "应该提及正确类型");
           assert!(message.contains("解决方案"), "应该提供解决方案");
           
           println!("JSON 约束错误消息:\n{}", message);
       }
   }
   
   #[cfg(test)]
   mod error_recovery_tests {
       /// 测试错误恢复机制
       #[test]
       fn test_multiple_errors_recovery() {
           let input = parse_quote! {
               #[derive(Node)]
               #[node_type = "multi_error_node"]
               pub struct MultiErrorNode {
                   #[attr(default = "not_a_number")]
                   number_field: i32,
                   
                   #[attr(default = r#"{"invalid": json"#)]
                   json_field: serde_json::Value,
                   
                   #[attr(default = "1000")]
                   small_number: i8,  // 超出范围
               }
           };
           
           let result = process_derive_node_with_recovery(input);
           
           // 应该收集到多个错误
           if let Err(errors) = result {
               assert!(errors.len() >= 3, "应该收集到至少 3 个错误");
               
               // 验证错误类型多样性
               let error_types: Vec<_> = errors.iter().map(|e| e.error_type()).collect();
               assert!(error_types.contains(&"DefaultValueTypeMismatch".to_string()));
               assert!(error_types.contains(&"InvalidJsonDefaultValue".to_string()));
           } else {
               panic!("应该返回错误结果");
           }
       }
   }
   ```

**验收标准**:
- [ ] 测试覆盖率 ≥ 95%
- [ ] 所有功能的正面和负面测试用例
- [ ] 性能基准测试验证性能要求
- [ ] 错误消息质量测试通过
- [ ] 向后兼容性测试 100% 通过
- [ ] 边界情况和异常情况测试覆盖

**技术要求**:
- 使用 `trybuild` 进行编译失败测试
- 性能测试提供具体的时间和内存数据
- 错误测试验证消息的友好性和有用性
- 集成测试覆盖完整的使用场景

**测试要求**:
- 单元测试：每个模块的详细功能测试
- 集成测试：端到端的完整流程测试
- 性能测试：编译时间和内存使用测试
- 错误测试：各种错误情况的处理测试

---

#### 任务 TASK-011: 文档和示例完善
**优先级**: P0 (必须完成)
**预估工时**: 2天
**依赖关系**: TASK-010
**负责模块**: `docs/`, `examples/`

**任务描述**:
完善项目文档，提供详细的使用指南、API 文档和示例代码。

**具体工作项**:
1. **API 文档完善**:
   ```rust
   //! # ModuForge-RS Default 属性扩展
   //! 
   //! 为 ModuForge-RS 的 Node 和 Mark 宏提供声明式的默认值支持。
   //! 
   //! ## 核心功能
   //! 
   //! - **声明式默认值**: 通过 `#[attr(default="value")]` 语法设置字段默认值
   //! - **编译时验证**: 严格的类型检查，确保默认值与字段类型匹配
   //! - **智能代码生成**: 自动生成支持默认值的构造函数和转换方法
   //! - **向后兼容**: 现有代码无需修改即可使用
   //! 
   //! ## 快速开始
   //! 
   //! ```rust
   //! use moduforge_derive::Node;
   //! use serde::{Serialize, Deserialize};
   //! 
   //! #[derive(Node, Serialize, Deserialize)]
   //! #[node_type = "paragraph"]
   //! pub struct Paragraph {
   //!     #[attr(default = "默认内容")]
   //!     content: String,
   //!     
   //!     #[attr(default = "16")]
   //!     font_size: i32,
   //!     
   //!     #[attr(default = "true")]
   //!     visible: bool,
   //!     
   //!     #[attr]
   //!     author: Option<String>,
   //! }
   //! 
   //! // 使用默认值创建实例
   //! let paragraph = Paragraph::new();
   //! assert_eq!(paragraph.content, "默认内容");
   //! assert_eq!(paragraph.font_size, 16);
   //! assert_eq!(paragraph.visible, true);
   //! assert_eq!(paragraph.author, None);
   //! 
   //! // 转换为 Node（支持默认值）
   //! let node = paragraph.to_node();
   //! ```
   //! 
   //! ## 支持的类型
   //! 
   //! ### 基本类型
   //! 
   //! ```rust
   //! #[derive(Node)]
   //! #[node_type = "example"]
   //! pub struct Example {
   //!     // 字符串类型
   //!     #[attr(default = "hello world")]
   //!     text: String,
   //!     
   //!     // 整数类型
   //!     #[attr(default = "42")]
   //!     count: i32,
   //!     
   //!     // 浮点数类型
   //!     #[attr(default = "3.14")]
   //!     pi: f64,
   //!     
   //!     // 布尔类型
   //!     #[attr(default = "true")]
   //!     enabled: bool,
   //! }
   //! ```
   //! 
   //! ### JSON 类型（复杂默认值）
   //! 
   //! ```rust
   //! #[derive(Node)]
   //! #[node_type = "config"]
   //! pub struct ConfigNode {
   //!     #[attr(default = r#"{"theme": "dark", "size": 12}"#)]
   //!     settings: serde_json::Value,
   //!     
   //!     #[attr(default = r#"["option1", "option2"]"#)]
   //!     options: serde_json::Value,
   //! }
   //! ```
   //! 
   //! ### Option 类型
   //! 
   //! ```rust
   //! #[derive(Node)]
   //! #[node_type = "article"]
   //! pub struct Article {
   //!     // None 默认值
   //!     #[attr(default = "null")]
   //!     author: Option<String>,
   //!     
   //!     // Some 默认值
   //!     #[attr(default = "未命名")]
   //!     title: Option<String>,
   //! }
   //! ```
   //! 
   //! ## 设计原则
   //! 
   //! 本扩展严格遵循以下设计原则：
   //! 
   //! - **单一职责原则**: 每个模块专注于特定的功能领域
   //! - **开闭原则**: 支持扩展而不修改现有代码
   //! - **里氏替换原则**: 新功能完全兼容现有接口
   //! - **接口隔离原则**: 提供最小化、专用的接口
   //! - **依赖倒置原则**: 依赖抽象而非具体实现
   //! 
   //! ## 性能特点
   //! 
   //! - **零运行时开销**: 所有默认值处理在编译期完成
   //! - **编译时验证**: 类型错误在编译时被捕获
   //! - **智能优化**: 自动选择最优的代码生成策略
   //! - **缓存优化**: 常用类型信息被缓存以提高编译速度
   
   /// 默认值表示
   /// 
   /// 存储解析后的默认值信息，包括原始值、类型化值和元数据。
   /// 
   /// # 设计原则体现
   /// 
   /// - **单一职责**: 专门负责默认值的数据表示
   /// - **不可变性**: 创建后不可修改，确保数据一致性
   /// 
   /// # 使用示例
   /// 
   /// ```rust
   /// use moduforge_derive::parser::default_value::*;
   /// 
   /// // 解析字符串默认值
   /// let default_value = DefaultValueParser::parse("hello", None)?;
   /// assert!(matches!(default_value.value_type, DefaultValueType::String(_)));
   /// 
   /// // 解析数值默认值
   /// let default_value = DefaultValueParser::parse("42", None)?;
   /// assert!(matches!(default_value.value_type, DefaultValueType::Integer(42)));
   /// ```
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
   ```

2. **使用指南编写**:
   ```markdown
   # ModuForge-RS Default 属性扩展使用指南
   
   ## 介绍
   
   这个扩展为 ModuForge-RS 的 Node 和 Mark 宏添加了声明式的默认值支持，让你可以通过简单的属性声明来设置字段的默认值，而不需要手写复杂的初始化代码。
   
   ## 安装和设置
   
   这个功能已经集成到 `moduforge-macros-derive` 库中，无需额外安装。确保你的 `Cargo.toml` 包含：
   
   ```toml
   [dependencies]
   moduforge-macros-derive = { version = "0.3.0", features = ["default-values"] }
   ```
   
   ## 基础用法
   
   ### 简单默认值
   
   最常见的用法是为基本类型设置默认值：
   
   ```rust
   use moduforge_derive::Node;
   
   #[derive(Node)]
   #[node_type = "document"]
   pub struct Document {
       #[attr(default = "新文档")]
       title: String,
       
       #[attr(default = "0")]
       word_count: i32,
       
       #[attr(default = "true")]
       auto_save: bool,
   }
   
   // 创建带默认值的实例
   let doc = Document::new();
   assert_eq!(doc.title, "新文档");
   assert_eq!(doc.word_count, 0);
   assert_eq!(doc.auto_save, true);
   ```
   
   ### 复杂默认值（JSON）
   
   对于复杂的配置数据，可以使用 JSON 格式的默认值：
   
   ```rust
   #[derive(Node)]
   #[node_type = "editor"]
   pub struct Editor {
       #[attr(default = r#"{
           "theme": "dark",
           "fontSize": 14,
           "wordWrap": true,
           "minimap": {
               "enabled": true,
               "side": "right"
           }
       }"#)]
       settings: serde_json::Value,
   }
   ```
   
   **注意**: JSON 格式的默认值只能用于 `serde_json::Value` 类型的字段。
   
   ### 可选字段（Option）
   
   Option 类型的字段支持两种默认值：
   
   ```rust
   #[derive(Node)]
   #[node_type = "user"]
   pub struct User {
       // 默认为 None
       #[attr(default = "null")]
       avatar: Option<String>,
       
       // 默认为 Some("游客")
       #[attr(default = "游客")]
       display_name: Option<String>,
   }
   ```
   
   ## 高级用法
   
   ### 混合使用新旧语法
   
   你可以在同一个结构体中混合使用有默认值和无默认值的字段：
   
   ```rust
   #[derive(Node)]
   #[node_type = "article"]
   pub struct Article {
       #[attr(default = "未命名文章")]
       title: String,
       
       #[attr(default = "0")]
       view_count: i32,
       
       #[attr]  // 无默认值，保持现有行为
       author: String,
       
       #[attr]  // 无默认值
       published_at: Option<chrono::DateTime<chrono::Utc>>,
   }
   ```
   
   ### 数值类型的范围注意事项
   
   编译器会验证数值默认值是否在目标类型的范围内：
   
   ```rust
   #[derive(Node)]
   #[node_type = "counter"]
   pub struct Counter {
       #[attr(default = "100")]
       small_value: i8,  // ✓ 正确：100 在 i8 范围内
       
       // #[attr(default = "1000")]
       // small_value: i8,  // ✗ 错误：1000 超出 i8 范围 (-128 到 127)
   }
   ```
   
   ## 生成的方法
   
   当你的结构体包含默认值字段时，宏会自动生成以下方法：
   
   ### `new()` 方法
   
   使用所有默认值创建实例：
   
   ```rust
   let instance = MyStruct::new();
   ```
   
   ### 增强的 `to_node()` 方法
   
   转换为 Node 时会智能处理默认值：
   
   ```rust
   let node = instance.to_node();
   // 如果字段值为空或未设置，会自动使用默认值
   ```
   
   ## 错误处理和调试
   
   ### 常见错误及解决方法
   
   #### 1. 类型不匹配错误
   
   ```
   error: 默认值类型不匹配: 字段 'age' 类型为 'i32'，但默认值 'abc' 不兼容
   ```
   
   **解决方法**: 确保默认值与字段类型匹配：
   ```rust
   #[attr(default = "25")]  // ✓ 正确
   age: i32,
   
   // #[attr(default = "abc")]  // ✗ 错误
   // age: i32,
   ```
   
   #### 2. JSON 类型约束错误
   
   ```
   error: JSON 默认值只能用于 serde_json::Value 类型字段
   ```
   
   **解决方法**: JSON 格式的默认值只能用于 `serde_json::Value` 类型：
   ```rust
   #[attr(default = r#"{"key": "value"}"#)]  // ✓ 正确
   config: serde_json::Value,
   
   // #[attr(default = r#"{"key": "value"}"#)]  // ✗ 错误
   // config: String,
   ```
   
   ### 调试技巧
   
   1. **使用 `cargo expand`** 查看生成的代码：
      ```bash
      cargo expand --bin your_binary
      ```
   
   2. **启用详细编译日志**：
      ```bash
      MODUFORGE_DERIVE_VERBOSE=1 cargo build
      ```
   
   3. **查看性能统计**：
      ```bash
      MODUFORGE_DERIVE_PERF=1 cargo build
      ```
   
   ## 最佳实践
   
   ### 1. 默认值的选择
   
   - **字符串**: 选择有意义的默认值，避免空字符串
   - **数值**: 选择合理的初始值，考虑业务逻辑
   - **布尔值**: 根据功能的默认状态选择 true 或 false
   - **JSON**: 保持结构简单，避免过度嵌套
   
   ### 2. 性能考虑
   
   - 简单类型的默认值处理开销极小
   - JSON 默认值会在编译时解析，运行时无额外开销
   - 避免在默认值中使用过大的 JSON 对象
   
   ### 3. 维护性
   
   - 为复杂的默认值添加注释说明
   - 定期检查默认值是否仍然合理
   - 考虑将常用的默认值定义为常量
   
   ## 迁移指南
   
   ### 从现有代码迁移
   
   现有的代码无需任何修改即可继续工作：
   
   ```rust
   // 现有代码（继续工作）
   #[derive(Node)]
   #[node_type = "paragraph"]
   pub struct Paragraph {
       #[attr]
       content: String,
       
       #[attr]
       author: Option<String>,
   }
   
   // 逐步添加默认值
   #[derive(Node)]
   #[node_type = "paragraph"]
   pub struct Paragraph {
       #[attr(default = "")]  // 添加默认值
       content: String,
       
       #[attr]  // 保持现有行为
       author: Option<String>,
   }
   ```
   
   ### 迁移策略
   
   1. **阶段1**: 保持现有代码不变，验证兼容性
   2. **阶段2**: 为新字段添加默认值
   3. **阶段3**: 逐步为现有字段添加合适的默认值
   4. **阶段4**: 利用生成的构造函数简化代码
   
   ## 故障排除
   
   ### 常见问题 FAQ
   
   **Q: 为什么我的结构体没有生成 `new()` 方法？**
   A: `new()` 方法只在结构体包含至少一个有默认值的字段时生成。
   
   **Q: 可以为自定义类型设置默认值吗？**
   A: 目前只支持基本类型（String、数值、bool）和 `serde_json::Value`。自定义类型支持在未来版本中考虑。
   
   **Q: 默认值会影响性能吗？**
   A: 不会。所有默认值处理都在编译期完成，运行时没有额外开销。
   
   **Q: 可以使用表达式作为默认值吗？**
   A: 目前只支持字面量。表达式默认值是未来的功能规划。
   ```

3. **示例代码库**:
   ```rust
   // examples/basic_usage.rs
   
   //! 基本使用示例
   //! 
   //! 展示 ModuForge-RS 默认值功能的基本用法
   
   use moduforge_derive::{Node, Mark};
   use serde::{Serialize, Deserialize};
   
   /// 基本的文档节点示例
   #[derive(Node, Serialize, Deserialize, Debug)]
   #[node_type = "document"]
   pub struct Document {
       /// 文档标题，默认为"新文档"
       #[attr(default = "新文档")]
       pub title: String,
       
       /// 字数统计，默认为0
       #[attr(default = "0")]
       pub word_count: i32,
       
       /// 是否自动保存，默认启用
       #[attr(default = "true")]
       pub auto_save: bool,
       
       /// 创建时间，无默认值
       #[attr]
       pub created_at: Option<String>,
   }
   
   /// 强调标记示例
   #[derive(Mark, Serialize, Deserialize, Debug)]
   #[mark_type = "emphasis"]
   pub struct EmphasisMark {
       /// 强调样式，默认为"normal"
       #[attr(default = "normal")]
       pub style: String,
       
       /// 权重值，默认为1.0
       #[attr(default = "1.0")]
       pub weight: f64,
       
       /// 是否斜体，默认为false
       #[attr(default = "false")]
       pub italic: bool,
   }
   
   fn main() -> Result<(), Box<dyn std::error::Error>> {
       println!("=== ModuForge-RS 默认值功能示例 ===\n");
       
       // 基本用法：使用默认值创建实例
       println!("1. 使用默认值创建文档:");
       let doc = Document::new();
       println!("   标题: {}", doc.title);
       println!("   字数: {}", doc.word_count);
       println!("   自动保存: {}", doc.auto_save);
       println!("   创建时间: {:?}", doc.created_at);
       
       // 转换为 Node
       println!("\n2. 转换为 Node:");
       let node = doc.to_node();
       println!("   Node 类型: {}", node.node_type());
       println!("   属性数量: {}", node.attributes().len());
       
       // 创建标记
       println!("\n3. 创建强调标记:");
       let mark = EmphasisMark::new();
       println!("   样式: {}", mark.style);
       println!("   权重: {}", mark.weight);
       println!("   斜体: {}", mark.italic);
       
       // 转换为 Mark
       let mark_obj = mark.to_mark();
       println!("   Mark 类型: {}", mark_obj.mark_type());
       
       Ok(())
   }
   ```

   ```rust
   // examples/advanced_defaults.rs
   
   //! 高级默认值示例
   //! 
   //! 展示 JSON 默认值、Option 类型等高级用法
   
   use moduforge_derive::Node;
   use serde::{Serialize, Deserialize};
   use serde_json::Value;
   
   /// 编辑器配置节点
   #[derive(Node, Serialize, Deserialize, Debug)]
   #[node_type = "editor_config"]
   pub struct EditorConfig {
       /// 编辑器设置（JSON 格式）
       #[attr(default = r#"{
           "theme": "dark",
           "fontSize": 14,
           "fontFamily": "Consolas",
           "wordWrap": true,
           "lineNumbers": true,
           "minimap": {
               "enabled": true,
               "side": "right"
           },
           "editor": {
               "tabSize": 4,
               "insertSpaces": true
           }
       }"#)]
       pub settings: Value,
       
       /// 插件列表（JSON 数组）
       #[attr(default = r#"[
           "syntax-highlighting",
           "auto-completion",
           "code-folding"
       ]"#)]
       pub plugins: Value,
       
       /// 用户偏好设置
       #[attr(default = r#"{"language": "zh-CN", "autoSave": true}"#)]
       pub preferences: Value,
   }
   
   /// 用户配置文件
   #[derive(Node, Serialize, Deserialize, Debug)]
   #[node_type = "user_profile"]
   pub struct UserProfile {
       /// 用户名，默认为"游客"
       #[attr(default = "游客")]
       pub username: String,
       
       /// 头像URL，默认为None
       #[attr(default = "null")]
       pub avatar_url: Option<String>,
       
       /// 显示名称，默认为"未命名用户"
       #[attr(default = "未命名用户")]
       pub display_name: Option<String>,
       
       /// 电子邮件，无默认值
       #[attr]
       pub email: Option<String>,
       
       /// 年龄，默认为18
       #[attr(default = "18")]
       pub age: Option<i32>,
   }
   
   /// 数值范围示例
   #[derive(Node, Serialize, Deserialize, Debug)]
   #[node_type = "numeric_example"]
   pub struct NumericExample {
       /// 小整数
       #[attr(default = "100")]
       pub small_int: i8,  // -128 到 127
       
       /// 大整数
       #[attr(default = "1000000")]
       pub large_int: i64,
       
       /// 无符号整数
       #[attr(default = "255")]
       pub unsigned_int: u8,  // 0 到 255
       
       /// 单精度浮点数
       #[attr(default = "3.14")]
       pub float_val: f32,
       
       /// 双精度浮点数
       #[attr(default = "2.718281828")]
       pub double_val: f64,
   }
   
   fn main() -> Result<(), Box<dyn std::error::Error>> {
       println!("=== 高级默认值功能示例 ===\n");
       
       // JSON 默认值示例
       println!("1. JSON 默认值:");
       let config = EditorConfig::new();
       
       println!("   编辑器设置:");
       if let Value::Object(settings) = &config.settings {
           for (key, value) in settings {
               println!("     {}: {}", key, value);
           }
       }
       
       println!("\n   插件列表:");
       if let Value::Array(plugins) = &config.plugins {
           for plugin in plugins {
               println!("     - {}", plugin);
           }
       }
       
       // Option 类型默认值示例
       println!("\n2. Option 类型默认值:");
       let profile = UserProfile::new();
       println!("   用户名: {}", profile.username);
       println!("   头像URL: {:?}", profile.avatar_url);
       println!("   显示名称: {:?}", profile.display_name);
       println!("   电子邮件: {:?}", profile.email);
       println!("   年龄: {:?}", profile.age);
       
       // 数值类型示例
       println!("\n3. 数值类型示例:");
       let numbers = NumericExample::new();
       println!("   小整数 (i8): {}", numbers.small_int);
       println!("   大整数 (i64): {}", numbers.large_int);
       println!("   无符号整数 (u8): {}", numbers.unsigned_int);
       println!("   单精度浮点数 (f32): {}", numbers.float_val);
       println!("   双精度浮点数 (f64): {}", numbers.double_val);
       
       // 转换测试
       println!("\n4. Node 转换测试:");
       let config_node = config.to_node();
       let profile_node = profile.to_node();
       let numbers_node = numbers.to_node();
       
       println!("   配置节点属性数: {}", config_node.attributes().len());
       println!("   用户节点属性数: {}", profile_node.attributes().len());
       println!("   数值节点属性数: {}", numbers_node.attributes().len());
       
       Ok(())
   }
   ```

**验收标准**:
- [ ] API 文档完整，包含所有公共接口
- [ ] 使用指南详细，覆盖所有功能和用法
- [ ] 示例代码可运行，演示各种使用场景
- [ ] 错误处理文档包含常见问题和解决方案
- [ ] 迁移指南提供平滑的升级路径
- [ ] 文档的用户友好度和可读性高

**技术要求**:
- 使用 `cargo doc` 生成 API 文档
- 示例代码必须能够编译和运行
- 文档格式符合 Rust 社区标准
- 提供中英文双语文档

**测试要求**:
- 文档中的代码示例必须通过 doctest
- 示例程序的集成测试
- 文档链接和引用的准确性测试
- 用户体验测试（可读性、完整性）

---

## 3. 项目管理

### 3.1 里程碑规划

#### 里程碑 M1: 基础架构完成（第2周末）
**交付物**:
- [ ] 默认值数据结构设计完成（TASK-001）
- [ ] FieldConfig 结构扩展完成（TASK-002）
- [ ] 属性解析器增强完成（TASK-003）
- [ ] 验证器系统基础架构完成（TASK-004）

**验收标准**:
- 基础数据结构定义完整且类型安全
- 属性解析支持新语法且保持向后兼容
- 验证器接口设计合理且可扩展
- 所有现有测试继续通过

#### 里程碑 M2: 核心功能实现（第4周末）
**交付物**:
- [ ] 类型验证器实现完成（TASK-005）
- [ ] 错误处理系统扩展完成（TASK-006）
- [ ] 代码生成器增强完成（TASK-007）

**验收标准**:
- 所有规划的类型都有对应的验证器
- 错误消息友好且包含修复建议
- 生成的代码类型安全且性能优良
- 基本功能端到端测试通过

#### 里程碑 M3: 性能和扩展性（第6周末）
**交付物**:
- [ ] 性能优化实现完成（TASK-008）
- [ ] 可扩展性架构实现完成（TASK-009）

**验收标准**:
- 编译时性能满足要求（< 10% 增加）
- 内存使用满足要求（< 20MB 增加）
- 验证器和模板系统支持插件扩展
- 性能基准测试通过

#### 里程碑 M4: 项目完成（第8周末）
**交付物**:
- [ ] 全面测试套件实现完成（TASK-010）
- [ ] 文档和示例完善完成（TASK-011）

**验收标准**:
- 测试覆盖率 ≥ 95%
- 所有功能的正面和负面测试
- API 文档完整且示例可运行
- 用户指南详细且易懂

### 3.2 风险管理

#### 高风险项目
1. **TASK-005 (类型验证器实现)**
   - **风险**: 复杂的类型系统兼容性问题
   - **缓解**: 建立全面的类型测试用例，分阶段实现

2. **TASK-007 (代码生成器增强)**
   - **风险**: 生成代码的正确性和兼容性
   - **缓解**: 大量的集成测试和现有代码验证

3. **TASK-008 (性能优化)**
   - **风险**: 性能目标可能难以达成
   - **缓解**: 早期性能基准测试，渐进式优化

#### 中风险项目
1. **TASK-004 (验证器系统架构)**
   - **风险**: 架构设计复杂性
   - **缓解**: 参考现有模式，简化设计

2. **TASK-009 (可扩展性架构)**
   - **风险**: 过度设计影响性能
   - **缓解**: 平衡扩展性和性能，渐进式实现

### 3.3 质量保证

#### 代码质量标准
- **测试覆盖率**: ≥ 95%
- **文档覆盖率**: ≥ 90%
- **中文注释**: 100% 覆盖公共 API
- **性能基准**: 所有性能要求必须满足

#### 代码审查要求
- 每个任务完成后进行代码审查
- 重点检查设计原则的遵循情况
- 验证向后兼容性
- 检查错误处理的完整性

#### 持续集成检查
- 编译检查（所有 Rust 版本）
- 测试执行（单元测试、集成测试）
- 性能基准测试
- 文档生成和 doctest

### 3.4 交付清单

#### 最终交付物
- [ ] **源代码**: 完整的功能实现，包含详细中文注释
- [ ] **测试套件**: 全面的单元测试、集成测试、性能测试
- [ ] **文档**: API 文档、使用指南、示例代码
- [ ] **性能报告**: 基准测试结果和性能分析
- [ ] **兼容性报告**: 向后兼容性验证结果

#### 质量检查清单
- [ ] 所有 P0 任务 100% 完成
- [ ] 所有 P1 任务 ≥ 80% 完成
- [ ] 测试覆盖率 ≥ 95%
- [ ] 性能指标满足要求
- [ ] 向后兼容性 100% 保证
- [ ] 文档完整且用户友好
- [ ] 代码质量符合项目标准

---

*此精确的开发任务清单为 ModuForge-RS Default 属性扩展项目提供了详细的实施指南，确保项目能够按计划高质量完成，同时严格遵循核心设计原则和技术要求。*
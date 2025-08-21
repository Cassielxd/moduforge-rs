# ModuForge-RS 宏模块扩展 - 开发任务清单

## 项目概述

基于现有的 `mf-derive` 模块，扩展实现 `#[derive(Node)]` 和 `#[derive(Mark)]` 派生宏功能，为 ModuForge-RS 框架提供声明式的节点和标记定义能力。

## 总体开发策略

### 核心设计原则遵循
- **单一职责原则 (SRP)**: 每个模块只负责一个明确的功能
- **接口隔离原则 (ISP)**: 提供精简、专用的接口
- **开闭原则 (OCP)**: 通过插件系统支持扩展
- **里氏替换原则 (LSP)**: 确保实现类型的可替换性

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

## 开发阶段规划

### 第一阶段：基础架构搭建 (Priority: P0)

#### T001: 更新 mf-derive 模块配置
**负责模块**: mf-derive/Cargo.toml, mf-derive/src/lib.rs  
**预估工时**: 4小时  
**复杂度**: 低  

**具体任务**:
1. **依赖配置更新**
   - 添加 `serde = { version = "1.0", features = ["derive"] }`
   - 添加 `serde_json = "1.0"`
   - 添加 `once_cell = "1.19"` (用于类型转换缓存)
   - 确保现有依赖 `syn`, `quote`, `proc-macro2` 版本兼容

2. **Cargo.toml 元信息更新**
   ```toml
   description = "ModuForge-RS 宏扩展模块，提供 Node 和 Mark 的派生宏"
   ```

3. **lib.rs 入口点扩展**
   - 保留现有 `derive_plugin_state` 功能
   - 添加新的派生宏导出声明
   - 设置模块结构导入

**验收标准**:
- [ ] Cargo.toml 配置正确，通过 `cargo check`
- [ ] 模块结构清晰，符合设计文档
- [ ] 现有功能不受影响

**前置条件**: 无  
**依赖任务**: 无

---

#### T002: 创建通用模块 (common/)
**负责模块**: mf-derive/src/common/  
**预估工时**: 8小时  
**复杂度**: 中等

**具体任务**:
1. **错误处理系统 (common/error.rs)**
   - 定义 `MacroError` 枚举类型
   - 实现友好的编译时错误消息
   - 提供错误位置定位和修复建议
   - 实现 `to_compile_error()` 方法

2. **工具函数库 (common/utils.rs)**
   - 实现 `is_option_type()` 函数
   - 实现 `extract_option_inner_type()` 函数  
   - 实现 `generate_field_conversion()` 函数
   - 实现 `generate_imports()` 函数

3. **常量定义 (common/constants.rs)**
   - 支持的基本类型列表
   - 默认属性名称映射
   - 错误消息模板

**生成代码示例**:
```rust
/// 宏处理过程中的错误类型
#[derive(Error, Debug)]
pub enum MacroError {
    #[error("缺少必需的宏属性: {attribute}")]
    MissingAttribute { attribute: String },
    
    #[error("不支持的字段类型 '{field_type}' 在字段 '{field_name}' 中")]
    UnsupportedFieldType { field_name: String, field_type: String },
    // ...其他错误类型
}
```

**验收标准**:
- [ ] 错误类型覆盖所有预期场景
- [ ] 工具函数通过单元测试
- [ ] 错误消息友好且具有指导性
- [ ] 代码符合项目规范，包含完整中文注释

**前置条件**: T001完成  
**依赖任务**: T001

---

#### T003: 实现属性解析模块 (parser/)
**负责模块**: mf-derive/src/parser/  
**预估工时**: 12小时  
**复杂度**: 中等

**具体任务**:
1. **属性解析器 (parser/attribute_parser.rs)**
   - 实现 `NodeAttributes` 配置结构体
   - 实现 `MarkAttributes` 配置结构体
   - 实现 `FieldAttributes` 配置结构体
   - 实现属性解析逻辑，支持字符串值解析

2. **字段分析器 (parser/field_analyzer.rs)**
   - 分析结构体字段的类型信息
   - 识别 `#[attr]` 标注的字段
   - 提取字段名称和类型信息

3. **验证逻辑 (parser/validation.rs)**
   - 验证 `node_type` 和 `mark_type` 格式
   - 验证字段类型兼容性
   - 验证属性配置的完整性

**核心接口设计**:
```rust
/// Node 属性配置
#[derive(Debug, Clone, Default)]
pub struct NodeAttributes {
    pub node_type: Option<String>,
    pub marks: Option<String>,
    pub content: Option<String>,
    pub attr_fields: Vec<FieldConfig>,
}

/// 属性解析器
impl AttributeParser {
    pub fn parse_node_attributes(input: &DeriveInput) -> MacroResult<NodeAttributes>;
    pub fn parse_mark_attributes(input: &DeriveInput) -> MacroResult<MarkAttributes>;
}
```

**验收标准**:
- [ ] 正确解析所有支持的属性类型
- [ ] 验证逻辑覆盖所有约束条件
- [ ] 错误场景提供清晰的错误消息
- [ ] 单元测试覆盖率 ≥ 90%
- [ ] 所有函数包含详细中文注释和使用示例

**前置条件**: T002完成  
**依赖任务**: T002

---

#### T004: 实现类型转换模块 (converter/)
**负责模块**: mf-derive/src/converter/  
**预估工时**: 10小时  
**复杂度**: 中等

**具体任务**:
1. **类型转换核心 (converter/type_converter.rs)**
   - 定义 `TypeConverter` trait (符合开闭原则)
   - 实现 `BuiltinTypeConverter` 基础转换器
   - 支持基本类型: String, i32, i64, f32, f64, bool
   - 支持可选类型: Option<T>

2. **内置转换器 (converter/builtin_converters.rs)**
   - 各种 Rust 基本类型到 `serde_json::Value` 的转换
   - 特殊类型处理 (如 Vec, HashMap 等)
   - 自定义序列化类型支持

3. **转换器注册表 (converter/converter_registry.rs)**
   - 全局转换器注册机制
   - 支持用户自定义转换器注册
   - 转换器优先级管理

**核心接口设计**:
```rust
/// 类型转换器 trait，支持扩展 (符合OCP)
pub trait TypeConverter {
    fn convert_field_to_json_value(&self, field: &Field) -> MacroResult<TokenStream2>;
    fn supports_type(&self, field_type: &Type) -> bool;
}

/// 转换器注册表
pub struct ConverterRegistry {
    converters: Vec<Box<dyn TypeConverter>>,
}
```

**验收标准**:
- [ ] 支持所有计划的基本类型转换
- [ ] 转换器接口设计符合开闭原则
- [ ] 注册机制支持运行时扩展
- [ ] 转换代码性能优化，避免不必要的分配
- [ ] 完整的类型转换测试套件
- [ ] 转换器代码包含详细中文文档

**前置条件**: T002完成  
**依赖任务**: T002

---

### 第二阶段：Node 派生宏实现 (Priority: P0)

#### T005: Node 代码生成器实现
**负责模块**: mf-derive/src/generator/node_generator.rs  
**预估工时**: 14小时  
**复杂度**: 高

**具体任务**:
1. **to_node() 方法生成**
   - 生成符合 mf-core API 的代码
   - 正确调用 `mf_core::node::Node::create()`
   - 集成 `NodeSpec` 构建逻辑

2. **属性设置代码生成**
   - 为 `#[attr]` 字段生成 `set_attr()` 调用
   - 处理可选字段的 None 值情况
   - 集成类型转换器调用

3. **NodeSpec 构建代码**
   - 根据 `content`, `marks` 属性生成 NodeSpec
   - 处理属性的默认值设置
   - 优化生成代码的性能

**生成代码示例**:
```rust
impl MyNode {
    /// 将结构体转换为 mf_core::node::Node 实例
    pub fn to_node(&self) -> mf_core::node::Node {
        use mf_model::node_type::NodeSpec;
        use std::collections::HashMap;
        
        // 构建属性映射
        let mut attrs = HashMap::with_capacity(2);
        attrs.insert("name".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::Value::String(self.name.clone()))
        });
        
        // 构建 NodeSpec
        let spec = NodeSpec {
            content: Some("*".to_string()),
            marks: Some("color".to_string()),
            attrs: Some(attrs),
            group: None,
            desc: None,
        };
        
        // 创建并返回 Node
        mf_core::node::Node::create("GCXM", spec)
    }
}
```

**验收标准**:
- [ ] 生成的代码通过编译且功能正确
- [ ] 与 mf-core API 完全兼容
- [ ] 支持所有计划的属性类型
- [ ] 生成代码性能优化，避免不必要分配
- [ ] 错误处理完整，提供友好错误消息
- [ ] 代码生成器包含详细中文注释

**前置条件**: T003, T004完成  
**依赖任务**: T003, T004

---

#### T006: Node 派生宏主实现
**负责模块**: mf-derive/src/node/derive_impl.rs  
**预估工时**: 8小时  
**复杂度**: 中等

**具体任务**:
1. **派生宏入口实现**
   - 实现 `process_derive_node()` 函数
   - 集成属性解析、验证、代码生成流程
   - 错误处理和编译时错误转换

2. **属性处理逻辑 (node/attribute_handler.rs)**
   - Node 特定的属性处理逻辑
   - 属性验证规则实现
   - 与通用解析器的集成

3. **代码生成集成 (node/code_gen.rs)**
   - 调用代码生成器
   - 最终 TokenStream 组装
   - 导入语句生成

**主要流程**:
```rust
pub fn process_derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    match process_node_derive_internal(&input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn process_node_derive_internal(input: &DeriveInput) -> MacroResult<proc_macro2::TokenStream> {
    // 1. 解析属性配置
    let config = AttributeParser::parse_node_attributes(input)?;
    
    // 2. 验证配置正确性
    Validator::validate_node_config(&config)?;
    
    // 3. 生成代码
    let generated = NodeGenerator::generate_to_node_method(input, &config)?;
    
    Ok(generated)
}
```

**验收标准**:
- [ ] 派生宏正确注册和导出
- [ ] 处理流程完整，包含所有步骤
- [ ] 错误处理覆盖所有异常情况
- [ ] 与其他模块集成正确
- [ ] 单元测试覆盖正常和异常场景
- [ ] 实现代码包含详细中文注释

**前置条件**: T005完成  
**依赖任务**: T005

---

### 第三阶段：Mark 派生宏实现 (Priority: P0)

#### T007: Mark 代码生成器实现
**负责模块**: mf-derive/src/generator/mark_generator.rs  
**预估工时**: 10小时  
**复杂度**: 中等

**具体任务**:
1. **to_mark() 方法生成**
   - 生成符合 mf-core Mark API 的代码
   - 正确调用 `mf_core::mark::Mark::new()`
   - 集成 `MarkSpec` 构建逻辑

2. **属性设置代码生成**
   - 为 `#[attr]` 字段生成 `set_attr()` 调用
   - 复用 Node 的类型转换逻辑
   - 处理 Mark 特定的属性约束

3. **MarkSpec 构建代码**
   - 根据 `mark_type` 属性生成 MarkSpec
   - 支持 excludes, group, spanning 属性
   - 优化代码生成性能

**生成代码示例**:
```rust
impl MyMark {
    /// 将结构体转换为 mf_core::mark::Mark 实例
    pub fn to_mark(&self) -> mf_core::mark::Mark {
        use mf_model::mark_type::MarkSpec;
        use std::collections::HashMap;
        
        // 构建属性映射
        let mut attrs = HashMap::with_capacity(1);
        attrs.insert("level".to_string(), mf_model::schema::AttributeSpec {
            default: Some(serde_json::Value::Number(
                serde_json::Number::from(self.level)
            ))
        });
        
        // 构建 MarkSpec
        let spec = MarkSpec {
            attrs: Some(attrs),
            excludes: None,
            group: None,
            spanning: None,
            desc: None,
        };
        
        // 创建并返回 Mark
        mf_core::mark::Mark::new("emphasis", spec)
    }
}
```

**验收标准**:
- [ ] 生成的代码通过编译且功能正确
- [ ] 与 mf-core Mark API 完全兼容
- [ ] 复用 Node 的类型转换逻辑
- [ ] 支持 Mark 特有的属性配置
- [ ] 生成代码性能优化
- [ ] 代码生成器包含详细中文注释

**前置条件**: T005完成  
**依赖任务**: T005

---

#### T008: Mark 派生宏主实现
**负责模块**: mf-derive/src/mark/derive_impl.rs  
**预估工时**: 6小时  
**复杂度**: 中等

**具体任务**:
1. **派生宏入口实现**
   - 实现 `process_derive_mark()` 函数
   - 复用属性解析和验证逻辑
   - Mark 特定的错误处理

2. **属性处理逻辑 (mark/attribute_handler.rs)**
   - Mark 特定的属性处理
   - 验证 `mark_type` 属性
   - 字段属性处理

3. **代码生成集成 (mark/code_gen.rs)**
   - 调用 Mark 代码生成器
   - 最终代码组装
   - 导入语句处理

**验收标准**:
- [ ] Mark 派生宏正确实现
- [ ] 与 Node 派生宏保持一致的接口
- [ ] 复用通用模块的功能
- [ ] 错误处理完整
- [ ] 测试覆盖率 ≥ 90%
- [ ] 实现代码包含详细中文注释

**前置条件**: T007完成  
**依赖任务**: T007

---

### 第四阶段：宏入口点和集成 (Priority: P0)

#### T009: 更新主入口点模块
**负责模块**: mf-derive/src/lib.rs  
**预估工时**: 4小时  
**复杂度**: 低

**具体任务**:
1. **派生宏注册**
   - 添加 `#[proc_macro_derive(Node, attributes(node_type, marks, content, attr))]`
   - 添加 `#[proc_macro_derive(Mark, attributes(mark_type, attr))]`
   - 保持现有 `derive_plugin_state` 功能

2. **模块导出**
   - 导出公共类型和接口
   - 重新组织模块结构
   - 提供清晰的公共 API

3. **文档完善**
   - 添加模块级文档
   - 提供使用示例
   - 说明各个属性的用法

**入口点代码**:
```rust
/// Node 派生宏
#[proc_macro_derive(Node, attributes(node_type, marks, content, attr))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    node::derive_impl::process_derive_node(input)
}

/// Mark 派生宏
#[proc_macro_derive(Mark, attributes(mark_type, attr))]
pub fn derive_mark(input: TokenStream) -> TokenStream {
    mark::derive_impl::process_derive_mark(input)
}
```

**验收标准**:
- [ ] 派生宏正确注册和导出
- [ ] 公共 API 清晰易用
- [ ] 文档完整，包含使用示例
- [ ] 现有功能不受影响
- [ ] 通过 `cargo doc` 生成完整文档
- [ ] 入口点代码包含详细中文注释

**前置条件**: T006, T008完成  
**依赖任务**: T006, T008

---

### 第五阶段：测试和验证 (Priority: P0)

#### T010: 单元测试实现
**负责模块**: mf-derive/src/*/mod.rs (各模块测试)  
**预估工时**: 16小时  
**复杂度**: 中等

**具体任务**:
1. **属性解析测试**
   - 测试各种属性配置的解析
   - 测试错误配置的处理
   - 边界条件测试

2. **类型转换测试**
   - 测试所有支持类型的转换
   - 测试 Option 类型处理
   - 测试自定义转换器注册

3. **代码生成测试**
   - 测试生成代码的语法正确性
   - 测试生成代码的功能正确性
   - 测试各种配置组合

4. **端到端宏测试**
   - 测试完整的派生宏流程
   - 测试生成的方法调用
   - 测试与 mf-core 集成

**测试覆盖率目标**: ≥ 90%

**验收标准**:
- [ ] 所有模块单元测试通过
- [ ] 测试覆盖率达到目标
- [ ] 边界条件和错误情况覆盖完整
- [ ] 测试代码具有良好的可维护性
- [ ] 测试代码包含详细中文注释

**前置条件**: T009完成  
**依赖任务**: T009

---

#### T011: 集成测试实现
**负责模块**: mf-derive/tests/  
**预估工时**: 12小时  
**复杂度**: 中等

**具体任务**:
1. **Node 派生宏集成测试 (tests/node_derive_tests.rs)**
   - 基础功能测试
   - 复杂配置测试
   - 与 mf-core 集成测试
   - 属性设置正确性测试

2. **Mark 派生宏集成测试 (tests/mark_derive_tests.rs)**
   - 基础功能测试
   - 属性处理测试
   - 与 mf-core 集成测试

3. **错误场景测试 (tests/error_cases.rs)**
   - 使用 `trybuild` 测试编译错误
   - 验证错误消息质量
   - 测试各种无效配置

4. **性能测试 (tests/performance.rs)**
   - 编译时间测试
   - 生成代码性能测试
   - 内存使用测试

**验收标准**:
- [ ] 所有集成测试通过
- [ ] 错误场景测试覆盖完整
- [ ] 性能测试满足要求
- [ ] 测试代码可维护且具有文档
- [ ] 集成测试包含详细中文注释

**前置条件**: T010完成  
**依赖任务**: T010

---

#### T012: 编译时错误消息优化
**负责模块**: mf-derive/src/common/error.rs 及相关  
**预估工时**: 6小时  
**复杂度**: 中等

**具体任务**:
1. **错误消息本地化**
   - 所有错误消息使用中文
   - 提供具体的修复建议
   - 包含相关文档链接

2. **错误位置精确定位**
   - 使用 `syn::spanned::Spanned` 精确定位
   - 高亮显示错误的具体代码位置
   - 提供上下文信息

3. **常见错误的专门处理**
   - 缺失必需属性
   - 不支持的字段类型
   - 属性格式错误
   - 循环依赖等问题

**错误消息示例**:
```
error: 缺少必需的宏属性: node_type
  --> src/lib.rs:5:10
   |
5  | #[derive(Node)]
   |          ^^^^
   |
help: 请添加 node_type 属性，例如: #[node_type = "GCXM"]
```

**验收标准**:
- [ ] 错误消息友好且具有指导性
- [ ] 错误位置定位准确
- [ ] 修复建议具体可行
- [ ] 所有错误类型都有相应的友好消息
- [ ] 错误处理代码包含详细中文注释

**前置条件**: T011完成  
**依赖任务**: T011

---

### 第六阶段：示例和文档 (Priority: P1)

#### T013: 使用示例实现
**负责模块**: mf-derive/examples/  
**预估工时**: 8小时  
**复杂度**: 低

**具体任务**:
1. **基础使用示例 (examples/basic_usage.rs)**
   - 简单 Node 和 Mark 定义示例
   - 基本属性配置示例
   - 生成方法调用示例

2. **复杂配置示例 (examples/complex_usage.rs)**
   - 多属性 Node 定义
   - Optional 字段处理
   - 自定义类型转换

3. **最佳实践示例 (examples/best_practices.rs)**
   - 项目结构建议
   - 错误处理模式
   - 性能优化技巧

4. **集成示例 (examples/integration.rs)**
   - 与 ModuForge-RS 其他模块集成
   - 完整的文档编辑器示例
   - 实际业务场景应用

**验收标准**:
- [ ] 示例代码可以正常编译和运行
- [ ] 覆盖所有主要功能点
- [ ] 代码注释详细，包含解释说明
- [ ] 示例具有实际参考价值
- [ ] 示例代码包含详细中文注释

**前置条件**: T012完成  
**依赖任务**: T012

---

#### T014: API 文档完善
**负责模块**: 各模块文档注释  
**预估工时**: 10小时  
**复杂度**: 中等

**具体任务**:
1. **公共 API 文档**
   - 为所有公开函数添加文档注释
   - 包含参数说明和返回值说明
   - 提供使用示例

2. **模块级文档**
   - 每个模块的整体功能说明
   - 模块间的关系说明
   - 设计原则体现说明

3. **宏使用指南**
   - 详细的宏属性说明
   - 支持的类型列表
   - 常见问题和解决方案

4. **架构文档**
   - 整体架构设计说明
   - 扩展点和插件机制
   - 性能优化建议

**文档质量标准**:
```rust
/// 将结构体转换为 mf_core::node::Node 实例
///
/// 此方法由 #[derive(Node)] 宏自动生成，根据结构体的字段
/// 和宏属性配置创建相应的 Node 实例。
///
/// # 前提条件
/// 结构体必须标注 #[derive(Node)] 和必需的宏属性
///
/// # 返回值
/// 返回配置好的 `mf_core::node::Node` 实例
///
/// # 示例
/// ```rust
/// #[derive(Node)]
/// #[node_type = "project"]
/// struct MyProject {
///     #[attr]
///     name: String,
/// }
///
/// let project = MyProject { name: "示例".to_string() };
/// let node = project.to_node();
/// ```
```

**验收标准**:
- [ ] 100% 公共 API 文档覆盖率
- [ ] 文档清晰易懂，包含实用示例
- [ ] 通过 `cargo doc` 生成完整文档
- [ ] 文档中的示例代码可以编译通过
- [ ] 所有文档使用中文，符合项目规范

**前置条件**: T013完成  
**依赖任务**: T013

---

### 第七阶段：性能优化和扩展性 (Priority: P1)

#### T015: 编译时性能优化
**负责模块**: 所有模块的性能优化  
**预估工时**: 8小时  
**复杂度**: 中等

**具体任务**:
1. **类型分析缓存**
   - 使用 `once_cell` 缓存重复的类型分析结果
   - 避免重复的 AST 解析
   - 优化字符串处理性能

2. **代码生成优化**
   - 减少不必要的 TokenStream 克隆
   - 优化 quote! 宏的使用
   - 预计算常量表达式

3. **内存管理优化**
   - 及时释放临时数据结构
   - 使用 String interning 减少内存占用
   - 优化 HashMap 的初始容量

4. **增量编译支持**
   - 确保宏支持增量编译
   - 避免不必要的重新编译
   - 优化依赖关系

**性能目标**:
- 编译时间增加 < 5%
- 内存使用峰值 < 10MB
- 支持增量编译

**验收标准**:
- [ ] 性能基准测试通过
- [ ] 编译时间满足目标要求
- [ ] 内存使用在合理范围内
- [ ] 增量编译正常工作
- [ ] 性能优化代码包含详细中文注释

**前置条件**: T014完成  
**依赖任务**: T014

---

#### T016: 扩展性机制实现
**负责模块**: mf-derive/src/extension/  
**预估工时**: 12小时  
**复杂度**: 高

**具体任务**:
1. **插件接口设计 (extension/plugin.rs)**
   - 定义宏处理插件接口
   - 支持自定义属性处理器
   - 提供代码生成钩子

2. **扩展点实现 (extension/extension_points.rs)**
   - 在关键位置添加扩展点
   - 支持多个插件的组合使用
   - 维护插件执行顺序

3. **注册机制 (extension/registry.rs)**
   - 插件动态注册机制
   - 插件依赖关系管理
   - 插件配置管理

4. **内置扩展示例 (extension/builtin/)**
   - 实现几个示例扩展
   - 展示扩展机制的使用方法
   - 提供扩展开发模板

**扩展接口设计**:
```rust
/// 宏处理插件接口
pub trait MacroPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn process_attributes(&self, attrs: &mut Vec<Attribute>);
    fn generate_additional_code(&self, config: &dyn Any) -> TokenStream;
}

/// 插件注册器
pub struct PluginRegistry {
    plugins: Vec<Box<dyn MacroPlugin>>,
}
```

**验收标准**:
- [ ] 扩展接口设计合理，符合开闭原则
- [ ] 支持插件的动态注册和管理
- [ ] 内置扩展示例完整可用
- [ ] 扩展机制文档详细
- [ ] 扩展机制代码包含详细中文注释

**前置条件**: T015完成  
**依赖任务**: T015

---

### 第八阶段：最终集成和发布准备 (Priority: P1)

#### T017: 版本兼容性验证
**负责模块**: 整体项目验证  
**预估工时**: 6小时  
**复杂度**: 中等

**具体任务**:
1. **ModuForge-RS 集成测试**
   - 与所有相关 crate 的集成测试
   - 版本兼容性验证
   - API 兼容性检查

2. **向后兼容性测试**
   - 确保现有功能不受影响
   - 测试现有代码迁移的便利性
   - 验证升级路径的顺畅性

3. **平台兼容性测试**
   - Windows, Linux, macOS 测试
   - 不同 Rust 版本测试
   - 不同 IDE 环境测试

4. **性能回归测试**
   - 对比优化前后的性能
   - 确保无性能回归
   - 建立性能基准

**验收标准**:
- [ ] 与 ModuForge-RS 生态完全兼容
- [ ] 现有功能零影响
- [ ] 跨平台兼容性良好
- [ ] 性能指标满足要求
- [ ] 兼容性测试代码包含详细中文注释

**前置条件**: T016完成  
**依赖任务**: T016

---

#### T018: 最终文档和发布材料
**负责模块**: 文档和发布材料  
**预估工时**: 6小时  
**复杂度**: 低

**具体任务**:
1. **README 更新**
   - 功能介绍和亮点
   - 快速开始指南
   - 迁移指南

2. **CHANGELOG 编写**
   - 详细的变更记录
   - 新功能介绍
   - 破坏性变更说明

3. **发布说明准备**
   - 版本亮点总结
   - 使用建议
   - 已知问题说明

4. **社区材料**
   - 博客文章草稿
   - 示例项目
   - 视频教程脚本

**验收标准**:
- [ ] 文档完整且易于理解
- [ ] 发布材料准备就绪
- [ ] 社区推广材料质量高
- [ ] 所有文档使用中文

**前置条件**: T017完成  
**依赖任务**: T017

---

## 任务优先级矩阵

### P0 (必须完成)
- T001-T012: 核心功能实现和基础验证
- 关键路径: T001 → T002 → T003,T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012

### P1 (重要功能)
- T013-T018: 文档、示例、优化和发布准备
- 可与 P0 任务并行进行部分工作

### P2 (可选增强)
- 高级扩展功能
- 实验性特性
- 社区反馈驱动的改进

## 工作量估算总结

| 阶段 | 任务数 | 总工时 | 平均复杂度 |
|------|-------|-------|-----------|
| 基础架构 | 4 | 34h | 中等 |
| Node实现 | 2 | 22h | 高 |
| Mark实现 | 2 | 16h | 中等 |
| 集成 | 1 | 4h | 低 |
| 测试验证 | 3 | 34h | 中等 |
| 文档示例 | 2 | 18h | 中等 |
| 优化扩展 | 2 | 20h | 高 |
| 发布准备 | 2 | 12h | 中等 |
| **总计** | **18** | **160h** | **中等** |

## 质量保证要求

### 代码质量标准
- **测试覆盖率**: ≥ 90%
- **文档覆盖率**: 100% (公共 API)
- **Clippy 检查**: 零警告策略
- **格式规范**: 严格遵循 rustfmt 配置

### 中文注释要求
- **函数文档**: 所有公共函数必须包含详细的中文文档注释
- **模块文档**: 每个模块都需要模块级别的中文说明
- **示例代码**: 所有示例必须包含中文说明
- **错误消息**: 所有用户可见的错误消息使用中文

### 性能要求
- **编译时间**: 宏处理对整体编译时间的影响 < 5%
- **内存使用**: 宏展开过程内存峰值 < 10MB
- **运行时性能**: 生成的代码性能与手写代码相当

### 设计原则验证
每个任务完成后需验证是否符合核心设计原则：
- **SRP**: 模块职责单一明确
- **ISP**: 接口精简专用
- **OCP**: 支持扩展不修改现有代码
- **LSP**: 实现类型可完全替换抽象类型

## 风险管控

### 技术风险
1. **宏复杂性**: 模块化设计，充分测试
2. **编译性能**: 持续性能监控，早期优化
3. **API兼容性**: 严格的集成测试

### 项目风险
1. **范围蔓延**: 严格按照设计文档执行
2. **质量问题**: 建立完善的质量保证流程
3. **时间延期**: 合理的任务优先级和依赖管理

通过这个详细的任务清单，ModuForge-RS 宏模块扩展项目将能够系统化、高质量地完成所有必要的开发工作，最终为 ModuForge-RS 生态提供强大的声明式节点和标记定义能力。
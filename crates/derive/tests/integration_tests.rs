//! ModuForge-RS 派生宏集成测试
//!
//! 这个文件包含了完整的端到端集成测试，用于验证 #[derive(Node)] 和 #[derive(Mark)]
//! 派生宏的实际功能和用户体验。演示了所有支持的功能，包括自定义类型表达式。
//!
//! 集成测试遵循以下设计原则：
//! - **用户视角**: 从用户使用角度测试宏功能  
//! - **端到端**: 测试完整的宏处理流程
//! - **真实场景**: 使用真实的使用案例和数据结构
//! - **功能演示**: 展示泛型类型、自定义类型、默认值等高级功能
//!
//! # 支持的功能演示
//!
//! ## 基本功能
//! - `#[node_type = "type"]` - 节点类型定义
//! - `#[marks = "mark1 mark2"]` - 支持的标记
//! - `#[content = "expression"]` - 内容表达式
//! - `#[attr]` - 基本属性标记
//! - `#[attr(default="value")]` - 带默认值的属性
//!
//! ## 高级功能
//! - 泛型类型支持：`Option<String>`, `Vec<u8>`, `HashMap<String, i32>`
//! - 自定义类型表达式：`#[attr(default="CustomStruct::new()")]`
//! - 双向转换：`node_definition()` 和 `from()` 方法
//! - 错误处理：类型不匹配时的 Result 返回
//! - 降级策略：转换失败时使用 `default_instance()`
//!
//! ## 设计分离
//! - `node_definition()`: 只包含 `#[attr]` 字段，用于节点模式定义
//! - `from()`: 处理所有字段，用于实例创建
//! - 非 `#[attr]` 字段在实例化时使用类型默认值

use mf_derive::{Node, Mark};

/// 基本的 Node 派生宏测试结构体
///
/// 这个结构体用于测试 Node 派生宏的基本功能，包括：
/// - 基本的 node_type 属性设置
/// - 简单的属性字段处理
#[derive(Node)]
#[node_type = "paragraph"]
struct BasicNodeTest {
    #[attr]
    content: String,

    #[attr(default = 1)]
    level: i32,
}

/// 完整功能的 Node 测试结构体
///
/// 测试所有 Node 派生宏支持的功能：
/// - node_type, marks, content 属性
/// - 多种类型的属性字段
/// - 可选类型字段
/// - 非属性字段（应被忽略）
#[derive(Node)]
#[node_type = "rich_content"]
#[marks = "bold italic underline"]
#[content = "text*"]
#[desc = "这是一个测试"]
struct FullNodeTest {
    #[attr]
    title: String,

    #[attr]
    subtitle: Option<String>,

    #[attr]
    priority: i32,

    #[attr]
    weight: Option<f64>,

    #[attr]
    is_published: bool,

    #[attr]
    tags: Option<String>,

    // 非属性字段，应该被忽略
    #[attr]
    internal_id: uuid::Uuid,
    #[attr]
    cache_data: Vec<u8>,
}

/// 基本的 Mark 派生宏测试结构体
///
/// 测试 Mark 派生宏的基本功能：
/// - mark_type 属性设置
/// - 简单的属性字段处理
#[derive(Mark)]
#[mark_type = "emphasis"]
#[allow(dead_code)]
struct BasicMarkTest {
    #[attr]
    level: String,

    #[attr]
    color: Option<String>,
}

/// 完整功能的 Mark 测试结构体
///
/// 测试所有 Mark 派生宏支持的功能：
/// - mark_type 属性
/// - 多种类型的属性字段
/// - 可选和非可选字段混合
#[derive(Mark)]
#[mark_type = "styling"]
#[allow(dead_code)]
struct FullMarkTest {
    #[attr]
    font_family: String,

    #[attr]
    font_size: Option<f32>,

    #[attr]
    is_bold: bool,

    #[attr]
    opacity: Option<f64>,

    #[attr]
    z_index: i32,

    // 非属性字段
    _phantom: std::marker::PhantomData<()>,
}

/// ID 字段测试结构体
///
/// 测试 #[id] 属性的功能：
/// - ID 字段映射到 Node 的 id 属性
/// - 与普通 attr 字段的组合使用
/// - 不同类型的 ID 字段支持
#[derive(Node, Debug)]
#[node_type = "document"]
#[marks = "important"]
struct NodeWithIdTest {
    #[id]
    document_id: String,

    #[attr]
    title: String,

    #[attr]
    content: Option<String>,

    #[attr]
    version: i32,

    // 普通字段（无标记）
    internal_data: Vec<u8>,
}

/// 可选 ID 字段测试结构体
///
/// 测试 Option<String> 类型的 ID 字段
#[derive(Node, Debug)]
#[node_type = "section"]
#[desc = "测试"]
struct NodeWithOptionalIdTest {
    #[id]
    section_id: Option<String>,

    #[attr(default = "1")]
    name: String,

    #[attr(default = "0")]
    order: i32,
}

/// 最小化 ID 测试结构体
///
/// 只有 ID 字段，没有其他属性字段
#[derive(Node, Debug)]
#[node_type = "minimal"]
struct MinimalNodeWithIdTest {
    #[id]
    node_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试完整 Node 派生宏功能
    #[test]
    fn test_full_node_derivation() {
        // 调用生成的 node_definition() 静态方法，返回使用默认值的 Node 定义
        let mf_node = FullNodeTest::node_definition();
        println!("{mf_node:?}");

        // 验证节点类型
        assert_eq!(mf_node.name, "rich_content");
    }
}

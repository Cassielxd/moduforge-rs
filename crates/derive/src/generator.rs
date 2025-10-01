//! 代码生成器模块
//!
//! 提供 Node 和 Mark 的代码生成功能，将解析后的配置转换为可执行的 Rust 代码。
//! 严格遵循单一职责原则，每个生成器只负责特定类型的代码生成。

// Library code may have unused items that are part of the public API
#![allow(dead_code, clippy::only_used_in_recursion)]

pub mod mark_generator;
pub mod node_generator;

use crate::common::MacroResult;
use proc_macro2::TokenStream as TokenStream2;

/// 代码生成器接口
///
/// 定义了代码生成的核心接口，遵循接口隔离原则。
/// 任何实现此接口的类型都能提供代码生成功能。
///
/// # 设计原则体现
///
/// - **接口隔离原则**: 只定义代码生成相关的必要方法
/// - **开闭原则**: 通过实现此接口可以扩展新的代码生成器
/// - **里氏替换原则**: 所有实现都能够无缝替换使用
pub trait CodeGenerator {
    /// 生成转换方法的代码
    ///
    /// 根据配置信息生成相应的转换方法代码。
    ///
    /// # 返回值
    ///
    /// 成功时返回生成的代码 TokenStream，失败时返回生成错误
    fn generate(&self) -> MacroResult<TokenStream2>;

    /// 获取生成器的名称
    ///
    /// 返回生成器的名称，用于调试和错误消息。
    ///
    /// # 返回值
    ///
    /// 返回生成器名称字符串
    fn name(&self) -> &'static str;
}

/// 生成器工厂
///
/// 提供创建各种代码生成器的工厂方法。
/// 遵循单一职责原则，专门负责生成器的创建和管理。
///
/// # 设计原则体现
///
/// - **单一职责**: 只负责生成器的创建
/// - **开闭原则**: 可扩展新的生成器类型
pub struct GeneratorFactory;

impl GeneratorFactory {
    /// 创建 Node 代码生成器
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入
    /// * `config` - Node 配置信息
    ///
    /// # 返回值
    ///
    /// 返回配置好的 Node 代码生成器
    pub fn create_node_generator<'a>(
        input: &'a syn::DeriveInput,
        config: &'a crate::parser::NodeConfig,
    ) -> node_generator::NodeGenerator<'a> {
        node_generator::NodeGenerator::new(input, config)
    }

    /// 创建 Mark 代码生成器
    ///
    /// # 参数
    ///
    /// * `input` - 派生宏的输入
    /// * `config` - Mark 配置信息
    ///
    /// # 返回值
    ///
    /// 返回配置好的 Mark 代码生成器
    pub fn create_mark_generator<'a>(
        input: &'a syn::DeriveInput,
        config: &'a crate::parser::MarkConfig,
    ) -> mark_generator::MarkGenerator<'a> {
        mark_generator::MarkGenerator::new(input, config)
    }
}

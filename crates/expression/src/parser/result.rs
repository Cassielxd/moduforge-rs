use crate::parser::{Node, ParserError};
use nohash_hasher::BuildNoHashHasher;
use std::collections::HashMap;

/// 语法分析结果结构体
/// 包含解析生成的AST根节点和相关元数据
#[derive(Debug)]
pub struct ParserResult<'a> {
    pub root: &'a Node<'a>, // AST的根节点
    pub is_complete: bool,  // 解析是否完整（是否处理了所有令牌）
    pub metadata:
        Option<HashMap<usize, NodeMetadata, BuildNoHashHasher<usize>>>, // 节点元数据映射
}

/// 节点元数据结构体
/// 存储节点在源代码中的位置信息
#[derive(Debug, Clone)]
pub struct NodeMetadata {
    pub span: (u32, u32), // 节点在源代码中的位置范围（开始位置，结束位置）
}

impl<'a> ParserResult<'a> {
    /// 检查解析结果是否包含错误
    /// 返回第一个遇到的错误，如果没有错误则返回Ok(())
    pub fn error(&self) -> Result<(), ParserError> {
        // 首先检查解析是否完整
        if !self.is_complete {
            return Err(ParserError::Incomplete);
        }

        // 检查AST中是否包含错误节点
        match self.root.first_error() {
            None => Ok(()),
            Some(err) => Err(ParserError::NodeError(err.to_string())),
        }
    }
}

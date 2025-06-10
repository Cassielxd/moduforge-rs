use crate::lexer::{Bracket, Identifier, TokenKind};
use crate::parser::ast::{AstNodeError, Node};
use crate::parser::constants::{Associativity, BINARY_OPERATORS, UNARY_OPERATORS};
use crate::parser::parser::Parser;
use crate::parser::result::ParserResult;
use crate::parser::NodeMetadata;

/// 标准解析器结构体
/// 用于解析完整的表达式，支持所有语法特性
#[derive(Debug)]
pub struct Standard;

impl<'arena, 'token_ref> Parser<'arena, 'token_ref, Standard> {
    /// 执行标准解析
    /// 解析完整的表达式并返回解析结果
    pub fn parse(&self) -> ParserResult<'arena> {
        let root = self.binary_expression(0);

        ParserResult {
            root,
            is_complete: self.is_done(),
            metadata: self.node_metadata.clone().map(|t| t.into_inner()),
        }
    }

    /// 解析二元表达式
    /// 使用算符优先级分析法解析二元操作符表达式
    /// 
    /// # 参数
    /// * `precedence` - 最小优先级，只处理优先级不低于此值的操作符
    #[cfg_attr(feature = "stack-protection", recursive::recursive)]
    fn binary_expression(&self, precedence: u8) -> &'arena Node<'arena> {
        // 解析左操作数
        let mut node_left = self.unary_expression();
        let Some(mut token) = self.current() else {
            return node_left;
        };

        // 处理二元操作符序列
        while let TokenKind::Operator(operator) = &token.kind {
            if self.is_done() {
                break;
            }

            // 检查操作符是否为有效的二元操作符
            let Some(op) = BINARY_OPERATORS.get(operator) else {
                break;
            };

            // 检查优先级：如果当前操作符优先级低于最小优先级，则停止
            if op.precedence < precedence {
                break;
            }

            self.next();
            // 根据结合性确定右操作数的最小优先级
            let node_right = match op.associativity {
                Associativity::Left => self.binary_expression(op.precedence + 1), // 左结合：提高优先级
                _ => self.binary_expression(op.precedence),                        // 右结合：保持优先级
            };

            // 创建二元操作节点
            node_left = self.node(
                Node::Binary {
                    operator: *operator,
                    left: node_left,
                    right: node_right,
                },
                |h| NodeMetadata {
                    span: h.span(node_left, node_right).unwrap_or_default(),
                },
            );

            // 获取下一个令牌继续处理
            let Some(t) = self.current() else {
                break;
            };
            token = t;
        }

        // 在最低优先级（0）时处理条件表达式（三元操作符）
        if precedence == 0 {
            if let Some(conditional_node) =
                self.conditional(node_left, |_| self.binary_expression(0))
            {
                node_left = conditional_node;
            }
        }

        node_left
    }

    /// 解析一元表达式
    /// 处理一元操作符、括号表达式、区间表达式和字面量
    fn unary_expression(&self) -> &'arena Node<'arena> {
        let Some(token) = self.current() else {
            return self.error(AstNodeError::Custom {
                message: self.bump.alloc_str("Unexpected end of unary expression"),
                span: (self.prev_token_end(), self.prev_token_end()),
            });
        };

        // 处理回调引用 (#)
        if self.depth() > 0 && token.kind == TokenKind::Identifier(Identifier::CallbackReference) {
            self.next();

            let node = self.node(Node::Pointer, |_| NodeMetadata { span: token.span });
            return self.with_postfix(node, |_| self.binary_expression(0));
        }

        // 处理一元操作符（如 +x, -x, !x）
        if let TokenKind::Operator(operator) = &token.kind {
            let Some(unary_operator) = UNARY_OPERATORS.get(operator) else {
                return self.error(AstNodeError::UnexpectedToken {
                    expected: "UnaryOperator",
                    received: self.bump.alloc_str(token.kind.to_string().as_str()),
                    span: token.span,
                });
            };

            self.next();
            let expr = self.binary_expression(unary_operator.precedence);
            let node = self.node(
                Node::Unary {
                    operator: *operator,
                    node: expr,
                },
                |h| NodeMetadata {
                    span: (
                        token.span.0,
                        h.metadata(expr).map(|n| n.span.1).unwrap_or_default(),
                    ),
                },
            );

            return node;
        }

        // 尝试解析区间表达式（如 [1, 10]、(0, 100) 等）
        if let Some(interval_node) = self.interval(|_| self.binary_expression(0)) {
            return interval_node;
        }

        // 处理括号表达式
        if token.kind == TokenKind::Bracket(Bracket::LeftParenthesis) {
            let p_start = self.current().map(|s| s.span.0);

            self.next();
            let binary_node = self.binary_expression(0);
            if let Some(error_node) = self.expect(TokenKind::Bracket(Bracket::RightParenthesis)) {
                return error_node;
            };

            let expr = self.node(Node::Parenthesized(binary_node), |_| NodeMetadata {
                span: (p_start.unwrap_or_default(), self.prev_token_end()),
            });

            return self.with_postfix(expr, |_| self.binary_expression(0));
        }

        // 解析字面量（数字、字符串、标识符等）
        self.literal(|_| self.binary_expression(0))
    }
}

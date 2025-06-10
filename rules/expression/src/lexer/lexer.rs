use crate::lexer::codes::{is_token_type, token_type};
use crate::lexer::cursor::{Cursor, CursorItem};
use crate::lexer::error::LexerResult;
use crate::lexer::token::{
    Bracket, ComparisonOperator, Identifier, LogicalOperator, Operator, Token, TokenKind,
};
use crate::lexer::{LexerError, QuotationMark, TemplateString};
use std::str::FromStr;

/// 词法分析器结构体
/// 负责将字符串转换为令牌序列
#[derive(Debug, Default)]
pub struct Lexer<'arena> {
    tokens: Vec<Token<'arena>>,    // 存储解析出的令牌
}

impl<'arena> Lexer<'arena> {
    /// 创建新的词法分析器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 对输入字符串进行词法分析
    /// 返回解析出的令牌数组的引用
    pub fn tokenize(&mut self, source: &'arena str) -> LexerResult<&[Token<'arena>]> {
        self.tokens.clear();

        Scanner::new(source, &mut self.tokens).scan()?;
        Ok(&self.tokens)
    }
}

/// 扫描器结构体
/// 执行实际的词法分析工作
struct Scanner<'arena, 'self_ref> {
    cursor: Cursor<'arena>,                 // 字符串游标
    tokens: &'self_ref mut Vec<Token<'arena>>, // 令牌向量的可变引用
    source: &'arena str,                    // 源字符串
}

impl<'arena, 'self_ref> Scanner<'arena, 'self_ref> {
    /// 创建新的扫描器
    pub fn new(source: &'arena str, tokens: &'self_ref mut Vec<Token<'arena>>) -> Self {
        Self {
            cursor: Cursor::from(source),
            source,
            tokens,
        }
    }

    /// 执行扫描过程
    /// 遍历整个源字符串并识别所有令牌
    pub fn scan(&mut self) -> LexerResult<()> {
        while let Some(cursor_item) = self.cursor.peek() {
            self.scan_cursor_item(cursor_item)?;
        }

        Ok(())
    }

    /// 扫描单个字符项
    /// 根据字符类型调用相应的处理方法
    pub(crate) fn scan_cursor_item(&mut self, cursor_item: CursorItem) -> LexerResult<()> {
        let (i, s) = cursor_item;

        match s {
            // 空白字符：跳过
            token_type!("space") => {
                self.cursor.next();
                Ok(())
            }
            '\'' => self.string(QuotationMark::SingleQuote),     // 单引号字符串
            '"' => self.string(QuotationMark::DoubleQuote),      // 双引号字符串
            token_type!("digit") => self.number(),               // 数字
            token_type!("bracket") => self.bracket(),            // 括号
            token_type!("cmp_operator") => self.cmp_operator(),  // 比较操作符
            token_type!("operator") => self.operator(),          // 其他操作符
            token_type!("question_mark") => self.question_mark(), // 问号
            '`' => self.template_string(),                       // 模板字符串
            '.' => self.dot(),                                   // 点操作符
            token_type!("alpha") => self.identifier(),           // 标识符
            _ => Err(LexerError::UnmatchedSymbol {               // 未知字符
                symbol: s,
                position: i as u32,
            }),
        }
    }

    /// 获取下一个字符
    /// 如果到达文件末尾则返回错误
    fn next(&self) -> LexerResult<CursorItem> {
        self.cursor.next().ok_or_else(|| {
            let (a, b) = self.cursor.peek_back().unwrap_or((0, ' '));

            LexerError::UnexpectedEof {
                symbol: b,
                position: a as u32,
            }
        })
    }

    /// 添加令牌到令牌向量
    fn push(&mut self, token: Token<'arena>) {
        self.tokens.push(token);
    }

    /// 处理模板字符串
    /// 解析反引号包围的字符串，处理其中的表达式插值 ${...}
    fn template_string(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;

        // 添加开始的反引号令牌
        self.tokens.push(Token {
            kind: TokenKind::QuotationMark(QuotationMark::Backtick),
            span: (start as u32, (start + 1) as u32),
            value: QuotationMark::Backtick.into(),
        });

        let mut in_expression = false;  // 是否在表达式内部
        let mut str_start = start + 1;  // 字符串内容开始位置
        loop {
            let (e, c) = self.next()?;

            match (c, in_expression) {
                // 遇到结束的反引号
                ('`', _) => {
                    if str_start < e {
                        // 添加剩余的字符串字面量
                        self.tokens.push(Token {
                            kind: TokenKind::Literal,
                            span: (str_start as u32, e as u32),
                            value: &self.source[str_start..e],
                        });
                    }

                    // 添加结束的反引号令牌
                    self.tokens.push(Token {
                        kind: TokenKind::QuotationMark(QuotationMark::Backtick),
                        span: (e as u32, (e + 1) as u32),
                        value: QuotationMark::Backtick.into(),
                    });

                    break;
                }
                // 在字符串中遇到 $，检查是否是表达式开始
                ('$', false) => {
                    in_expression = self.cursor.next_if_is("{");
                    if in_expression {
                        // 添加表达式前的字符串字面量
                        self.tokens.push(Token {
                            kind: TokenKind::Literal,
                            span: (str_start as u32, e as u32),
                            value: &self.source[str_start..e],
                        });

                        // 添加表达式开始标记
                        self.tokens.push(Token {
                            kind: TokenKind::TemplateString(TemplateString::ExpressionStart),
                            span: (e as u32, (e + 2) as u32),
                            value: TemplateString::ExpressionStart.into(),
                        });
                    }
                }
                // 在表达式中遇到 }，表达式结束
                ('}', true) => {
                    in_expression = false;
                    self.tokens.push(Token {
                        kind: TokenKind::TemplateString(TemplateString::ExpressionEnd),
                        span: (str_start as u32, e as u32),
                        value: TemplateString::ExpressionEnd.into(),
                    });

                    str_start = e + 1;
                }
                // 在字符串中继续读取
                (_, false) => {
                    // Continue reading string
                }
                // 在表达式中，递归解析字符
                (_, true) => {
                    self.cursor.back();
                    self.scan_cursor_item((e, c))?;
                }
            }
        }

        Ok(())
    }

    /// 处理普通字符串（单引号或双引号包围）
    fn string(&mut self, quote_kind: QuotationMark) -> LexerResult<()> {
        let (start, opener) = self.next()?;
        let end: usize;

        // 寻找匹配的结束引号
        loop {
            let (e, c) = self.next()?;
            if c == opener {
                end = e;
                break;
            }
        }

        // 添加开始引号令牌
        self.push(Token {
            kind: TokenKind::QuotationMark(quote_kind),
            span: (start as u32, (start + 1) as u32),
            value: quote_kind.into(),
        });

        // 添加字符串内容令牌
        self.push(Token {
            kind: TokenKind::Literal,
            span: ((start + 1) as u32, end as u32),
            value: &self.source[start + 1..end],
        });

        // 添加结束引号令牌
        self.push(Token {
            kind: TokenKind::QuotationMark(quote_kind),
            span: (end as u32, (end + 1) as u32),
            value: quote_kind.into(),
        });

        Ok(())
    }

    /// 处理数字
    /// 支持整数、小数和科学计数法
    fn number(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;
        let mut end = start;
        let mut fractal = false;  // 是否已有小数点

        // 读取数字字符、下划线和小数点
        while let Some((e, c)) = self
            .cursor
            .next_if(|c| is_token_type!(c, "digit") || c == '_' || c == '.')
        {
            // 避免多个小数点
            if fractal && c == '.' {
                self.cursor.back();
                break;
            }

            if c == '.' {
                // 检查是否是范围操作符 ..
                if let Some((_, p)) = self.cursor.peek() {
                    if p == '.' {
                        self.cursor.back();
                        break;
                    }

                    fractal = true
                }
            }

            end = e;
        }

        // 处理科学计数法 (e/E)
        if let Some((e_pos, _)) = self.cursor.next_if(|c| c == 'e') {
            end = e_pos;

            // 处理可选的正负号
            if let Some((sign_pos, _)) = self.cursor.next_if(|c| c == '+' || c == '-') {
                end = sign_pos;
            }

            // 读取指数部分的数字
            let mut has_exponent_digits = false;
            while let Some((exp_pos, _)) = self.cursor.next_if(|c| is_token_type!(c, "digit")) {
                end = exp_pos;
                has_exponent_digits = true;
            }

            // 如果没有指数数字，回退到 e 之前
            if !has_exponent_digits {
                while self.cursor.position() > e_pos {
                    self.cursor.back();
                }

                end = e_pos - 1;
            }
        }

        // 添加数字令牌
        self.push(Token {
            kind: TokenKind::Number,
            span: (start as u32, (end + 1) as u32),
            value: &self.source[start..=end],
        });

        Ok(())
    }

    /// 处理括号
    fn bracket(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;

        let value = &self.source[start..=start];
        let span = (start as u32, (start + 1) as u32);
        self.push(Token {
            kind: TokenKind::Bracket(Bracket::from_str(value).map_err(|_| {
                LexerError::UnexpectedSymbol {
                    symbol: value.to_string(),
                    span,
                }
            })?),
            span,
            value,
        });

        Ok(())
    }

    /// 处理点操作符
    /// 支持单个点 . 和范围操作符 ..
    fn dot(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;
        let mut end = start;

        // 检查是否是范围操作符 ..
        if self.cursor.next_if(|c| c == '.').is_some() {
            end += 1;
        }

        let value = &self.source[start..=end];
        let span = (start as u32, (end + 1) as u32);
        self.push(Token {
            kind: TokenKind::Operator(Operator::from_str(value).map_err(|_| {
                LexerError::UnexpectedSymbol {
                    symbol: value.to_string(),
                    span,
                }
            })?),
            span,
            value,
        });

        Ok(())
    }

    /// 处理比较操作符
    /// 支持 <, >, !, = 及其组合
    fn cmp_operator(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;
        let mut end = start;

        // 检查是否有后续的 = 组成复合操作符
        if self.cursor.next_if(|c| c == '=').is_some() {
            end += 1;
        }

        let value = &self.source[start..=end];
        self.push(Token {
            kind: TokenKind::Operator(Operator::from_str(value).map_err(|_| {
                LexerError::UnexpectedSymbol {
                    symbol: value.to_string(),
                    span: (start as u32, (end + 1) as u32),
                }
            })?),
            span: (start as u32, (end + 1) as u32),
            value,
        });

        Ok(())
    }

    /// 处理问号操作符
    /// 支持单个问号 ? 和空值合并操作符 ??
    fn question_mark(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;
        let mut kind = TokenKind::Operator(Operator::QuestionMark);
        let mut end = start;

        // 检查是否是空值合并操作符 ??
        if self.cursor.next_if(|c| c == '?').is_some() {
            kind = TokenKind::Operator(Operator::Logical(LogicalOperator::NullishCoalescing));
            end += 1;
        }

        let value = &self.source[start..=end];
        self.push(Token {
            kind,
            value,
            span: (start as u32, (end + 1) as u32),
        });

        Ok(())
    }

    /// 处理其他操作符
    /// 包括算术操作符：+ - * / % ^ 和其他符号：, :
    fn operator(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;

        let value = &self.source[start..=start];
        let span = (start as u32, (start + 1) as u32);
        self.push(Token {
            kind: TokenKind::Operator(Operator::from_str(value).map_err(|_| {
                LexerError::UnexpectedSymbol {
                    symbol: value.to_string(),
                    span,
                }
            })?),
            span,
            value,
        });

        Ok(())
    }

    /// 处理 not 关键字
    /// 支持 not 和 not in 两种形式
    fn not(&mut self, start: usize) -> LexerResult<()> {
        if self.cursor.next_if_is(" in ") {
            // not in 操作符
            let end = self.cursor.position();

            self.push(Token {
                kind: TokenKind::Operator(Operator::Comparison(ComparisonOperator::NotIn)),
                span: (start as u32, (end - 1) as u32),
                value: "not in",
            })
        } else {
            // not 操作符
            let end = self.cursor.position();

            self.push(Token {
                kind: TokenKind::Operator(Operator::Logical(LogicalOperator::Not)),
                span: (start as u32, end as u32),
                value: "not",
            })
        }

        Ok(())
    }

    /// 处理标识符和关键字
    /// 包括变量名、布尔值、逻辑操作符等
    fn identifier(&mut self) -> LexerResult<()> {
        let (start, _) = self.next()?;
        let mut end = start;

        // 读取完整的标识符（字母、数字、下划线等）
        while let Some((e, _)) = self.cursor.next_if(|c| is_token_type!(c, "alphanumeric")) {
            end = e;
        }

        let value = &self.source[start..=end];
        match value {
            // 逻辑操作符
            "and" => self.push(Token {
                kind: TokenKind::Operator(Operator::Logical(LogicalOperator::And)),
                span: (start as u32, (end + 1) as u32),
                value,
            }),
            "or" => self.push(Token {
                kind: TokenKind::Operator(Operator::Logical(LogicalOperator::Or)),
                span: (start as u32, (end + 1) as u32),
                value,
            }),
            // 比较操作符
            "in" => self.push(Token {
                kind: TokenKind::Operator(Operator::Comparison(ComparisonOperator::In)),
                span: (start as u32, (end + 1) as u32),
                value,
            }),
            // 布尔值
            "true" => self.push(Token {
                kind: TokenKind::Boolean(true),
                span: (start as u32, (end + 1) as u32),
                value,
            }),
            "false" => self.push(Token {
                kind: TokenKind::Boolean(false),
                span: (start as u32, (end + 1) as u32),
                value,
            }),
            // not 关键字（可能是 not 或 not in）
            "not" => self.not(start)?,
            // 其他标识符或字面量
            _ => self.push(Token {
                kind: Identifier::try_from(value)
                    .map(|identifier| TokenKind::Identifier(identifier))
                    .unwrap_or(TokenKind::Literal),
                span: (start as u32, (end + 1) as u32),
                value,
            }),
        }

        Ok(())
    }
}

use crate::compiler::error::{CompilerError, CompilerResult};
use crate::compiler::opcode::{FetchFastTarget, Jump};
use crate::compiler::{Compare, Opcode};
use crate::functions::registry::FunctionRegistry;
use crate::functions::{
    ClosureFunction, FunctionKind, InternalFunction, MethodRegistry,
};
use crate::lexer::{
    ArithmeticOperator, ComparisonOperator, LogicalOperator, Operator,
};
use crate::parser::Node;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;

/// 编译器主结构体
/// 负责将AST编译为虚拟机操作码序列
#[derive(Debug)]
pub struct Compiler {
    bytecode: Vec<Opcode>, // 生成的操作码序列
}

impl Compiler {
    /// 创建新的编译器实例
    pub fn new() -> Self {
        Self { bytecode: Default::default() }
    }

    /// 编译AST节点为操作码
    ///
    /// # 参数
    /// * `root` - 要编译的AST根节点
    ///
    /// # 返回值
    /// 返回生成的操作码序列或编译错误
    pub fn compile(
        &mut self,
        root: &Node,
    ) -> CompilerResult<&[Opcode]> {
        self.bytecode.clear();

        CompilerInner::new(&mut self.bytecode, root).compile()?;
        Ok(self.bytecode.as_slice())
    }

    /// 获取当前的操作码序列
    pub fn get_bytecode(&self) -> &[Opcode] {
        self.bytecode.as_slice()
    }
}

/// 编译器内部实现结构体
/// 执行实际的编译工作，管理编译状态
#[derive(Debug)]
struct CompilerInner<'arena, 'bytecode_ref> {
    root: &'arena Node<'arena>,               // AST根节点
    bytecode: &'bytecode_ref mut Vec<Opcode>, // 操作码向量的可变引用
}

impl<'arena, 'bytecode_ref> CompilerInner<'arena, 'bytecode_ref> {
    /// 创建新的编译器内部实例
    pub fn new(
        bytecode: &'bytecode_ref mut Vec<Opcode>,
        root: &'arena Node<'arena>,
    ) -> Self {
        Self { root, bytecode }
    }

    /// 执行编译过程
    pub fn compile(&mut self) -> CompilerResult<()> {
        self.compile_node(self.root)?;
        Ok(())
    }

    /// 生成一个操作码并返回其位置
    ///
    /// # 参数
    /// * `op` - 要生成的操作码
    ///
    /// # 返回值
    /// 返回操作码在序列中的位置（1-based）
    fn emit(
        &mut self,
        op: Opcode,
    ) -> usize {
        self.bytecode.push(op);
        self.bytecode.len()
    }

    /// 生成循环结构的操作码
    ///
    /// # 参数
    /// * `body` - 循环体编译函数
    fn emit_loop<F>(
        &mut self,
        body: F,
    ) -> CompilerResult<()>
    where
        F: FnOnce(&mut Self) -> CompilerResult<()>,
    {
        let begin = self.bytecode.len();
        let end = self.emit(Opcode::Jump(Jump::IfEnd, 0)); // 循环结束跳转占位符

        body(self)?; // 编译循环体

        self.emit(Opcode::IncrementIt); // 增加迭代器
        let e = self.emit(Opcode::Jump(
            Jump::Backward,
            self.calc_backward_jump(begin) as u32,
        )); // 跳回循环开始

        // 回填循环结束跳转的目标地址
        self.replace(end, Opcode::Jump(Jump::IfEnd, (e - end) as u32));
        Ok(())
    }

    /// 生成条件结构的操作码
    ///
    /// # 参数
    /// * `body` - 条件体编译函数
    fn emit_cond<F>(
        &mut self,
        mut body: F,
    ) where
        F: FnMut(&mut Self),
    {
        let noop = self.emit(Opcode::Jump(Jump::IfFalse, 0)); // 条件为假时跳转
        self.emit(Opcode::Pop); // 弹出条件值

        body(self); // 编译条件体

        let jmp = self.emit(Opcode::Jump(Jump::Forward, 0)); // 跳过else部分
        self.replace(noop, Opcode::Jump(Jump::IfFalse, (jmp - noop) as u32));
        let e = self.emit(Opcode::Pop); // 清理栈
        self.replace(jmp, Opcode::Jump(Jump::Forward, (e - jmp) as u32));
    }

    /// 替换指定位置的操作码
    ///
    /// # 参数
    /// * `at` - 要替换的位置（1-based）
    /// * `op` - 新的操作码
    fn replace(
        &mut self,
        at: usize,
        op: Opcode,
    ) {
        let _ = std::mem::replace(&mut self.bytecode[at - 1], op);
    }

    /// 计算向后跳转的距离
    ///
    /// # 参数
    /// * `to` - 目标位置
    ///
    /// # 返回值
    /// 返回跳转距离
    fn calc_backward_jump(
        &self,
        to: usize,
    ) -> usize {
        self.bytecode.len() + 1 - to
    }

    /// 编译函数或方法的参数
    ///
    /// # 参数
    /// * `function_kind` - 函数类型（用于错误报告）
    /// * `arguments` - 参数节点数组
    /// * `index` - 要编译的参数索引
    fn compile_argument<T: ToString>(
        &mut self,
        function_kind: T,
        arguments: &[&'arena Node<'arena>],
        index: usize,
    ) -> CompilerResult<usize> {
        let arg = arguments.get(index).ok_or_else(|| {
            CompilerError::ArgumentNotFound {
                index,
                function: function_kind.to_string(),
            }
        })?;

        self.compile_node(arg)
    }

    /// 尝试为成员访问生成快速路径
    /// 将连续的成员访问优化为单个FetchFast操作
    ///
    /// # 参数
    /// * `node` - 要分析的节点
    ///
    /// # 返回值
    /// 如果可以优化，返回快速访问路径；否则返回None
    #[cfg_attr(feature = "stack-protection", recursive::recursive)]
    fn compile_member_fast(
        &mut self,
        node: &'arena Node<'arena>,
    ) -> Option<Vec<FetchFastTarget>> {
        match node {
            Node::Root => Some(vec![FetchFastTarget::Root]),
            Node::Identifier(v) => Some(vec![
                FetchFastTarget::Root,
                FetchFastTarget::String(Arc::from(*v)),
            ]),
            Node::Member { node, property } => {
                let mut path = self.compile_member_fast(node)?;
                match property {
                    Node::String(v) => {
                        path.push(FetchFastTarget::String(Arc::from(*v)));
                        Some(path)
                    },
                    Node::Number(v) => {
                        if let Some(idx) = v.to_u32() {
                            path.push(FetchFastTarget::Number(idx));
                            Some(path)
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            },
            _ => None,
        }
    }

    /// 编译AST节点为操作码
    /// 这是编译器的核心方法，递归处理各种类型的AST节点
    ///
    /// # 参数
    /// * `node` - 要编译的AST节点
    ///
    /// # 返回值
    /// 返回最后生成的操作码位置或编译错误
    #[cfg_attr(feature = "stack-protection", recursive::recursive)]
    fn compile_node(
        &mut self,
        node: &'arena Node<'arena>,
    ) -> CompilerResult<usize> {
        match node {
            // 基本值类型
            Node::Null => Ok(self.emit(Opcode::PushNull)),
            Node::Bool(v) => Ok(self.emit(Opcode::PushBool(*v))),
            Node::Number(v) => Ok(self.emit(Opcode::PushNumber(*v))),
            Node::String(v) => Ok(self.emit(Opcode::PushString(Arc::from(*v)))),
            Node::Pointer => Ok(self.emit(Opcode::Pointer)),
            Node::Root => Ok(self.emit(Opcode::FetchRootEnv)),

            // 数组：编译所有元素，然后创建数组
            Node::Array(v) => {
                v.iter().try_for_each(|&n| self.compile_node(n).map(|_| ()))?;
                self.emit(Opcode::PushNumber(Decimal::from(v.len())));
                Ok(self.emit(Opcode::Array))
            },

            // 对象：编译所有键值对，然后创建对象
            Node::Object(v) => {
                v.iter().try_for_each(|&(key, value)| {
                    self.compile_node(key).map(|_| ())?;
                    // 将键转换为字符串
                    self.emit(Opcode::CallFunction {
                        arg_count: 1,
                        kind: FunctionKind::Internal(InternalFunction::String),
                    });
                    self.compile_node(value).map(|_| ())?;
                    Ok(())
                })?;

                self.emit(Opcode::PushNumber(Decimal::from(v.len())));
                Ok(self.emit(Opcode::Object))
            },

            // 标识符：从环境中获取值
            Node::Identifier(v) => {
                Ok(self.emit(Opcode::FetchEnv(Arc::from(*v))))
            },

            // 闭包和括号表达式：直接编译内部节点
            Node::Closure(v) => self.compile_node(v),
            Node::Parenthesized(v) => self.compile_node(v),

            // 成员访问：尝试快速路径，否则使用通用方法
            Node::Member { node: n, property: p } => {
                if let Some(path) = self.compile_member_fast(node) {
                    Ok(self.emit(Opcode::FetchFast(path)))
                } else {
                    self.compile_node(n)?;
                    self.compile_node(p)?;
                    Ok(self.emit(Opcode::Fetch))
                }
            },

            // 模板字符串：编译所有部分并连接
            Node::TemplateString(parts) => {
                parts.iter().try_for_each(|&n| {
                    self.compile_node(n).map(|_| ())?;
                    // 将每部分转换为字符串
                    self.emit(Opcode::CallFunction {
                        arg_count: 1,
                        kind: FunctionKind::Internal(InternalFunction::String),
                    });
                    Ok(())
                })?;

                self.emit(Opcode::PushNumber(Decimal::from(parts.len())));
                self.emit(Opcode::Array);
                self.emit(Opcode::PushString(Arc::from("")));
                Ok(self.emit(Opcode::Join))
            },

            // 切片操作：编译对象和边界，然后执行切片
            Node::Slice { node, to, from } => {
                self.compile_node(node)?;
                if let Some(t) = to {
                    self.compile_node(t)?;
                } else {
                    // 如果没有指定结束位置，使用长度-1
                    self.emit(Opcode::Len);
                    self.emit(Opcode::PushNumber(dec!(1)));
                    self.emit(Opcode::Subtract);
                }

                if let Some(f) = from {
                    self.compile_node(f)?;
                } else {
                    // 如果没有指定开始位置，使用0
                    self.emit(Opcode::PushNumber(dec!(0)));
                }

                Ok(self.emit(Opcode::Slice))
            },

            // 区间：编译左右边界
            Node::Interval { left, right, left_bracket, right_bracket } => {
                self.compile_node(left)?;
                self.compile_node(right)?;
                Ok(self.emit(Opcode::Interval {
                    left_bracket: *left_bracket,
                    right_bracket: *right_bracket,
                }))
            },

            // 条件表达式：三元操作符 condition ? on_true : on_false
            Node::Conditional { condition, on_true, on_false } => {
                self.compile_node(condition)?;
                let otherwise = self.emit(Opcode::Jump(Jump::IfFalse, 0)); // 条件为假时跳转

                self.emit(Opcode::Pop); // 清理条件值
                self.compile_node(on_true)?; // 编译真值分支
                let end = self.emit(Opcode::Jump(Jump::Forward, 0)); // 跳过假值分支

                // 回填假值分支跳转
                self.replace(
                    otherwise,
                    Opcode::Jump(Jump::IfFalse, (end - otherwise) as u32),
                );
                self.emit(Opcode::Pop); // 清理栈
                let b = self.compile_node(on_false)?; // 编译假值分支
                self.replace(
                    end,
                    Opcode::Jump(Jump::Forward, (b - end) as u32),
                );

                Ok(b)
            },

            // 一元操作：+x, -x, !x
            Node::Unary { node, operator } => {
                let curr = self.compile_node(node)?;
                match *operator {
                    Operator::Arithmetic(ArithmeticOperator::Add) => Ok(curr), // +x 不变
                    Operator::Arithmetic(ArithmeticOperator::Subtract) => {
                        Ok(self.emit(Opcode::Negate)) // -x 取负
                    },
                    Operator::Logical(LogicalOperator::Not) => {
                        Ok(self.emit(Opcode::Not))
                    }, // !x 逻辑非
                    _ => Err(CompilerError::UnknownUnaryOperator {
                        operator: operator.to_string(),
                    }),
                }
            },

            // 二元操作：处理各种二元操作符
            Node::Binary { left, right, operator } => match *operator {
                // 相等比较
                Operator::Comparison(ComparisonOperator::Equal) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Equal))
                },

                // 不等比较：等于操作后取非
                Operator::Comparison(ComparisonOperator::NotEqual) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    self.emit(Opcode::Equal);
                    Ok(self.emit(Opcode::Not))
                },

                // 逻辑或：短路求值
                Operator::Logical(LogicalOperator::Or) => {
                    self.compile_node(left)?;
                    let end = self.emit(Opcode::Jump(Jump::IfTrue, 0)); // 左值为真时跳过右值
                    self.emit(Opcode::Pop);
                    let r = self.compile_node(right)?;
                    self.replace(
                        end,
                        Opcode::Jump(Jump::IfTrue, (r - end) as u32),
                    );
                    Ok(r)
                },

                // 逻辑与：短路求值
                Operator::Logical(LogicalOperator::And) => {
                    self.compile_node(left)?;
                    let end = self.emit(Opcode::Jump(Jump::IfFalse, 0)); // 左值为假时跳过右值
                    self.emit(Opcode::Pop);
                    let r = self.compile_node(right)?;
                    self.replace(
                        end,
                        Opcode::Jump(Jump::IfFalse, (r - end) as u32),
                    );
                    Ok(r)
                },

                // 空值合并：左值为null时使用右值
                Operator::Logical(LogicalOperator::NullishCoalescing) => {
                    self.compile_node(left)?;
                    let end = self.emit(Opcode::Jump(Jump::IfNotNull, 0)); // 左值不为null时跳过右值
                    self.emit(Opcode::Pop);
                    let r = self.compile_node(right)?;
                    self.replace(
                        end,
                        Opcode::Jump(Jump::IfNotNull, (r - end) as u32),
                    );
                    Ok(r)
                },

                // 包含检查
                Operator::Comparison(ComparisonOperator::In) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::In))
                },

                // 不包含检查：包含检查后取非
                Operator::Comparison(ComparisonOperator::NotIn) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    self.emit(Opcode::In);
                    Ok(self.emit(Opcode::Not))
                },

                // 各种比较操作
                Operator::Comparison(ComparisonOperator::LessThan) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Compare(Compare::Less)))
                },
                Operator::Comparison(ComparisonOperator::LessThanOrEqual) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Compare(Compare::LessOrEqual)))
                },
                Operator::Comparison(ComparisonOperator::GreaterThan) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Compare(Compare::More)))
                },
                Operator::Comparison(
                    ComparisonOperator::GreaterThanOrEqual,
                ) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Compare(Compare::MoreOrEqual)))
                },

                // 算术操作
                Operator::Arithmetic(ArithmeticOperator::Add) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Add))
                },
                Operator::Arithmetic(ArithmeticOperator::Subtract) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Subtract))
                },
                Operator::Arithmetic(ArithmeticOperator::Multiply) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Multiply))
                },
                Operator::Arithmetic(ArithmeticOperator::Divide) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Divide))
                },
                Operator::Arithmetic(ArithmeticOperator::Modulus) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Modulo))
                },
                Operator::Arithmetic(ArithmeticOperator::Power) => {
                    self.compile_node(left)?;
                    self.compile_node(right)?;
                    Ok(self.emit(Opcode::Exponent))
                },
                _ => Err(CompilerError::UnknownBinaryOperator {
                    operator: operator.to_string(),
                }),
            },

            // 函数调用：验证参数数量并生成调用指令
            Node::FunctionCall { kind, arguments } => match kind {
                FunctionKind::Internal(_)
                | FunctionKind::Deprecated(_)
                | FunctionKind::Custom(_) => {
                    let function = FunctionRegistry::get_definition(kind)
                        .ok_or_else(|| CompilerError::UnknownFunction {
                            name: kind.to_string(),
                        })?;

                    // 验证参数数量
                    let min_params = function.required_parameters();
                    let max_params =
                        min_params + function.optional_parameters();
                    if arguments.len() < min_params
                        || arguments.len() > max_params
                    {
                        return Err(CompilerError::InvalidFunctionCall {
                            name: kind.to_string(),
                            message: "无效的参数数量".to_string(),
                        });
                    }

                    // 编译所有参数
                    for i in 0..arguments.len() {
                        self.compile_argument(kind, arguments, i)?;
                    }

                    Ok(self.emit(Opcode::CallFunction {
                        kind: kind.clone(),
                        arg_count: arguments.len() as u32,
                    }))
                },

                // 闭包函数：特殊处理各种闭包操作
                FunctionKind::Closure(c) => match c {
                    // all函数：检查所有元素是否满足条件
                    ClosureFunction::All => {
                        self.compile_argument(kind, arguments, 0)?; // 编译数组参数
                        self.emit(Opcode::Begin);
                        let mut loop_break: usize = 0;
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?; // 编译条件表达式
                            loop_break = c.emit(Opcode::Jump(Jump::IfFalse, 0)); // 条件为假时跳出
                            c.emit(Opcode::Pop);
                            Ok(())
                        })?;
                        let e = self.emit(Opcode::PushBool(true)); // 默认返回true
                        self.replace(
                            loop_break,
                            Opcode::Jump(
                                Jump::IfFalse,
                                (e - loop_break) as u32,
                            ),
                        );
                        Ok(self.emit(Opcode::End))
                    },

                    // none函数：检查没有元素满足条件
                    ClosureFunction::None => {
                        self.compile_argument(kind, arguments, 0)?;
                        self.emit(Opcode::Begin);
                        let mut loop_break: usize = 0;
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?;
                            c.emit(Opcode::Not); // 对条件取非
                            loop_break = c.emit(Opcode::Jump(Jump::IfFalse, 0));
                            c.emit(Opcode::Pop);
                            Ok(())
                        })?;
                        let e = self.emit(Opcode::PushBool(true));
                        self.replace(
                            loop_break,
                            Opcode::Jump(
                                Jump::IfFalse,
                                (e - loop_break) as u32,
                            ),
                        );
                        Ok(self.emit(Opcode::End))
                    },

                    // some函数：检查至少有一个元素满足条件
                    ClosureFunction::Some => {
                        self.compile_argument(kind, arguments, 0)?;
                        self.emit(Opcode::Begin);
                        let mut loop_break: usize = 0;
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?;
                            loop_break = c.emit(Opcode::Jump(Jump::IfTrue, 0)); // 条件为真时跳出
                            c.emit(Opcode::Pop);
                            Ok(())
                        })?;
                        let e = self.emit(Opcode::PushBool(false)); // 默认返回false
                        self.replace(
                            loop_break,
                            Opcode::Jump(Jump::IfTrue, (e - loop_break) as u32),
                        );
                        Ok(self.emit(Opcode::End))
                    },

                    // one函数：检查恰好有一个元素满足条件
                    ClosureFunction::One => {
                        self.compile_argument(kind, arguments, 0)?;
                        self.emit(Opcode::Begin);
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?;
                            c.emit_cond(|c| {
                                c.emit(Opcode::IncrementCount); // 满足条件时增加计数
                            });
                            Ok(())
                        })?;
                        self.emit(Opcode::GetCount);
                        self.emit(Opcode::PushNumber(dec!(1)));
                        self.emit(Opcode::Equal); // 检查计数是否等于1
                        Ok(self.emit(Opcode::End))
                    },

                    // filter函数：过滤满足条件的元素
                    ClosureFunction::Filter => {
                        self.compile_argument(kind, arguments, 0)?;
                        self.emit(Opcode::Begin);
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?;
                            c.emit_cond(|c| {
                                c.emit(Opcode::IncrementCount);
                                c.emit(Opcode::Pointer); // 添加当前元素到结果
                            });
                            Ok(())
                        })?;
                        self.emit(Opcode::GetCount);
                        self.emit(Opcode::End);
                        Ok(self.emit(Opcode::Array))
                    },

                    // map函数：转换每个元素
                    ClosureFunction::Map => {
                        self.compile_argument(kind, arguments, 0)?;
                        self.emit(Opcode::Begin);
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?; // 应用转换表达式
                            Ok(())
                        })?;
                        self.emit(Opcode::GetLen);
                        self.emit(Opcode::End);
                        Ok(self.emit(Opcode::Array))
                    },

                    // flatMap函数：转换并展平结果
                    ClosureFunction::FlatMap => {
                        self.compile_argument(kind, arguments, 0)?;
                        self.emit(Opcode::Begin);
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?;
                            Ok(())
                        })?;
                        self.emit(Opcode::GetLen);
                        self.emit(Opcode::End);
                        self.emit(Opcode::Array);
                        Ok(self.emit(Opcode::Flatten)) // 展平结果数组
                    },

                    // count函数：计算满足条件的元素数量
                    ClosureFunction::Count => {
                        self.compile_argument(kind, arguments, 0)?;
                        self.emit(Opcode::Begin);
                        self.emit_loop(|c| {
                            c.compile_argument(kind, arguments, 1)?;
                            c.emit_cond(|c| {
                                c.emit(Opcode::IncrementCount);
                            });
                            Ok(())
                        })?;
                        self.emit(Opcode::GetCount);
                        Ok(self.emit(Opcode::End))
                    },
                },
            },

            // 方法调用：验证参数数量并生成调用指令
            Node::MethodCall { kind, this, arguments } => {
                let method =
                    MethodRegistry::get_definition(kind).ok_or_else(|| {
                        CompilerError::UnknownFunction {
                            name: kind.to_string(),
                        }
                    })?;

                self.compile_node(this)?; // 编译调用对象

                // 验证参数数量（方法的第一个参数是this，所以减1）
                let min_params = method.required_parameters() - 1;
                let max_params = min_params + method.optional_parameters();
                if arguments.len() < min_params || arguments.len() > max_params
                {
                    return Err(CompilerError::InvalidMethodCall {
                        name: kind.to_string(),
                        message: "Invalid number of arguments".to_string(),
                    });
                }

                // 编译所有参数
                for i in 0..arguments.len() {
                    self.compile_argument(kind, arguments, i)?;
                }

                Ok(self.emit(Opcode::CallMethod {
                    kind: kind.clone(),
                    arg_count: arguments.len() as u32,
                }))
            },

            // 错误节点：不应该在编译时遇到
            Node::Error { .. } => Err(CompilerError::UnexpectedErrorNode),
        }
    }
}

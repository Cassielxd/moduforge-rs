//! 虚拟机核心实现
//! 
//! 实现基于栈的虚拟机，用于执行编译器生成的操作码。
//! 支持变量操作、函数调用、控制流、算术运算等完整的表达式执行功能。

use crate::compiler::{Compare, FetchFastTarget, Jump, Opcode};
use crate::functions::arguments::Arguments;
use crate::functions::registry::FunctionRegistry;
use crate::functions::{internal, MethodRegistry};
use crate::variable::Variable;
use crate::variable::Variable::*;
use crate::vm::error::VMError::*;
use crate::vm::error::VMResult;
use crate::vm::interval::{VmInterval, VmIntervalData};
use ahash::{HashMap, HashMapExt};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use std::rc::Rc;
use std::string::String as StdString;

/// 作用域结构
/// 
/// 用于闭包函数执行时的作用域管理，保存数组迭代状态
#[derive(Debug)]
pub struct Scope {
    /// 当前迭代的数组
    array: Variable,
    /// 数组长度
    len: usize,
    /// 当前迭代位置
    iter: usize,
    /// 满足条件的元素计数
    count: usize,
}

/// 虚拟机主结构
/// 
/// 基于栈的虚拟机实现，负责执行操作码序列
#[derive(Debug)]
pub struct VM {
    /// 作用域栈：用于嵌套作用域管理
    scopes: Vec<Scope>,
    /// 操作数栈：存储运算过程中的中间值
    stack: Vec<Variable>
}

impl VM {
    /// 创建新的虚拟机实例
    /// 
    /// 初始化空的作用域栈和操作数栈
    pub fn new() -> Self {
        Self {
            scopes: Default::default(),
            stack: Default::default()
        }
    }

    /// 运行字节码
    /// 
    /// 清空栈状态并执行给定的操作码序列
    /// 
    /// # 参数
    /// * `bytecode` - 要执行的操作码数组
    /// * `env` - 执行环境变量
    /// 
    /// # 返回值
    /// * `VMResult<Variable>` - 执行结果或错误
    pub fn run(&mut self, bytecode: &[Opcode], env: Variable) -> VMResult<Variable> {
        self.stack.clear();
        self.scopes.clear();

        let s = VMInner::new(bytecode, &mut self.stack, &mut self.scopes).run(env);
        Ok(s?)
    }
}

/// 虚拟机内部执行器
/// 
/// 实际的字节码执行逻辑，管理程序计数器和栈操作
struct VMInner<'parent_ref, 'bytecode_ref> {
    /// 作用域栈引用
    scopes: &'parent_ref mut Vec<Scope>,
    /// 操作数栈引用
    stack: &'parent_ref mut Vec<Variable>,
    /// 字节码数组引用
    bytecode: &'bytecode_ref [Opcode],
    /// 程序计数器：指向当前执行的操作码位置
    ip: u32,
}

impl<'arena, 'parent_ref, 'bytecode_ref> VMInner<'parent_ref, 'bytecode_ref> {
    /// 创建新的虚拟机内部执行器
    /// 
    /// # 参数
    /// * `bytecode` - 字节码数组
    /// * `stack` - 操作数栈
    /// * `scopes` - 作用域栈
    pub fn new(
        bytecode: &'bytecode_ref [Opcode],
        stack: &'parent_ref mut Vec<Variable>,
        scopes: &'parent_ref mut Vec<Scope>,
    ) -> Self {
        Self {
            ip: 0,
            scopes,
            stack,
            bytecode,
        }
    }

    /// 向栈中压入值
    /// 
    /// # 参数
    /// * `var` - 要压入的变量
    fn push(&mut self, var: Variable) {
        self.stack.push(var);
    }

    /// 从栈中弹出值
    /// 
    /// # 返回值
    /// * `VMResult<Variable>` - 弹出的变量或栈为空的错误
    fn pop(&mut self) -> VMResult<Variable> {
        self.stack.pop().ok_or_else(|| StackOutOfBounds {
            stack: format!("{:?}", self.stack),
        })
    }

    /// 执行字节码
    /// 
    /// 主执行循环，逐个处理操作码直到程序结束
    /// 
    /// # 参数
    /// * `env` - 执行环境变量
    /// 
    /// # 返回值
    /// * `VMResult<Variable>` - 执行结果或错误
    pub fn run(&mut self, env: Variable) -> VMResult<Variable> {
        if self.ip != 0 {
            self.ip = 0;
        }

        while self.ip < self.bytecode.len() as u32 {
            let op = self
                .bytecode
                .get(self.ip as usize)
                .ok_or_else(|| OpcodeOutOfBounds {
                    bytecode: format!("{:?}", self.bytecode),
                    index: self.ip as usize,
                })?;

            self.ip += 1;

            match op {
                // 基本值压栈操作
                // 压入空值
                Opcode::PushNull => self.push(Null),
                // 压入布尔值
                Opcode::PushBool(b) => self.push(Bool(*b)),
                // 压入数字
                Opcode::PushNumber(n) => self.push(Number(*n)),
                // 压入字符串
                Opcode::PushString(s) => self.push(String(Rc::from(s.as_ref()))),
                // 弹出栈顶值（丢弃）
                Opcode::Pop => {
                    self.pop()?;
                }
                
                // 变量访问操作
                // 通用属性访问：object[key] 或 array[index]
                Opcode::Fetch => {
                    let b = self.pop()?; // 索引或键
                    let a = self.pop()?; // 对象或数组

                    match (a, b) {
                        // 对象属性访问
                        (Object(o), String(s)) => {
                            let obj = o.borrow();
                            self.push(obj.get(s.as_ref()).cloned().unwrap_or(Null));
                        }
                        // 数组索引访问
                        (Array(a), Number(n)) => {
                            let arr = a.borrow();
                            self.push(
                                arr.get(n.to_usize().ok_or_else(|| OpcodeErr {
                                    opcode: "Fetch".into(),
                                    message: "转换为 usize 失败".into(),
                                })?)
                                .cloned()
                                .unwrap_or(Null),
                            )
                        }
                        // 字符串字符访问
                        (String(str), Number(n)) => {
                            let index = n.to_usize().ok_or_else(|| OpcodeErr {
                                opcode: "Fetch".into(),
                                message: "转换为 usize 失败".into(),
                            })?;

                            if let Some(slice) = str.get(index..index + 1) {
                                self.push(String(Rc::from(slice)));
                            } else {
                                self.push(Null)
                            };
                        }
                        _ => self.push(Null),
                    }
                }
                
                // 快速路径访问：优化的属性访问
                Opcode::FetchFast(path) => {
                    let variable = path.iter().fold(Null, |v, p| match p {
                        FetchFastTarget::Root => env.clone(),
                        FetchFastTarget::String(key) => match v {
                            Object(obj) => {
                                let obj_ref = obj.borrow();
                                obj_ref.get(key.as_ref()).cloned().unwrap_or(Null)
                            }
                            _ => Null,
                        },
                        FetchFastTarget::Number(num) => match v {
                            Array(arr) => {
                                let arr_ref = arr.borrow();
                                arr_ref.get(*num as usize).cloned().unwrap_or(Null)
                            }
                            _ => Null,
                        },
                    });

                    self.push(variable);
                }
                
                // 获取环境变量
                Opcode::FetchEnv(f) => match &env {
                    Object(o) => {
                        let obj = o.borrow();
                        match obj.get(f.as_ref()) {
                            None => self.push(Null),
                            Some(v) => self.push(v.clone()),
                        }
                    }
                    Null => self.push(Null),
                    _ => {
                        return Err(OpcodeErr {
                            opcode: "FetchEnv".into(),
                            message: "不支持的类型".into(),
                        });
                    }
                },
                
                // 获取根环境
                Opcode::FetchRootEnv => {
                    self.push(env.clone());
                }
                
                // 一元运算操作
                // 数字取负
                Opcode::Negate => {
                    let a = self.pop()?;
                    match a {
                        Number(n) => {
                            self.push(Number(-n));
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Negate".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                
                // 逻辑非
                Opcode::Not => {
                    let a = self.pop()?;
                    match a {
                        Bool(b) => self.push(Bool(!b)),
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Not".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                
                // 比较运算操作
                // 相等比较
                Opcode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        // 数字相等比较
                        (Number(a), Number(b)) => {
                            self.push(Bool(a == b));
                        }
                        // 布尔值相等比较
                        (Bool(a), Bool(b)) => {
                            self.push(Bool(a == b));
                        }
                        // 字符串相等比较
                        (String(a), String(b)) => {
                            self.push(Bool(a == b));
                        }
                        // 空值相等比较
                        (Null, Null) => {
                            self.push(Bool(true));
                        }
                        // 动态类型（日期）相等比较
                        (Dynamic(a), Dynamic(b)) => {
                            let a = a.as_date();
                            let b = b.as_date();

                            self.push(Bool(a.is_some() && b.is_some() && a == b));
                        }
                        // 不同类型不相等
                        _ => {
                            self.push(Bool(false));
                        }
                    }
                }
                
                // 控制流操作
                // 跳转指令：根据不同条件执行跳转
                Opcode::Jump(kind, j) => match kind {
                    // 无条件向前跳转
                    Jump::Forward => self.ip += j,
                    // 无条件向后跳转
                    Jump::Backward => self.ip -= j,
                    // 条件跳转：栈顶值为true时跳转
                    Jump::IfTrue => {
                        let a = self.stack.last().ok_or_else(|| OpcodeErr {
                            opcode: "JumpIfTrue".into(),
                            message: "未定义的对象键".into(),
                        })?;
                        match a {
                            Bool(a) => {
                                if *a {
                                    self.ip += j;
                                }
                            }
                            _ => {
                                return Err(OpcodeErr {
                                    opcode: "JumpIfTrue".into(),
                                    message: "Unsupported type".into(),
                                });
                            }
                        }
                    }
                    // 条件跳转：栈顶值为false时跳转
                    Jump::IfFalse => {
                        let a = self.stack.last().ok_or_else(|| OpcodeErr {
                            opcode: "JumpIfFalse".into(),
                            message: "空数组".into(),
                        })?;

                        match a {
                            Bool(a) => {
                                if !*a {
                                    self.ip += j;
                                }
                            }
                            _ => {
                                return Err(OpcodeErr {
                                    opcode: "JumpIfFalse".into(),
                                    message: "不支持的类型".into(),
                                });
                            }
                        }
                    }
                    // 条件跳转：栈顶值不为null时跳转
                    Jump::IfNotNull => {
                        let a = self.stack.last().ok_or_else(|| OpcodeErr {
                            opcode: "JumpIfNull".into(),
                            message: "空数组".into(),
                        })?;

                        match a {
                            Null => {}
                            _ => {
                                self.ip += j;
                            }
                        }
                    }
                    // 条件跳转：迭代结束时跳转
                    Jump::IfEnd => {
                        let scope = self.scopes.last().ok_or_else(|| OpcodeErr {
                            opcode: "JumpIfEnd".into(),
                            message: "空栈".into(),
                        })?;

                        if scope.iter >= scope.len {
                            self.ip += j;
                        }
                    }
                },
                
                // 成员关系操作
                // 包含检查：检查元素是否在集合或区间中
                Opcode::In => {
                    let b = self.pop()?; // 容器（数组或区间）
                    let a = self.pop()?; // 要检查的元素

                    match (a, &b) {
                        // 检查数字是否在数组中
                        (Number(a), Array(b)) => {
                            let arr = b.borrow();
                            let is_in = arr.iter().any(|b| match b {
                                Number(b) => a == *b,
                                _ => false,
                            });

                            self.push(Bool(is_in));
                        }
                        // 检查数字是否在区间中
                        (Number(v), Dynamic(d)) => {
                            let Some(i) = d.as_any().downcast_ref::<VmInterval>() else {
                                return Err(OpcodeErr {
                                    opcode: "In".into(),
                                    message: "不支持的类型".into(),
                                });
                            };

                            self.push(Bool(i.includes(VmIntervalData::Number(v)).map_err(
                                |err| OpcodeErr {
                                    opcode: "In".into(),
                                    message: err.to_string(),
                                },
                            )?));
                        }
                        (Dynamic(d), Dynamic(i)) => {
                            let Some(d) = d.as_date() else {
                                return Err(OpcodeErr {
                                    opcode: "In".into(),
                                    message: "不支持的类型".into(),
                                });
                            };

                            let Some(i) = i.as_any().downcast_ref::<VmInterval>() else {
                                return Err(OpcodeErr {
                                    opcode: "In".into(),
                                    message: "不支持的类型".into(),
                                });
                            };

                            self.push(Bool(i.includes(VmIntervalData::Date(d.clone())).map_err(
                                |err| OpcodeErr {
                                    opcode: "In".into(),
                                    message: err.to_string(),
                                },
                            )?));
                        }
                        (Dynamic(a), Array(arr)) => {
                            let Some(a) = a.as_date() else {
                                return Err(OpcodeErr {
                                    opcode: "In".into(),
                                    message: "不支持的类型".into(),
                                });
                            };

                            let arr = arr.borrow();
                            let is_in = arr.iter().any(|b| match b {
                                Dynamic(b) => Some(a) == b.as_date(),
                                _ => false,
                            });

                            self.push(Bool(is_in));
                        }
                        (String(a), Array(b)) => {
                            let arr = b.borrow();
                            let is_in = arr.iter().any(|b| match b {
                                String(b) => &a == b,
                                _ => false,
                            });

                            self.push(Bool(is_in));
                        }
                        (String(a), Object(b)) => {
                            let obj = b.borrow();
                            self.push(Bool(obj.contains_key(a.as_ref())));
                        }
                        (Bool(a), Array(b)) => {
                            let arr = b.borrow();
                            let is_in = arr.iter().any(|b| match b {
                                Bool(b) => a == *b,
                                _ => false,
                            });

                            self.push(Bool(is_in));
                        }
                        (Null, Array(b)) => {
                            let arr = b.borrow();
                            let is_in = arr.iter().any(|b| match b {
                                Null => true,
                                _ => false,
                            });

                            self.push(Bool(is_in));
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "In".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Compare(comparison) => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    fn compare<T: Ord>(a: &T, b: &T, comparison: &Compare) -> bool {
                        match comparison {
                            Compare::More => a > b,
                            Compare::MoreOrEqual => a >= b,
                            Compare::Less => a < b,
                            Compare::LessOrEqual => a <= b,
                        }
                    }

                    match (a, b) {
                        (Number(a), Number(b)) => self.push(Bool(compare(&a, &b, comparison))),
                        (Dynamic(a), Dynamic(b)) => {
                            let (a, b) = match (a.as_date(), b.as_date()) {
                                (Some(a), Some(b)) => (a, b),
                                _ => {
                                    return Err(OpcodeErr {
                                        opcode: "Compare".into(),
                                        message: "不支持的类型".into(),
                                    })
                                }
                            };

                            self.push(Bool(compare(a, b, comparison)));
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Compare".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Number(a), Number(b)) => self.push(Number(a + b)),
                        (String(a), String(b)) => {
                            let mut c = StdString::with_capacity(a.len() + b.len());

                            c.push_str(a.as_ref());
                            c.push_str(b.as_ref());

                            self.push(String(Rc::from(c.as_str())));
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Add".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Subtract => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Number(a), Number(b)) => self.push(Number(a - b)),
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Subtract".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Multiply => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Number(a), Number(b)) => self.push(Number(a * b)),
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Multiply".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Divide => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Number(a), Number(b)) => self.push(Number(a / b)),
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Divide".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Modulo => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Number(a), Number(b)) => self.push(Number(a % b)),
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Modulo".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Exponent => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Number(a), Number(b)) => {
                            let result = a
                                .checked_powd(b)
                                .or_else(|| Decimal::from_f64(a.to_f64()?.powf(b.to_f64()?)))
                                .ok_or_else(|| OpcodeErr {
                                    opcode: "Exponent".into(),
                                    message: "计算指数失败".into(),
                                })?;

                            self.push(Number(result));
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Exponent".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Interval {
                    left_bracket,
                    right_bracket,
                } => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (&a, &b) {
                        (Number(a), Number(b)) => {
                            let interval = VmInterval {
                                left_bracket: *left_bracket,
                                right_bracket: *right_bracket,
                                left: VmIntervalData::Number(*a),
                                right: VmIntervalData::Number(*b),
                            };

                            self.push(Dynamic(Rc::new(interval)));
                        }
                        (Dynamic(a), Dynamic(b)) => {
                            let (a, b) = match (a.as_date(), b.as_date()) {
                                (Some(a), Some(b)) => (a, b),
                                _ => {
                                    return Err(OpcodeErr {
                                        opcode: "Interval".into(),
                                        message: "不支持的类型".into(),
                                    })
                                }
                            };

                            let interval = VmInterval {
                                left_bracket: *left_bracket,
                                right_bracket: *right_bracket,
                                left: VmIntervalData::Date(a.clone()),
                                right: VmIntervalData::Date(b.clone()),
                            };

                            self.push(Dynamic(Rc::new(interval)));
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Interval".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Join => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    let (Array(a), String(separator)) = (a, &b) else {
                        return Err(OpcodeErr {
                            opcode: "Join".into(),
                            message: "不支持的类型".into(),
                        });
                    };

                    let arr = a.borrow();
                    let parts = arr
                        .iter()
                        .enumerate()
                        .map(|(i, var)| match var {
                            String(str) => Ok(str.clone()),
                            _ => Err(OpcodeErr {
                                opcode: "Join".into(),
                                message: format!("数组中索引 {i} 的类型不支持"),
                            }),
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    let str_capacity = parts
                        .iter()
                        .fold(separator.len() * (parts.len() - 1), |acc, s| acc + s.len());

                    let mut s = StdString::with_capacity(str_capacity);
                    let mut it = parts.into_iter().peekable();
                    while let Some(part) = it.next() {
                        s.push_str(part.as_ref());
                        if it.peek().is_some() {
                            s.push_str(separator);
                        }
                    }

                    self.push(String(Rc::from(s)));
                }
                Opcode::Slice => {
                    let from_var = self.pop()?;
                    let to_var = self.pop()?;
                    let current = self.pop()?;

                    match (from_var, to_var) {
                        (Number(f), Number(t)) => {
                            let from = f.to_usize().ok_or_else(|| OpcodeErr {
                                opcode: "Slice".into(),
                                message: "获取范围失败".into(),
                            })?;
                            let to = t.to_usize().ok_or_else(|| OpcodeErr {
                                opcode: "Slice".into(),
                                message: "获取范围失败".into(),
                            })?;

                            match current {
                                Array(a) => {
                                    let arr = a.borrow();
                                    let slice = arr.get(from..=to).ok_or_else(|| OpcodeErr {
                                        opcode: "Slice".into(),
                                        message: "索引超出范围".into(),
                                    })?;

                                    self.push(Variable::from_array(slice.to_vec()));
                                }
                                String(s) => {
                                    let slice = s.get(from..=to).ok_or_else(|| OpcodeErr {
                                        opcode: "Slice".into(),
                                        message: "索引超出范围".into(),
                                    })?;

                                    self.push(String(Rc::from(slice)));
                                }
                                _ => {
                                    return Err(OpcodeErr {
                                        opcode: "Slice".into(),
                                        message: "不支持的类型".into(),
                                    });
                                }
                            }
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Slice".into(),
                                message: "不支持的类型".into(),
                            });
                        }
                    }
                }
                Opcode::Array => {
                    let size = self.pop()?;
                    let Number(s) = size else {
                        return Err(OpcodeErr {
                            opcode: "Array".into(),
                            message: "不支持的类型".into(),
                        });
                    };

                    let to = s.round().to_usize().ok_or_else(|| OpcodeErr {
                        opcode: "Array".into(),
                        message: "提取参数失败".into(),
                    })?;

                    let mut arr = Vec::with_capacity(to);
                    for _ in 0..to {
                        arr.push(self.pop()?);
                    }
                    arr.reverse();

                    self.push(Variable::from_array(arr));
                }
                Opcode::Object => {
                    let size = self.pop()?;
                    let Number(s) = size else {
                        return Err(OpcodeErr {
                            opcode: "Array".into(),
                            message: "不支持的类型".into(),
                        });
                    };

                    let to = s.round().to_usize().ok_or_else(|| OpcodeErr {
                        opcode: "Array".into(),
                        message: "提取参数失败".into(),
                    })?;

                    let mut map = HashMap::with_capacity(to);
                    for _ in 0..to {
                        let value = self.pop()?;
                        let String(key) = self.pop()? else {
                            return Err(OpcodeErr {
                                opcode: "Object".into(),
                                message: "意外的键值".to_string(),
                            });
                        };

                        map.insert(key.clone(), value);
                    }

                    self.push(Variable::from_object(map));
                }
                Opcode::Len => {
                    let current = self.stack.last().ok_or_else(|| OpcodeErr {
                        opcode: "Len".into(),
                        message: "空栈".into(),
                    })?;

                    let len_var =
                        internal::imp::len(Arguments(&[current.clone()])).map_err(|err| {
                            OpcodeErr {
                                opcode: "Len".into(),
                                message: err.to_string(),
                            }
                        })?;

                    self.push(len_var);
                }
                Opcode::Flatten => {
                    let current = self.pop()?;
                    let Array(a) = current else {
                        return Err(OpcodeErr {
                            opcode: "Flatten".into(),
                            message: "不支持的类型".into(),
                        });
                    };

                    let arr = a.borrow();

                    let mut flat_arr = Vec::with_capacity(arr.len());
                    arr.iter().for_each(|v| match v {
                        Array(b) => {
                            let arr = b.borrow();
                            arr.iter().for_each(|v| flat_arr.push(v.clone()))
                        }
                        _ => flat_arr.push(v.clone()),
                    });

                    self.push(Variable::from_array(flat_arr));
                }
                Opcode::IncrementIt => {
                    let scope = self.scopes.last_mut().ok_or_else(|| OpcodeErr {
                        opcode: "IncrementIt".into(),
                        message: "空作用域".into(),
                    })?;

                    scope.iter += 1;
                }
                Opcode::IncrementCount => {
                    let scope = self.scopes.last_mut().ok_or_else(|| OpcodeErr {
                        opcode: "IncrementCount".into(),
                        message: "空作用域".into(),
                    })?;

                    scope.count += 1;
                }
                Opcode::GetCount => {
                    let scope = self.scopes.last().ok_or_else(|| OpcodeErr {
                        opcode: "GetCount".into(),
                        message: "空作用域".into(),
                    })?;

                    self.push(Number(scope.count.into()));
                }
                Opcode::GetLen => {
                    let scope = self.scopes.last().ok_or_else(|| OpcodeErr {
                        opcode: "GetLen".into(),
                        message: "空作用域".into(),
                    })?;

                    self.push(Number(scope.len.into()));
                }
                Opcode::Pointer => {
                    let scope = self.scopes.last().ok_or_else(|| OpcodeErr {
                        opcode: "Pointer".into(),
                        message: "空作用域".into(),
                    })?;

                    match &scope.array {
                        Array(a) => {
                            let a_cloned = a.clone();
                            let arr = a_cloned.borrow();
                            let variable =
                                arr.get(scope.iter).cloned().ok_or_else(|| OpcodeErr {
                                    opcode: "Pointer".into(),
                                    message: "作用域数组超出范围".into(),
                                })?;

                            self.push(variable);
                        }
                        _ => {
                            return Err(OpcodeErr {
                                opcode: "Pointer".into(),
                                message: "不支持的作用域类型".into(),
                            });
                        }
                    }
                }
                Opcode::Begin => {
                    let var = self.pop()?;
                    let maybe_scope = match &var {
                        Array(a) => {
                            let arr = a.borrow();
                            Some(Scope {
                                len: arr.len(),
                                array: var.clone(),
                                count: 0,
                                iter: 0,
                            })
                        }
                        _ => match var.dynamic::<VmInterval>().map(|s| s.to_array()).flatten() {
                            None => None,
                            Some(arr) => Some(Scope {
                                len: arr.len(),
                                array: Variable::from_array(arr),
                                count: 0,
                                iter: 0,
                            }),
                        },
                    };

                    let Some(scope) = maybe_scope else {
                        return Err(OpcodeErr {
                            opcode: "Begin".into(),
                            message: "不支持的类型".into(),
                        });
                    };

                    self.scopes.push(scope);
                }
                Opcode::End => {
                    self.scopes.pop();
                }
                Opcode::CallFunction { kind, arg_count } => {
                    let function =
                        FunctionRegistry::get_definition(kind).ok_or_else(|| OpcodeErr {
                            opcode: "CallFunction".into(),
                            message: format!("函数 `{kind}` 未找到"),
                        })?;

                    let params_start = self.stack.len().saturating_sub(*arg_count as usize);
                    let result = function
                        .call(Arguments(&self.stack[params_start..]))
                        .map_err(|err| OpcodeErr {
                            opcode: "CallFunction".into(),
                            message: format!("函数 `{kind}` 调用失败: {err}"),
                        })?;

                    self.stack.drain(params_start..);
                    self.push(result);
                }
                Opcode::CallMethod { kind, arg_count } => {
                    let method = MethodRegistry::get_definition(kind).ok_or_else(|| OpcodeErr {
                        opcode: "CallMethod".into(),
                        message: format!("方法 `{kind}` 未找到"),
                    })?;

                    let params_start = self.stack.len().saturating_sub(*arg_count as usize) - 1;
                    let result = method
                        .call(Arguments(&self.stack[params_start..]))
                        .map_err(|err| OpcodeErr {
                            opcode: "CallMethod".into(),
                            message: format!("方法 `{kind}` 调用失败: {err}"),
                        })?;

                    self.stack.drain(params_start..);
                    self.push(result);
                }
            }
        }

        self.pop()
    }
}

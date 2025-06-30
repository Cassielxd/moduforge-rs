# Expression 项目架构分析

## 项目概述

Expression 是一个高性能的表达式语言库，专门用于评估和处理 JSON 数据。它采用编译器+虚拟机的架构设计，实现了从源码到字节码再到执行结果的完整流水线。

## 核心架构

```
源码表达式 → 词法分析 → 语法分析 → 编译器 → 字节码 → 虚拟机 → 执行结果
     ↓           ↓         ↓        ↓       ↓       ↓        ↓
   "a + b"    Tokens     AST    Opcodes  Bytecode   VM    Variable
```

## 编译过程详解

### 1\. 词法分析 (Lexer)

**功能**: 将源码字符串分解为标记(Token)序列

```rust
// 输入: "a + b * 2"
// 输出: [Identifier("a"), Plus, Identifier("b"), Multiply, Number(2)]
```

### 2\. 语法分析 (Parser)

**功能**: 将 Token 序列构建为抽象语法树(AST)

```rust
// AST 结构示例:
//     +
//   /   \
//  a     *
//       / \
//      b   2
```

### 3\. 编译器 (Compiler)

**功能**: 将 AST 转换为字节码指令序列

核心操作码类型:

- **数据操作**: `PushNull`, `PushBool`, `PushString`, `PushNumber`

- **栈操作**: `Pop`, `Flatten`, `Join`

- **访问操作**: `Fetch`, `FetchEnv`, `FetchFast`

- **运算操作**: `Add`, `Subtract`, `Multiply`, `Divide`, `Negate`, `Not`

- **比较操作**: `Equal`, `Compare(More/Less/MoreOrEqual/LessOrEqual)`

- **控制流**: `Jump(Forward/Backward/IfTrue/IfFalse)`

- **函数调用**: `CallFunction`, `CallMethod`

## VM 执行过程详解

### VM 核心组件

```rust
pub struct VM {
    scopes: Vec<Scope>,    // 作用域栈，支持嵌套循环
    stack: Vec<Variable>,  // 操作数栈，存储中间计算结果
}

pub struct Scope {
    array: Variable,  // 当前迭代的数组
    len: usize,      // 数组长度
    iter: usize,     // 当前迭代位置
    count: usize,    // 计数器
}
```

### 执行机制

1. **栈机架构**: 基于操作数栈的计算模型

2. **指令指针**: `ip` 指向当前执行的指令

3. **环境变量**: `env` 提供表达式执行的上下文数据

## 具体案例分析

### 案例1: 基础算术表达式

**表达式**: `"50 * tax.percentage / 100"`

**编译过程**:

```rust
// 1. 词法分析
[Number(50), Multiply, Identifier("tax"), Dot, Identifier("percentage"), Divide, Number(100)]

// 2. 语法分析 (AST)
//        /
//      /   \
//     *     100
//   /   \
//  50    .
//       / \
//     tax  percentage

// 3. 编译为字节码
[
    PushNumber(50),                          // 推入 50
    FetchFast(vec![Root, String("tax"), String("percentage")]), // 获取 tax.percentage
    Multiply,                                // 50 * tax.percentage
    PushNumber(100),                        // 推入 100
    Divide,                                 // (50 * tax.percentage) / 100
]
```

**VM执行过程**:

```rust
// 环境: {"tax": {"percentage": 10}}
// 栈状态变化:

// 1. PushNumber(50)
stack: [50]

// 2. FetchFast([Root, "tax", "percentage"])
//    从环境中获取 tax.percentage = 10
stack: [50, 10]

// 3. Multiply
//    弹出 10 和 50，计算 50 * 10 = 500
stack: [500]

// 4. PushNumber(100)
stack: [500, 100]

// 5. Divide
//    弹出 100 和 500，计算 500 / 100 = 5
stack: [5]

// 最终结果: 5
```

### 案例2: 条件表达式与区间

**表达式**: `"age >= 18 and score in [60..100]"`

**编译过程**:

```rust
// 字节码序列
[
    FetchFast(vec![Root, String("age")]),    // 获取 age
    PushNumber(18),                          // 推入 18
    Compare(MoreOrEqual),                    // age >= 18
    FetchFast(vec![Root, String("score")]),  // 获取 score
    PushNumber(60),                          // 推入 60
    PushNumber(100),                         // 推入 100
    Interval {                               // 创建区间 [60..100]
        left_bracket: LeftSquareBracket,
        right_bracket: RightSquareBracket,
    },
    In,                                      // score in [60..100]
    // 逻辑与运算的跳转指令...
]
```

**VM执行过程**:

```rust
// 环境: {"age": 20, "score": 85}

// 1. 计算 age >= 18
stack: [20, 18] → [true]  // 20 >= 18 = true

// 2. 计算 score in [60..100]
stack: [true, 85, 60, 100] 
→ [true, 85, Interval{left:60, right:100, brackets:[,]}]
→ [true, true]  // 85 in [60..100] = true

// 3. 逻辑与运算
stack: [true, true] → [true]  // true and true = true
```

### 案例3: 高性能重复执行

**使用 Isolate 进行高性能计算**:

```rust
use zen_expression::Isolate;
use serde_json::json;

fn performance_example() {
    let context = json!({ "items": [1, 2, 3, 4, 5] });
    let mut isolate = Isolate::with_environment(context.into());

    // 编译一次，重复执行多次 - 高性能关键
    for _ in 0..1_000_000 {
        let result = isolate.run_standard("sum(items) * 0.1").unwrap();
        // 结果: (1+2+3+4+5) * 0.1 = 1.5
    }
}
```

**性能优化机制**:

1. **内存重用**: `UnsafeArena` 分配器重用内存块

2. **编译缓存**: 字节码可以预编译并重复使用

3. **无垃圾回收**: 基于 Rust 的零成本抽象

## Isolate 的核心价值

### 内存管理优化

```rust
pub struct Isolate<'arena> {
    lexer: Lexer<'arena>,
    compiler: Compiler,
    vm: VM,
    bump: UnsafeArena<'arena>,  // Arena分配器
    environment: Option<Variable>,
    references: HashMap<String, Variable>,
}
```

### 执行模式

1. **一次性执行**:

```rust
let result = evaluate_expression("a + b", context.into())?;
```

1. **高性能重复执行**:

```rust
let mut isolate = Isolate::with_environment(context.into());
for _ in 0..1000 {
    let result = isolate.run_standard("a + b")?;
}
```

1. **预编译模式**:

```rust
let expr = isolate.compile_standard("a + b")?;
for context in contexts {
    let result = expr.evaluate(context)?;
}
```

## 支持的数据类型和操作

### 变量类型

- **基础类型**: `Null`, `Bool`, `Number(Decimal)`, `String`

- **集合类型**: `Array`, `Object`

- **特殊类型**: `Interval`(区间), `Date`(日期)

### 内置函数

- **数学函数**: `abs()`, `sum()`, `min()`, `max()`, `round()`

- **字符串函数**: `len()`, `contains()`, `startsWith()`, `endsWith()`

- **日期函数**: `date()`, `time()`, `now()`

- **数组函数**: `map()`, `filter()`, `reduce()`

### 操作符支持

- **算术**: `+`, `-`, `*`, `/`, `%`, `^`

- **比较**: `==`, `!=`, `>`, `<`, `>=`, `<=`

- **逻辑**: `and`, `or`, `not`

- **成员**: `in`, `not in`

- **区间**: `[a..b]`, `(a..b)`, `[a..b)`, `(a..b]`

## 错误处理机制

```rust
pub enum IsolateError {
    LexerError { source: LexerError },     // 词法错误
    ParserError { source: ParserError },   // 语法错误  
    CompilerError { source: CompilerError }, // 编译错误
    VMError { source: VMError },           // 运行时错误
    ValueCastError,                        // 类型转换错误
    ReferenceError,                        // 引用错误
    MissingContextReference,               // 缺失上下文引用
}
```

## 总结

 Expression 通过精心设计的编译器+VM架构，实现了：

1. **高性能**: 基于字节码的执行，内存重用机制

2. **类型安全**: Rust 的类型系统保证内存安全

3. **易用性**: 直观的表达式语法，丰富的内置函数

4. **可扩展**: 支持自定义函数和操作符

5. **跨平台**: 可编译到包括 WASM 在内的多种目标平台

这使得它特别适合于规则引擎、数据处理、条件评估等高频计算场景。
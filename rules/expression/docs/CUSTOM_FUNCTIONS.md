# ModuForge Expression 自定义函数功能

ModuForge Expression 支持在运行时注册自定义函数，这些函数可以在表达式中直接调用，就像内置函数（如 `date()`、`len()` 等）一样。

## 核心功能

### 1. 函数注册

使用 `Isolate::register_custom_function()` 注册自定义函数：

```rust
use moduforge_rules_expression::{Isolate, Variable, VariableType};

// 注册一个简单的函数
Isolate::register_custom_function(
    "myFunction".to_string(),                    // 函数名
    vec![VariableType::String],                  // 参数类型列表
    VariableType::Number,                        // 返回类型
    |args, state_opt| {                          // 执行器闭包
        let input = args.str(0)?;                // 获取第一个参数
        let result = input.len() as f64;
        Ok(Variable::Number(rust_decimal::Decimal::from_f64_retain(result).unwrap_or_default()))
    },
)?;
```

### 2. State 访问

自定义函数可以访问运行时传入的 State：

```rust
Isolate::register_custom_function(
    "getStateVersion".to_string(),
    vec![],                                      // 无参数
    VariableType::Number,
    |_args, state_opt| {
        if let Some(state) = state_opt {
            // 访问 State 数据
            let version = state.version as f64;
            Ok(Variable::Number(rust_decimal::Decimal::from_f64_retain(version).unwrap_or_default()))
        } else {
            Ok(Variable::Number(rust_decimal::Decimal::ZERO))
        }
    },
)?;
```

### 3. 运行时使用

#### 不传递 State
```rust
let mut isolate = Isolate::new();
let result = isolate.run_standard("myFunction('hello')")?;
```

#### 传递 State
```rust
let mut isolate = Isolate::new();
let state = Arc::new(/* 你的 State 实例 */);
let result = isolate.run_standard_with_state("getStateVersion()", state)?;
```

## 完整示例

```rust
use moduforge_rules_expression::{Isolate, Variable, VariableType};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    // 1. 注册自定义函数
    Isolate::register_custom_function(
        "addNumbers".to_string(),
        vec![VariableType::Number, VariableType::Number],
        VariableType::Number,
        |args, _state| {
            let a = args.number(0)?;
            let b = args.number(1)?;
            Ok(Variable::Number(a + b))
        },
    )?;

    Isolate::register_custom_function(
        "toUpper".to_string(),
        vec![VariableType::String],
        VariableType::String,
        |args, _state| {
            let text = args.str(0)?;
            Ok(Variable::String(std::rc::Rc::from(text.to_uppercase())))
        },
    )?;

    // 2. 创建 Isolate 并运行表达式
    let mut isolate = Isolate::new();
    
    // 使用自定义函数
    let result1 = isolate.run_standard("addNumbers(10, 20)")?;
    println!("10 + 20 = {}", result1); // 输出: 30
    
    let result2 = isolate.run_standard("toUpper('hello')")?;
    println!("upper('hello') = {}", result2); // 输出: "HELLO"
    
    // 组合使用
    let result3 = isolate.run_standard("addNumbers(5, len(toUpper('test')))")?;
    println!("5 + len('TEST') = {}", result3); // 输出: 9

    Ok(())
}
```

## API 参考

### 函数注册

```rust
pub fn register_custom_function<F>(
    name: String,                                     // 函数名
    params: Vec<VariableType>,                        // 参数类型
    return_type: VariableType,                        // 返回类型
    executor: F,                                      // 执行器
) -> Result<(), String>
where
    F: Fn(&Arguments, Option<&Arc<State>>) -> Result<Variable, anyhow::Error> + 'static,
```

### 运行方法

```rust
// 不传递 State
pub fn run_standard(&mut self, source: &str) -> Result<Variable, IsolateError>

// 传递 State
pub fn run_standard_with_state(&mut self, source: &str, state: Arc<State>) -> Result<Variable, IsolateError>

// 对于一元表达式
pub fn run_unary_with_state(&mut self, source: &str, state: Arc<State>) -> Result<bool, IsolateError>
```

### 管理函数

```rust
// 列出已注册的函数
pub fn list_custom_functions() -> Vec<String>

// 清空所有自定义函数
pub fn clear_custom_functions()
```

## 支持的参数类型

- `VariableType::String` - 字符串
- `VariableType::Number` - 数字 (Decimal)
- `VariableType::Bool` - 布尔值
- `VariableType::Date` - 日期
- `VariableType::Array(T)` - 数组
- `VariableType::Object(fields)` - 对象
- `VariableType::Null` - 空值
- `VariableType::Any` - 任意类型

## 获取参数值

```rust
|args, state| {
    let string_param = args.str(0)?;           // 获取字符串参数
    let number_param = args.number(1)?;        // 获取数字参数
    let bool_param = args.bool(2)?;            // 获取布尔参数
    let array_param = args.array(3)?;          // 获取数组参数
    let object_param = args.object(4)?;        // 获取对象参数
    let any_param = args.var(5)?;              // 获取任意类型参数
    
    // ... 处理逻辑
    
    Ok(Variable::String(std::rc::Rc::from("result")))
}
```

## 最佳实践

1. **函数命名**: 使用清晰的函数名，避免与内置函数冲突
2. **参数验证**: 在函数中进行适当的参数验证
3. **错误处理**: 使用 `anyhow::Error` 返回详细的错误信息
4. **State 访问**: 只在需要时访问 State，保持函数的独立性
5. **性能考虑**: 避免在函数中执行耗时操作

## 注意事项

- 自定义函数是全局注册的，在整个应用程序中共享
- 函数注册应该在应用程序启动时完成
- 相同名称的函数会覆盖之前注册的函数
- State 的传递是可选的，函数应该能够处理没有 State 的情况
- 函数执行器需要是线程安全的 (`'static`)

## 与内置函数的集成

自定义函数与内置函数完全集成，可以在同一个表达式中混合使用：

```rust
// 同时使用内置函数和自定义函数
let result = isolate.run_standard("len(toUpper(d('2023-01-01').format()))")?;
```

这个例子中：
- `d()` 是内置的日期函数
- `format()` 是内置的日期格式化方法
- `toUpper()` 是自定义函数
- `len()` 是内置的长度函数 
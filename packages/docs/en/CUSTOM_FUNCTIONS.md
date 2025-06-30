# ModuForge Expression 自定义函数功能

ModuForge Expression 支持在运行时注册自定义函数，这些函数可以在表达式中直接调用，就像内置函数（如 `date()`、`len()` 等）一样。

## 核心功能

### 1. 函数注册

使用 `CustomFunctionHelper::register_function()` 注册自定义函数：

```rust
#[derive(Debug)]
    struct MyTestState {
        counter: AtomicU32,
    }

    impl MyTestState {
        fn new() -> Self {
            Self {
                counter: AtomicU32::new(0),
            }
        }

        fn get_info(&self) -> String {
            let count = self.counter.fetch_add(1, Ordering::SeqCst);
            format!("State call count: {}", count)
        }
    }

    let helper = CustomFunctionHelper::<MyTestState>::new();
    helper.register_function(
        "getStateInfo".to_string(),
        vec![],
        VariableType::String,
        Box::new(|_args, state_opt| {
            if let Some(state) = state_opt {
                Ok(Variable::String(state.get_info().into()))
            } else {
                Ok(Variable::String("No state provided".into()))
            }
        }),
    ).unwrap();
```

### 2. State 访问

自定义函数可以访问运行时传入的 State：

```rust
 helper.register_function(
        "getStateInfo".to_string(),
        vec![],
        VariableType::String,
        Box::new(|_args, state_opt| {
            if let Some(state) = state_opt {
                Ok(Variable::String(state.get_info().into()))
            } else {
                Ok(Variable::String("No state provided".into()))
            }
        }),
    ).unwrap();
```

### 3. 运行时使用

#### 不传递 State
```rust
let mut isolate = Isolate::new();
let result = isolate.run_standard("getStateInfo()")?;
```

#### 传递 State
```rust
let mut isolate = Isolate::new();
let state = Arc::new(/* 你的 State 实例 */);
let result = isolate.run_standard_with_state("getStateInfo()", state)?;
```

## 完整示例

```rust
    #[derive(Debug)]
    struct MyTestState {
        counter: AtomicU32,
    }

    impl MyTestState {
        fn new() -> Self {
            Self {
                counter: AtomicU32::new(0),
            }
        }

        fn get_info(&self) -> String {
            let count = self.counter.fetch_add(1, Ordering::SeqCst);
            format!("State call count: {}", count)
        }
    }

    let helper = CustomFunctionHelper::<MyTestState>::new();
    helper.register_function(
        "getStateInfo".to_string(),
        vec![],
        VariableType::String,
        Box::new(|_args, state_opt| {
            if let Some(state) = state_opt {
                Ok(Variable::String(state.get_info().into()))
            } else {
                Ok(Variable::String("No state provided".into()))
            }
        }),
    ).unwrap();

    let state = Arc::new(MyTestState::new());
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());
    let result = engine
        .evaluate_with_state_and_opts(
            "http-function.json", 
            json!({ "input": 12 }).into(), 
            state.clone(),
            EvaluationOptions {
                trace: Some(true),
                max_depth: None,
            }
        )
        .await
        .unwrap();

    assert!(result.result.to_value().is_object(), "结果应该是一个对象");
    
    // Loop to test caching/reuse
    for _ in 0..10 {
        engine
            .evaluate_with_state_and_opts(
                "http-function.json",
                json!({ "input": 12 }).into(),
                state.clone(),
                EvaluationOptions {
                    trace: Some(true),
                    max_depth: None,
                },
            )
            .await
            .unwrap();
    }

    CustomFunctionRegistry::clear();

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
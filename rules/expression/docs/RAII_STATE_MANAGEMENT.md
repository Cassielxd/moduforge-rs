# RAII 模式的 State 管理

## 概述

为了解决自定义函数中 State 管理的异常安全问题，我们引入了 RAII（Resource Acquisition Is Initialization）模式。这确保了无论是正常执行还是异常情况，State 都能被正确清理。

## 问题背景

### 之前的问题

```rust
// ❌ 旧的不安全方式
CustomFunctionRegistry::set_current_state(Some(state));
let result = isolate.run_standard(source); // 如果这里 panic，State 不会被清理
CustomFunctionRegistry::set_current_state(None);
```

**缺点：**
- 异常情况下 State 不会被清理
- 手动管理容易出错
- 代码冗长且重复

### 现在的解决方案

```rust
// ✅ 新的 RAII 方式
let _guard = StateGuard::new(state);
let result = isolate.run_standard(source); // 即使 panic，State 也会被自动清理
// StateGuard 在这里自动销毁，清理 State
```

## 核心组件

### 1. StateGuard

```rust
pub struct StateGuard {
    _private: (),
}

impl StateGuard {
    /// 创建新的 State 守卫，自动设置 State
    pub fn new(state: Arc<State>) -> Self;
    
    /// 创建空守卫，立即清理 State
    pub fn empty() -> Self;
    
    /// 检查是否有活跃的 State
    pub fn has_active_state() -> bool;
}

impl Drop for StateGuard {
    fn drop(&mut self) {
        // 自动清理 State，无论是正常还是异常情况
        CustomFunctionRegistry::set_current_state(None);
    }
}
```

### 2. 便利宏

```rust
// 同步版本
with_state!(state => {
    // 在这个块内，State 是活跃的
    let result = isolate.run_standard("expression()")?;
    // State 会被自动清理
});
```

### 3. 异步支持

```rust
// 异步版本
let result = with_state_async(state, || async {
    // 异步操作
    tokio::time::sleep(Duration::from_millis(100)).await;
    isolate.run_standard("expression()")
}).await?;
```

## 使用方式

### 基本用法

```rust
use moduforge_rules_expression::{Isolate, StateGuard};

let mut isolate = Isolate::new();
let state = create_state()?;

// 方式1：直接使用 StateGuard
{
    let _guard = StateGuard::new(state.clone());
    let result = isolate.run_standard("customFunction()")?;
    println!("结果: {}", result);
} // State 在这里自动清理

// 方式2：使用便利方法
let result = isolate.run_standard_with_state("customFunction()", state)?;
```

### 异常安全示例

```rust
// 即使发生 panic，State 也会被正确清理
let panic_result = std::panic::catch_unwind(|| {
    let _guard = StateGuard::new(state);
    panic!("模拟错误"); // State 仍会被清理
});

assert!(panic_result.is_err());
assert!(!StateGuard::has_active_state()); // State 已被清理
```

### 嵌套作用域

```rust
{
    let _outer_guard = StateGuard::new(state1);
    println!("外层 State 活跃");
    
    {
        let _inner_guard = StateGuard::new(state2); // 覆盖外层 State
        println!("内层 State 活跃");
        // 内层 StateGuard 在这里销毁
    }
    
    // 外层 StateGuard 在这里销毁
}
```

### 引擎级别的使用

```rust
use moduforge_rules_engine::DecisionEngine;

let engine = DecisionEngine::default();

// 自动使用 RAII 管理
let result = engine.evaluate_with_state(
    "decision_key", 
    context, 
    state
).await?;
```

## 最佳实践

### 1. 优先使用便利方法

```rust
// ✅ 推荐：使用内建方法
let result = isolate.run_standard_with_state("expr", state)?;

// ⚠️ 可以但不必要：手动管理
let _guard = StateGuard::new(state);
let result = isolate.run_standard("expr")?;
```

### 2. 异步场景

```rust
// ✅ 异步环境下的正确使用
let result = with_state_async(state, || async {
    let result1 = some_async_operation().await?;
    let result2 = isolate.run_standard("expr")?;
    Ok((result1, result2))
}).await?;
```

### 3. 错误处理

```rust
// ✅ 结合错误处理
let result = with_state!(state => {
    isolate.run_standard("expr")
        .map_err(|e| anyhow::anyhow!("表达式执行失败: {}", e))?
});
```

### 4. 条件性 State 设置

```rust
fn run_with_optional_state(
    isolate: &mut Isolate,
    expr: &str,
    state: Option<Arc<State>>
) -> Result<Variable, IsolateError> {
    match state {
        Some(s) => {
            let _guard = StateGuard::new(s);
            isolate.run_standard(expr)
        }
        None => isolate.run_standard(expr)
    }
}
```

## 性能考虑

### 1. 零成本抽象

- `StateGuard` 是零大小类型（ZST）
- 编译器会优化掉大部分开销
- 只在析构时有少量开销

### 2. 内存安全

- 不会增加内存使用
- 避免内存泄漏
- 线程安全（基于 thread_local）

## 迁移指南

### 从旧代码迁移

```rust
// 旧代码
CustomFunctionRegistry::set_current_state(Some(state));
let result = operation();
CustomFunctionRegistry::set_current_state(None);

// 新代码
let _guard = StateGuard::new(state);
let result = operation();
```

### 批量替换

1. 找到所有的 `set_current_state(Some(...))` 调用
2. 替换为 `let _guard = StateGuard::new(...)`
3. 删除对应的 `set_current_state(None)` 调用
4. 确保作用域正确

## 测试

```rust
#[test]
fn test_raii_state_management() {
    let state = create_test_state();
    
    // 测试基本功能
    assert!(!StateGuard::has_active_state());
    {
        let _guard = StateGuard::new(state);
        assert!(StateGuard::has_active_state());
    }
    assert!(!StateGuard::has_active_state());
    
    // 测试异常安全
    let result = std::panic::catch_unwind(|| {
        let _guard = StateGuard::new(state);
        panic!("测试");
    });
    assert!(result.is_err());
    assert!(!StateGuard::has_active_state());
}
```

## 总结

RAII 模式的引入为 State 管理带来了：

1. **异常安全**：无论如何退出作用域，State 都会被清理
2. **简化代码**：减少手动管理的样板代码
3. **降低错误**：编译器保证资源清理
4. **更好的可读性**：意图更明确

这种模式确保了在"每次执行都需要新 State"的场景下，资源管理的安全性和简洁性。 
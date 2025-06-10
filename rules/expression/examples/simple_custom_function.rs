use moduforge_rules_expression::{Isolate, Variable};
use moduforge_rules_expression::variable::VariableType;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("=== 简单自定义函数演示 ===\n");

    // 注册一个简单的自定义函数：getUserCount
    println!("注册自定义函数: getUserCount()");
    
    Isolate::register_custom_function(
        "getUserCount".to_string(),
        vec![], // 无参数
        VariableType::Number,
        |_args, state_opt| {
            if let Some(state) = state_opt {
                // 如果有State，返回State的版本号作为用户数量的模拟
                let count = state.version as f64;
                Ok(Variable::Number(rust_decimal::Decimal::from_f64_retain(count).unwrap_or_default()))
            } else {
                // 如果没有State，返回0
                Ok(Variable::Number(rust_decimal::Decimal::ZERO))
            }
        },
    ).map_err(|e| anyhow::anyhow!(e))?;

    // 注册一个有参数的自定义函数：addNumbers
    println!("注册自定义函数: addNumbers(a, b)");
    
    Isolate::register_custom_function(
        "addNumbers".to_string(),
        vec![VariableType::Number, VariableType::Number], // 两个数字参数
        VariableType::Number,
        |args, _state_opt| {
            let a = args.number(0)?;
            let b = args.number(1)?;
            Ok(Variable::Number(a + b))
        },
    ).map_err(|e| anyhow::anyhow!(e))?;

    // 注册一个字符串处理函数：toUpper
    println!("注册自定义函数: toUpper(text)");
    
    Isolate::register_custom_function(
        "toUpper".to_string(),
        vec![VariableType::String], // 一个字符串参数
        VariableType::String,
        |args, _state_opt| {
            let text = args.str(0)?;
            Ok(Variable::String(std::rc::Rc::from(text.to_uppercase())))
        },
    ).map_err(|e| anyhow::anyhow!(e))?;

    // 创建 Isolate
    let mut isolate = Isolate::new();

    println!("\n=== 测试自定义函数 ===");

    // 测试1: 不传递State运行
    println!("1. 不传递State运行表达式:");
    let result1 = isolate.run_standard("getUserCount()")?;
    println!("   getUserCount() = {}", result1);

    // 测试2: 数学运算
    println!("\n2. 数学运算:");
    let result2 = isolate.run_standard("addNumbers(10, 20)")?;
    println!("   addNumbers(10, 20) = {}", result2);

    // 测试3: 字符串处理
    println!("\n3. 字符串处理:");
    let result3 = isolate.run_standard("toUpper('hello world')")?;
    println!("   toUpper('hello world') = {}", result3);

    // 测试4: 组合使用
    println!("\n4. 组合表达式:");
    let result4 = isolate.run_standard("addNumbers(getUserCount(), 5)")?;
    println!("   addNumbers(getUserCount(), 5) = {}", result4);

    // 现在测试带有State的情况（创建一个模拟的State）
    println!("\n=== 使用模拟State测试 ===");
    
    // 创建一个模拟的State（这里我们可以传递任何Arc<T>，只要它有version字段）
    // 但是为了简化，我们直接跳过复杂的State创建，专注于展示API的使用
    
    println!("注意：完整的State集成需要适当的schema和配置设置");
    println!("这个演示展示了自定义函数的基本用法");

    // 显示所有已注册的自定义函数
    println!("\n=== 已注册的自定义函数 ===");
    let functions = Isolate::list_custom_functions();
    for func in functions {
        println!("- {}", func);
    }

    // 清理
    println!("\n清理自定义函数...");
    Isolate::clear_custom_functions();
    
    println!("演示完成！");
    Ok(())
} 
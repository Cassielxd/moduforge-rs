use moduforge_rules_expression::{Isolate, Variable};
use moduforge_rules_expression::variable::VariableType;
use moduforge_state::{State, ops::GlobalResourceManager};
use moduforge_model::schema::Schema;
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    println!("=== 自定义函数与State集成演示 ===\n");

    // 创建 State
    use moduforge_model::schema::{SchemaSpec};
    use moduforge_model::node_type::NodeSpec;
    use std::collections::HashMap;

    let mut nodes = HashMap::new();
    nodes.insert(
        "doc".to_string(),
        NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("Document root".to_string()),
            attrs: None,
        },
    );

    let schema_spec = SchemaSpec {
        nodes,
        marks: HashMap::new(),
        top_node: Some("doc".to_string()),
    };
    let schema = Arc::new(Schema::compile(schema_spec).unwrap());
    let resource_manager = Arc::new(GlobalResourceManager::new());
    let config = moduforge_state::state::Configuration::new(
        schema,
        None,
        None,
        Some(resource_manager),
    )
    .unwrap();
    let state = Arc::new(State::new(Arc::new(config)).unwrap());

    // 模拟向 State 中添加一些数据
    // 注意：这里只是演示，实际使用时需要根据 State 的 API 来添加数据
    println!("创建 State...");

    // 注册一个简单的自定义函数：getStateInfo
    println!("注册自定义函数: getStateInfo()");

    Isolate::register_custom_function(
        "getStateInfo".to_string(),
        vec![], // 无参数
        VariableType::String,
        |_args, state_opt| {
            if let Some(state) = state_opt {
                // 从 State 中获取信息
                let info = format!("State版本: {}", state.version);
                Ok(Variable::String(std::rc::Rc::from(info)))
            } else {
                Ok(Variable::String(std::rc::Rc::from("未提供State")))
            }
        },
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    // 注册另一个自定义函数：getDocumentTitle
    println!("注册自定义函数: getDocumentTitle()");

    Isolate::register_custom_function(
        "getDocumentTitle".to_string(),
        vec![], // 无参数
        VariableType::String,
        |_args, state_opt| {
            if let Some(state) = state_opt {
                // 从 State 获取基本信息
                let info = format!("State版本: {}", state.version);
                Ok(Variable::String(std::rc::Rc::from(info)))
            } else {
                Ok(Variable::String(std::rc::Rc::from("无法访问文档")))
            }
        },
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    // 注册一个有参数的自定义函数：checkPlugin
    println!("注册自定义函数: checkPlugin(name)");

    Isolate::register_custom_function(
        "checkPlugin".to_string(),
        vec![VariableType::String], // 一个字符串参数
        VariableType::Bool,
        |args, state_opt| {
            let plugin_name = args.str(0)?;

            if let Some(state) = state_opt {
                // 检查插件是否存在（简化版本）
                let has_plugin =
                    state.plugins().iter().any(|p| &p.key == plugin_name);
                Ok(Variable::Bool(has_plugin))
            } else {
                Ok(Variable::Bool(false))
            }
        },
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    // 创建 Isolate
    let mut isolate = Isolate::new();

    println!("\n=== 测试自定义函数 ===");

    // 测试1: 不传递State运行
    println!("1. 不传递State运行表达式:");
    let result1 = isolate.run_standard("getStateInfo()")?;
    println!("   结果: {}", result1);

    // 测试2: 传递State运行
    println!("\n2. 传递State运行表达式:");
    let result2 =
        isolate.run_standard_with_state("getStateInfo()", state.clone())?;
    println!("   结果: {}", result2);

    // 测试3: 获取文档标题
    println!("\n3. 获取文档标题:");
    let result3 =
        isolate.run_standard_with_state("getDocumentTitle()", state.clone())?;
    println!("   结果: {}", result3);

    // 测试4: 检查插件是否存在
    println!("\n4. 检查插件是否存在:");
    let result4 = isolate.run_standard_with_state(
        "checkPlugin('example-plugin')",
        state.clone(),
    )?;
    println!("   结果: {}", result4);

    // 测试5: 在表达式中组合使用
    println!("\n5. 组合表达式:");
    let result5 = isolate.run_standard_with_state(
        "getDocumentTitle() + ' - ' + getStateInfo()",
        state.clone(),
    )?;
    println!("   结果: {}", result5);

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

use crate::support::{create_fs_loader, load_raw_test_data, load_test_data, test_data_root};
use moduforge_model::schema::Schema;
use moduforge_rules_expression::variable::VariableType;
use moduforge_rules_expression::Isolate;
use moduforge_state::ops::GlobalResourceManager;
use moduforge_state::State;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::io::Read;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Builder;
use moduforge_rules_engine::loader::{LoaderError, MemoryLoader};
use moduforge_rules_engine::model::{DecisionContent, DecisionNode, DecisionNodeKind, FunctionNodeContent};
use moduforge_rules_engine::Variable;
use moduforge_rules_engine::{DecisionEngine, EvaluationError, EvaluationOptions};

mod support;


#[tokio::test]
async fn engine_custom_function() {
    println!("=== 自定义函数与State集成演示 ===\n");

    // 创建 State
    use moduforge_model::schema::{SchemaSpec};
    use moduforge_model::node_type::NodeSpec;
    use std::collections::HashMap;
    
    let mut nodes = HashMap::new();
    nodes.insert("doc".to_string(), NodeSpec {
        content: None,
        marks: None,
        group: None,
        desc: Some("Document root".to_string()),
        attrs: None,
    });
    
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
        Some(resource_manager)
    ).unwrap();
    let state = Arc::new(State::new(Arc::new(config)).unwrap());
    println!("注册自定义函数: getStateInfo()");
    
    let _ = Isolate::register_custom_function(
        "getStateInfo".to_string(),
        vec![VariableType::String], // 无参数
        VariableType::String,
        |args, state_opt| {
            let p1 = args.str(0)?;
            println!("p1: {:?}", p1);
            if let Some(state) = state_opt {
                // 从 State 中获取信息
                let info = format!("State版本: {}", state.version);
                Ok(Variable::String(std::rc::Rc::from(info)))
            } else {
                Ok(Variable::String(std::rc::Rc::from("未提供State")))
            }
        },
    ).map_err(|e| anyhow::anyhow!(e));

    // 注册另一个自定义函数：getDocumentTitle
    println!("注册自定义函数: getDocumentTitle()");
    
    let _ = Isolate::register_custom_function(
        "getDocumentTitle".to_string(),
        vec![], // 无参数
        VariableType::String,
        |_args, state_opt| {
            if let Some(state) = state_opt {
                // 从 State 获取基本信息
                let info = format!("标题版本: {}", state.version);
                Ok(Variable::String(std::rc::Rc::from(info)))
            } else {
                Ok(Variable::String(std::rc::Rc::from("无法访问文档")))
            }
        },
    ).map_err(|e| anyhow::anyhow!(e));

    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());
    let result = engine
        .evaluate_with_state_and_opts("http-function.json", json!({ "input": 12 }).into(), state.clone(),EvaluationOptions {
            trace: Some(true),
            max_depth: None,
        })
        .await
        .unwrap();
    
    println!("测试结果: {:?}", result);
    assert!(result.result.to_value().is_object(), "结果应该是一个对象");
}



#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn engine_memory_loader() {
    let memory_loader = Arc::new(MemoryLoader::default());
    memory_loader.add("table", load_test_data("table.json"));
    memory_loader.add("function", load_test_data("function.json"));

    let engine = DecisionEngine::default().with_loader(memory_loader.clone());
    let table = engine
        .evaluate("table", json!({ "input": 12 }).into())
        .await;
    let function = engine
        .evaluate("function", json!({ "input": 12 }).into())
        .await;

    memory_loader.remove("function");
    let not_found = engine.evaluate("function", json!({}).into()).await;

    assert_eq!(table.unwrap().result, json!({"output": 10}).into());
    assert_eq!(function.unwrap().result, json!({"output": 24}).into());
    assert_eq!(not_found.unwrap_err().to_string(), "加载器错误");
}

#[tokio::test]

async fn engine_filesystem_loader() {
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());
    let table = engine
        .evaluate("table.json", json!({ "input": 12 }).into())
        .await;
    let function = engine
        .evaluate("function.json", json!({ "input": 12 }).into())
        .await;
    let not_found = engine.evaluate("invalid_file", json!({}).into()).await;

    assert_eq!(table.unwrap().result, json!({"output": 10}).into());
    assert_eq!(function.unwrap().result, json!({"output": 24}).into());
    assert_eq!(not_found.unwrap_err().to_string(), "加载器错误");
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn engine_closure_loader() {
    let engine = DecisionEngine::default().with_closure_loader(|key| async {
        match key.as_str() {
            "function" => Ok(Arc::new(load_test_data("function.json"))),
            "table" => Ok(Arc::new(load_test_data("table.json"))),
            _ => Err(LoaderError::NotFound(key).into()),
        }
    });

    let table = engine
        .evaluate("table", json!({ "input": 12 }).into())
        .await;
    let function = engine
        .evaluate("function", json!({ "input": 12 }).into())
        .await;
    let not_found = engine.evaluate("invalid_file", json!({}).into()).await;

    assert_eq!(table.unwrap().result, json!({"output": 10}).into());
    assert_eq!(function.unwrap().result, json!({"output": 24}).into());
    assert_eq!(not_found.unwrap_err().to_string(), "Loader error");
}

#[test]
fn engine_noop_loader() {
    let rt = Builder::new_current_thread().build().unwrap();
    // Default engine is noop
    let engine = DecisionEngine::default();
    let result = rt.block_on(engine.evaluate("any.json", json!({}).into()));

    assert_eq!(result.unwrap_err().to_string(), "Loader error");
}

#[test]
fn engine_get_decision() {
    let rt = Builder::new_current_thread().build().unwrap();
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());

    assert!(rt.block_on(engine.get_decision("table.json")).is_ok());
    assert!(rt.block_on(engine.get_decision("any.json")).is_err());
}

#[test]
fn engine_create_decision() {
    let engine = DecisionEngine::default();
    engine.create_decision(load_test_data("table.json").into());
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn engine_errors() {
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());

    let infinite_fn = engine
        .evaluate("infinite-function.json", json!({}).into())
        .await;
    match infinite_fn.unwrap_err().deref() {
        EvaluationError::NodeError(e) => {
            assert_eq!(e.node_id, "e0fd96d0-44dc-4f0e-b825-06e56b442d78");
            assert!(e.source.to_string().contains("interrupted"));
        }
        _ => assert!(false, "Wrong error type"),
    }

    let recursive = engine
        .evaluate("recursive-table1.json", json!({}).into())
        .await;
    match recursive.unwrap_err().deref() {
        EvaluationError::NodeError(e) => {
            assert_eq!(e.source.to_string(), "Depth limit exceeded")
        }
        _ => assert!(false, "Depth limit not exceeded"),
    }
}

#[test]
fn engine_with_trace() {
    let rt = Builder::new_current_thread().build().unwrap();
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());

    let table_r = rt.block_on(engine.evaluate("table.json", json!({ "input": 12 }).into()));
    let table_opt_r = rt.block_on(engine.evaluate_with_opts(
        "table.json",
        json!({ "input": 12 }).into(),
        EvaluationOptions {
            trace: Some(true),
            max_depth: None,
        },
    ));

    let table = table_r.unwrap();
    let table_opt = table_opt_r.unwrap();

    assert!(table.trace.is_none());
    assert!(table_opt.trace.is_some());

    let trace = table_opt.trace.unwrap();
    assert_eq!(trace.len(), 3); // trace for each node
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn engine_function_imports() {
    let function_content = load_test_data("function.json");

    let imports_js_path = Path::new("js").join("imports.js");
    let mut replace_buffer = load_raw_test_data(imports_js_path.to_str().unwrap());
    let mut replace_data = String::new();
    replace_buffer.read_to_string(&mut replace_data).unwrap();

    let new_nodes = function_content
        .nodes
        .into_iter()
        .map(|node| match &node.kind {
            DecisionNodeKind::FunctionNode { .. } => {
                let new_kind = DecisionNodeKind::FunctionNode {
                    content: FunctionNodeContent::Version1(replace_data.clone()),
                };

                Arc::new(DecisionNode {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    kind: new_kind,
                })
            }
            _ => node,
        })
        .collect::<Vec<_>>();

    let function_content = DecisionContent {
        edges: function_content.edges,
        nodes: new_nodes,
    };
    let decision = DecisionEngine::default().create_decision(function_content.into());
    let response = decision.evaluate(json!({}).into()).await.unwrap();

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct GraphResult {
        bigjs_tests: Vec<bool>,
        bigjs_valid: bool,
        dayjs_valid: bool,
        moment_valid: bool,
    }

    let result = serde_json::from_value::<GraphResult>(response.result.to_value()).unwrap();

    assert!(result.bigjs_tests.iter().all(|v| *v));
    assert!(result.bigjs_valid);
    assert!(result.dayjs_valid);
    assert!(result.moment_valid);
}

#[tokio::test]
async fn engine_switch_node() {
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());

    let switch_node_r = engine
        .evaluate("switch-node.json", json!({ "color": "yellow" }).into())
        .await;

    let table = switch_node_r.unwrap();
    println!("{table:?}");
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn engine_graph_tests() {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestCase {
        input: Variable,
        output: Variable,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestData {
        tests: Vec<TestCase>,
        #[serde(flatten)]
        decision_content: DecisionContent,
    }

    let engine = DecisionEngine::default();

    let graphs_path = Path::new(test_data_root().as_str()).join("graphs");
    let file_list = fs::read_dir(graphs_path).unwrap();
    for maybe_file in file_list {
        let Ok(file) = maybe_file else {
            panic!("Failed to read DirEntry {maybe_file:?}");
        };

        let file_name = file.file_name().to_str().map(|s| s.to_string()).unwrap();
        let file_contents = fs::read_to_string(file.path()).expect("valid file data");
        let test_data: TestData = serde_json::from_str(&file_contents).expect("Valid JSON");

        let decision = engine.create_decision(test_data.decision_content.into());
        for test_case in test_data.tests {
            let input = test_case.input.clone();
            let result = decision.evaluate(input.clone()).await.unwrap().result;

            assert_eq!(
                test_case.output, result,
                "Decision file: {file_name}.\nInput:\n {input:#?}"
            );
        }
    }
}

#[tokio::test]
async fn engine_function_v2() {
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());

    for _ in 0..1_000 {
        let function_opt_r = engine
            .evaluate_with_opts(
                "function-v2.json",
                json!({ "input": 12 }).into(),
                EvaluationOptions {
                    trace: Some(true),
                    max_depth: None,
                },
            )
            .await;

        assert!(function_opt_r.is_ok(), "function v2 has errored");

        let function_opt = function_opt_r.unwrap();
        let trace = function_opt.trace.unwrap();
        assert_eq!(trace.len(), 3); // trace for each node

        assert_eq!(
            function_opt.result,
            json!({ "hello": "world", "multiplied": 24 }).into()
        )
    }
}

#[tokio::test]
async fn test_validation() {
    let engine = DecisionEngine::default().with_loader(create_fs_loader().into());

    let context_valid = json!({
        "color": "red",
        "customer": {
            "firstName": "John",
            "lastName": "Doe",
            "email": "john@doe.com",
            "age": 20
        }
    });

    let context_invalid = json!({
         "color": "redd",
        "customer": {
            "firstName": "John",
            "lastName": "Doe",
            "email": "john@doe.com",
            "age": 20
        }
    });

    assert!(engine
        .evaluate("customer-input-schema.json", context_valid.clone().into())
        .await
        .is_ok());
    assert!(engine
        .evaluate("customer-input-schema.json", context_invalid.clone().into())
        .await
        .is_err());

    assert!(engine
        .evaluate("customer-output-schema.json", context_valid.clone().into())
        .await
        .is_ok());
    assert!(engine
        .evaluate(
            "customer-output-schema.json",
            context_invalid.clone().into()
        )
        .await
        .is_err());
}

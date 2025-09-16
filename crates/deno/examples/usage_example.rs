//! ModuForge Deno 集成使用示例
//!
//! 演示如何在 ModuForge 项目中集成和使用 Deno 插件

use std::sync::Arc;
use mf_deno::*;
use moduforge_state::{State, StateConfig};
use moduforge_model::schema::Schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::init();

    println!("🚀 ModuForge Deno 集成示例");

    // 1. 创建基础 Schema 和 State
    let schema = Arc::new(Schema::default());
    let state_config = StateConfig {
        schema: Some(schema),
        doc: None,
        stored_marks: None,
        plugins: None,
        resource_manager: None,
    };

    let initial_state = Arc::new(
        State::create(state_config)
            .await
            .expect("Failed to create initial state")
    );

    // 2. 创建 ModuForge Deno 集成实例
    let deno = ModuForgeDeno::new(initial_state.clone(), Some(4));
    deno.initialize().await?;

    println!("✅ Deno 运行时池初始化完成");

    // 3. 加载示例插件
    let plugin = deno
        .load_plugin_from_file("simple-plugin", "examples/simple_plugin.js")
        .await?;

    println!("✅ 插件加载完成: {}", plugin.get_name());

    // 4. 测试插件方法调用
    let manager = deno.manager();

    // 测试 validateDocument 方法
    let validate_result = manager
        .execute_plugin_method(
            "simple-plugin",
            "validateDocument",
            serde_json::json!({}),
        )
        .await?;

    println!("📄 文档验证结果: {}", serde_json::to_string_pretty(&validate_result)?);

    // 测试 getNodeStats 方法
    let stats_result = manager
        .execute_plugin_method(
            "simple-plugin",
            "getNodeStats",
            serde_json::json!({
                "startNodeId": 1,
                "endNodeId": 5
            }),
        )
        .await?;

    println!("📊 节点统计结果: {}", serde_json::to_string_pretty(&stats_result)?);

    // 5. 创建更多插件进行测试
    let advanced_plugin_code = r#"
        function processData(args) {
            const { data, operation } = args;

            switch (operation) {
                case 'reverse':
                    return { result: data.split('').reverse().join('') };
                case 'uppercase':
                    return { result: data.toUpperCase() };
                case 'length':
                    return { result: data.length };
                default:
                    return { error: 'Unknown operation: ' + operation };
            }
        }

        function analyzeState() {
            const version = ModuForge.State.getVersion();
            const hasDocField = ModuForge.State.hasField('doc');

            return {
                stateVersion: version,
                hasDocField: hasDocField,
                timestamp: Date.now(),
                analysis: 'State analysis completed'
            };
        }

        console.log('🔧 Advanced plugin loaded');
    "#;

    let advanced_plugin = deno
        .create_plugin_from_code("advanced-plugin", advanced_plugin_code)
        .await?;

    println!("✅ 高级插件加载完成: {}", advanced_plugin.get_name());

    // 测试高级插件方法
    let process_result = manager
        .execute_plugin_method(
            "advanced-plugin",
            "processData",
            serde_json::json!({
                "data": "Hello ModuForge",
                "operation": "reverse"
            }),
        )
        .await?;

    println!("🔄 数据处理结果: {}", serde_json::to_string_pretty(&process_result)?);

    let analyze_result = manager
        .execute_plugin_method(
            "advanced-plugin",
            "analyzeState",
            serde_json::json!({}),
        )
        .await?;

    println!("🔍 状态分析结果: {}", serde_json::to_string_pretty(&analyze_result)?);

    // 6. 使用插件构建器创建自定义插件
    let custom_plugin = deno
        .build_plugin(
            DenoPluginBuilder::new("custom-plugin")
                .code(r#"
                    function greet(args) {
                        const name = args.name || 'World';
                        return {
                            message: `Hello, ${name}! Welcome to ModuForge with Deno.`,
                            timestamp: Date.now()
                        };
                    }

                    function appendTransaction(args) {
                        console.log('Custom plugin appendTransaction:', args);
                        return null;
                    }

                    function filterTransaction(args) {
                        console.log('Custom plugin filterTransaction:', args);
                        return true;
                    }
                "#)
                .priority(5)
                .enabled(true),
        )
        .await?;

    println!("✅ 自定义插件创建完成: {}", custom_plugin.get_name());

    let greet_result = manager
        .execute_plugin_method(
            "custom-plugin",
            "greet",
            serde_json::json!({"name": "ModuForge Developer"}),
        )
        .await?;

    println!("👋 问候结果: {}", serde_json::to_string_pretty(&greet_result)?);

    // 7. 列出所有已加载的插件
    let all_plugins = deno.list_plugins().await;
    println!("📋 已加载的插件列表: {:?}", all_plugins);

    // 8. 状态更新测试
    println!("🔄 测试状态更新...");
    let new_state = Arc::new(
        State::create(StateConfig {
            schema: Some(Arc::new(Schema::default())),
            doc: None,
            stored_marks: None,
            plugins: None,
            resource_manager: None,
        })
        .await?
    );

    deno.update_state(new_state).await;
    println!("✅ 状态更新完成");

    // 9. 性能测试
    println!("⚡ 开始性能测试...");
    let start_time = std::time::Instant::now();

    let tasks = (0..10).map(|i| {
        let manager_ref = &manager;
        async move {
            manager_ref
                .execute_plugin_method(
                    "advanced-plugin",
                    "processData",
                    serde_json::json!({
                        "data": format!("Test data {}", i),
                        "operation": "uppercase"
                    }),
                )
                .await
        }
    });

    let results = futures::future::try_join_all(tasks).await?;
    let duration = start_time.elapsed();

    println!("⚡ 性能测试完成:");
    println!("   - 执行 10 个并发调用");
    println!("   - 耗时: {:?}", duration);
    println!("   - 平均每个调用: {:?}", duration / 10);
    println!("   - 成功执行: {} 个", results.len());

    // 10. 清理资源
    println!("🧹 清理资源...");
    deno.unload_plugin("simple-plugin").await?;
    deno.unload_plugin("advanced-plugin").await?;
    deno.unload_plugin("custom-plugin").await?;

    let remaining_plugins = deno.list_plugins().await;
    println!("📋 剩余插件: {:?}", remaining_plugins);

    deno.shutdown().await;
    println!("✅ ModuForge Deno 集成示例完成");

    Ok(())
}

/// 创建复杂的插件示例
fn create_complex_plugin_example() -> &'static str {
    r#"
    // 复杂插件示例：文档编辑助手
    class DocumentHelper {
        constructor() {
            this.name = "DocumentHelper";
            this.version = "1.0.0";
            this.operations = new Map();
            this.history = [];
        }

        logOperation(operation, details) {
            const entry = {
                operation,
                details,
                timestamp: Date.now()
            };
            this.history.push(entry);
            console.log(`📝 ${this.name}:`, operation, details);
        }

        processNode(nodeId, operation) {
            const nodeExists = ModuForge.Node.findById(nodeId);
            if (!nodeExists) {
                return { error: `Node ${nodeId} not found` };
            }

            const nodeInfo = ModuForge.Node.getInfo(nodeId);
            this.logOperation('processNode', { nodeId, operation, nodeExists });

            return {
                success: true,
                nodeId,
                operation,
                nodeInfo: nodeInfo ? JSON.parse(nodeInfo) : null
            };
        }

        batchProcess(nodes, operation) {
            const results = [];

            for (const nodeId of nodes) {
                const result = this.processNode(nodeId, operation);
                results.push(result);
            }

            this.logOperation('batchProcess', { nodeCount: nodes.length, operation });
            return { results, summary: { total: nodes.length, processed: results.length } };
        }

        getHistory() {
            return {
                history: this.history,
                count: this.history.length,
                plugin: this.name
            };
        }
    }

    // 创建全局实例
    const documentHelper = new DocumentHelper();

    // 导出方法
    function processNode(args) {
        return documentHelper.processNode(args.nodeId, args.operation);
    }

    function batchProcess(args) {
        return documentHelper.batchProcess(args.nodes, args.operation);
    }

    function getHistory(args) {
        return documentHelper.getHistory();
    }

    function appendTransaction(args) {
        documentHelper.logOperation('appendTransaction', args);
        return null;
    }

    function filterTransaction(args) {
        documentHelper.logOperation('filterTransaction', args);
        return true;
    }

    console.log('🏭 Complex DocumentHelper plugin loaded');
    "#
}
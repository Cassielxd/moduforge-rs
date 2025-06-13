use moduforge_core::{
    async_runtime::AsyncEditor,
    types::{EditorOptionsBuilder, Extensions},
    node::Node,
    extension::Extension,
    middleware::MiddlewareStack,
};
use moduforge_model::node_type::NodeSpec;
use moduforge_state::{
    plugin::{Plugin, PluginSpec},
    transaction::Command,
};
use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;

// 使用已定义的模块
use crate::resources::*;
use crate::plugins::*;
use crate::edit_commands::*;
use crate::middleware::*;
use crate::document_nodes::*;

/// 简单演示命令
#[derive(Debug)]
pub struct SimpleCommand {
    pub name: String,
    pub action: String,
}

impl SimpleCommand {
    pub fn new(
        name: &str,
        action: &str,
    ) -> Self {
        Self { name: name.to_string(), action: action.to_string() }
    }
}

#[async_trait]
impl Command for SimpleCommand {
    async fn execute(
        &self,
        tr: &mut moduforge_state::transaction::Transaction,
    ) -> moduforge_transform::TransformResult<()> {
        tr.set_meta("action", self.action.clone());
        tr.set_meta("source", "simple_demo");

        println!("⚡ 执行命令: {} (动作: {})", self.name, self.action);
        Ok(())
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

/// 运行完整的 ModuForge-RS 演示
/// 展示插件机制、资源管理、中间件和多插件协作流程
pub async fn run_simple_demo() -> Result<()> {
    println!("🚀 ModuForge-RS 完整演示");
    println!("🎯 展示插件机制、资源管理、中间件、多插件协作");
    println!("{}", "=".repeat(60));

    // 1. 创建完整的文档节点系统
    println!("\n📋 第1步: 设置完整的文档架构");
    println!("   🏗️ 创建丰富的节点类型生态系统...");

    // 首先创建基础节点类型
    let text_node = create_text_node();
    let inline_node = create_inline_node();
    let block_node = create_block_node();

    // 文档结构节点
    let doc_node = create_document_node();
    let paragraph_node = create_paragraph_node();
    let heading_node = create_heading_node();

    // 列表相关节点
    let list_node = create_list_node();
    let list_item_node = create_list_item_node();

    // 表格相关节点
    let table_node = create_table_node();
    let table_row_node = create_table_row_node();
    let table_cell_node = create_table_cell_node();

    // 特殊内容节点
    let code_block_node = create_code_block_node();
    let blockquote_node = create_blockquote_node();
    let hr_node = create_horizontal_rule_node();

    println!("   ✅ 基础节点: text, inline, block");
    println!("   ✅ 文档节点: document, paragraph, heading");
    println!("   ✅ 列表节点: list, list_item");
    println!("   ✅ 表格节点: table, table_row, table_cell");
    println!("   ✅ 特殊节点: code_block, blockquote, horizontal_rule");

    // 2. 创建多个功能插件
    println!("\n🔌 第2步: 创建插件生态系统");
    let mut extension = Extension::new();

    // 用户管理插件 (最高优先级)
    let user_plugin = Plugin::new(PluginSpec {
        key: ("user_manager".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(UserStateField)),
        tr: Some(Arc::new(UserPlugin)),
        priority: 10,
    });
    println!("   ✅ 用户管理插件 (优先级: 10)");

    // 权限验证插件
    let auth_plugin = Plugin::new(PluginSpec {
        key: ("auth_system".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(AuthStateField)),
        tr: Some(Arc::new(AuthPlugin)),
        priority: 20,
    });
    println!("   ✅ 权限验证插件 (优先级: 20)");

    // 审计日志插件
    let audit_plugin = Plugin::new(PluginSpec {
        key: ("audit_log".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(AuditStateField)),
        tr: Some(Arc::new(AuditPlugin)),
        priority: 30,
    });
    println!("   ✅ 审计日志插件 (优先级: 30)");

    // 缓存管理插件
    let cache_plugin = Plugin::new(PluginSpec {
        key: ("cache_manager".to_string(), "v1".to_string()),
        state_field: Some(Arc::new(CacheStateField)),
        tr: Some(Arc::new(CachePlugin)),
        priority: 40,
    });
    println!("   ✅ 缓存管理插件 (优先级: 40)");

    extension.add_plugin(Arc::new(user_plugin));
    extension.add_plugin(Arc::new(auth_plugin));
    extension.add_plugin(Arc::new(audit_plugin));
    extension.add_plugin(Arc::new(cache_plugin));

    // 3. 创建中间件堆栈
    println!("\n🛡️ 第3步: 构建中间件管道");
    let mut middleware_stack = MiddlewareStack::new();
    middleware_stack.add(ValidationMiddleware::new());
    middleware_stack.add(LoggingMiddleware::new());
    middleware_stack.add(MetricsMiddleware::new());
    println!("   ✅ 验证中间件 -> 日志中间件 -> 性能监控中间件");

    // 4. 构建编辑器
    println!("\n⚙️ 第4步: 构建编辑器");
    let options = EditorOptionsBuilder::new()
        .extensions(vec![
            // 首先注册基础节点类型
            Extensions::N(text_node),
            Extensions::N(inline_node),
            Extensions::N(block_node),
            // 然后注册完整的节点系统
            Extensions::N(doc_node),
            Extensions::N(paragraph_node),
            Extensions::N(heading_node),
            Extensions::N(list_node),
            Extensions::N(list_item_node),
            Extensions::N(table_node),
            Extensions::N(table_row_node),
            Extensions::N(table_cell_node),
            Extensions::N(code_block_node),
            Extensions::N(blockquote_node),
            Extensions::N(hr_node),
            Extensions::E(extension),
        ])
        .middleware_stack(middleware_stack)
        .history_limit(50)
        .build();

    let mut editor = AsyncEditor::create(options)
        .await
        .map_err(|e| anyhow::anyhow!("创建编辑器失败: {}", e))?;

    println!("   ✅ 编辑器创建成功");

    // 输入文档内容
    let doc = editor.get_state().doc();
    println!("🔍 文档内容");
    dbg!(doc);

    // 5. 执行多插件协作工作流
    println!("\n🎬 第5步: 多插件协作演示");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // 用户登录流程
    println!("\n👤 用户管理流程:");
    let login_cmd = UserLoginCommand::new("alice", "editor");
    editor
        .command(Arc::new(login_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("用户登录失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 文档创建流程
    println!("\n📄 文档管理流程:");
    let create_doc_cmd =
        CreateDocumentCommand::new("协作文档示例", "展示多插件协作的示例文档");
    editor
        .command(Arc::new(create_doc_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("创建文档失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 内容编辑流程
    println!("\n✏️ 内容编辑流程:");
    let add_heading_cmd = AddHeadingCommand::new(1, "ModuForge-RS 特性介绍");
    editor
        .command(Arc::new(add_heading_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("添加标题失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let add_para_cmd = AddParagraphCommand::new(
        "ModuForge-RS 是一个基于 Rust 的现代化文档编辑框架，具有强大的插件系统、中间件支持和事务化状态管理。",
    );
    editor
        .command(Arc::new(add_para_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("添加段落失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let add_list_cmd = AddListCommand::new(vec![
        "🔌 强大的插件系统".to_string(),
        "🛡️ 中间件管道".to_string(),
        "💾 事务化状态管理".to_string(),
        "🔄 实时协作支持".to_string(),
    ]);
    editor
        .command(Arc::new(add_list_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("添加列表失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // 添加表格展示数据
    let add_table_cmd = AddTableCommand::new(
        vec!["功能".to_string(), "状态".to_string(), "优先级".to_string()],
        vec![
            vec![
                "用户管理".to_string(),
                "✅ 活跃".to_string(),
                "高".to_string(),
            ],
            vec![
                "权限控制".to_string(),
                "✅ 活跃".to_string(),
                "高".to_string(),
            ],
            vec![
                "审计日志".to_string(),
                "✅ 活跃".to_string(),
                "中".to_string(),
            ],
            vec![
                "缓存管理".to_string(),
                "✅ 活跃".to_string(),
                "低".to_string(),
            ],
        ],
    );
    editor
        .command(Arc::new(add_table_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("添加表格失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 版本控制流程
    println!("\n💾 版本控制流程:");
    let snapshot_cmd = CreateSnapshotCommand::new("初始版本 - 添加基础内容");
    editor
        .command(Arc::new(snapshot_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("创建快照失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 协作同步流程
    println!("\n🔄 协作同步流程:");
    let sync_cmd = SyncDocumentCommand::new("sync_001".to_string());
    editor
        .command(Arc::new(sync_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("同步文档失败: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 一致性验证
    println!("\n🔍 一致性验证:");
    let validate_cmd = ValidateConsistencyCommand::new();
    editor
        .command(Arc::new(validate_cmd))
        .await
        .map_err(|e| anyhow::anyhow!("一致性验证失败: {}", e))?;

    // 6. 展示最终状态
    println!("\n📊 第6步: 系统状态总览");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let state = editor.get_state();

    println!("🎯 编辑器状态:");
    println!("   • 状态版本: {}", state.version);
    println!("   • 激活插件: {}", state.plugins().len());
    println!("   • 状态字段: {}", state.fields_instances.len());

    println!("\n🔌 插件状态详情:");
    for plugin in state.plugins() {
        println!("   • {} (优先级: {})", plugin.key, plugin.spec.priority);
    }

    println!("\n📐 节点系统详情:");
    println!("   • 文档根节点: document (顶级容器)");
    println!("   • 内容节点: paragraph, heading (1-6级)");
    println!("   • 列表系统: list, list_item (支持有序/无序/待办)");
    println!("   • 表格系统: table, table_row, table_cell (支持合并单元格)");
    println!("   • 特殊节点: code_block, blockquote, horizontal_rule");

    println!("\n💡 演示要点:");
    println!("   ✅ 完整节点系统: 11种节点类型支持丰富的文档结构");
    println!("   ✅ 多插件协作: 用户管理 → 权限验证 → 审计日志 → 缓存管理");
    println!("   ✅ 中间件管道: 验证 → 日志 → 性能监控");
    println!("   ✅ 事务化操作: 所有状态变更都通过事务处理");
    println!("   ✅ 状态持久化: 插件状态通过 Resource trait 管理");
    println!("   ✅ 异步处理: 完全异步的命令执行流程");
    println!("   ✅ 属性系统: 节点支持丰富的配置属性(对齐、缩进、样式等)");

    println!("\n🎉 ModuForge-RS 多插件协作演示完成!");

    // 手动释放编辑器避免运行时冲突
    std::mem::drop(editor);

    Ok(())
}

/// 创建基础文本节点（叶子节点）
fn create_text_node() -> Node {
    let spec = NodeSpec {
        content: None, // 文本节点是叶子节点，不包含其他节点
        marks: None,
        attrs: None,
        desc: Some("基础文本节点".to_string()),
        ..Default::default()
    };

    Node::create("text", spec)
}

/// 创建内联节点（可包含文本和其他内联元素）
fn create_inline_node() -> Node {
    let spec = NodeSpec {
        content: Some("text*".to_string()), // 可以包含文本节点
        marks: None,
        attrs: None,
        desc: Some("内联节点，用于内联内容".to_string()),
        ..Default::default()
    };

    Node::create("inline", spec)
}

/// 创建块级节点（可包含段落等块级元素）
fn create_block_node() -> Node {
    let spec = NodeSpec {
        content: Some("table paragraph  list heading".to_string()), // 简化为文本内容
        marks: None,
        attrs: None,
        desc: Some("块级节点，用于块级内容".to_string()),
        ..Default::default()
    };

    Node::create("block", spec)
}

# 创建编辑器

本章将指导您创建第一个基于 ModuForge-RS 的编辑器应用。

## 基本设置

### 1. 创建新项目

```bash
cargo new my-editor
cd my-editor
```

### 2. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
mf_core = "0.7.0"
mf_model = "0.7.0"
mf_state = "0.7.0"
mf_transform = "0.7.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 最简单的编辑器

创建一个支持基本文本编辑的最小编辑器：

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};
use mf_model::{Schema, Node};
use mf_state::State;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建运行时
    let runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Sync)  // 使用同步运行时
        .build()
        .await?;

    // 创建 Schema
    let schema = Schema::default();

    // 创建初始文档
    let doc = Node::new("doc", vec![
        Node::new("paragraph", vec![
            Node::text("欢迎使用 ModuForge 编辑器！")
        ])
    ]);

    // 创建状态
    let state = State::create(doc, schema, vec![]);

    println!("编辑器已初始化！");
    println!("当前文档内容：{}", state.doc().to_text());

    Ok(())
}
```

## 添加交互功能

### 命令行交互编辑器

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};
use mf_model::{Schema, Node};
use mf_state::{State, Transaction};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Sync)
        .build()
        .await?;

    let schema = Schema::default();
    let doc = Node::new("doc", vec![
        Node::new("paragraph", vec![
            Node::text("开始编辑...")
        ])
    ]);

    let mut state = State::create(doc, schema, vec![]);

    loop {
        println!("\n当前内容：");
        println!("{}", state.doc().to_text());
        println!("\n命令：");
        println!("1. 添加段落");
        println!("2. 编辑段落");
        println!("3. 删除段落");
        println!("4. 撤销");
        println!("5. 重做");
        println!("0. 退出");

        print!("选择操作: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => add_paragraph(&mut state)?,
            "2" => edit_paragraph(&mut state)?,
            "3" => delete_paragraph(&mut state)?,
            "4" => undo(&mut state)?,
            "5" => redo(&mut state)?,
            "0" => break,
            _ => println!("无效选择"),
        }
    }

    Ok(())
}

fn add_paragraph(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    print!("输入段落内容: ");
    io::stdout().flush()?;

    let mut content = String::new();
    io::stdin().read_line(&mut content)?;

    let mut tr = state.tr();
    let new_para = Node::new("paragraph", vec![
        Node::text(&content.trim())
    ]);

    tr.add_node("doc", vec![new_para])?;
    *state = state.apply(tr)?;

    println!("段落已添加！");
    Ok(())
}

fn edit_paragraph(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    // 显示所有段落
    let paragraphs = state.doc().find_all("paragraph");

    for (i, para) in paragraphs.iter().enumerate() {
        println!("{}. {}", i + 1, para.to_text());
    }

    print!("选择要编辑的段落 (序号): ");
    io::stdout().flush()?;

    let mut index = String::new();
    io::stdin().read_line(&mut index)?;
    let index: usize = index.trim().parse()?;

    if index > 0 && index <= paragraphs.len() {
        print!("输入新内容: ");
        io::stdout().flush()?;

        let mut content = String::new();
        io::stdin().read_line(&mut content)?;

        let mut tr = state.tr();
        let para_id = &paragraphs[index - 1].id;

        // 替换段落内容
        tr.replace_node_content(
            para_id.clone(),
            vec![Node::text(&content.trim())]
        )?;

        *state = state.apply(tr)?;
        println!("段落已更新！");
    }

    Ok(())
}

fn delete_paragraph(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    // 实现删除逻辑
    Ok(())
}

fn undo(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    if state.can_undo() {
        *state = state.undo()?;
        println!("已撤销！");
    } else {
        println!("没有可撤销的操作");
    }
    Ok(())
}

fn redo(state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    if state.can_redo() {
        *state = state.redo()?;
        println!("已重做！");
    } else {
        println!("没有可重做的操作");
    }
    Ok(())
}
```

## 使用异步运行时

对于需要处理并发操作的编辑器，使用异步运行时：

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType, ForgeAsyncRuntime};
use mf_model::{Schema, Node};
use mf_state::State;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建异步运行时
    let runtime = ForgeRuntimeBuilder::new()
        .runtime_type(RuntimeType::Async)
        .max_concurrent_tasks(10)  // 最大并发任务数
        .task_timeout(5000)         // 任务超时时间（毫秒）
        .build()
        .await?;

    let runtime = match runtime {
        RuntimeType::Async(async_runtime) => async_runtime,
        _ => unreachable!(),
    };

    // 创建共享状态
    let schema = Schema::default();
    let doc = Node::new("doc", vec![]);
    let state = Arc::new(RwLock::new(State::create(doc, schema, vec![])));

    // 启动多个并发编辑任务
    let handles = (0..5).map(|i| {
        let state = state.clone();
        let runtime = runtime.clone();

        tokio::spawn(async move {
            // 模拟并发编辑
            let mut state = state.write().await;
            let mut tr = state.tr();

            tr.add_node("doc", vec![
                Node::new("paragraph", vec![
                    Node::text(&format!("来自任务 {} 的段落", i))
                ])
            ])?;

            *state = state.apply(tr)?;
            Ok::<(), Box<dyn std::error::Error>>(())
        })
    });

    // 等待所有任务完成
    for handle in handles {
        handle.await??;
    }

    // 显示最终结果
    let state = state.read().await;
    println!("最终文档：");
    println!("{}", state.doc().to_text());

    Ok(())
}
```

## 添加插件支持

创建自定义插件来扩展编辑器功能：

```rust
use mf_state::plugin::{Plugin, PluginTrait};
use mf_macro::{mf_plugin, mf_meta};

// 字数统计插件
mf_plugin!(
    word_count_plugin,
    metadata = mf_meta!(
        version = "1.0.0",
        description = "字数统计插件",
        author = "MyEditor"
    ),

    append_transaction = async |trs, old_state, new_state| {
        // 计算字数
        let text = new_state.doc().to_text();
        let word_count = text.split_whitespace().count();

        println!("当前字数：{}", word_count);

        // 可以返回新的事务来更新元数据
        let mut meta_tr = new_state.tr();
        meta_tr.set_meta("word_count", json!(word_count));
        Ok(Some(meta_tr))
    }
);

// 在运行时中注册插件
let runtime = ForgeRuntimeBuilder::new()
    .runtime_type(RuntimeType::Async)
    .plugin(word_count_plugin::new())
    .build()
    .await?;
```

## 保存和加载文档

使用文件系统持久化文档：

```rust
use mf_file::{Writer, Reader};
use std::path::Path;

async fn save_document(state: &State, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let writer = Writer::create(path)?;

    // 序列化状态
    let data = serde_json::to_vec(&state)?;

    // 写入文件
    writer.append(&data)?;
    writer.flush()?;

    println!("文档已保存到 {:?}", path);
    Ok(())
}

async fn load_document(path: &Path) -> Result<State, Box<dyn std::error::Error>> {
    let reader = Reader::open(path)?;

    // 读取数据
    let data = reader.get_at(0)?;

    // 反序列化状态
    let state: State = serde_json::from_slice(&data)?;

    println!("文档已从 {:?} 加载", path);
    Ok(state)
}
```

## 完整示例：简单的 Markdown 编辑器

```rust
use mf_core::{ForgeRuntimeBuilder, RuntimeType};
use mf_model::{Schema, Node};
use mf_state::State;
use mf_derive::Node;

// 定义 Markdown 节点类型
#[derive(Node, Debug, Clone)]
#[node_type = "heading"]
#[content = "text*"]
pub struct HeadingNode {
    #[attr(default = 1)]
    pub level: u8,  // 1-6 级标题
}

#[derive(Node, Debug, Clone)]
#[node_type = "code_block"]
#[content = "text*"]
pub struct CodeBlockNode {
    #[attr]
    pub language: Option<String>,
}

#[derive(Node, Debug, Clone)]
#[node_type = "list"]
#[content = "list_item+"]
pub struct ListNode {
    #[attr(default = false)]
    pub ordered: bool,
}

#[derive(Node, Debug, Clone)]
#[node_type = "list_item"]
#[content = "paragraph+"]
pub struct ListItemNode {}

async fn create_markdown_editor() -> Result<(), Box<dyn std::error::Error>> {
    // 创建带有 Markdown 节点的 Schema
    let mut schema = Schema::new();
    schema.register(HeadingNode::node_definition());
    schema.register(CodeBlockNode::node_definition());
    schema.register(ListNode::node_definition());
    schema.register(ListItemNode::node_definition());

    let compiled_schema = schema.compile()?;

    // 创建示例文档
    let doc = Node::new("doc", vec![
        HeadingNode::new(1, "ModuForge Markdown 编辑器"),
        Node::new("paragraph", vec![
            Node::text("这是一个支持 Markdown 的编辑器示例。")
        ]),
        HeadingNode::new(2, "特性"),
        ListNode::new(false, vec![
            ListItemNode::new(vec![
                Node::new("paragraph", vec![Node::text("支持多级标题")])
            ]),
            ListItemNode::new(vec![
                Node::new("paragraph", vec![Node::text("代码块高亮")])
            ]),
            ListItemNode::new(vec![
                Node::new("paragraph", vec![Node::text("有序和无序列表")])
            ])
        ]),
        CodeBlockNode::new(Some("rust".to_string()), vec![
            Node::text("fn main() {\n    println!(\"Hello, ModuForge!\");\n}")
        ])
    ]);

    let state = State::create(doc, compiled_schema, vec![]);

    // 渲染为 Markdown
    println!("{}", render_to_markdown(&state));

    Ok(())
}

fn render_to_markdown(state: &State) -> String {
    let mut output = String::new();

    fn render_node(node: &Node, output: &mut String) {
        match node.node_type.as_str() {
            "heading" => {
                let level = node.attrs.get("level")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1) as usize;
                output.push_str(&"#".repeat(level));
                output.push(' ');
                output.push_str(&node.to_text());
                output.push_str("\n\n");
            }
            "paragraph" => {
                output.push_str(&node.to_text());
                output.push_str("\n\n");
            }
            "code_block" => {
                output.push_str("```");
                if let Some(lang) = node.attrs.get("language") {
                    output.push_str(lang.as_str().unwrap_or(""));
                }
                output.push('\n');
                output.push_str(&node.to_text());
                output.push_str("\n```\n\n");
            }
            "list" => {
                for child in node.children() {
                    render_node(child, output);
                }
            }
            "list_item" => {
                output.push_str("- ");
                for child in node.children() {
                    output.push_str(&child.to_text());
                }
                output.push('\n');
            }
            _ => {
                for child in node.children() {
                    render_node(child, output);
                }
            }
        }
    }

    let doc = state.doc();
    for child in doc.children() {
        render_node(child, &mut output);
    }

    output
}
```

## 最佳实践

### 1. 选择合适的运行时

- **同步运行时**：简单应用，单用户编辑
- **异步运行时**：需要并发，中等复杂度
- **Actor 运行时**：高并发，复杂协作场景

### 2. 状态管理

```rust
// 使用事务进行批量更新
let mut tr = state.tr();
tr.add_node(...)?;
tr.set_node_attribute(...)?;
tr.remove_node(...)?;
state = state.apply(tr)?;  // 一次性应用所有更改

// 避免频繁的小更新
// 不好
for item in items {
    let tr = state.tr();
    tr.add_node(item)?;
    state = state.apply(tr)?;
}

// 好
let mut tr = state.tr();
for item in items {
    tr.add_node(item)?;
}
state = state.apply(tr)?;
```

### 3. 错误处理

```rust
use mf_state::error::StateError;

match state.apply(tr) {
    Ok(new_state) => state = new_state,
    Err(StateError::ValidationFailed(msg)) => {
        eprintln!("验证失败：{}", msg);
        // 恢复或提示用户
    }
    Err(e) => {
        eprintln!("操作失败：{}", e);
    }
}
```

### 4. 性能优化

- 使用批量操作而不是多次单独操作
- 合理使用插件，避免在插件中执行耗时操作
- 对于大文档，考虑使用懒加载和虚拟化

## 下一步

现在您已经创建了基本的编辑器，可以继续学习：

- [文档模型](./document-model.md) - 深入理解节点和树结构
- [状态管理](./state-management.md) - 掌握事务和状态更新
- [插件系统](./plugins.md) - 开发自定义插件扩展功能
- [实际示例](../examples/basic-editor.md) - 查看完整的编辑器实现
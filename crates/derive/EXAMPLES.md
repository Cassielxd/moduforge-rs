# ModuForge-RS 派生宏示例集

本文档提供了 ModuForge-RS 派生宏的完整示例集，从基础到高级应用。

## 目录

- [基础示例](#基础示例)
- [高级示例](#高级示例)
- [实际应用场景](#实际应用场景)
- [命令模式示例](#命令模式示例)
- [最佳实践示例](#最佳实践示例)

## 基础示例

### 1. 最简单的 Node

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "text"]
pub struct TextNode {
    #[attr]
    content: String,
}

// 使用
fn example() {
    let text = TextNode {
        content: "Hello, World!".to_string(),
    };

    let node = text.to_node();
    println!("节点类型: {}", node.node_type);
    println!("内容: {:?}", node.attrs.get("content"));
}
```

### 2. 带 ID 的 Node

```rust
#[derive(Node)]
#[node_type = "document"]
pub struct DocumentNode {
    #[id]
    doc_id: String,

    #[attr]
    title: String,

    #[attr]
    author: String,
}

// 使用
fn create_document() {
    let doc = DocumentNode {
        doc_id: "doc_001".to_string(),
        title: "技术文档".to_string(),
        author: "张三".to_string(),
    };

    let node = doc.to_node();
    assert_eq!(node.id, Some("doc_001".to_string()));
}
```

### 3. 带默认值的 Node

```rust
#[derive(Node)]
#[node_type = "config"]
pub struct ConfigNode {
    #[attr(default="localhost")]
    host: String,

    #[attr(default=8080)]
    port: i32,

    #[attr(default=true)]
    enabled: bool,

    #[attr(default=3.14)]
    timeout: f64,
}

// 使用默认值
impl Default for ConfigNode {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            enabled: true,
            timeout: 3.14,
        }
    }
}
```

### 4. 带可选字段的 Node

```rust
#[derive(Node)]
#[node_type = "user"]
pub struct UserNode {
    #[attr]
    username: String,

    #[attr]
    email: Option<String>,

    #[attr]
    phone: Option<String>,

    #[attr(default="user")]
    role: String,
}

// 使用
fn create_user() {
    let user = UserNode {
        username: "john_doe".to_string(),
        email: Some("john@example.com".to_string()),
        phone: None,
        role: "admin".to_string(),
    };

    let node = user.to_node();
    // phone 字段为 None，不会出现在 attrs 中
}
```

### 5. 简单的 Mark

```rust
use mf_derive::Mark;

#[derive(Mark)]
#[mark_type = "emphasis"]
pub struct EmphasisMark {
    #[attr(default="bold")]
    style: String,

    #[attr]
    level: Option<i32>,
}

// 使用
fn apply_emphasis() {
    let mark = EmphasisMark {
        style: "italic".to_string(),
        level: Some(2),
    };

    let mark_instance = mark.to_mark();
    println!("标记类型: {}", mark_instance.mark_type);
}
```

## 高级示例

### 1. 完整的文档节点系统

```rust
use mf_derive::{Node, Mark};
use lazy_static::lazy_static;

// 段落节点
#[derive(Node)]
#[node_type = "paragraph"]
#[desc = "文本段落"]
#[marks = "bold italic underline link"]
#[content = "text*"]
pub struct ParagraphNode {
    #[attr]
    alignment: Option<String>,

    #[attr(default=1.5)]
    line_height: f64,

    #[attr]
    indent: Option<i32>,
}

// 标题节点
#[derive(Node)]
#[node_type = "heading"]
#[desc = "标题"]
#[marks = "bold italic"]
#[content = "text*"]
pub struct HeadingNode {
    #[attr(default=1)]
    level: i32,  // 1-6

    #[attr]
    anchor: Option<String>,
}

// 代码块节点
#[derive(Node)]
#[node_type = "code_block"]
#[desc = "代码块"]
pub struct CodeBlockNode {
    #[attr]
    language: Option<String>,

    #[attr]
    code: String,

    #[attr(default=false)]
    line_numbers: bool,

    #[attr]
    highlight_lines: Option<String>,  // "1,3-5,7"
}

// 列表节点
#[derive(Node)]
#[node_type = "list"]
#[desc = "列表"]
#[content = "list_item+"]
pub struct ListNode {
    #[attr(default="bullet")]
    list_type: String,  // "bullet" | "ordered"

    #[attr(default=1)]
    start: i32,  // 起始编号（仅对有序列表）

    #[attr]
    tight: Option<bool>,  // 紧凑列表
}

// 列表项节点
#[derive(Node)]
#[node_type = "list_item"]
#[desc = "列表项"]
#[content = "(paragraph | list)*"]
pub struct ListItemNode {
    #[attr]
    checked: Option<bool>,  // 任务列表复选框
}

// 粗体标记
#[derive(Mark)]
#[mark_type = "bold"]
pub struct BoldMark {
    #[attr(default=700)]
    weight: i32,
}

// 斜体标记
#[derive(Mark)]
#[mark_type = "italic"]
pub struct ItalicMark {}

// 链接标记
#[derive(Mark)]
#[mark_type = "link"]
pub struct LinkMark {
    #[attr]
    href: String,

    #[attr]
    title: Option<String>,

    #[attr(default="_blank")]
    target: String,
}

// 注册所有节点和标记
lazy_static! {
    pub static ref PARAGRAPH: mf_core::node::Node = ParagraphNode::node_definition();
    pub static ref HEADING: mf_core::node::Node = HeadingNode::node_definition();
    pub static ref CODE_BLOCK: mf_core::node::Node = CodeBlockNode::node_definition();
    pub static ref LIST: mf_core::node::Node = ListNode::node_definition();
    pub static ref LIST_ITEM: mf_core::node::Node = ListItemNode::node_definition();

    pub static ref BOLD: mf_core::mark::Mark = BoldMark::mark_definition();
    pub static ref ITALIC: mf_core::mark::Mark = ItalicMark::mark_definition();
    pub static ref LINK: mf_core::mark::Mark = LinkMark::mark_definition();
}

pub fn register_document_schema() -> (Vec<mf_core::node::Node>, Vec<mf_core::mark::Mark>) {
    let nodes = vec![
        PARAGRAPH.clone(),
        HEADING.clone(),
        CODE_BLOCK.clone(),
        LIST.clone(),
        LIST_ITEM.clone(),
    ];

    let marks = vec![
        BOLD.clone(),
        ITALIC.clone(),
        LINK.clone(),
    ];

    (nodes, marks)
}
```

### 2. 项目管理系统节点

```rust
#[derive(Node)]
#[node_type = "project"]
#[desc = "项目节点"]
#[content = "(milestone | task | subtask)*"]
pub struct ProjectNode {
    #[id]
    project_id: String,

    #[attr]
    name: String,

    #[attr]
    description: Option<String>,

    #[attr(default="planning")]
    status: String,  // planning, active, completed, archived

    #[attr]
    start_date: Option<String>,  // ISO 8601

    #[attr]
    end_date: Option<String>,

    #[attr(default=1)]
    priority: i32,  // 1-5

    #[attr]
    owner: String,

    #[attr]
    team: Option<String>,  // JSON array of team members

    #[attr(default=0.0)]
    progress: f64,  // 0.0 - 100.0

    #[attr]
    budget: Option<f64>,

    #[attr]
    tags: Option<String>,  // Comma-separated
}

#[derive(Node)]
#[node_type = "milestone"]
#[desc = "里程碑"]
pub struct MilestoneNode {
    #[id]
    milestone_id: String,

    #[attr]
    title: String,

    #[attr]
    due_date: String,

    #[attr(default=false)]
    completed: bool,

    #[attr]
    deliverables: Option<String>,  // JSON array
}

#[derive(Node)]
#[node_type = "task"]
#[desc = "任务"]
#[content = "subtask*"]
#[marks = "priority status assignee"]
pub struct TaskNode {
    #[id]
    task_id: String,

    #[attr]
    title: String,

    #[attr]
    description: Option<String>,

    #[attr(default="todo")]
    status: String,  // todo, in_progress, review, done, cancelled

    #[attr(default=3)]
    priority: i32,

    #[attr]
    assignee: Option<String>,

    #[attr]
    due_date: Option<String>,

    #[attr(default=0)]
    estimated_hours: i32,

    #[attr(default=0)]
    actual_hours: i32,

    #[attr]
    dependencies: Option<String>,  // JSON array of task_ids

    #[attr]
    labels: Option<String>,  // Comma-separated
}
```

## 实际应用场景

### 1. 博客系统

```rust
#[derive(Node)]
#[node_type = "blog_post"]
#[desc = "博客文章"]
#[content = "(heading | paragraph | image | code_block | quote)*"]
#[marks = "highlight"]
pub struct BlogPostNode {
    #[id]
    post_id: String,

    #[attr]
    title: String,

    #[attr]
    slug: String,

    #[attr]
    author: String,

    #[attr]
    published_at: Option<String>,

    #[attr(default="draft")]
    status: String,  // draft, published, archived

    #[attr]
    excerpt: Option<String>,

    #[attr]
    featured_image: Option<String>,

    #[attr]
    categories: Option<String>,

    #[attr]
    tags: Option<String>,

    #[attr(default=0)]
    views: i32,

    #[attr(default=0)]
    likes: i32,

    #[attr(default=true)]
    comments_enabled: bool,
}

// 使用示例
impl BlogPostNode {
    pub fn publish(&mut self) {
        self.status = "published".to_string();
        self.published_at = Some(chrono::Utc::now().to_rfc3339());
    }

    pub fn increment_views(&mut self) {
        self.views += 1;
    }
}
```

### 2. 表单系统

```rust
#[derive(Node)]
#[node_type = "form"]
#[desc = "表单"]
#[content = "form_field+"]
pub struct FormNode {
    #[id]
    form_id: String,

    #[attr]
    name: String,

    #[attr]
    action: String,

    #[attr(default="POST")]
    method: String,

    #[attr]
    validation_rules: Option<String>,  // JSON
}

#[derive(Node)]
#[node_type = "form_field"]
#[desc = "表单字段"]
pub struct FormFieldNode {
    #[attr]
    name: String,

    #[attr]
    field_type: String,  // text, email, password, select, checkbox, etc.

    #[attr]
    label: String,

    #[attr]
    placeholder: Option<String>,

    #[attr(default=false)]
    required: bool,

    #[attr]
    default_value: Option<String>,

    #[attr]
    validation: Option<String>,  // regex pattern or validation rule

    #[attr]
    options: Option<String>,  // JSON array for select/radio

    #[attr(default=false)]
    disabled: bool,
}
```

## 命令模式示例

### 1. 基础命令

```rust
use mf_derive::impl_command;
use moduforge_model::Transaction;

#[impl_command(CreateUser)]
async fn create_user(
    tr: &mut Transaction,
    username: String,
    email: String,
    role: String,
) -> Result<(), Error> {
    // 创建用户节点
    let user = UserNode {
        username: username.clone(),
        email: Some(email),
        role,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    tr.insert_node(user.to_node())?;
    tr.commit().await?;
    Ok(())
}

// 使用生成的命令
async fn example_usage(tr: &mut Transaction) {
    let cmd = CreateUser::new(
        "john_doe".to_string(),
        "john@example.com".to_string(),
        "user".to_string(),
    );

    cmd.execute(tr).await.unwrap();
}
```

### 2. 复杂命令

```rust
#[impl_command(UpdateProjectStatus)]
async fn update_project_status(
    tr: &mut Transaction,
    project_id: String,
    new_status: String,
    update_tasks: bool,
    notify_team: bool,
) -> Result<ProjectUpdateResult, Error> {
    // 获取项目
    let project = tr.get_node(&project_id)?;

    // 更新状态
    project.attrs.insert("status".to_string(), json!(new_status));
    tr.update_node(project)?;

    // 可选：更新所有任务
    if update_tasks {
        let tasks = tr.query_nodes(json!({
            "parent_id": project_id
        }))?;

        for mut task in tasks {
            task.attrs.insert("status".to_string(), json!(new_status));
            tr.update_node(task)?;
        }
    }

    // 可选：通知团队
    if notify_team {
        // 发送通知逻辑
    }

    tr.commit().await?;

    Ok(ProjectUpdateResult {
        updated_count: tasks.len() + 1,
        notified: notify_team,
    })
}
```

### 3. 批处理命令

```rust
#[impl_command(BatchImportTasks)]
async fn batch_import_tasks(
    tr: &mut Transaction,
    project_id: String,
    csv_data: String,
) -> Result<ImportResult, Error> {
    let mut reader = csv::Reader::from_reader(csv_data.as_bytes());
    let mut imported = 0;
    let mut errors = Vec::new();

    for result in reader.records() {
        match result {
            Ok(record) => {
                let task = TaskNode {
                    task_id: uuid::Uuid::new_v4().to_string(),
                    title: record.get(0).unwrap_or("").to_string(),
                    description: record.get(1).map(|s| s.to_string()),
                    status: "todo".to_string(),
                    priority: record.get(2)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(3),
                    assignee: record.get(3).map(|s| s.to_string()),
                    due_date: record.get(4).map(|s| s.to_string()),
                    estimated_hours: 0,
                    actual_hours: 0,
                    dependencies: None,
                    labels: record.get(5).map(|s| s.to_string()),
                };

                tr.insert_node(task.to_node())?;
                imported += 1;
            }
            Err(e) => {
                errors.push(format!("行解析错误: {}", e));
            }
        }
    }

    tr.commit().await?;

    Ok(ImportResult {
        imported,
        errors,
    })
}
```

## 最佳实践示例

### 1. 使用 Builder 模式

```rust
#[derive(Node)]
#[node_type = "article"]
pub struct ArticleNode {
    #[id]
    article_id: String,

    #[attr]
    title: String,

    #[attr]
    content: String,

    #[attr]
    author: Option<String>,

    #[attr]
    tags: Option<String>,

    #[attr(default="draft")]
    status: String,
}

// Builder 实现
pub struct ArticleBuilder {
    article_id: Option<String>,
    title: Option<String>,
    content: Option<String>,
    author: Option<String>,
    tags: Option<String>,
    status: String,
}

impl ArticleBuilder {
    pub fn new() -> Self {
        Self {
            article_id: None,
            title: None,
            content: None,
            author: None,
            tags: None,
            status: "draft".to_string(),
        }
    }

    pub fn id(mut self, id: String) -> Self {
        self.article_id = Some(id);
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags.join(","));
        self
    }

    pub fn status(mut self, status: String) -> Self {
        self.status = status;
        self
    }

    pub fn build(self) -> Result<ArticleNode, String> {
        Ok(ArticleNode {
            article_id: self.article_id
                .ok_or_else(|| "article_id is required".to_string())?,
            title: self.title
                .ok_or_else(|| "title is required".to_string())?,
            content: self.content
                .ok_or_else(|| "content is required".to_string())?,
            author: self.author,
            tags: self.tags,
            status: self.status,
        })
    }
}

// 使用示例
fn create_article() -> Result<ArticleNode, String> {
    ArticleBuilder::new()
        .id(uuid::Uuid::new_v4().to_string())
        .title("Rust 宏编程指南".to_string())
        .content("详细内容...".to_string())
        .author("技术专家".to_string())
        .tags(vec!["rust".to_string(), "macro".to_string()])
        .build()
}
```

### 2. 验证器模式

```rust
trait NodeValidator {
    fn validate(&self) -> Result<(), Vec<String>>;
}

impl NodeValidator for TaskNode {
    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // 验证标题
        if self.title.is_empty() {
            errors.push("标题不能为空".to_string());
        }

        if self.title.len() > 200 {
            errors.push("标题不能超过200个字符".to_string());
        }

        // 验证优先级
        if self.priority < 1 || self.priority > 5 {
            errors.push("优先级必须在1-5之间".to_string());
        }

        // 验证日期格式
        if let Some(ref due_date) = self.due_date {
            if chrono::DateTime::parse_from_rfc3339(due_date).is_err() {
                errors.push("截止日期格式无效".to_string());
            }
        }

        // 验证状态
        let valid_statuses = ["todo", "in_progress", "review", "done", "cancelled"];
        if !valid_statuses.contains(&self.status.as_str()) {
            errors.push(format!("无效的状态: {}", self.status));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// 使用示例
fn validate_before_save(task: &TaskNode) -> Result<(), String> {
    task.validate()
        .map_err(|errors| errors.join("; "))
}
```

### 3. 工厂模式

```rust
pub trait NodeFactory {
    type Output;
    fn create(&self) -> Self::Output;
}

pub struct DocumentNodeFactory {
    template: String,
}

impl DocumentNodeFactory {
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }
}

impl NodeFactory for DocumentNodeFactory {
    type Output = DocumentNode;

    fn create(&self) -> Self::Output {
        match self.template.as_str() {
            "report" => DocumentNode {
                doc_id: uuid::Uuid::new_v4().to_string(),
                title: "报告模板".to_string(),
                author: "系统".to_string(),
                template: Some("report".to_string()),
                sections: Some("introduction,analysis,conclusion".to_string()),
            },
            "memo" => DocumentNode {
                doc_id: uuid::Uuid::new_v4().to_string(),
                title: "备忘录模板".to_string(),
                author: "系统".to_string(),
                template: Some("memo".to_string()),
                sections: Some("to,from,subject,body".to_string()),
            },
            _ => DocumentNode {
                doc_id: uuid::Uuid::new_v4().to_string(),
                title: "空白文档".to_string(),
                author: "系统".to_string(),
                template: None,
                sections: None,
            },
        }
    }
}

// 使用示例
fn create_from_template() {
    let factory = DocumentNodeFactory::new("report");
    let document = factory.create();
    println!("创建的文档: {}", document.title);
}
```

## 总结

这些示例展示了 ModuForge-RS 派生宏的强大功能和灵活性。通过这些模式，您可以：

1. **快速定义数据模型** - 使用声明式语法
2. **保持类型安全** - 编译时验证
3. **实现复杂业务逻辑** - 结合设计模式
4. **构建可扩展系统** - 模块化架构
5. **简化代码维护** - 清晰的结构

更多示例请参考：
- [测试用例](tests/)
- [集成示例](examples/)
- [API 文档](API_REFERENCE.md)
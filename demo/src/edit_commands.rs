use mf_state::transaction::{Transaction, Command};
use mf_transform::TransformResult;
use async_trait::async_trait;
use std::fmt::Debug;

/// 安全截断UTF-8字符串，确保不会在字符边界中间切断
fn safe_truncate(
    s: &str,
    max_chars: usize,
) -> &str {
    if s.chars().count() <= max_chars {
        s
    } else {
        let mut end = 0;
        for (i, _) in s.char_indices().take(max_chars) {
            end = i;
        }
        // 找到最后一个字符的结束位置
        if let Some((last_start, ch)) = s.char_indices().nth(max_chars - 1) {
            end = last_start + ch.len_utf8();
        }
        &s[..end]
    }
}

// ===== 用户管理命令 =====

/// 用户登录命令
#[derive(Debug)]
pub struct UserLoginCommand {
    pub username: String,
    pub role: String,
}

impl UserLoginCommand {
    pub fn new(
        username: &str,
        role: &str,
    ) -> Self {
        Self { username: username.to_string(), role: role.to_string() }
    }
}

#[async_trait]
impl Command for UserLoginCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "user_login");
        tr.set_meta("username", self.username.clone());
        tr.set_meta("role", self.role.clone());

        println!("👤 用户 {} ({}) 正在登录", self.username, self.role);
        Ok(())
    }

    fn name(&self) -> String {
        format!("UserLogin({})", self.username)
    }
}

// ===== 文档操作命令 =====

/// 创建文档命令
#[derive(Debug)]
pub struct CreateDocumentCommand {
    pub title: String,
    pub description: String,
}

impl CreateDocumentCommand {
    pub fn new(
        title: &str,
        description: &str,
    ) -> Self {
        Self { title: title.to_string(), description: description.to_string() }
    }
}

#[async_trait]
impl Command for CreateDocumentCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "create_document");
        tr.set_meta("title", self.title.clone());
        tr.set_meta("description", self.description.clone());

        println!("📄 创建文档: {}", self.title);
        Ok(())
    }

    fn name(&self) -> String {
        format!("CreateDocument({})", self.title)
    }
}

// ===== 内容编辑命令 =====

/// 添加标题命令
#[derive(Debug)]
pub struct AddHeadingCommand {
    pub level: u32,
    pub text: String,
}

impl AddHeadingCommand {
    pub fn new(
        level: u32,
        text: &str,
    ) -> Self {
        Self { level, text: text.to_string() }
    }
}

#[async_trait]
impl Command for AddHeadingCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "add_heading");
        tr.set_meta("level", self.level);
        tr.set_meta("text", self.text.clone());

        println!("📝 添加 H{} 标题: {}", self.level, self.text);
        Ok(())
    }

    fn name(&self) -> String {
        format!("AddHeading(H{}: {})", self.level, self.text)
    }
}

/// 添加段落命令
#[derive(Debug)]
pub struct AddParagraphCommand {
    pub text: String,
}

impl AddParagraphCommand {
    pub fn new(text: &str) -> Self {
        Self { text: text.to_string() }
    }
}

#[async_trait]
impl Command for AddParagraphCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "add_paragraph");
        tr.set_meta("text", self.text.clone());

        println!(
            "📝 添加段落: {}...",
            if self.text.chars().count() > 50 {
                safe_truncate(&self.text, 50)
            } else {
                &self.text
            }
        );
        Ok(())
    }

    fn name(&self) -> String {
        format!(
            "AddParagraph({}...)",
            if self.text.chars().count() > 20 {
                safe_truncate(&self.text, 20)
            } else {
                &self.text
            }
        )
    }
}

/// 编辑段落命令
#[derive(Debug)]
pub struct EditParagraphCommand {
    pub paragraph_id: String,
    pub new_text: String,
}

impl EditParagraphCommand {
    pub fn new(
        paragraph_id: String,
        new_text: String,
    ) -> Self {
        Self { paragraph_id, new_text }
    }
}

#[async_trait]
impl Command for EditParagraphCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "edit_paragraph");
        tr.set_meta("paragraph_id", self.paragraph_id.clone());
        tr.set_meta("new_text", self.new_text.clone());

        println!(
            "✏️ 编辑段落 {}: {}...",
            self.paragraph_id,
            if self.new_text.chars().count() > 30 {
                safe_truncate(&self.new_text, 30)
            } else {
                &self.new_text
            }
        );
        Ok(())
    }

    fn name(&self) -> String {
        format!("EditParagraph({})", self.paragraph_id)
    }
}

/// 添加列表命令
#[derive(Debug)]
pub struct AddListCommand {
    pub items: Vec<String>,
}

impl AddListCommand {
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

#[async_trait]
impl Command for AddListCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "add_list");
        tr.set_meta("items", self.items.clone());
        tr.set_meta("item_count", self.items.len() as u32);

        println!("📋 添加列表，包含 {} 项", self.items.len());
        for (i, item) in self.items.iter().enumerate() {
            println!("   {}. {}", i + 1, item);
        }
        Ok(())
    }

    fn name(&self) -> String {
        format!("AddList({} items)", self.items.len())
    }
}

/// 添加表格命令
#[derive(Debug)]
pub struct AddTableCommand {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl AddTableCommand {
    pub fn new(
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    ) -> Self {
        Self { headers, rows }
    }
}

#[async_trait]
impl Command for AddTableCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "add_table");
        tr.set_meta("headers", self.headers.clone());
        tr.set_meta("row_count", self.rows.len() as u32);
        tr.set_meta("col_count", self.headers.len() as u32);

        println!(
            "📊 添加表格: {} x {} (列 x 行)",
            self.headers.len(),
            self.rows.len() + 1
        ); // +1 for header
        println!("   表头: {:?}", self.headers);
        Ok(())
    }

    fn name(&self) -> String {
        format!("AddTable({}x{})", self.headers.len(), self.rows.len() + 1)
    }
}

// ===== 协作和冲突解决命令 =====

/// 解决冲突命令
#[derive(Debug)]
pub struct ResolveConflictCommand {
    pub element_id: String,
    pub resolved_content: String,
    pub resolver: String,
}

impl ResolveConflictCommand {
    pub fn new(
        element_id: String,
        resolved_content: String,
        resolver: String,
    ) -> Self {
        Self { element_id, resolved_content, resolver }
    }
}

#[async_trait]
impl Command for ResolveConflictCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "resolve_conflict");
        tr.set_meta("element_id", self.element_id.clone());
        tr.set_meta("resolved_content", self.resolved_content.clone());
        tr.set_meta("resolver", self.resolver.clone());

        println!("⚖️ {} 解决冲突: {}", self.resolver, self.element_id);
        Ok(())
    }

    fn name(&self) -> String {
        format!("ResolveConflict({}, by: {})", self.element_id, self.resolver)
    }
}

/// 同步文档命令
#[derive(Debug)]
pub struct SyncDocumentCommand {
    pub sync_id: String,
}

impl SyncDocumentCommand {
    pub fn new(sync_id: String) -> Self {
        Self { sync_id }
    }
}

#[async_trait]
impl Command for SyncDocumentCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "sync_document");
        tr.set_meta("sync_id", self.sync_id.clone());

        println!("🔄 同步文档: {}", self.sync_id);
        Ok(())
    }

    fn name(&self) -> String {
        format!("SyncDocument({})", self.sync_id)
    }
}

// ===== 版本控制命令 =====

/// 创建快照命令
#[derive(Debug)]
pub struct CreateSnapshotCommand {
    pub description: String,
}

impl CreateSnapshotCommand {
    pub fn new(description: &str) -> Self {
        Self { description: description.to_string() }
    }
}

#[async_trait]
impl Command for CreateSnapshotCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "create_snapshot");
        tr.set_meta("description", self.description.clone());

        println!("📸 创建版本快照: {}", self.description);
        Ok(())
    }

    fn name(&self) -> String {
        format!("CreateSnapshot({})", self.description)
    }
}

/// 验证数据一致性命令
#[derive(Debug)]
pub struct ValidateConsistencyCommand;

impl ValidateConsistencyCommand {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Command for ValidateConsistencyCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "validate_consistency");

        println!("🔍 验证数据一致性");
        Ok(())
    }

    fn name(&self) -> String {
        "ValidateConsistency".to_string()
    }
}

// ===== 批量操作命令 =====

/// 批量编辑命令
#[derive(Debug)]
pub struct BatchEditCommand {
    pub commands: Vec<Box<dyn Command>>,
}

impl BatchEditCommand {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn add_command(
        mut self,
        command: Box<dyn Command>,
    ) -> Self {
        self.commands.push(command);
        self
    }
}

#[async_trait]
impl Command for BatchEditCommand {
    async fn execute(
        &self,
        tr: &mut Transaction,
    ) -> TransformResult<()> {
        tr.set_meta("action", "batch_edit");
        tr.set_meta("command_count", self.commands.len() as u32);

        println!("📦 执行批量编辑，包含 {} 个命令", self.commands.len());

        // 执行所有子命令
        for (i, command) in self.commands.iter().enumerate() {
            println!(
                "   执行命令 {}/{}: {}",
                i + 1,
                self.commands.len(),
                command.name()
            );
            command.execute(tr).await?;
        }

        Ok(())
    }

    fn name(&self) -> String {
        format!("BatchEdit({} commands)", self.commands.len())
    }
}

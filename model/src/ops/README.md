# ModuForge 操作符扩展系统

## 概述

ModuForge 通过 Rust 的操作符重载机制，提供了一套直观且强大的 Tree 操作 API。这些操作符大大简化了文档树的操作，使代码更加简洁和易读。

## 操作符总览

| 操作符 | 功能 | 应用场景 | 示例 |
|--------|------|----------|------|
| `+` | 添加/插入 | 添加节点、标记、属性 | `doc + paragraph` |
| `-` | 删除/移除 | 删除节点、标记 | `doc - node_id` |
| `*` | 复制/克隆 | 复制节点 | `doc * 3` |
| `/` | 分割/提取 | 分页、按类型过滤 | `doc / 10` |
| `\|` | 合并/联合 | 合并内容、去重标记 | `doc1 \| doc2` |
| `&` | 交集/过滤 | 保留特定内容 | `doc & "paragraph"` |
| `^` | 切换/反转 | 切换标记、属性 | `marks ^ bold` |
| `<<` | 左移/优先 | 插入到开头 | `doc << urgent_node` |
| `>>` | 右移/追加 | 插入到末尾、移动 | `doc >> footer_node` |
| `%` | 采样/选择 | 按条件选择节点 | `doc % 2` |

## 详细说明

### 1. 加法运算符 `+` (add.rs)

**用途**: 添加内容到树结构中

**NodeRef 支持的操作**:
- `NodeRef + Node` - 添加单个节点
- `NodeRef + (usize, Node)` - 在指定位置添加节点
- `NodeRef + Vec<Node>` - 添加多个节点
- `NodeRef + NodeEnum` - 添加枚举节点

**MarkRef 支持的操作**:
- `MarkRef + Mark` - 添加单个标记
- `MarkRef + Vec<Mark>` - 添加多个标记

**AttrsRef 支持的操作**:
- `AttrsRef + Attrs` - 添加属性对象
- `AttrsRef + (String, Value)` - 添加键值对
- `AttrsRef + HashMap<String, Value>` - 添加属性映射

```rust
// 示例
let doc = tree.node("doc")?;
(doc + paragraph_node)?;  // 添加段落
(doc + (0, urgent_node))?;  // 在开头插入紧急节点
```

### 2. 减法运算符 `-` (sub.rs)

**用途**: 从树结构中移除内容

**NodeRef 支持的操作**:
- `NodeRef - NodeId` - 删除指定节点
- `NodeRef - Vec<NodeId>` - 删除多个节点
- `NodeRef - usize` - 按索引删除节点

**MarkRef 支持的操作**:
- `MarkRef - Mark` - 删除指定标记
- `MarkRef - String` - 按名称删除标记
- `MarkRef - Vec<Mark>` - 删除多个标记

```rust
// 示例
(doc - old_node_id)?;  // 删除旧节点
(marks - "bold")?;  // 删除粗体标记
```

### 3. 乘法运算符 `*` (mul.rs)

**用途**: 复制和克隆节点

**NodeRef 支持的操作**:
- `NodeRef * usize` - 复制当前节点N次
- `NodeRef * NodeId` - 复制指定节点
- `NodeRef * Vec<NodeId>` - 复制多个指定节点

```rust
// 示例
(doc * 3)?;  // 复制当前节点3次
(doc * template_id)?;  // 复制模板节点
```

### 4. 除法运算符 `/` (div.rs)

**用途**: 分割和提取内容

**NodeRef 支持的操作**:
- `NodeRef / usize` - 按页大小分割（返回第一页）
- `NodeRef / String` - 按节点类型过滤
- `NodeRef / Vec<String>` - 按多个类型过滤
- `NodeRef / (usize, usize)` - 按索引范围提取

```rust
// 示例
let first_page = (doc / 10)?;  // 每页10个节点
let paragraphs = (doc / "paragraph")?;  // 获取所有段落
let range = (doc / (5, 15))?;  // 获取索引5-15的节点
```

### 5. 位或运算符 `|` (bitor.rs)

**用途**: 合并和联合操作

**NodeRef 支持的操作**:
- `NodeRef | NodeId` - 合并另一个节点的子节点
- `NodeRef | Vec<NodeId>` - 合并多个节点的子节点
- `NodeRef | Vec<Node>` - 直接合并节点列表

**MarkRef 支持的操作**:
- `MarkRef | Mark` - 合并标记（自动去重）
- `MarkRef | Vec<Mark>` - 合并多个标记（自动去重）

```rust
// 示例
(doc1 | doc2_id)?;  // 将doc2的内容合并到doc1
(marks | new_mark)?;  // 添加新标记（如果不存在）
```

### 6. 位与运算符 `&` (bitand.rs)

**用途**: 交集和过滤操作

**NodeRef 支持的操作**:
- `NodeRef & String` - 只保留指定类型的节点
- `NodeRef & Vec<String>` - 保留多个指定类型的节点
- `NodeRef & Vec<NodeId>` - 只保留指定的节点列表

**MarkRef 支持的操作**:
- `MarkRef & String` - 只保留指定名称的标记
- `MarkRef & Vec<String>` - 保留多个指定名称的标记
- `MarkRef & Vec<Mark>` - 只保留指定的标记列表

```rust
// 示例
(doc & "paragraph")?;  // 只保留段落节点
(marks & vec!["bold", "italic"])?;  // 只保留粗体和斜体标记
```

### 7. 异或运算符 `^` (bitxor.rs)

**用途**: 切换和反转操作

**MarkRef 支持的操作**:
- `MarkRef ^ Mark` - 切换标记状态
- `MarkRef ^ Vec<Mark>` - 切换多个标记状态
- `MarkRef ^ String` - 按名称切换标记

**AttrsRef 支持的操作**:
- `AttrsRef ^ String` - 切换布尔属性
- `AttrsRef ^ Vec<String>` - 切换多个布尔属性
- `AttrsRef ^ (String, Value, Value)` - 在两个值之间切换

```rust
// 示例
(marks ^ bold_mark)?;  // 切换粗体（有则删，无则加）
(attrs ^ "hidden")?;  // 切换隐藏属性
(attrs ^ ("theme", "light", "dark"))?;  // 在浅色和深色主题间切换
```

### 8. 左移运算符 `<<` (shl.rs)

**用途**: 插入到开头或向左移动

**NodeRef 支持的操作**:
- `NodeRef << Node` - 在开头插入节点
- `NodeRef << Vec<Node>` - 在开头插入多个节点
- `NodeRef << usize` - 向左移动指定位置

**MarkRef 支持的操作**:
- `MarkRef << Mark` - 在开头插入标记（最高优先级）
- `MarkRef << Vec<Mark>` - 在开头插入多个标记
- `MarkRef << String` - 按名称创建高优先级标记

```rust
// 示例
(doc << urgent_node)?;  // 插入紧急内容到开头
(marks << priority_mark)?;  // 添加高优先级标记
```

### 9. 右移运算符 `>>` (shr.rs)

**用途**: 插入到末尾或向右移动

**NodeRef 支持的操作**:
- `NodeRef >> Node` - 在末尾添加节点
- `NodeRef >> Vec<Node>` - 在末尾添加多个节点
- `NodeRef >> usize` - 向右移动指定位置
- `NodeRef >> (usize, bool)` - 移动到绝对位置

**MarkRef 支持的操作**:
- `MarkRef >> Mark` - 在末尾添加标记
- `MarkRef >> Vec<Mark>` - 在末尾添加多个标记
- `MarkRef >> String` - 按名称创建低优先级标记

```rust
// 示例
(doc >> footer_node)?;  // 添加页脚到末尾
(node >> 3)?;  // 向右移动3个位置
```

### 10. 取模运算符 `%` (rem.rs)

**用途**: 选择和采样操作

**NodeRef 支持的操作**:
- `NodeRef % usize` - 按步长选择节点
- `NodeRef % String` - 按模式匹配选择节点
- `NodeRef % (usize, String)` - 随机或规则采样
- `NodeRef % Vec<String>` - 按多条件选择
- `NodeRef % (usize, usize, String)` - 分页选择

```rust
// 示例
let every_third = (doc % 3)?;  // 选择每第3个节点
let paragraphs = (doc % "paragraph")?;  // 选择所有段落
let random_5 = (doc % (5, "random"))?;  // 随机选择5个节点
let page_2 = (doc % (2, 10, "page"))?;  // 第2页，每页10个
```

## 组合使用示例

### 富文本编辑器场景

```rust
// 1. 创建文档结构
(doc + title)?;
(doc + paragraphs)?;

// 2. 插入紧急通知
(doc << alert)?;

// 3. 复制重要内容
(doc * important_id)?;

// 4. 格式化文本
(marks ^ bold_mark)?;
(marks | italic_mark)?;

// 5. 过滤和清理
(doc & vec!["paragraph", "heading"])?;

// 6. 分页显示
let page = (doc / 10)?;
```

### 内容管理系统场景

```rust
// 批量操作
(doc + articles)?;          // 添加文章
(doc & "published")?;       // 只保留已发布内容
let featured = (doc % (5, "random"))?;  // 随机选择5篇推荐
(sidebar << featured_list)?;            // 侧边栏添加推荐

// 标记管理
(article.marks() ^ "featured")?;        // 切换推荐状态
(article.attrs() ^ "published")?;       // 切换发布状态
```

## 性能考虑

1. **惰性计算**: 大部分操作符返回结果而不是立即修改树结构
2. **批量操作**: 使用 `Vec<>` 版本进行批量操作以提高性能
3. **内存效率**: 利用不可变数据结构的结构共享特性
4. **错误处理**: 所有操作返回 `PoolResult<>` 以提供完整的错误信息

## 扩展指南

如需添加新的操作符：

1. 在 `ops/` 目录下创建新的 `.rs` 文件
2. 实现相应的 trait（如 `std::ops::Add`）
3. 在 `mod.rs` 中添加模块声明
4. 添加测试用例和文档示例

## 最佳实践

1. **链式操作**: 优先使用操作符链式调用
2. **类型安全**: 利用 Rust 的类型系统确保操作安全
3. **错误处理**: 始终处理操作符返回的 `Result`
4. **性能优化**: 对大量数据使用批量操作版本
5. **可读性**: 选择最能表达意图的操作符组合

这套操作符系统让 ModuForge 的 Tree 操作变得直观、高效且类型安全，大大提升了开发体验和代码质量。 
# 自定义标记

ModuForge-RS 提供了灵活的标记（Mark）系统，用于为文本添加样式、注释和元数据。本章将介绍如何创建和使用自定义标记，包括背景颜色、多媒体链接和批注等实用功能。

## 标记系统概述

标记（Mark）是应用于文本内容的格式化和元数据信息，不同于节点（Node），标记不会改变文档结构，而是为内容添加视觉样式或附加信息。

### 核心结构

```rust
use serde::{Deserialize, Serialize};
use mf_model::mark::Mark;
use mf_model::attrs::Attrs;

/// 基础标记结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    pub r#type: String,    // 标记类型
    pub attrs: Attrs,      // 标记属性
}
```

## 内置标记类型

### 基础文本格式

```rust
// 粗体标记
let bold_mark = Mark {
    r#type: "bold".to_string(),
    attrs: HashMap::new(),
};

// 斜体标记
let italic_mark = Mark {
    r#type: "italic".to_string(),
    attrs: HashMap::new(),
};

// 下划线标记
let underline_mark = Mark {
    r#type: "underline".to_string(),
    attrs: HashMap::new(),
};

// 删除线标记
let strike_mark = Mark {
    r#type: "strike".to_string(),
    attrs: HashMap::new(),
};

// 代码标记
let code_mark = Mark {
    r#type: "code".to_string(),
    attrs: HashMap::new(),
};
```

### 链接标记

```rust
// 超链接标记
let link_mark = Mark {
    r#type: "link".to_string(),
    attrs: hashmap!{
        "href".to_string() => json!("https://example.com"),
        "title".to_string() => json!("示例链接"),
        "target".to_string() => json!("_blank"),
    },
};

// 内部引用标记
let ref_mark = Mark {
    r#type: "reference".to_string(),
    attrs: hashmap!{
        "ref_id".to_string() => json!("node_123"),
        "ref_type".to_string() => json!("internal"),
    },
};
```

## 自定义标记实现

### 背景颜色标记

创建支持多种颜色的背景高亮标记：

```rust
use mf_derive::Mark;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 背景颜色标记 - 用于高亮显示重要内容
#[derive(Mark, Debug, Clone, Serialize, Deserialize)]
#[mark_type = "background"]
#[desc = "背景颜色高亮"]
pub struct BackgroundMark {
    #[attr]
    pub color: String,      // 颜色值，如 "#FFE4B5" 或 "yellow"

    #[attr]
    pub opacity: f32,       // 透明度，0.0 到 1.0

    #[attr]
    pub style: String,      // 样式类型：solid, gradient, pattern
}

impl BackgroundMark {
    /// 创建预定义的高亮颜色
    pub fn highlight_yellow() -> Mark {
        Mark {
            r#type: "background".to_string(),
            attrs: hashmap!{
                "color".to_string() => json!("#FFFF00"),
                "opacity".to_string() => json!(0.3),
                "style".to_string() => json!("solid"),
            },
        }
    }

    pub fn highlight_green() -> Mark {
        Mark {
            r#type: "background".to_string(),
            attrs: hashmap!{
                "color".to_string() => json!("#90EE90"),
                "opacity".to_string() => json!(0.4),
                "style".to_string() => json!("solid"),
            },
        }
    }

    pub fn highlight_red() -> Mark {
        Mark {
            r#type: "background".to_string(),
            attrs: hashmap!{
                "color".to_string() => json!("#FFB6C1"),
                "opacity".to_string() => json!(0.3),
                "style".to_string() => json!("solid"),
            },
        }
    }

    /// 创建渐变背景
    pub fn gradient_background(from: &str, to: &str) -> Mark {
        Mark {
            r#type: "background".to_string(),
            attrs: hashmap!{
                "color".to_string() => json!(format!("{},{}", from, to)),
                "opacity".to_string() => json!(0.5),
                "style".to_string() => json!("gradient"),
            },
        }
    }
}
```

### 批注/备注标记

实现支持多用户协作的批注系统：

```rust
/// 批注标记 - 用于添加评论和备注
#[derive(Mark, Debug, Clone, Serialize, Deserialize)]
#[mark_type = "annotation"]
#[desc = "批注和备注"]
pub struct AnnotationMark {
    #[attr]
    pub id: String,             // 批注唯一ID

    #[attr]
    pub author: String,         // 批注作者

    #[attr]
    pub author_id: String,      // 作者ID

    #[attr]
    pub content: String,        // 批注内容

    #[attr]
    pub created_at: String,     // 创建时间

    #[attr]
    pub updated_at: Option<String>,  // 更新时间

    #[attr]
    pub resolved: bool,         // 是否已解决

    #[attr]
    pub priority: String,       // 优先级：low, medium, high, critical

    #[attr]
    pub category: String,       // 分类：comment, question, issue, suggestion

    #[attr]
    pub replies: Vec<String>,   // 回复ID列表
}

impl AnnotationMark {
    /// 创建新批注
    pub fn new(author: &str, content: &str, category: &str) -> Mark {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        Mark {
            r#type: "annotation".to_string(),
            attrs: hashmap!{
                "id".to_string() => json!(id),
                "author".to_string() => json!(author),
                "content".to_string() => json!(content),
                "created_at".to_string() => json!(now),
                "resolved".to_string() => json!(false),
                "priority".to_string() => json!("medium"),
                "category".to_string() => json!(category),
                "replies".to_string() => json!([]),
            },
        }
    }

    /// 创建问题标记
    pub fn question(author: &str, question: &str) -> Mark {
        Self::new(author, question, "question")
    }

    /// 创建建议标记
    pub fn suggestion(author: &str, suggestion: &str) -> Mark {
        Self::new(author, suggestion, "suggestion")
    }

    /// 创建问题报告
    pub fn issue(author: &str, issue: &str, priority: &str) -> Mark {
        let mut mark = Self::new(author, issue, "issue");
        if let Some(attrs) = mark.attrs.as_mut() {
            attrs.insert("priority".to_string(), json!(priority));
        }
        mark
    }
}
```

### 多媒体标记

支持视频、音频和图片的嵌入：

```rust
/// 多媒体标记 - 用于嵌入视频、音频等媒体内容
#[derive(Mark, Debug, Clone, Serialize, Deserialize)]
#[mark_type = "media"]
#[desc = "多媒体内容"]
pub struct MediaMark {
    #[attr]
    pub media_type: String,     // 媒体类型：video, audio, image

    #[attr]
    pub url: String,            // 媒体资源URL

    #[attr]
    pub title: Option<String>,  // 标题

    #[attr]
    pub alt: Option<String>,    // 替代文本

    #[attr]
    pub width: Option<i32>,     // 宽度

    #[attr]
    pub height: Option<i32>,    // 高度

    #[attr]
    pub duration: Option<f64>,  // 持续时间（秒）

    #[attr]
    pub thumbnail: Option<String>,  // 缩略图URL

    #[attr]
    pub autoplay: bool,         // 是否自动播放

    #[attr]
    pub loop_play: bool,        // 是否循环播放

    #[attr]
    pub controls: bool,         // 是否显示控制条

    #[attr]
    pub muted: bool,           // 是否静音
}

impl MediaMark {
    /// 创建视频标记
    pub fn video(url: &str, title: &str) -> Mark {
        Mark {
            r#type: "media".to_string(),
            attrs: hashmap!{
                "media_type".to_string() => json!("video"),
                "url".to_string() => json!(url),
                "title".to_string() => json!(title),
                "controls".to_string() => json!(true),
                "autoplay".to_string() => json!(false),
                "loop_play".to_string() => json!(false),
                "muted".to_string() => json!(false),
            },
        }
    }

    /// 创建音频标记
    pub fn audio(url: &str, title: &str) -> Mark {
        Mark {
            r#type: "media".to_string(),
            attrs: hashmap!{
                "media_type".to_string() => json!("audio"),
                "url".to_string() => json!(url),
                "title".to_string() => json!(title),
                "controls".to_string() => json!(true),
                "autoplay".to_string() => json!(false),
                "loop_play".to_string() => json!(false),
            },
        }
    }

    /// 创建图片标记
    pub fn image(url: &str, alt: &str, width: i32, height: i32) -> Mark {
        Mark {
            r#type: "media".to_string(),
            attrs: hashmap!{
                "media_type".to_string() => json!("image"),
                "url".to_string() => json!(url),
                "alt".to_string() => json!(alt),
                "width".to_string() => json!(width),
                "height".to_string() => json!(height),
            },
        }
    }
}
```

### 工程造价专用标记

为 price-rs 项目定制的标记类型：

```rust
/// 价格标记 - 用于标注价格信息
#[derive(Mark, Debug, Clone, Serialize, Deserialize)]
#[mark_type = "price"]
#[desc = "价格信息标记"]
pub struct PriceMark {
    #[attr]
    pub value: f64,             // 价格值

    #[attr]
    pub currency: String,       // 货币单位：CNY, USD, EUR

    #[attr]
    pub unit: Option<String>,   // 单位：m², m³, 个

    #[attr]
    pub price_type: String,     // 价格类型：unit_price, total_price, tax_included

    #[attr]
    pub tax_rate: Option<f64>,  // 税率

    #[attr]
    pub discount: Option<f64>,  // 折扣率

    #[attr]
    pub source: Option<String>, // 价格来源

    #[attr]
    pub valid_until: Option<String>, // 有效期至
}

/// 审核标记 - 用于标记审核状态
#[derive(Mark, Debug, Clone, Serialize, Deserialize)]
#[mark_type = "review"]
#[desc = "审核状态标记"]
pub struct ReviewMark {
    #[attr]
    pub status: String,         // 状态：pending, approved, rejected, revised

    #[attr]
    pub reviewer: String,       // 审核人

    #[attr]
    pub review_date: String,    // 审核日期

    #[attr]
    pub comment: Option<String>, // 审核意见

    #[attr]
    pub level: String,          // 审核级别：initial, final, expert
}
```

## 标记定义与注册

### 使用 MarkSpec 定义标记

```rust
use mf_model::mark_definition::{MarkSpec, MarkDefinition};
use mf_model::schema::AttributeSpec;

pub fn register_custom_marks() -> HashMap<String, MarkSpec> {
    let mut marks = HashMap::new();

    // 注册背景颜色标记
    marks.insert(
        "background".to_string(),
        MarkSpec {
            attrs: Some(hashmap!{
                "color" => AttributeSpec::required(AttrType::String),
                "opacity" => AttributeSpec::optional(AttrType::Number, 0.5),
                "style" => AttributeSpec::optional(AttrType::String, "solid"),
            }),
            excludes: Some("highlight".to_string()),  // 不能与高亮标记同时使用
            group: Some("formatting".to_string()),
            spanning: Some(true),  // 可以跨越多个节点
            desc: Some("背景颜色标记".to_string()),
        },
    );

    // 注册批注标记
    marks.insert(
        "annotation".to_string(),
        MarkSpec {
            attrs: Some(hashmap!{
                "id" => AttributeSpec::required(AttrType::String),
                "author" => AttributeSpec::required(AttrType::String),
                "content" => AttributeSpec::required(AttrType::String),
                "created_at" => AttributeSpec::required(AttrType::String),
                "resolved" => AttributeSpec::optional(AttrType::Boolean, false),
                "priority" => AttributeSpec::optional(AttrType::String, "medium"),
                "category" => AttributeSpec::required(AttrType::String),
            }),
            excludes: None,
            group: Some("annotation".to_string()),
            spanning: Some(true),
            desc: Some("批注和备注".to_string()),
        },
    );

    // 注册多媒体标记
    marks.insert(
        "media".to_string(),
        MarkSpec {
            attrs: Some(hashmap!{
                "media_type" => AttributeSpec::required(AttrType::String),
                "url" => AttributeSpec::required(AttrType::String),
                "title" => AttributeSpec::optional(AttrType::String),
                "controls" => AttributeSpec::optional(AttrType::Boolean, true),
            }),
            excludes: None,
            group: Some("media".to_string()),
            spanning: Some(false),  // 不跨越节点
            desc: Some("多媒体内容".to_string()),
        },
    );

    marks
}
```

## 应用标记到文档

### 使用 Transform 添加标记

```rust
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};
use mf_state::Transaction;

// 为文本添加背景色
pub async fn highlight_text(
    tr: &mut Transaction,
    from: usize,
    to: usize,
    color: &str,
) -> Result<()> {
    let mark = BackgroundMark::highlight_yellow();

    let step = AddMarkStep {
        from,
        to,
        mark,
    };

    tr.step(Arc::new(step))?;
    Ok(())
}

// 添加批注
pub async fn add_annotation(
    tr: &mut Transaction,
    from: usize,
    to: usize,
    author: &str,
    content: &str,
) -> Result<()> {
    let mark = AnnotationMark::new(author, content, "comment");

    let step = AddMarkStep {
        from,
        to,
        mark,
    };

    tr.step(Arc::new(step))?;
    Ok(())
}

// 嵌入视频
pub async fn embed_video(
    tr: &mut Transaction,
    position: usize,
    url: &str,
    title: &str,
) -> Result<()> {
    let mark = MediaMark::video(url, title);

    let step = AddMarkStep {
        from: position,
        to: position + 1,  // 占位符长度
        mark,
    };

    tr.step(Arc::new(step))?;
    Ok(())
}
```

### 查询和管理标记

```rust
/// 获取文档中的所有批注
pub fn get_annotations(doc: &Document) -> Vec<(usize, usize, AnnotationMark)> {
    let mut annotations = Vec::new();

    doc.descendants(|node| {
        if let Some(marks) = &node.marks {
            for mark in marks {
                if mark.r#type == "annotation" {
                    // 解析批注标记
                    let annotation = serde_json::from_value::<AnnotationMark>(
                        &serde_json::to_value(&mark.attrs).unwrap()
                    ).unwrap();

                    annotations.push((node.from, node.to, annotation));
                }
            }
        }
        true
    });

    annotations
}

/// 解决批注
pub async fn resolve_annotation(
    tr: &mut Transaction,
    annotation_id: &str,
) -> Result<()> {
    let doc = tr.doc();

    // 找到对应的批注标记
    doc.descendants(|node| {
        if let Some(marks) = &node.marks {
            for mark in marks {
                if mark.r#type == "annotation" {
                    if let Some(id) = mark.attrs.get("id") {
                        if id.as_str() == Some(annotation_id) {
                            // 更新批注状态
                            let mut updated_mark = mark.clone();
                            updated_mark.attrs.insert(
                                "resolved".to_string(),
                                json!(true),
                            );

                            // 替换标记
                            let remove_step = RemoveMarkStep {
                                from: node.from,
                                to: node.to,
                                mark: mark.clone(),
                            };

                            let add_step = AddMarkStep {
                                from: node.from,
                                to: node.to,
                                mark: updated_mark,
                            };

                            tr.step(Arc::new(remove_step))?;
                            tr.step(Arc::new(add_step))?;

                            return false;  // 停止遍历
                        }
                    }
                }
            }
        }
        true
    });

    Ok(())
}
```

## 渲染标记

### HTML 渲染器示例

```rust
pub struct MarkRenderer;

impl MarkRenderer {
    /// 将标记渲染为 HTML
    pub fn render_mark(mark: &Mark, content: &str) -> String {
        match mark.r#type.as_str() {
            "bold" => format!("<strong>{}</strong>", content),

            "italic" => format!("<em>{}</em>", content),

            "background" => {
                let color = mark.attrs.get("color")
                    .and_then(|v| v.as_str())
                    .unwrap_or("#FFFF00");
                let opacity = mark.attrs.get("opacity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);

                format!(
                    r#"<span style="background-color: {}; opacity: {}">{}</span>"#,
                    color, opacity, content
                )
            }

            "annotation" => {
                let id = mark.attrs.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let author = mark.attrs.get("author")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Anonymous");
                let comment = mark.attrs.get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                format!(
                    r#"<span class="annotation" data-id="{}" data-author="{}" title="{}">{}</span>"#,
                    id, author, comment, content
                )
            }

            "media" => {
                let media_type = mark.attrs.get("media_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("video");
                let url = mark.attrs.get("url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                match media_type {
                    "video" => format!(
                        r#"<video src="{}" controls></video>"#,
                        url
                    ),
                    "audio" => format!(
                        r#"<audio src="{}" controls></audio>"#,
                        url
                    ),
                    "image" => format!(
                        r#"<img src="{}" alt="{}" />"#,
                        url, content
                    ),
                    _ => content.to_string(),
                }
            }

            _ => content.to_string(),
        }
    }

    /// 渲染多个标记
    pub fn render_marks(marks: &[Mark], content: &str) -> String {
        marks.iter().fold(content.to_string(), |acc, mark| {
            Self::render_mark(mark, &acc)
        })
    }
}
```

## 标记组合与冲突处理

### 标记组规则

```rust
/// 定义标记组和互斥规则
pub struct MarkRules;

impl MarkRules {
    /// 检查两个标记是否可以组合
    pub fn can_combine(mark1: &Mark, mark2: &Mark) -> bool {
        match (mark1.r#type.as_str(), mark2.r#type.as_str()) {
            // 背景色和高亮不能同时存在
            ("background", "highlight") | ("highlight", "background") => false,

            // 多个批注可以重叠
            ("annotation", "annotation") => true,

            // 媒体标记不能与其他媒体标记重叠
            ("media", "media") => false,

            // 默认允许组合
            _ => true,
        }
    }

    /// 合并相同类型的相邻标记
    pub fn merge_marks(marks: Vec<Mark>) -> Vec<Mark> {
        let mut merged = Vec::new();
        let mut current: Option<Mark> = None;

        for mark in marks {
            if let Some(ref mut curr) = current {
                if curr.r#type == mark.r#type && curr.attrs == mark.attrs {
                    // 相同标记，继续累积
                    continue;
                } else {
                    // 不同标记，保存当前并开始新的
                    merged.push(curr.clone());
                    current = Some(mark);
                }
            } else {
                current = Some(mark);
            }
        }

        if let Some(curr) = current {
            merged.push(curr);
        }

        merged
    }
}
```

## 性能优化

### 标记缓存

```rust
use lru::LruCache;
use std::sync::Arc;

/// 标记缓存系统
pub struct MarkCache {
    cache: LruCache<String, Arc<Vec<Mark>>>,
}

impl MarkCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(capacity),
        }
    }

    /// 缓存节点的标记
    pub fn cache_marks(&mut self, node_id: &str, marks: Vec<Mark>) {
        self.cache.put(node_id.to_string(), Arc::new(marks));
    }

    /// 获取缓存的标记
    pub fn get_marks(&mut self, node_id: &str) -> Option<Arc<Vec<Mark>>> {
        self.cache.get(node_id).cloned()
    }
}
```

## 最佳实践

### 1. 标记设计原则

- **轻量化**：标记应该只包含必要的属性
- **可组合**：设计标记时考虑与其他标记的组合
- **语义化**：标记类型名称应该清晰表达其用途
- **可扩展**：预留扩展字段以适应未来需求

### 2. 性能考虑

- 避免在大量文本上频繁添加/删除标记
- 使用批量操作处理多个标记变更
- 对频繁访问的标记使用缓存
- 合并相邻的相同标记以减少开销

### 3. 用户体验

- 为标记提供清晰的视觉反馈
- 支持键盘快捷键快速应用常用标记
- 提供标记的撤销/重做功能
- 实现标记的导出/导入功能

## 总结

ModuForge-RS 的标记系统提供了：

1. **灵活的定义**：通过 derive 宏快速定义自定义标记
2. **丰富的类型**：支持格式、批注、多媒体等多种标记类型
3. **强大的组合**：标记可以灵活组合和嵌套
4. **高效的管理**：提供查询、更新、缓存等完整功能
5. **实际应用**：在 price-rs 项目中验证了标记系统的实用性

通过本章的学习，您可以为自己的项目创建各种自定义标记，实现丰富的文本格式化和元数据管理功能。

下一章：[命令系统](./commands.md)
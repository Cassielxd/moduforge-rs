//! 完整的 mf_extension 宏使用示例
//!
//! 此示例展示如何使用 mf_extension 宏创建一个功能完整的编辑器扩展，
//! 包含操作函数、节点定义、标记定义和节点转换功能。

use mf_macro::{mf_extension, mf_op, mf_global_attr, node, mark};
use mf_core::{ForgeResult, node::Node, types::Extensions};
use mf_state::ops::GlobalResourceManager;
use serde_json::Value;

// ==================== 操作函数定义 ====================

// 初始化编辑器工具栏
mf_op!(init_toolbar, |manager| {
    println!("初始化编辑器工具栏");
    // 这里可以设置工具栏相关资源
    // manager.set_resource("toolbar_config", toolbar_config);
    Ok(())
});

// 注册编辑器快捷键
mf_op!(register_shortcuts, {
    println!("注册编辑器快捷键");
    println!("  Ctrl+B: 粗体");
    println!("  Ctrl+I: 斜体");
    println!("  Ctrl+U: 下划线");
    println!("  Ctrl+K: 插入链接");
    Ok(())
});

/// 初始化拼写检查
fn init_spellcheck(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("初始化拼写检查服务");
    // 加载词典
    // let dictionary = load_dictionary()?;
    // _manager.set_resource("spellcheck_dict", dictionary);
    Ok(())
}

/// 设置自动保存
fn setup_autosave(_manager: &GlobalResourceManager) -> ForgeResult<()> {
    println!("配置自动保存");
    println!("  自动保存间隔: 30秒");
    println!("  保存位置: 本地存储");
    Ok(())
}

// ==================== 节点转换函数 ====================

/// 节点转换函数：处理特殊节点类型
fn transform_nodes(node: &mut Node) -> ForgeResult<()> {
    match node.name.as_str() {
        "paragraph" => {
            // 空段落显示占位符 - 检查 content 属性
            if node.r#type.content.as_ref().map_or(true, |c| c.is_empty()) {
                node.set_attr(
                    "placeholder",
                    Some(Value::String("输入文本...".to_string())),
                );
            }
        },
        "code_block" => {
            // 代码块添加默认语言
            let needs_language = node
                .r#type
                .attrs
                .as_ref()
                .and_then(|attrs| attrs.get("language"))
                .and_then(|spec| spec.default.as_ref())
                .is_none();

            if needs_language {
                node.set_attr(
                    "language",
                    Some(Value::String("plaintext".to_string())),
                );
            }
            // 添加行号支持
            node.set_attr(
                "line_numbers",
                Some(Value::String("true".to_string())),
            );
        },
        "image" => {
            // 图片节点添加懒加载
            node.set_attr("loading", Some(Value::String("lazy".to_string())));

            let needs_alt = node
                .r#type
                .attrs
                .as_ref()
                .and_then(|attrs| attrs.get("alt"))
                .and_then(|spec| spec.default.as_ref())
                .is_none();

            if needs_alt {
                node.set_attr("alt", Some(Value::String("图片".to_string())));
            }
        },
        _ => {},
    }
    Ok(())
}

// ==================== 创建编辑器扩展 ====================

mf_extension!(
    rich_text_editor,
    // 操作函数列表
    ops = [init_toolbar, register_shortcuts, init_spellcheck, setup_autosave],
    // 全局属性配置
    global_attributes = [
        // 编辑器配置
        mf_global_attr!(
            vec!["editor"],
            vec![
                (
                    "theme",
                    AttributeSpec {
                        default: Some(Value::String("light".to_string()))
                    }
                ),
                (
                    "font_size",
                    AttributeSpec { default: Some(Value::Number(14.into())) }
                ),
                (
                    "line_height",
                    AttributeSpec {
                        default: Some(Value::String("1.5".to_string()))
                    }
                )
            ]
        ),
        // 工具栏配置
        mf_global_attr!("toolbar", "visible", "true"),
        mf_global_attr!("toolbar", "position", "top")
    ],
    // 节点转换函数
    node_transform = transform_nodes,
    // 节点定义
    nodes = [
        // 基础文本节点
        node!("paragraph", "段落节点"),
        node!("heading", "标题节点", "", "level" => "1"),
        node!("text", "纯文本节点", ""),
        // 列表节点
        node!("bullet_list", "无序列表"),
        node!("ordered_list", "有序列表"),
        node!("list_item", "列表项"),
        node!("task_list", "任务列表"),
        node!("task_item", "任务项", "", "checked" => "false"),
        // 块级节点
        node!("blockquote", "引用块"),
        node!("code_block", "代码块", "", "language" => "plaintext", "line_numbers" => "false"),
        node!("horizontal_rule", "水平分割线"),
        // 媒体节点
        node!("image", "图片", "", "src" => "", "alt" => "", "width" => "", "height" => ""),
        node!("video", "视频", "", "src" => "", "poster" => "", "controls" => "true"),
        node!("audio", "音频", "", "src" => "", "controls" => "true"),
        // 表格节点
        node!("table", "表格"),
        node!("table_row", "表格行"),
        node!("table_cell", "表格单元格", "", "colspan" => "1", "rowspan" => "1"),
        node!("table_header", "表格头", "", "colspan" => "1", "rowspan" => "1"),
        // 特殊节点
        node!("math_inline", "内联数学公式", ""),
        node!("math_block", "块级数学公式", ""),
        node!("footnote", "脚注", "", "id" => ""),
        node!("toc", "目录", "", "max_depth" => "3")
    ],
    // 标记定义
    marks = [
        // 基础文本标记
        mark!("bold", "粗体"),
        mark!("italic", "斜体"),
        mark!("underline", "下划线"),
        mark!("strike", "删除线"),
        mark!("code", "内联代码"),
        // 高级标记
        mark!("link", "超链接", "href" => "", "title" => "", "target" => "_blank"),
        mark!("highlight", "高亮", "color" => "yellow"),
        mark!("comment", "批注", "author" => "", "timestamp" => ""),
        // 语义标记
        mark!("em", "强调"),
        mark!("strong", "重要"),
        mark!("mark", "标记"),
        mark!("abbr", "缩写", "title" => ""),
        mark!("cite", "引用"),
        // 格式标记
        mark!("subscript", "下标"),
        mark!("superscript", "上标"),
        mark!("small", "小字"),
        mark!("kbd", "键盘按键"),
        mark!("var", "变量"),
        // 自定义标记
        mark!("tooltip", "工具提示", "content" => "", "position" => "top"),
        mark!("spoiler", "剧透遮罩", "revealed" => "false"),
        mark!("emoji", "表情符号", "name" => "", "unicode" => "")
    ],
    docs = "功能完整的富文本编辑器扩展，提供丰富的节点类型和标记类型，支持现代编辑器的所有基础功能"
);

// ==================== 使用示例 ====================

fn main() -> ForgeResult<()> {
    println!("=== 富文本编辑器扩展示例 ===\n");

    // 初始化扩展
    let extensions = rich_text_editor::init();

    // 统计各类型扩展数量
    let mut extension_count = 0;
    let mut node_count = 0;
    let mut mark_count = 0;

    for ext in &extensions {
        match ext {
            Extensions::E(_) => extension_count += 1,
            Extensions::N(_) => node_count += 1,
            Extensions::M(_) => mark_count += 1,
        }
    }

    println!("扩展初始化完成！");
    println!("  Extension 数量: {}", extension_count);
    println!("  Node 定义数量: {}", node_count);
    println!("  Mark 定义数量: {}", mark_count);
    println!("  总计: {} 个组件\n", extensions.len());

    // 打印所有节点
    println!("已注册的节点类型:");
    for ext in &extensions {
        if let Extensions::N(node) = ext {
            let desc = node.r#type.desc.as_deref().unwrap_or("无描述");
            println!("  - {}: {}", node.name, desc);
        }
    }

    println!("\n已注册的标记类型:");
    for ext in &extensions {
        if let Extensions::M(mark) = ext {
            let desc = mark.r#type.desc.as_deref().unwrap_or("无描述");
            println!("  - {}: {}", mark.name, desc);
        }
    }

    // 演示节点转换
    println!("\n=== 节点转换演示 ===");

    let mut test_paragraph = node!("paragraph", "测试段落", "");
    let placeholder_before = test_paragraph
        .r#type
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("placeholder"))
        .and_then(|spec| spec.default.as_ref());
    println!("空段落转换前 placeholder: {:?}", placeholder_before);

    transform_nodes(&mut test_paragraph).unwrap();

    let placeholder_after = test_paragraph
        .r#type
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("placeholder"))
        .and_then(|spec| spec.default.as_ref());
    println!("空段落转换后 placeholder: {:?}", placeholder_after);

    let mut test_code_block =
        node!("code_block", "代码块", "console.log('Hello');");
    let lang_before = test_code_block
        .r#type
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("language"))
        .and_then(|spec| spec.default.as_ref());
    let line_nums_before = test_code_block
        .r#type
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("line_numbers"))
        .and_then(|spec| spec.default.as_ref());

    println!(
        "\n代码块转换前 - 语言: {:?}, 行号: {:?}",
        lang_before, line_nums_before
    );

    transform_nodes(&mut test_code_block).unwrap();

    let lang_after = test_code_block
        .r#type
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("language"))
        .and_then(|spec| spec.default.as_ref());
    let line_nums_after = test_code_block
        .r#type
        .attrs
        .as_ref()
        .and_then(|attrs| attrs.get("line_numbers"))
        .and_then(|spec| spec.default.as_ref());

    println!(
        "代码块转换后 - 语言: {:?}, 行号: {:?}",
        lang_after, line_nums_after
    );

    println!("\n=== 扩展使用场景 ===");
    println!("1. 可以将这些扩展注册到编辑器框架");
    println!("2. 节点定义用于构建文档树结构");
    println!("3. 标记定义用于文本格式化");
    println!("4. 操作函数在初始化时自动执行");
    println!("5. 节点转换函数可以预处理节点");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_init() {
        let extensions = rich_text_editor::init();

        // 验证至少有一个 Extension
        let has_extension =
            extensions.iter().any(|e| matches!(e, Extensions::E(_)));
        assert!(has_extension, "应该至少有一个 Extension");

        // 验证有节点定义
        let node_count =
            extensions.iter().filter(|e| matches!(e, Extensions::N(_))).count();
        assert!(node_count > 0, "应该有节点定义");

        // 验证有标记定义
        let mark_count =
            extensions.iter().filter(|e| matches!(e, Extensions::M(_))).count();
        assert!(mark_count > 0, "应该有标记定义");
    }

    #[test]
    fn test_node_transform() {
        // 测试空段落转换
        let mut empty_para = node!("paragraph", "段落", "");
        transform_nodes(&mut empty_para).unwrap();
        let placeholder = empty_para
            .r#type
            .attrs
            .as_ref()
            .and_then(|attrs| attrs.get("placeholder"))
            .and_then(|spec| spec.default.as_ref())
            .and_then(|v| v.as_str());
        assert_eq!(placeholder, Some("输入文本..."));

        // 测试代码块转换
        let mut code_block = node!("code_block", "代码", "let x = 1;");
        transform_nodes(&mut code_block).unwrap();
        let language = code_block
            .r#type
            .attrs
            .as_ref()
            .and_then(|attrs| attrs.get("language"))
            .and_then(|spec| spec.default.as_ref())
            .and_then(|v| v.as_str());
        let line_numbers = code_block
            .r#type
            .attrs
            .as_ref()
            .and_then(|attrs| attrs.get("line_numbers"))
            .and_then(|spec| spec.default.as_ref())
            .and_then(|v| v.as_str());
        assert_eq!(language, Some("plaintext"));
        assert_eq!(line_numbers, Some("true"));
    }

    #[test]
    fn test_extensions_enumeration() {
        let extensions = rich_text_editor::init();

        for ext in extensions {
            match ext {
                Extensions::E(e) => {
                    // Extension 应该已经配置了操作函数和属性
                    println!("Extension 已配置");
                },
                Extensions::N(n) => {
                    // 验证节点有名称
                    assert!(!n.name.is_empty());
                },
                Extensions::M(m) => {
                    // 验证标记有名称
                    assert!(!m.name.is_empty());
                },
            }
        }
    }
}

use std::collections::HashMap;

use moduforge_model::schema::{AttributeSpec, Schema, SchemaSpec};

use crate::{
    types::{Extensions, GlobalAttributeItem},
    ForgeResult,
};

/// 根据已解析的扩展列表生成完整的 Schema
/// 
/// # 参数
/// * `extensions` - 扩展列表，包含节点(N)、标记(M)和扩展(E)三种类型
/// 
/// # 返回值
/// * `EditorResult<Schema>` - 返回编译后的 Schema 或错误
/// 
/// # 功能说明
/// 1. 收集所有扩展中定义的全局属性
/// 2. 处理节点扩展，构建节点定义
/// 3. 处理标记扩展，构建标记定义
/// 4. 确定顶层节点名称
/// 5. 编译生成最终的 Schema
pub fn get_schema_by_resolved_extensions(
    extensions: &Vec<Extensions>
) -> ForgeResult<Schema> {
    // 收集所有扩展中定义的全局属性
    let mut extension_attributes = vec![];
    for extension in extensions {
        if let Extensions::E(extension) = extension {
            for item in extension.get_global_attributes().iter() {
                extension_attributes.push(item);
            }
        }
    }

    // 初始化节点和标记的存储
    let mut nodes = HashMap::new();
    let mut marks = HashMap::new();
    let mut top_name = "doc".to_string();

    // 处理每个扩展
    for extension in extensions {
        match extension {
            // 处理节点扩展
            Extensions::N(node) => {
                let name = node.name.clone();
                // 检查是否为顶层节点
                if node.is_top_node() {
                    top_name = node.name.clone();
                }
                // 获取节点的属性定义
                let mut attrs = get_attr_dfn(name, &extension_attributes);

                // 合并节点类型中定义的属性
                let attrs_def = match &node.r#type.attrs {
                    Some(m) => {
                        m.iter().for_each(|e| {
                            attrs.insert(e.0.clone(), e.1.clone());
                        });
                        attrs
                    },
                    None => attrs,
                };
                let mut t = node.r#type.clone();
                t.attrs = Some(attrs_def);
                nodes.insert(node.name.clone(), t);
            },
            // 处理标记扩展
            Extensions::M(mark) => {
                marks.insert(mark.name.clone(), mark.r#type.clone());
            },
            _ => {},
        }
    }

    // 创建 Schema 规范并编译
    let instance_spec = SchemaSpec { nodes, marks, top_node: Some(top_name) };
    let schema = Schema::compile(instance_spec)?;
    Ok(schema)
}

/// 获取指定节点名称的属性定义
/// 
/// # 参数
/// * `name` - 节点名称
/// * `extension_attributes` - 全局属性列表
/// 
/// # 返回值
/// * `HashMap<String, AttributeSpec>` - 节点对应的属性定义映射
fn get_attr_dfn(
    name: String,
    extension_attributes: &Vec<&GlobalAttributeItem>,
) -> HashMap<String, AttributeSpec> {
    let mut attributes: HashMap<String, AttributeSpec> = HashMap::new();
    // 遍历全局属性，找出适用于当前节点的属性
    for attr in extension_attributes.iter() {
        if attr.types.contains(&name) {
            attr.attributes.iter().for_each(|e| {
                attributes.insert(e.0.clone(), e.1.clone());
            });
        }
    }
    attributes
}

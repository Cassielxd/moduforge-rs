use super::attrs::Attrs;
use super::content::ContentMatch;
use super::mark_type::{MarkSpec, MarkType};
use super::node_type::{NodeSpec, NodeType};
use im::HashMap as ImHashMap;
use serde::Serialize;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
/// 属性定义结构体
/// 用于定义节点或标记的属性特征
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct Attribute {
    pub has_default: bool,
    pub default: Option<Value>,
}

impl Attribute {
    /// 从 AttributeSpec 创建新的 Attribute 实例
    pub(crate) fn new(options: AttributeSpec) -> Self {
        Attribute {
            has_default: options.default.is_some(),
            default: options.default,
        }
    }
    /// 检查属性是否为必需的
    /// 如果没有默认值，则属性为必需
    pub fn is_required(&self) -> bool {
        !self.has_default
    }
}
/// Schema 结构体定义
/// 用于管理文档模型的整体结构，包括节点和标记的类型定义
#[derive(Clone, Debug)]
pub struct Schema {
    /// Schema 的规范定义
    pub spec: SchemaSpec,
    /// 顶级节点类型
    pub top_node_type: Option<NodeType>,
    /// 全局缓存
    pub cached: Arc<Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    /// 节点类型映射表
    pub nodes: HashMap<String, NodeType>,
    /// 标记类型映射表
    pub marks: HashMap<String, MarkType>,
}
impl PartialEq for Schema {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.spec == other.spec
            && self.top_node_type == other.top_node_type
            && self.nodes == other.nodes
            && self.marks == other.marks
    }
}
impl Eq for Schema {}
impl Schema {
    /// 创建新的 Schema 实例
    pub fn new(spec: SchemaSpec) -> Self {
        let mut instance_spec = SchemaSpec {
            nodes: HashMap::new(),
            marks: HashMap::new(),
            top_node: spec.top_node,
        };
        // 复制 spec 属性
        for (key, value) in spec.nodes {
            instance_spec.nodes.insert(key, value);
        }
        for (key, value) in spec.marks {
            instance_spec.marks.insert(key, value);
        }
        Schema {
            spec: instance_spec,
            top_node_type: None,
            cached: Arc::new(Mutex::new(HashMap::new())),
            nodes: HashMap::new(),
            marks: HashMap::new(),
        }
    }
    /// 编译 Schema 定义
    /// 处理节点和标记的定义，建立它们之间的关系
    pub fn compile(
        instance_spec: SchemaSpec
    ) -> Result<Schema, Box<dyn Error>> {
        let mut schema: Schema = Schema::new(instance_spec);
        let nodes: HashMap<String, NodeType> =
            NodeType::compile(schema.spec.nodes.clone());
        let marks = MarkType::compile(schema.spec.marks.clone());
        let mut content_expr_cache = HashMap::new();
        let mut updated_nodes = HashMap::new();
        for (prop, type_) in &nodes {
            if marks.contains_key(prop) {
                return Err(format!("{} 不能既是节点又是标记", prop).into());
            }

            let content_expr = type_.spec.content.as_deref().unwrap_or("");
            let mark_expr = type_.spec.marks.as_deref();

            let content_match = content_expr_cache
                .entry(content_expr.to_string())
                .or_insert_with(|| {
                    ContentMatch::parse(content_expr.to_string(), &nodes)
                })
                .clone();

            let mark_set = match mark_expr {
                Some("_") => None,
                Some(expr) => {
                    let marks_result = gather_marks(
                        &schema,
                        expr.split_whitespace().collect(),
                    );
                    match marks_result {
                        Ok(marks) => Some(marks.into_iter().cloned().collect()), // Convert Vec<&MarkType> to Vec<MarkType>
                        Err(e) => return Err(e.into()),
                    }
                },
                None => None,
            };

            let mut node = type_.clone();
            node.content_match = Some(content_match);
            node.mark_set = mark_set;
            updated_nodes.insert(prop.clone(), node);
        }
        schema.nodes = updated_nodes;
        schema.marks = marks;
        schema.top_node_type = schema
            .nodes
            .get(
                &schema
                    .spec
                    .top_node
                    .clone()
                    .unwrap_or_else(|| "doc".to_string()),
            )
            .cloned();

        Ok(schema)
    }
}
/// Schema 规范定义
/// 包含节点和标记的原始定义信息
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SchemaSpec {
    pub nodes: HashMap<String, NodeSpec>,
    pub marks: HashMap<String, MarkSpec>,
    pub top_node: Option<String>,
}

// 其他辅助函数...
/// 获取属性的默认值映射
/// 如果所有属性都有默认值，返回包含所有默认值的映射
/// 如果任一属性没有默认值，返回 None
pub fn default_attrs(
    attrs: &HashMap<String, Attribute>
) -> Option<HashMap<String, Value>> {
    let mut defaults = HashMap::new();

    for (attr_name, attr) in attrs {
        if let Some(default) = &attr.default {
            defaults.insert(attr_name.clone(), default.clone());
        } else {
            return None;
        }
    }

    Some(defaults)
}
/// 属性规范定义
#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize)]
pub struct AttributeSpec {
    /// 属性的默认值
    pub default: Option<Value>,
}
/// 收集标记类型
/// 根据给定的标记名称列表，收集对应的标记类型
fn gather_marks<'a>(
    schema: &'a Schema,
    marks: Vec<&'a str>,
) -> Result<Vec<&'a MarkType>, String> {
    let mut found = Vec::new();

    for name in marks {
        if let Some(mark) = schema.marks.get(name) {
            found.push(mark);
        } else {
            let mut ok = None;
            for mark_ref in schema.marks.values() {
                if name == "_"
                    || mark_ref.spec.group.as_ref().is_some_and(|group| {
                        group.split_whitespace().any(|g| g == name)
                    })
                {
                    found.push(mark_ref);
                    ok = Some(mark_ref);
                    break;
                }
            }
            if ok.is_none() {
                return Err(format!("未知的标记类型: '{}'", name));
            }
        }
    }
    Ok(found)
}
/// 计算属性值
/// 根据属性定义和提供的值计算最终的属性值
pub fn compute_attrs(
    attrs: &HashMap<String, Attribute>,
    value: Option<&HashMap<String, Value>>,
) -> Attrs {
    let mut built = ImHashMap::new();

    for (name, attr) in attrs {
        let given = value.and_then(|v| v.get(name));

        let given = match given {
            Some(val) => val.clone(),
            None => {
                if attr.has_default {
                    attr.default.clone().unwrap_or_else(|| {
                        panic!("没有为属性提供默认值 {}", name)
                    })
                } else {
                    Value::Null
                }
            },
        };

        built.insert(name.clone(), given);
    }

    built
}

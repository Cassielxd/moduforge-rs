use super::attrs::Attrs;
use super::content::ContentMatch;
use super::mark_type::{MarkSpec, MarkType};
use super::node_type::{NodeSpec, NodeType};
use im::HashMap as ImHashMap;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct Attribute {
    pub has_default: bool,
    pub default: Option<String>,
}

impl Attribute {
    pub(crate) fn new(options: AttributeSpec) -> Self {
        Attribute {
            has_default: options.default.is_some(),
            default: options.default,
        }
    }

    pub fn is_required(&self) -> bool {
        !self.has_default
    }
}
/**
 * Schema 定义,包含 节点类型定义 和 标记类型定义 fragment工厂 和顶级节点
 * @property nodes 节点定义
 * @property marks 标记定义
 * @property topNode 顶级节点名称
 * @property cached 全局缓存
 * @author string<348040933@qq.com>
 */
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Schema {
    pub spec: SchemaSpec,
    pub top_node_type: Option<NodeType>,
    pub cached: HashMap<String, String>,
    pub nodes: HashMap<String, NodeType>,
    pub marks: HashMap<String, MarkType>,
}

impl Schema {
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
            cached: HashMap::new(),
            nodes: HashMap::new(),
            marks: HashMap::new(),
        }
    }

    pub fn compile(instance_spec: SchemaSpec) -> Result<Schema, Box<dyn Error>> {
        let mut schema: Schema = Schema::new(instance_spec);
        let nodes: HashMap<String, NodeType> = NodeType::compile(schema.spec.nodes.clone());
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
                .or_insert_with(|| ContentMatch::parse(content_expr.to_string(), &nodes))
                .clone();

            let mark_set = match mark_expr {
                Some("_") => None,
                Some(expr) => {
                    let marks_result = gather_marks(&schema, expr.split_whitespace().collect());
                    match marks_result {
                        Ok(marks) => Some(marks.into_iter().cloned().collect()), // Convert Vec<&MarkType> to Vec<MarkType>
                        Err(e) => return Err(e.into()),
                    }
                }
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SchemaSpec {
    pub nodes: HashMap<String, NodeSpec>,
    pub marks: HashMap<String, MarkSpec>,
    pub top_node: Option<String>,
}

// 其他辅助函数...

pub fn default_attrs(attrs: &HashMap<String, Attribute>) -> Option<HashMap<String, String>> {
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

#[derive(Clone, PartialEq, Debug, Eq, Hash, Serialize)]
pub struct AttributeSpec {
    pub default: Option<String>,
}

fn gather_marks<'a>(schema: &'a Schema, marks: Vec<&'a str>) -> Result<Vec<&'a MarkType>, String> {
    let mut found = Vec::new();

    for name in marks {
        if let Some(mark) = schema.marks.get(name) {
            found.push(mark);
        } else {
            let mut ok = None;
            for mark_ref in schema.marks.values() {
                if name == "_"
                    || mark_ref
                        .spec
                        .group
                        .as_ref()
                        .map_or(false, |group| group.split_whitespace().any(|g| g == name))
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

pub fn compute_attrs(
    attrs: &HashMap<String, Attribute>,
    value: Option<&HashMap<String, String>>,
) -> Attrs {
    let mut built = ImHashMap::new();

    for (name, attr) in attrs {
        let given = value.and_then(|v| v.get(name));

        let given = match given {
            Some(val) => val.clone(),
            None => {
                if attr.has_default {
                    attr.default.clone().unwrap_or_else(|| "".to_string())
                } else {
                    panic!("没有为属性提供值 {}", name);
                }
            }
        };

        built.insert(name.clone(), given);
    }

    built
}

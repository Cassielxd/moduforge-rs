use super::attrs::Attrs;
use super::content::ContentMatch;
use super::id_generator::IdGenerator;
use super::mark::Mark;
use super::mark_type::MarkType;
use super::node::Node;
use super::schema::{compute_attrs, Attribute, AttributeSpec, Schema};
use super::types::NodeId;
use im::HashMap as ImHashMap;
use serde_json::Value;
use std::collections::HashMap;

/**
 * 节点的类型定义
 * @property name 节点类型名称
 * @property schema 架构
 * @property spec 节点类型定义
 * @property desc 描述
 * @property contentMatch 内容匹配
 * @property defaultContentType 默认内容类型
 * @author string<348040933@qq.com>
 */

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NodeType {
    pub name: String,
    pub spec: NodeSpec,
    pub desc: String,
    pub groups: Vec<String>,
    pub attrs: HashMap<String, Attribute>,
    pub default_attrs: HashMap<String, String>,
    pub content_match: Option<ContentMatch>,
    pub mark_set: Option<Vec<MarkType>>,
}

impl NodeType {
    pub fn compile(nodes: HashMap<String, NodeSpec>) -> HashMap<String, NodeType> {
        let mut result = HashMap::new();
        for (name, spec) in nodes {
            result.insert(name.clone(), NodeType::new(name.clone(), spec.clone()));
        }
        result
    }

    pub fn new(name: String, spec: NodeSpec) -> Self {
        let attrs = spec.attrs.as_ref().map_or_else(HashMap::new, |attrs| {
            attrs
                .iter()
                .map(|(name, spec)| (name.clone(), Attribute::new(spec.clone())))
                .collect()
        });

        let default_attrs = attrs
            .iter()
            .filter_map(|(name, attr)| {
                if attr.has_default {
                    Some((name.clone(), attr.default.clone().unwrap()))
                } else {
                    None
                }
            })
            .collect();

        NodeType {
            name,
            spec,
            desc: "".to_string(),
            groups: vec![],
            attrs,
            default_attrs,
            content_match: None,
            mark_set: None,
        }
    }

    pub fn check_attrs(&self, values: Attrs) {
        for (key, _value) in &values {
            if !self.attrs.contains_key(key) {
                panic!("节点 {} 属性 {}没有定义", self.name, key);
            }
        }
        for (key, value) in &self.attrs {
            if value.is_required() && !&values.contains_key(key) {
                panic!("节点 {} 属性 {} 没有值，这个属性必填", self.name, key);
            }
        }
    }

    pub fn has_required_attrs(&self) -> bool {
        self.attrs
            .values()
            .any(|attr: &Attribute| attr.is_required())
    }

    pub fn create_and_fill(
        &self,
        id: Option<String>,
        attrs: Option<&HashMap<String, String>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
        schema: &Schema,
    ) -> Vec<Node> {
        let id: String = id.unwrap_or_else(|| IdGenerator::get_id());
        let attrs = self.compute_attrs(attrs);
        let mut content_ref = content
            .iter()
            .map(|i| i.id.clone())
            .collect::<Vec<NodeId>>();
        let binding = &self.content_match.clone();
        let matched = match binding {
            Some(bind) => bind.match_fragment(&content, schema),
            None => None,
        };
        let mut nodes = vec![];
        let mut after = matched.and_then(|m| m.fill(&content, true, schema));
        if let Some(after_content) = &mut after {
            let mut ids: Vec<NodeId> = after_content.iter().map(|i| i.id.clone()).collect();
            content_ref.append(&mut ids);
            nodes.append(after_content);
        }
        let node = Node::new(
            &id,
            self.name.clone(),
            attrs,
            content_ref,
            Mark::set_from(marks),
        );
        nodes.push(node);
        nodes
    }

    pub fn create(
        &self,
        id: Option<String>,
        attrs: Option<&HashMap<String, String>>,
        content: Vec<NodeId>,
        marks: Option<Vec<Mark>>,
    ) -> Node {
        // 实现...
        let id: String = id.unwrap_or_else(|| {
            let mut id_generator: std::sync::MutexGuard<'_, IdGenerator> =
                IdGenerator::get_instance().lock().unwrap();
            id_generator.get_next_id()
        });
        Node::new(
            &id,
            self.name.clone(),
            self.compute_attrs(attrs),
            content,
            Mark::set_from(marks),
        )
    }
    fn compute_attrs(&self, attrs: Option<&HashMap<String, String>>) -> ImHashMap<String, String> {
        match attrs {
            Some(attr) => compute_attrs(&self.attrs, Some(&attr)),
            None => compute_attrs(&self.attrs, Some(&self.default_attrs)),
        }
    }
    // 其他方法...
}

/**
 * 节点的类型约束
 * @interface NodeSpec
 * @property {string} [content] 内容类型
 * @property {string} [marks] mark类型
 * @property {string} [group] 节点类型分组
 * @property {object} [attrs] 属性定义
 * @property {function} [render] 渲染函数
 * @property {any} [other] 其他属性
 * @author string<348040933@qq.com>
 */
#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct NodeSpec {
    pub content: Option<String>,
    pub marks: Option<String>,
    pub group: Option<String>,
    pub desc: Option<String>,
    pub attrs: Option<HashMap<String, AttributeSpec>>,
}

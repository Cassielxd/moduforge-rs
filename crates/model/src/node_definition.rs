use super::attrs::Attrs;
use super::content::ContentMatch;
use super::id_generator::IdGenerator;
use super::mark::Mark;
use super::mark_definition::MarkDefinition;
use super::node::Node;
use super::schema::{compute_attrs, Attribute, AttributeSpec, Schema};
use super::types::NodeId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{self, Debug};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeTree(pub Node, pub Vec<NodeTree>);

impl NodeTree {
    pub fn into_parts(self) -> (Node, Vec<NodeTree>) {
        match self {
            NodeTree(node, children) => (node, children),
        }
    }
    pub fn from(
        node: Node,
        childs: Vec<Node>,
    ) -> Self {
        NodeTree(
            node,
            childs.into_iter().map(|n| NodeTree(n, vec![])).collect(),
        )
    }
}
/// 用于描述节点类型的行为规则和属性约束，通过[Schema](super::schema::Schema)进行统一管理
#[derive(Clone, PartialEq, Eq)]
pub struct NodeDefinition {
    /// 节点类型的唯一标识符（例如："dw", "dxgc"）
    pub name: String,
    /// 节点类型的详细配置规范
    pub spec: NodeSpec,
    /// 节点类型的描述信息
    pub desc: String,
    /// 节点所属的逻辑分组
    pub groups: Vec<String>,
    /// 节点支持的属性集合（属性名 -> 属性定义）
    pub attrs: HashMap<String, Attribute>,
    /// 节点属性的默认值集合
    pub default_attrs: HashMap<String, Value>,
    /// 内容匹配规则，定义允许的子节点结构
    pub content_match: Option<ContentMatch>,
    /// 允许附加的Mark类型集合
    pub mark_set: Option<Vec<MarkDefinition>>,
}
impl Debug for NodeDefinition {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("NodeType")
            .field("name", &self.name)
            .field("spec", &self.spec)
            .field("desc", &self.desc)
            .field("groups", &self.groups)
            .field("attrs", &self.attrs)
            .field("default_attrs", &self.default_attrs)
            .field("mark_set", &self.mark_set)
            .finish()
    }
}

impl NodeDefinition {
    /// 将原始节点规范编译为可用的节点类型集合
    ///
    /// # 参数
    /// - `nodes`: 节点名称到[NodeSpec]的映射
    ///
    /// # 返回值
    /// 返回[HashMap]<String, [NodeType]> 类型节点集合
    pub fn compile(
        nodes: HashMap<String, NodeSpec>
    ) -> HashMap<String, NodeDefinition> {
        let mut result = HashMap::new();

        // First create all node types without content_match
        for (name, spec) in &nodes {
            result.insert(
                name.clone(),
                NodeDefinition::new(name.clone(), spec.clone()),
            );
        }

        // Then set up content_match for each node type
        let result_clone = result.clone();
        for (_, node_type) in result.iter_mut() {
            if let Some(content) = &node_type.spec.content {
                node_type.content_match =
                    Some(ContentMatch::parse(content.clone(), &result_clone));
            }
        }

        result
    }
    /// 创建新的节点类型实例
    ///
    /// # 参数
    /// - `name`: 节点类型名称  
    /// - `spec`: 节点规范定义  
    ///
    /// # 注意
    /// 自动从spec中推导默认属性和内容匹配规则
    pub fn new(
        name: String,
        spec: NodeSpec,
    ) -> Self {
        let attrs = spec.attrs.as_ref().map_or_else(HashMap::new, |attrs| {
            attrs
                .iter()
                .map(|(name, spec)| {
                    (name.clone(), Attribute::new(spec.clone()))
                })
                .collect()
        });

        let default_attrs = attrs
            .iter()
            .filter_map(|(name, attr)| {
                match (&attr.has_default, &attr.default) {
                    (true, Some(v)) => Some((name.clone(), v.clone())),
                    _ => None,
                }
            })
            .collect();

        NodeDefinition {
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
    /// 验证节点内容是否符合类型约束
    ///
    /// # 参数
    /// - `content`: 子节点切片  
    /// - `schema`: 当前使用的文档模式  
    ///
    /// # 返回值
    /// 返回`true`表示内容合法，`false`表示不合法
    pub fn check_content(
        &self,
        content: &[Node],
        schema: &Schema,
    ) -> bool {
        if let Some(content_match) = &self.content_match {
            if let Some(result) = content_match.match_fragment(content, schema)
            {
                if !result.valid_end {
                    return false;
                }
            }
        }
        true
    }
    /// 验证节点属性是否符合规范
    ///
    /// # 参数
    /// - `values`: 待验证的属性集合  
    ///
    /// # Panics
    /// 当遇到以下情况会panic：  
    /// - 包含未定义的属性  
    /// - 缺少必须的属性
    pub fn check_attrs(
        &self,
        values: &Attrs,
    ) {
        for (key, _value) in values.attrs.iter() {
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

    /// 检查节点是否包含必须的属性
    pub fn has_required_attrs(&self) -> bool {
        self.attrs.values().any(|attr: &Attribute| attr.is_required())
    }

    /// 创建节点并填充内容（保持旧 API 兼容，内部委托给 [`crate::NodeFactory`]）。
    pub fn create_and_fill(
        &self,
        id: Option<NodeId>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
        schema: &Schema,
    ) -> NodeTree {
        schema.factory()
            .create_tree_with_type(self, id, attrs, content, marks)
            .expect("NodeFactory::create_tree_with_type should succeed for compiled schema")
    }

    /// 创建节点（保持旧 API 兼容）。
    pub fn create(
        &self,
        id: Option<NodeId>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<NodeId>,
        marks: Option<Vec<Mark>>,
    ) -> Node {
        let id: NodeId = id.unwrap_or_else(IdGenerator::get_id);

        Node::new(
            &id,
            self.name.clone(),
            self.compute_attrs(attrs),
            content,
            self.compute_marks(marks),
        )
    }

    pub(crate) fn compute_marks(
        &self,
        marks: Option<Vec<Mark>>,
    ) -> Vec<Mark> {
        match (&self.mark_set, marks) {
            (Some(def), Some(marks)) => def
                .iter()
                .filter_map(|mark_type| {
                    marks.iter().find(|m| m.r#type == mark_type.name).cloned()
                })
                .collect(),
            (None, Some(marks)) => marks,
            _ => vec![],
        }
    }

    pub(crate) fn compute_attrs(
        &self,
        attrs: Option<&HashMap<String, Value>>,
    ) -> Attrs {
        match attrs {
            Some(attr) => compute_attrs(&self.attrs, Some(attr)),
            None => compute_attrs(&self.attrs, Some(&self.default_attrs)),
        }
    }
}

/// 定义节点类型的约束规范
///
/// 用于配置节点类型的元数据和行为规则，通过[NodeType::compile]转换为可用类型
#[derive(Clone, PartialEq, Debug, Eq, Default)]
pub struct NodeSpec {
    /// 内容约束表达式（例如："*"）
    pub content: Option<String>,
    // 允许附加的Mark类型表达式（例如："color"）
    pub marks: Option<String>,
    /// 所属的逻辑分组
    pub group: Option<String>,
    /// 类型描述信息
    pub desc: Option<String>,
    /// 属性规范定义（属性名 -> 属性规范）
    pub attrs: Option<HashMap<String, AttributeSpec>>,
}






use super::attrs::Attrs;
use super::content::ContentMatch;
use super::id_generator::IdGenerator;
use super::mark::Mark;
use super::mark_type::MarkType;
use super::node::Node;
use super::schema::{compute_attrs, Attribute, AttributeSpec, Schema};
use super::types::NodeId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone,Debug,Serialize, Deserialize)]
pub struct NodeEnum(pub Node, pub Vec<NodeEnum>);

impl NodeEnum {
    pub fn into_parts(self) -> (Node, Vec<NodeEnum>) {
        match self {
            NodeEnum(node, children) => (node, children),
        }
    }
}
/// 用于描述节点类型的行为规则和属性约束，通过[Schema](super::schema::Schema)进行统一管理
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NodeType {
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
    pub mark_set: Option<Vec<MarkType>>,
}

impl NodeType {
    /// 将原始节点规范编译为可用的节点类型集合
    ///
    /// # 参数
    /// - `nodes`: 节点名称到[NodeSpec]的映射
    ///
    /// # 返回值
    /// 返回[HashMap]<String, [NodeType]> 类型节点集合
    pub fn compile(
        nodes: HashMap<String, NodeSpec>
    ) -> HashMap<String, NodeType> {
        let mut result = HashMap::new();

        // First create all node types without content_match
        for (name, spec) in &nodes {
            result.insert(
                name.clone(),
                NodeType::new(name.clone(), spec.clone()),
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

    /// 创建节点并填充内容
    ///
    /// # 参数
    /// - `id`: 可选的节点ID，如果未提供则自动生成
    /// - `attrs`: 可选的属性映射，用于设置节点属性
    /// - `content`: 子节点列表
    /// - `marks`: 可选的标记列表，用于设置节点标记
    /// - `schema`: 当前使用的文档模式
    ///
    /// # 返回值
    /// 返回包含新创建的节点及其所有子节点的向量
    pub fn create_and_fill(
        &self,
        id: Option<String>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
        schema: &Schema,
    ) -> NodeEnum {
        let id: String = id.unwrap_or_else(IdGenerator::get_id);
        let attrs = self.compute_attrs(attrs);

        // 首先创建需要填充的内容
        let mut filled_nodes = Vec::new();
        let mut content_ids = Vec::new();
        if let Some(content_match) = &self.content_match {
            if let Some(matched) =
                content_match.match_fragment(&content, schema)
            {
                if let Some(filled) = matched.fill(&content, true, schema) {
                    // 对每个填充的节点，递归创建其子节点
                    for node in filled {
                        if let Some(node_type) = schema.nodes.get(&node.r#type)
                        {
                            content_ids.push(node.id.clone());
                            // 递归创建节点及其子节点
                            let child_nodes = node_type.create_and_fill(
                                Some(node.id.clone()),
                                None,
                                vec![],
                                None,
                                schema,
                            );
                            filled_nodes.push(child_nodes);
                        }
                    }
                }
            }
        }

        NodeEnum(
            Node::new(
                &id,
                self.name.clone(),
                attrs,
                content_ids,
                Mark::set_from(marks),
            ),
            filled_nodes,
        )
    }

    /// 创建节点
    pub fn create(
        &self,
        id: Option<String>,
        attrs: Option<&HashMap<String, Value>>,
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

    fn compute_attrs(
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

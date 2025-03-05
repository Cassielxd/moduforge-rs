use super::attrs::Attrs;
use super::content::ContentMatch;
use super::id_generator::IdGenerator;
use super::mark::Mark;
use super::mark_type::MarkType;
use super::node::Node;
use super::schema::{Attribute, AttributeSpec, Schema, compute_attrs};
use super::types::NodeId;
use im::HashMap as ImHashMap;
use std::collections::HashMap;

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
    pub default_attrs: HashMap<String, String>,
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
    pub fn compile(nodes: HashMap<String, NodeSpec>) -> HashMap<String, NodeType> {
        let mut result = HashMap::new();
        for (name, spec) in nodes {
            result.insert(name.clone(), NodeType::new(name.clone(), spec.clone()));
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
    /// 验证节点内容是否符合类型约束
    ///
    /// # 参数
    /// - `content`: 子节点切片  
    /// - `schema`: 当前使用的文档模式  
    ///
    /// # 返回值
    /// 返回`true`表示内容合法，`false`表示不合法
    pub fn check_content(&self, content: &[Node], schema: &Schema) -> bool {
        if let Some(content_match) = &self.content_match {
            if let Some(result) = content_match.match_fragment(content, schema) {
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
    pub fn check_attrs(&self, values: &Attrs) {
        for (key, _value) in values {
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
        self.attrs
            .values()
            .any(|attr: &Attribute| attr.is_required())
    }
    /// 创建节点并填充内容
    pub fn create_and_fill(
        &self,
        id: Option<String>,
        attrs: Option<&HashMap<String, String>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
        schema: &Schema,
    ) -> Vec<Node> {
        let id: String = id.unwrap_or_else(IdGenerator::get_id);
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
    /// 创建节点
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
            Some(attr) => compute_attrs(&self.attrs, Some(attr)),
            None => compute_attrs(&self.attrs, Some(&self.default_attrs)),
        }
    }
    // 其他方法...
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

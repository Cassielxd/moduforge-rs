use std::collections::HashMap;

use serde_json::Value;

use crate::{
    error::{error_helpers::schema_error, PoolResult},
    id_generator::IdGenerator,
    mark::Mark,
    mark_definition::MarkDefinition,
    node::Node,
    node_definition::{NodeDefinition, NodeTree},
    schema::Schema,
    types::NodeId,
};

/// 工厂负责基于 [`Schema`] 生成各类节点，复用 [`NodeType`] 的编译信息。
#[derive(Clone)]
pub struct NodeFactory<'schema> {
    schema: &'schema Schema,
}

impl<'schema> NodeFactory<'schema> {
    /// 创建新的工厂实例，保存对 [`Schema`] 的只读引用。
    pub fn new(schema: &'schema Schema) -> Self {
        Self { schema }
    }

    /// 暴露内部引用，便于调用方读取原始 Schema。
    pub fn schema(&self) -> &'schema Schema {
        self.schema
    }

    /// 按类型名称创建单节点。
    pub fn create_node(
        &self,
        type_name: &str,
        id: Option<NodeId>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<NodeId>,
        marks: Option<Vec<Mark>>,
    ) -> PoolResult<Node> {
        let node_type = self.schema.nodes.get(type_name).ok_or_else(|| {
            schema_error(&format!("无法在 schema 中找到节点类型: {type_name}"))
        })?;

        Ok(Self::instantiate_node(node_type, id, attrs, content, marks))
    }

    /// 获取节点类型定义引用，便于上层读取配置。
    pub fn node_definition(
        &self,
        type_name: &str,
    ) -> Option<&NodeDefinition> {
        self.schema.nodes.get(type_name)
    }

    /// 按类型名称创建标记。
    pub fn create_mark(
        &self,
        type_name: &str,
        attrs: Option<&HashMap<String, Value>>,
    ) -> PoolResult<Mark> {
        let mark_def = self.schema.marks.get(type_name).ok_or_else(|| {
            schema_error(&format!("无法在 schema 中找到标记类型: {type_name}"))
        })?;

        Ok(Self::instantiate_mark(mark_def, attrs))
    }

    /// 获取标记类型定义引用。
    pub fn mark_definition(
        &self,
        type_name: &str,
    ) -> Option<&MarkDefinition> {
        self.schema.marks.get(type_name)
    }

    /// 获取整个 Schema 的节点与标记定义映射，便于上层做批量/调试读取。
    pub fn definitions(
        &self
    ) -> (&HashMap<String, NodeDefinition>, &HashMap<String, MarkDefinition>)
    {
        (&self.schema.nodes, &self.schema.marks)
    }

    /// 以顶级节点为根构建整棵子树。
    pub fn create_top_node(
        &self,
        id: Option<NodeId>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
    ) -> PoolResult<NodeTree> {
        let top_node_type = self
            .schema
            .top_node_type
            .as_ref()
            .ok_or_else(|| schema_error("未找到顶级节点类型定义"))?;

        self.create_tree_with_type(top_node_type, id, attrs, content, marks)
    }

    /// 为指定类型构建并填充子树。
    pub fn create_tree(
        &self,
        type_name: &str,
        id: Option<NodeId>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
    ) -> PoolResult<NodeTree> {
        let node_type = self.schema.nodes.get(type_name).ok_or_else(|| {
            schema_error(&format!("无法在 schema 中找到节点类型: {type_name}"))
        })?;

        self.create_tree_with_type(node_type, id, attrs, content, marks)
    }

    /// 暴露给 [`NodeDefinition`] 的内部构建逻辑。
    pub(crate) fn create_tree_with_type(
        &self,
        node_type: &NodeDefinition,
        id: Option<NodeId>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
    ) -> PoolResult<NodeTree> {
        let id: NodeId = id.unwrap_or_else(IdGenerator::get_id);
        let computed_attrs = node_type.compute_attrs(attrs);
        let computed_marks = node_type.compute_marks(marks);

        let mut filled_nodes: Vec<NodeTree> = Vec::new();
        let mut final_content_ids: Vec<NodeId> = Vec::new();

        if let Some(content_match) = &node_type.content_match {
            if let Some(matched) =
                content_match.match_fragment(&content, self.schema)
            {
                if let Some(needed_type_names) =
                    matched.fill(&content, true, self.schema)
                {
                    for type_name in needed_type_names {
                        if let Some(existing_node) =
                            content.iter().find(|n| n.r#type == type_name)
                        {
                            let attrs_map: HashMap<String, Value> =
                                existing_node
                                    .attrs
                                    .attrs
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.clone()))
                                    .collect();
                            let marks_vec: Vec<Mark> =
                                existing_node.marks.iter().cloned().collect();
                            let child_type = self
                                .schema
                                .nodes
                                .get(&type_name)
                                .ok_or_else(|| {
                                    schema_error(&format!(
                                        "无法在 schema 中找到节点类型: {type_name}"
                                    ))
                                })?;

                            let child_tree = self.create_tree_with_type(
                                child_type,
                                Some(existing_node.id.clone()),
                                Some(&attrs_map),
                                vec![],
                                Some(marks_vec),
                            )?;
                            let child_id = child_tree.0.id.clone();
                            final_content_ids.push(child_id);
                            filled_nodes.push(child_tree);
                        } else {
                            let child_type = self
                                .schema
                                .nodes
                                .get(&type_name)
                                .ok_or_else(|| {
                                    schema_error(&format!(
                                        "无法在 schema 中找到节点类型: {type_name}"
                                    ))
                                })?;

                            let child_tree = self.create_tree_with_type(
                                child_type,
                                None,
                                None,
                                vec![],
                                None,
                            )?;
                            let child_id = child_tree.0.id.clone();
                            final_content_ids.push(child_id);
                            filled_nodes.push(child_tree);
                        }
                    }
                }
            }
        }

        let node = Node::new(
            &id,
            node_type.name.clone(),
            computed_attrs,
            final_content_ids,
            computed_marks,
        );

        Ok(NodeTree(node, filled_nodes))
    }

    fn instantiate_node(
        node_type: &NodeDefinition,
        id: Option<NodeId>,
        attrs: Option<&HashMap<String, Value>>,
        content: Vec<NodeId>,
        marks: Option<Vec<Mark>>,
    ) -> Node {
        let id: NodeId = id.unwrap_or_else(IdGenerator::get_id);
        let attrs = node_type.compute_attrs(attrs);
        let marks = node_type.compute_marks(marks);

        Node::new(&id, node_type.name.clone(), attrs, content, marks)
    }

    pub(crate) fn instantiate_mark(
        mark_def: &MarkDefinition,
        attrs: Option<&HashMap<String, Value>>,
    ) -> Mark {
        Mark {
            r#type: mark_def.name.clone(),
            attrs: mark_def.compute_attrs(attrs),
        }
    }
}

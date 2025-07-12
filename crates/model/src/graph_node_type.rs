use std::collections::HashMap;
use std::fmt::{self, Debug};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use im::HashMap as ImHashMap;
use im::Vector as ImVector;

use crate::{
    node::Node,
    types::NodeId,
    error::PoolResult,
    graph::{GraphNode, Relation, RelationType},
    versioned_graph::{VersionedGraph, GraphSnapshot},
    node_type::{NodeType, NodeSpec},
    schema::Schema,
    attrs::Attrs,
    mark::Mark,
    id_generator::IdGenerator,
};

/// 图节点类型定义，支持复杂的节点关系和行为
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeType {
    /// 节点类型名称
    pub name: String,
    /// 节点类型描述
    pub description: String,
    /// 节点类型规范
    pub spec: NodeSpec,
    /// 节点分组
    pub groups: ImVector<String>,
    /// 节点属性定义
    pub attributes: ImHashMap<String, Value>,
    /// 默认属性值
    pub default_attributes: ImHashMap<String, Value>,
    /// 允许的关系类型
    pub allowed_relations: ImVector<RelationType>,
    /// 禁止的关系类型
    pub forbidden_relations: ImVector<RelationType>,
    /// 子节点类型约束
    pub child_constraints: ImHashMap<String, ChildConstraint>,
    /// 父节点类型约束
    pub parent_constraints: ImHashMap<String, ParentConstraint>,
    /// 节点创建规则
    pub creation_rules: ImVector<CreationRule>,
    /// 节点验证规则
    pub validation_rules: ImVector<ValidationRule>,
    /// 节点行为定义
    pub behaviors: ImHashMap<String, NodeBehavior>,
}

/// 子节点约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildConstraint {
    /// 允许的子节点类型
    pub allowed_types: ImVector<String>,
    /// 禁止的子节点类型
    pub forbidden_types: ImVector<String>,
    /// 最小子节点数量
    pub min_count: usize,
    /// 最大子节点数量
    pub max_count: Option<usize>,
    /// 关系类型
    pub relation_type: RelationType,
    /// 是否必需
    pub required: bool,
}

/// 父节点约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentConstraint {
    /// 允许的父节点类型
    pub allowed_types: ImVector<String>,
    /// 禁止的父节点类型
    pub forbidden_types: ImVector<String>,
    /// 关系类型
    pub relation_type: RelationType,
    /// 是否必需
    pub required: bool,
}

/// 节点创建规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreationRule {
    /// 规则名称
    pub name: String,
    /// 触发条件
    pub condition: CreationCondition,
    /// 创建动作
    pub action: CreationAction,
    /// 规则优先级
    pub priority: i32,
    /// 是否启用
    pub enabled: bool,
}

/// 创建条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreationCondition {
    /// 当父节点类型为指定类型时
    ParentType(String),
    /// 当子节点数量少于指定值时
    ChildCountLess(usize),
    /// 当缺少指定类型的子节点时
    MissingChildType(String),
    /// 当属性满足指定条件时
    AttributeCondition(String, Value),
    /// 复合条件
    And(Vec<CreationCondition>),
    Or(Vec<CreationCondition>),
    Not(Box<CreationCondition>),
}

/// 创建动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreationAction {
    /// 创建指定类型的子节点
    CreateChild(String),
    /// 创建指定类型的父节点
    CreateParent(String),
    /// 创建指定类型的兄弟节点
    CreateSibling(String),
    /// 设置属性
    SetAttribute(String, Value),
    /// 复合动作
    Sequence(Vec<CreationAction>),
}

/// 节点验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// 规则名称
    pub name: String,
    /// 验证条件
    pub condition: ValidationCondition,
    /// 错误消息
    pub error_message: String,
    /// 规则优先级
    pub priority: i32,
    /// 是否启用
    pub enabled: bool,
}

/// 验证条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationCondition {
    /// 检查子节点数量
    ChildCount(usize, usize), // min, max
    /// 检查必需的子节点类型
    RequiredChildType(String),
    /// 检查属性值
    AttributeValue(String, Value),
    /// 检查关系数量
    RelationCount(RelationType, usize, usize), // min, max
    /// 复合条件
    And(Vec<ValidationCondition>),
    Or(Vec<ValidationCondition>),
    Not(Box<ValidationCondition>),
}

/// 节点行为定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBehavior {
    /// 行为名称
    pub name: String,
    /// 行为类型
    pub behavior_type: BehaviorType,
    /// 行为参数
    pub parameters: ImHashMap<String, Value>,
    /// 是否启用
    pub enabled: bool,
}

/// 行为类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorType {
    /// 自动创建子节点
    AutoCreateChildren,
    /// 自动创建父节点
    AutoCreateParent,
    /// 自动设置属性
    AutoSetAttributes,
    /// 自动建立关系
    AutoCreateRelations,
    /// 自定义行为
    Custom(String),
}

impl GraphNodeType {
    /// 创建新的图节点类型
    pub fn new(name: String, spec: NodeSpec) -> Self {
        Self {
            name,
            description: spec.desc.unwrap_or_default(),
            spec,
            groups: ImVector::new(),
            attributes: ImHashMap::new(),
            default_attributes: ImHashMap::new(),
            allowed_relations: ImVector::new(),
            forbidden_relations: ImVector::new(),
            child_constraints: ImHashMap::new(),
            parent_constraints: ImHashMap::new(),
            creation_rules: ImVector::new(),
            validation_rules: ImVector::new(),
            behaviors: ImHashMap::new(),
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// 添加分组
    pub fn add_group(&mut self, group: String) {
        self.groups = self.groups.push_back(group);
    }

    /// 设置属性
    pub fn set_attribute(&mut self, key: String, value: Value) {
        self.attributes = self.attributes.update(key, value);
    }

    /// 设置默认属性
    pub fn set_default_attribute(&mut self, key: String, value: Value) {
        self.default_attributes = self.default_attributes.update(key, value);
    }

    /// 添加允许的关系类型
    pub fn add_allowed_relation(&mut self, relation_type: RelationType) {
        self.allowed_relations = self.allowed_relations.push_back(relation_type);
    }

    /// 添加禁止的关系类型
    pub fn add_forbidden_relation(&mut self, relation_type: RelationType) {
        self.forbidden_relations = self.forbidden_relations.push_back(relation_type);
    }

    /// 添加子节点约束
    pub fn add_child_constraint(&mut self, constraint_type: String, constraint: ChildConstraint) {
        self.child_constraints = self.child_constraints.update(constraint_type, constraint);
    }

    /// 添加父节点约束
    pub fn add_parent_constraint(&mut self, constraint_type: String, constraint: ParentConstraint) {
        self.parent_constraints = self.parent_constraints.update(constraint_type, constraint);
    }

    /// 添加创建规则
    pub fn add_creation_rule(&mut self, rule: CreationRule) {
        self.creation_rules = self.creation_rules.push_back(rule);
    }

    /// 添加验证规则
    pub fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules = self.validation_rules.push_back(rule);
    }

    /// 添加行为
    pub fn add_behavior(&mut self, name: String, behavior: NodeBehavior) {
        self.behaviors = self.behaviors.update(name, behavior);
    }

    /// 递归创建节点及其子节点
    pub fn create_and_fill(
        &self,
        graph: &mut VersionedGraph,
        id: Option<NodeId>,
        attrs: Option<&ImHashMap<String, Value>>,
        content: Vec<Node>,
        marks: Option<Vec<Mark>>,
        schema: &Schema,
        node_types: &ImHashMap<String, GraphNodeType>,
    ) -> PoolResult<NodeId> {
        // 生成节点ID
        let node_id = id.unwrap_or_else(|| NodeId::from(IdGenerator::get_id()));
        
        // 计算属性
        let computed_attrs = self.compute_attributes(attrs);
        
        // 创建基础节点
        let node = Node::new(
            node_id.clone(),
            self.name.clone(),
            computed_attrs,
            vec![], // 初始为空，稍后填充
            marks.unwrap_or_default(),
        );
        
        // 添加到图中
        graph.add_node(node)?;
        
        // 递归创建子节点
        let mut child_ids = Vec::new();
        for child_node in content {
            let child_id = child_node.id.clone();
            
            // 检查是否有对应的图节点类型
            if let Some(child_type) = node_types.get(&child_node.r#type) {
                // 递归创建子节点
                let created_child_id = child_type.create_and_fill(
                    graph,
                    Some(child_id),
                    Some(&child_node.attrs.attrs),
                    vec![], // 子节点的内容
                    Some(child_node.marks.clone()),
                    schema,
                    node_types,
                )?;
                child_ids.push(created_child_id);
                
                // 建立父子关系
                let relation = Relation::new(RelationType::ParentChild);
                graph.add_relation(&node_id, &created_child_id, relation)?;
            } else {
                // 直接添加子节点
                graph.add_node(child_node)?;
                child_ids.push(child_id.clone());
                
                // 建立父子关系
                let relation = Relation::new(RelationType::ParentChild);
                graph.add_relation(&node_id, &child_id, relation)?;
            }
        }
        
        // 应用创建规则
        self.apply_creation_rules(graph, &node_id, node_types)?;
        
        // 应用验证规则
        self.apply_validation_rules(graph, &node_id)?;
        
        Ok(node_id)
    }

    /// 计算节点属性
    fn compute_attributes(&self, attrs: Option<&ImHashMap<String, Value>>) -> Attrs {
        let mut result_attrs = self.default_attributes.clone();
        
        if let Some(provided_attrs) = attrs {
            for (key, value) in provided_attrs.iter() {
                result_attrs = result_attrs.update(key.clone(), value.clone());
            }
        }
        
        Attrs::new(result_attrs)
    }

    /// 应用创建规则
    fn apply_creation_rules(
        &self,
        graph: &mut VersionedGraph,
        node_id: &NodeId,
        node_types: &ImHashMap<String, GraphNodeType>,
    ) -> PoolResult<()> {
        for rule in self.creation_rules.iter() {
            if !rule.enabled {
                continue;
            }
            
            if self.evaluate_creation_condition(graph, node_id, &rule.condition)? {
                self.execute_creation_action(graph, node_id, &rule.action, node_types)?;
            }
        }
        Ok(())
    }

    /// 评估创建条件
    fn evaluate_creation_condition(
        &self,
        graph: &VersionedGraph,
        node_id: &NodeId,
        condition: &CreationCondition,
    ) -> PoolResult<bool> {
        match condition {
            CreationCondition::ParentType(expected_type) => {
                if let Some(parent) = graph.get_parent(node_id) {
                    Ok(parent.node_type() == expected_type)
                } else {
                    Ok(false)
                }
            }
            CreationCondition::ChildCountLess(count) => {
                let children = graph.get_children(node_id);
                Ok(children.len() < *count)
            }
            CreationCondition::MissingChildType(child_type) => {
                let children = graph.get_children(node_id);
                Ok(!children.iter().any(|child| child.node_type() == child_type))
            }
            CreationCondition::AttributeCondition(attr_name, expected_value) => {
                if let Some(node) = graph.get_node(node_id) {
                    if let Some(attr_value) = node.get_property(attr_name) {
                        Ok(attr_value == expected_value)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            CreationCondition::And(conditions) => {
                for condition in conditions {
                    if !self.evaluate_creation_condition(graph, node_id, condition)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            CreationCondition::Or(conditions) => {
                for condition in conditions {
                    if self.evaluate_creation_condition(graph, node_id, condition)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            CreationCondition::Not(condition) => {
                let result = self.evaluate_creation_condition(graph, node_id, condition)?;
                Ok(!result)
            }
        }
    }

    /// 执行创建动作
    fn execute_creation_action(
        &self,
        graph: &mut VersionedGraph,
        node_id: &NodeId,
        action: &CreationAction,
        node_types: &ImHashMap<String, GraphNodeType>,
    ) -> PoolResult<()> {
        match action {
            CreationAction::CreateChild(child_type_name) => {
                if let Some(child_type) = node_types.get(child_type_name) {
                    let child_id = NodeId::from(IdGenerator::get_id());
                    child_type.create_and_fill(
                        graph,
                        Some(child_id.clone()),
                        None,
                        vec![],
                        None,
                        &Schema::default(), // 简化处理
                        node_types,
                    )?;
                    
                    let relation = Relation::new(RelationType::ParentChild);
                    graph.add_relation(node_id, &child_id, relation)?;
                }
            }
            CreationAction::CreateParent(parent_type_name) => {
                if let Some(parent_type) = node_types.get(parent_type_name) {
                    let parent_id = NodeId::from(IdGenerator::get_id());
                    parent_type.create_and_fill(
                        graph,
                        Some(parent_id.clone()),
                        None,
                        vec![],
                        None,
                        &Schema::default(), // 简化处理
                        node_types,
                    )?;
                    
                    let relation = Relation::new(RelationType::ParentChild);
                    graph.add_relation(&parent_id, node_id, relation)?;
                }
            }
            CreationAction::CreateSibling(sibling_type_name) => {
                if let Some(sibling_type) = node_types.get(sibling_type_name) {
                    let sibling_id = NodeId::from(IdGenerator::get_id());
                    sibling_type.create_and_fill(
                        graph,
                        Some(sibling_id.clone()),
                        None,
                        vec![],
                        None,
                        &Schema::default(), // 简化处理
                        node_types,
                    )?;
                    
                    // 如果有父节点，建立兄弟关系
                    if let Some(parent) = graph.get_parent(node_id) {
                        let relation = Relation::new(RelationType::ParentChild);
                        graph.add_relation(&parent.node.id, &sibling_id, relation)?;
                    }
                }
            }
            CreationAction::SetAttribute(attr_name, value) => {
                if let Some(node) = graph.get_node(node_id) {
                    // 这里需要修改节点属性，但 GraphNode 是不可变的
                    // 在实际实现中，可能需要创建一个新的节点或使用其他方法
                }
            }
            CreationAction::Sequence(actions) => {
                for action in actions {
                    self.execute_creation_action(graph, node_id, action, node_types)?;
                }
            }
        }
        Ok(())
    }

    /// 应用验证规则
    fn apply_validation_rules(
        &self,
        graph: &VersionedGraph,
        node_id: &NodeId,
    ) -> PoolResult<()> {
        for rule in self.validation_rules.iter() {
            if !rule.enabled {
                continue;
            }
            
            if !self.evaluate_validation_condition(graph, node_id, &rule.condition)? {
                return Err(crate::error::PoolError::ValidationFailed(rule.error_message.clone()));
            }
        }
        Ok(())
    }

    /// 评估验证条件
    fn evaluate_validation_condition(
        &self,
        graph: &VersionedGraph,
        node_id: &NodeId,
        condition: &ValidationCondition,
    ) -> PoolResult<bool> {
        match condition {
            ValidationCondition::ChildCount(min, max) => {
                let children = graph.get_children(node_id);
                let count = children.len();
                Ok(count >= *min && count <= *max)
            }
            ValidationCondition::RequiredChildType(child_type) => {
                let children = graph.get_children(node_id);
                Ok(children.iter().any(|child| child.node_type() == child_type))
            }
            ValidationCondition::AttributeValue(attr_name, expected_value) => {
                if let Some(node) = graph.get_node(node_id) {
                    if let Some(attr_value) = node.get_property(attr_name) {
                        Ok(attr_value == expected_value)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            ValidationCondition::RelationCount(relation_type, min, max) => {
                let relations = graph.get_relations_by_type(node_id, relation_type);
                let count = relations.len();
                Ok(count >= *min && count <= *max)
            }
            ValidationCondition::And(conditions) => {
                for condition in conditions {
                    if !self.evaluate_validation_condition(graph, node_id, condition)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ValidationCondition::Or(conditions) => {
                for condition in conditions {
                    if self.evaluate_validation_condition(graph, node_id, condition)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            ValidationCondition::Not(condition) => {
                let result = self.evaluate_validation_condition(graph, node_id, condition)?;
                Ok(!result)
            }
        }
    }

    /// 验证节点是否符合类型约束
    pub fn validate_node(&self, graph: &VersionedGraph, node_id: &NodeId) -> PoolResult<()> {
        // 验证子节点约束
        for (constraint_type, constraint) in self.child_constraints.iter() {
            let children = graph.get_relations_by_type(node_id, &constraint.relation_type);
            let child_count = children.len();
            
            if child_count < constraint.min_count {
                return Err(crate::error::PoolError::ValidationFailed(
                    format!("节点 {} 的子节点数量 {} 少于最小值 {}", 
                        node_id, child_count, constraint.min_count)
                ));
            }
            
            if let Some(max_count) = constraint.max_count {
                if child_count > max_count {
                    return Err(crate::error::PoolError::ValidationFailed(
                        format!("节点 {} 的子节点数量 {} 超过最大值 {}", 
                            node_id, child_count, max_count)
                    ));
                }
            }
            
            // 检查允许的类型
            for (child_node, _) in children {
                if !constraint.allowed_types.contains(&child_node.node_type().to_string()) {
                    return Err(crate::error::PoolError::ValidationFailed(
                        format!("节点 {} 包含不允许的子节点类型 {}", 
                            node_id, child_node.node_type())
                    ));
                }
            }
        }
        
        // 验证父节点约束
        for (constraint_type, constraint) in self.parent_constraints.iter() {
            if constraint.required {
                if let Some(parent) = graph.get_parent(node_id) {
                    if !constraint.allowed_types.contains(&parent.node_type().to_string()) {
                        return Err(crate::error::PoolError::ValidationFailed(
                            format!("节点 {} 的父节点类型 {} 不在允许列表中", 
                                node_id, parent.node_type())
                        ));
                    }
                } else {
                    return Err(crate::error::PoolError::ValidationFailed(
                        format!("节点 {} 缺少必需的父节点", node_id)
                    ));
                }
            }
        }
        
        Ok(())
    }
}

impl Default for GraphNodeType {
    fn default() -> Self {
        Self::new("default".to_string(), NodeSpec::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node_type::NodeSpec;

    #[test]
    fn test_graph_node_type_creation() {
        let spec = NodeSpec::default();
        let node_type = GraphNodeType::new("test_type".to_string(), spec);
        
        assert_eq!(node_type.name, "test_type");
        assert!(node_type.groups.is_empty());
        assert!(node_type.attributes.is_empty());
    }

    #[test]
    fn test_add_group() {
        let mut node_type = GraphNodeType::new("test_type".to_string(), NodeSpec::default());
        node_type.add_group("test_group".to_string());
        
        assert_eq!(node_type.groups.len(), 1);
        assert_eq!(node_type.groups[0], "test_group");
    }

    #[test]
    fn test_set_attribute() {
        let mut node_type = GraphNodeType::new("test_type".to_string(), NodeSpec::default());
        node_type.set_attribute("test_attr".to_string(), Value::String("test_value".to_string()));
        
        assert_eq!(node_type.attributes.len(), 1);
        assert_eq!(node_type.attributes.get("test_attr").unwrap(), &Value::String("test_value".to_string()));
    }
}
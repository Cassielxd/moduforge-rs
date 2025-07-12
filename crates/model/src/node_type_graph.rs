use std::collections::HashMap;
use std::fmt::{self, Debug};
use serde::{Deserialize, Serialize};
use im::HashMap as ImHashMap;
use im::Vector as ImVector;

use crate::{
    node::Node,
    types::NodeId,
    graph::{RelationType, Relation},
    content_relation::{ContentRelationRule, RelationConstraints, ValidationResult},
    error::PoolResult,
};

/// 图结构节点类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNodeType {
    /// 节点类型名称
    pub name: String,
    /// 节点类型描述
    pub description: Option<String>,
    /// 节点类型分组
    pub groups: ImVector<String>,
    /// 节点属性定义
    pub attributes: ImHashMap<String, AttributeDefinition>,
    /// 节点默认属性
    pub default_attributes: ImHashMap<String, serde_json::Value>,
    /// 内容关系规则
    pub content_rules: ImVector<ContentRelationRule>,
    /// 节点约束
    pub constraints: NodeConstraints,
    /// 节点行为定义
    pub behaviors: NodeBehaviors,
    /// 是否启用
    pub enabled: bool,
}

/// 属性定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    /// 属性名称
    pub name: String,
    /// 属性类型
    pub attribute_type: AttributeType,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default: Option<serde_json::Value>,
    /// 验证规则
    pub validation: Option<String>,
    /// 属性描述
    pub description: Option<String>,
}

/// 属性类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeType {
    /// 字符串类型
    String,
    /// 数字类型
    Number,
    /// 布尔类型
    Boolean,
    /// 数组类型
    Array,
    /// 对象类型
    Object,
    /// 自定义类型
    Custom(String),
}

/// 节点约束定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConstraints {
    /// 最大子节点数量
    pub max_children: Option<usize>,
    /// 最小子节点数量
    pub min_children: Option<usize>,
    /// 允许的子节点类型
    pub allowed_children: ImVector<String>,
    /// 禁止的子节点类型
    pub forbidden_children: ImVector<String>,
    /// 最大深度
    pub max_depth: Option<usize>,
    /// 是否允许循环引用
    pub allow_cycles: bool,
    /// 是否允许自引用
    pub allow_self_reference: bool,
}

/// 节点行为定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBehaviors {
    /// 是否可编辑
    pub editable: bool,
    /// 是否可删除
    pub deletable: bool,
    /// 是否可移动
    pub movable: bool,
    /// 是否可复制
    pub copyable: bool,
    /// 是否可克隆
    pub clonable: bool,
    /// 是否可展开
    pub expandable: bool,
    /// 是否可折叠
    pub collapsible: bool,
    /// 自定义行为
    pub custom_behaviors: ImHashMap<String, serde_json::Value>,
}

impl GraphNodeType {
    /// 创建新的图节点类型
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            groups: ImVector::new(),
            attributes: ImHashMap::new(),
            default_attributes: ImHashMap::new(),
            content_rules: ImVector::new(),
            constraints: NodeConstraints::new(),
            behaviors: NodeBehaviors::new(),
            enabled: true,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// 添加分组
    pub fn with_group(mut self, group: String) -> Self {
        self.groups = self.groups.push_back(group);
        self
    }

    /// 添加属性定义
    pub fn with_attribute(mut self, name: String, definition: AttributeDefinition) -> Self {
        self.attributes = self.attributes.update(name.clone(), definition.clone());
        if let Some(default) = definition.default {
            self.default_attributes = self.default_attributes.update(name, default);
        }
        self
    }

    /// 添加内容关系规则
    pub fn with_content_rule(mut self, rule: ContentRelationRule) -> Self {
        self.content_rules = self.content_rules.push_back(rule);
        self
    }

    /// 设置约束
    pub fn with_constraints(mut self, constraints: NodeConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// 设置行为
    pub fn with_behaviors(mut self, behaviors: NodeBehaviors) -> Self {
        self.behaviors = behaviors;
        self
    }

    /// 验证节点是否符合类型定义
    pub fn validate_node(&self, node: &Node) -> PoolResult<ValidationResult> {
        // 验证属性
        for (attr_name, attr_def) in &self.attributes {
            if attr_def.required {
                if !node.attrs.contains_key(attr_name) {
                    return Ok(ValidationResult::InvalidAttribute);
                }
            }
        }

        // 验证约束
        if let Some(max_children) = self.constraints.max_children {
            if node.content.len() > max_children {
                return Ok(ValidationResult::InvalidCount);
            }
        }

        if let Some(min_children) = self.constraints.min_children {
            if node.content.len() < min_children {
                return Ok(ValidationResult::InvalidCount);
            }
        }

        Ok(ValidationResult::Valid)
    }

    /// 创建节点实例
    pub fn create_node(
        &self,
        id: Option<String>,
        attributes: Option<ImHashMap<String, serde_json::Value>>,
        content: ImVector<NodeId>,
    ) -> PoolResult<Node> {
        let node_id = id.unwrap_or_else(|| format!("{}_{}", self.name, uuid::Uuid::new_v4()));
        
        // 合并默认属性和提供的属性
        let mut final_attrs = self.default_attributes.clone();
        if let Some(attrs) = attributes {
            for (key, value) in attrs {
                final_attrs = final_attrs.update(key, value);
            }
        }

        let node = Node::new(
            &node_id,
            self.name.clone(),
            crate::attrs::Attrs::from(final_attrs),
            content.into_iter().collect(),
            vec![],
        );

        // 验证创建的节点
        self.validate_node(&node)?;

        Ok(node)
    }

    /// 获取属性定义
    pub fn get_attribute_definition(&self, name: &str) -> Option<&AttributeDefinition> {
        self.attributes.get(name)
    }

    /// 获取所有属性名称
    pub fn get_attribute_names(&self) -> Vec<&String> {
        self.attributes.keys().collect()
    }

    /// 检查是否支持特定行为
    pub fn supports_behavior(&self, behavior: &str) -> bool {
        match behavior {
            "editable" => self.behaviors.editable,
            "deletable" => self.behaviors.deletable,
            "movable" => self.behaviors.movable,
            "copyable" => self.behaviors.copyable,
            "clonable" => self.behaviors.clonable,
            "expandable" => self.behaviors.expandable,
            "collapsible" => self.behaviors.collapsible,
            _ => self.behaviors.custom_behaviors.contains_key(behavior),
        }
    }

    /// 获取自定义行为
    pub fn get_custom_behavior(&self, name: &str) -> Option<&serde_json::Value> {
        self.behaviors.custom_behaviors.get(name)
    }
}

impl NodeConstraints {
    /// 创建新的节点约束
    pub fn new() -> Self {
        Self {
            max_children: None,
            min_children: None,
            allowed_children: ImVector::new(),
            forbidden_children: ImVector::new(),
            max_depth: None,
            allow_cycles: false,
            allow_self_reference: false,
        }
    }

    /// 设置子节点数量约束
    pub fn with_children_count(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_children = min;
        self.max_children = max;
        self
    }

    /// 添加允许的子节点类型
    pub fn with_allowed_child(mut self, child_type: String) -> Self {
        self.allowed_children = self.allowed_children.push_back(child_type);
        self
    }

    /// 添加禁止的子节点类型
    pub fn with_forbidden_child(mut self, child_type: String) -> Self {
        self.forbidden_children = self.forbidden_children.push_back(child_type);
        self
    }

    /// 设置深度限制
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// 设置循环引用策略
    pub fn with_cycle_policy(mut self, allow_cycles: bool, allow_self: bool) -> Self {
        self.allow_cycles = allow_cycles;
        self.allow_self_reference = allow_self;
        self
    }
}

impl Default for NodeConstraints {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeBehaviors {
    /// 创建新的节点行为
    pub fn new() -> Self {
        Self {
            editable: true,
            deletable: true,
            movable: true,
            copyable: true,
            clonable: true,
            expandable: false,
            collapsible: false,
            custom_behaviors: ImHashMap::new(),
        }
    }

    /// 设置编辑行为
    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    /// 设置删除行为
    pub fn with_deletable(mut self, deletable: bool) -> Self {
        self.deletable = deletable;
        self
    }

    /// 设置移动行为
    pub fn with_movable(mut self, movable: bool) -> Self {
        self.movable = movable;
        self
    }

    /// 设置复制行为
    pub fn with_copyable(mut self, copyable: bool) -> Self {
        self.copyable = copyable;
        self
    }

    /// 设置克隆行为
    pub fn with_clonable(mut self, clonable: bool) -> Self {
        self.clonable = clonable;
        self
    }

    /// 设置展开/折叠行为
    pub fn with_expand_collapse(mut self, expandable: bool, collapsible: bool) -> Self {
        self.expandable = expandable;
        self.collapsible = collapsible;
        self
    }

    /// 添加自定义行为
    pub fn with_custom_behavior(mut self, name: String, value: serde_json::Value) -> Self {
        self.custom_behaviors = self.custom_behaviors.update(name, value);
        self
    }
}

impl Default for NodeBehaviors {
    fn default() -> Self {
        Self::new()
    }
}

/// 图节点类型管理器
#[derive(Debug)]
pub struct GraphNodeTypeManager {
    /// 节点类型集合
    node_types: ImHashMap<String, GraphNodeType>,
    /// 类型索引（按分组）
    group_index: ImHashMap<String, ImVector<String>>,
}

impl GraphNodeTypeManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            node_types: ImHashMap::new(),
            group_index: ImHashMap::new(),
        }
    }

    /// 注册节点类型
    pub fn register_node_type(&mut self, node_type: GraphNodeType) -> PoolResult<()> {
        let name = node_type.name.clone();
        
        // 添加到类型集合
        self.node_types = self.node_types.update(name.clone(), node_type.clone());
        
        // 更新分组索引
        for group in &node_type.groups {
            let group_types = self.group_index.get(group)
                .cloned()
                .unwrap_or_else(ImVector::new);
            self.group_index = self.group_index.update(
                group.clone(),
                group_types.push_back(name.clone())
            );
        }
        
        Ok(())
    }

    /// 获取节点类型
    pub fn get_node_type(&self, name: &str) -> Option<&GraphNodeType> {
        self.node_types.get(name)
    }

    /// 获取所有节点类型
    pub fn get_all_node_types(&self) -> Vec<&GraphNodeType> {
        self.node_types.values().collect()
    }

    /// 获取特定分组的节点类型
    pub fn get_node_types_by_group(&self, group: &str) -> Vec<&GraphNodeType> {
        self.group_index.get(group)
            .map(|type_names| {
                type_names.iter()
                    .filter_map(|name| self.node_types.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 移除节点类型
    pub fn remove_node_type(&mut self, name: &str) -> PoolResult<()> {
        if let Some(node_type) = self.node_types.get(name) {
            // 从分组索引中移除
            for group in &node_type.groups {
                if let Some(group_types) = self.group_index.get(group) {
                    let updated_group_types = group_types
                        .iter()
                        .filter(|type_name| type_name != name)
                        .cloned()
                        .collect();
                    self.group_index = self.group_index.update(
                        group.clone(),
                        updated_group_types
                    );
                }
            }
            
            // 从类型集合中移除
            self.node_types = self.node_types.without(name);
        }
        
        Ok(())
    }

    /// 检查节点类型是否存在
    pub fn has_node_type(&self, name: &str) -> bool {
        self.node_types.contains_key(name)
    }

    /// 获取所有分组
    pub fn get_all_groups(&self) -> Vec<&String> {
        self.group_index.keys().collect()
    }
}

impl Default for GraphNodeTypeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::attrs::Attrs;

    fn create_test_node(id: &str, node_type: &str) -> Node {
        Node::new(
            id,
            node_type.to_string(),
            Attrs::default(),
            vec![],
            vec![],
        )
    }

    #[test]
    fn test_graph_node_type_creation() {
        let node_type = GraphNodeType::new("test_type".to_string())
            .with_description("Test node type".to_string())
            .with_group("test_group".to_string());
        
        assert_eq!(node_type.name, "test_type");
        assert_eq!(node_type.description, Some("Test node type".to_string()));
        assert!(node_type.groups.contains(&"test_group".to_string()));
    }

    #[test]
    fn test_node_type_with_attribute() {
        let attr_def = AttributeDefinition {
            name: "test_attr".to_string(),
            attribute_type: AttributeType::String,
            required: true,
            default: Some(serde_json::Value::String("default".to_string())),
            validation: None,
            description: Some("Test attribute".to_string()),
        };

        let node_type = GraphNodeType::new("test_type".to_string())
            .with_attribute("test_attr".to_string(), attr_def);
        
        assert!(node_type.attributes.contains_key("test_attr"));
        assert!(node_type.default_attributes.contains_key("test_attr"));
    }

    #[test]
    fn test_create_node() {
        let node_type = GraphNodeType::new("test_type".to_string());
        let node = node_type.create_node(None, None, ImVector::new()).unwrap();
        
        assert_eq!(node.r#type, "test_type");
        assert!(node.id.to_string().starts_with("test_type_"));
    }

    #[test]
    fn test_node_type_manager() {
        let mut manager = GraphNodeTypeManager::new();
        let node_type = GraphNodeType::new("test_type".to_string())
            .with_group("test_group".to_string());
        
        manager.register_node_type(node_type).unwrap();
        assert!(manager.has_node_type("test_type"));
        assert_eq!(manager.get_node_types_by_group("test_group").len(), 1);
    }

    #[test]
    fn test_node_validation() {
        let mut node_type = GraphNodeType::new("test_type".to_string());
        node_type.constraints = NodeConstraints::new()
            .with_children_count(Some(1), Some(3));
        
        let valid_node = create_test_node("test1", "test_type");
        let result = node_type.validate_node(&valid_node).unwrap();
        assert!(result.is_valid());
    }
}
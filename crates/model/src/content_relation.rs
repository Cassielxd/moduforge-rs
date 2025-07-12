use std::collections::HashMap;
use std::fmt::{self, Debug};
use serde::{Deserialize, Serialize};
use im::HashMap as ImHashMap;
use im::Vector as ImVector;

use crate::{
    node::Node,
    types::NodeId,
    graph::{RelationType, Relation},
    error::PoolResult,
};

/// 内容关系规则定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRelationRule {
    /// 规则名称
    pub name: String,
    /// 规则描述
    pub description: Option<String>,
    /// 源节点类型
    pub source_type: String,
    /// 目标节点类型
    pub target_type: String,
    /// 关系类型
    pub relation_type: RelationType,
    /// 关系约束
    pub constraints: RelationConstraints,
    /// 规则优先级
    pub priority: i32,
    /// 是否启用
    pub enabled: bool,
}

/// 关系约束定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationConstraints {
    /// 最小关系数量
    pub min_count: Option<usize>,
    /// 最大关系数量
    pub max_count: Option<usize>,
    /// 必需属性
    pub required_attrs: ImVector<String>,
    /// 禁止属性
    pub forbidden_attrs: ImVector<String>,
    /// 条件表达式
    pub condition: Option<String>,
    /// 循环检测
    pub allow_cycles: bool,
    /// 深度限制
    pub max_depth: Option<usize>,
}

impl RelationConstraints {
    /// 创建默认约束
    pub fn new() -> Self {
        Self {
            min_count: None,
            max_count: None,
            required_attrs: ImVector::new(),
            forbidden_attrs: ImVector::new(),
            condition: None,
            allow_cycles: false,
            max_depth: None,
        }
    }

    /// 设置数量约束
    pub fn with_count(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_count = min;
        self.max_count = max;
        self
    }

    /// 添加必需属性
    pub fn with_required_attr(mut self, attr: String) -> Self {
        self.required_attrs = self.required_attrs.push_back(attr);
        self
    }

    /// 添加禁止属性
    pub fn with_forbidden_attr(mut self, attr: String) -> Self {
        self.forbidden_attrs = self.forbidden_attrs.push_back(attr);
        self
    }

    /// 设置条件表达式
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// 设置循环检测
    pub fn with_cycle_detection(mut self, allow: bool) -> Self {
        self.allow_cycles = allow;
        self
    }

    /// 设置深度限制
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
}

impl Default for RelationConstraints {
    fn default() -> Self {
        Self::new()
    }
}

/// 内容关系验证器
#[derive(Debug)]
pub struct ContentRelationValidator {
    /// 关系规则集合
    rules: ImHashMap<String, ContentRelationRule>,
    /// 规则索引（按源类型分组）
    source_index: ImHashMap<String, ImVector<String>>,
    /// 规则索引（按目标类型分组）
    target_index: ImHashMap<String, ImVector<String>>,
}

impl ContentRelationValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self {
            rules: ImHashMap::new(),
            source_index: ImHashMap::new(),
            target_index: ImHashMap::new(),
        }
    }

    /// 添加关系规则
    pub fn add_rule(&mut self, rule: ContentRelationRule) -> PoolResult<()> {
        let rule_name = rule.name.clone();
        
        // 添加到规则集合
        self.rules = self.rules.update(rule_name.clone(), rule.clone());
        
        // 更新源类型索引
        let source_rules = self.source_index.get(&rule.source_type)
            .cloned()
            .unwrap_or_else(ImVector::new);
        self.source_index = self.source_index.update(
            rule.source_type.clone(),
            source_rules.push_back(rule_name.clone())
        );
        
        // 更新目标类型索引
        let target_rules = self.target_index.get(&rule.target_type)
            .cloned()
            .unwrap_or_else(ImVector::new);
        self.target_index = self.target_index.update(
            rule.target_type.clone(),
            target_rules.push_back(rule_name)
        );
        
        Ok(())
    }

    /// 移除关系规则
    pub fn remove_rule(&mut self, rule_name: &str) -> PoolResult<()> {
        if let Some(rule) = self.rules.get(rule_name) {
            // 从源类型索引中移除
            if let Some(source_rules) = self.source_index.get(&rule.source_type) {
                let updated_source_rules = source_rules
                    .iter()
                    .filter(|name| name != rule_name)
                    .cloned()
                    .collect();
                self.source_index = self.source_index.update(
                    rule.source_type.clone(),
                    updated_source_rules
                );
            }
            
            // 从目标类型索引中移除
            if let Some(target_rules) = self.target_index.get(&rule.target_type) {
                let updated_target_rules = target_rules
                    .iter()
                    .filter(|name| name != rule_name)
                    .cloned()
                    .collect();
                self.target_index = self.target_index.update(
                    rule.target_type.clone(),
                    updated_target_rules
                );
            }
            
            // 从规则集合中移除
            self.rules = self.rules.without(rule_name);
        }
        
        Ok(())
    }

    /// 验证关系是否合法
    pub fn validate_relation(
        &self,
        source_node: &Node,
        target_node: &Node,
        relation: &Relation,
    ) -> PoolResult<ValidationResult> {
        let source_type = &source_node.r#type;
        let target_type = &target_node.r#type;
        let relation_type = &relation.relation_type;
        
        // 查找适用的规则
        let applicable_rules = self.find_applicable_rules(source_type, target_type, relation_type);
        
        if applicable_rules.is_empty() {
            return Ok(ValidationResult::Valid);
        }
        
        // 按优先级排序规则
        let mut sorted_rules = applicable_rules;
        sorted_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // 验证每个规则
        for rule in sorted_rules {
            let result = self.validate_rule(rule, source_node, target_node, relation)?;
            if !result.is_valid() {
                return Ok(result);
            }
        }
        
        Ok(ValidationResult::Valid)
    }

    /// 查找适用的规则
    fn find_applicable_rules(
        &self,
        source_type: &str,
        target_type: &str,
        relation_type: &RelationType,
    ) -> Vec<&ContentRelationRule> {
        let mut rules = Vec::new();
        
        // 查找源类型匹配的规则
        if let Some(source_rule_names) = self.source_index.get(source_type) {
            for rule_name in source_rule_names {
                if let Some(rule) = self.rules.get(rule_name) {
                    if rule.enabled 
                        && rule.source_type == source_type 
                        && rule.target_type == target_type
                        && rule.relation_type == *relation_type {
                        rules.push(rule);
                    }
                }
            }
        }
        
        rules
    }

    /// 验证单个规则
    fn validate_rule(
        &self,
        rule: &ContentRelationRule,
        source_node: &Node,
        target_node: &Node,
        relation: &Relation,
    ) -> PoolResult<ValidationResult> {
        let constraints = &rule.constraints;
        
        // 检查属性约束
        if !self.check_attr_constraints(constraints, source_node, target_node, relation)? {
            return Ok(ValidationResult::InvalidAttribute);
        }
        
        // 检查数量约束
        if !self.check_count_constraints(constraints, source_node, target_node)? {
            return Ok(ValidationResult::InvalidCount);
        }
        
        // 检查条件约束
        if !self.check_condition_constraints(constraints, source_node, target_node, relation)? {
            return Ok(ValidationResult::InvalidCondition);
        }
        
        Ok(ValidationResult::Valid)
    }

    /// 检查属性约束
    fn check_attr_constraints(
        &self,
        constraints: &RelationConstraints,
        source_node: &Node,
        target_node: &Node,
        relation: &Relation,
    ) -> PoolResult<bool> {
        // 检查必需属性
        for required_attr in &constraints.required_attrs {
            if !source_node.attrs.contains_key(required_attr)
                && !target_node.attrs.contains_key(required_attr)
                && !relation.attrs.contains_key(required_attr) {
                return Ok(false);
            }
        }
        
        // 检查禁止属性
        for forbidden_attr in &constraints.forbidden_attrs {
            if source_node.attrs.contains_key(forbidden_attr)
                || target_node.attrs.contains_key(forbidden_attr)
                || relation.attrs.contains_key(forbidden_attr) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    /// 检查数量约束
    fn check_count_constraints(
        &self,
        constraints: &RelationConstraints,
        _source_node: &Node,
        _target_node: &Node,
    ) -> PoolResult<bool> {
        // 这里需要实现更复杂的数量检查逻辑
        // 当前简化实现
        Ok(true)
    }

    /// 检查条件约束
    fn check_condition_constraints(
        &self,
        constraints: &RelationConstraints,
        _source_node: &Node,
        _target_node: &Node,
        _relation: &Relation,
    ) -> PoolResult<bool> {
        // 这里需要实现表达式求值逻辑
        // 当前简化实现
        Ok(true)
    }

    /// 获取所有规则
    pub fn get_rules(&self) -> Vec<&ContentRelationRule> {
        self.rules.values().collect()
    }

    /// 获取特定类型的规则
    pub fn get_rules_by_source_type(&self, source_type: &str) -> Vec<&ContentRelationRule> {
        self.source_index.get(source_type)
            .map(|rule_names| {
                rule_names.iter()
                    .filter_map(|name| self.rules.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取特定目标类型的规则
    pub fn get_rules_by_target_type(&self, target_type: &str) -> Vec<&ContentRelationRule> {
        self.target_index.get(target_type)
            .map(|rule_names| {
                rule_names.iter()
                    .filter_map(|name| self.rules.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for ContentRelationValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 验证结果枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// 验证通过
    Valid,
    /// 属性验证失败
    InvalidAttribute,
    /// 数量验证失败
    InvalidCount,
    /// 条件验证失败
    InvalidCondition,
    /// 循环检测失败
    InvalidCycle,
    /// 深度验证失败
    InvalidDepth,
    /// 其他验证失败
    Invalid(String),
}

impl ValidationResult {
    /// 检查是否有效
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    /// 获取错误消息
    pub fn error_message(&self) -> Option<&str> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::InvalidAttribute => Some("属性验证失败"),
            ValidationResult::InvalidCount => Some("数量验证失败"),
            ValidationResult::InvalidCondition => Some("条件验证失败"),
            ValidationResult::InvalidCycle => Some("循环检测失败"),
            ValidationResult::InvalidDepth => Some("深度验证失败"),
            ValidationResult::Invalid(msg) => Some(msg),
        }
    }
}

/// 内容关系管理器
#[derive(Debug)]
pub struct ContentRelationManager {
    /// 验证器
    validator: ContentRelationValidator,
    /// 关系缓存
    relation_cache: ImHashMap<String, ImVector<Relation>>,
}

impl ContentRelationManager {
    /// 创建新的关系管理器
    pub fn new() -> Self {
        Self {
            validator: ContentRelationValidator::new(),
            relation_cache: ImHashMap::new(),
        }
    }

    /// 添加关系规则
    pub fn add_rule(&mut self, rule: ContentRelationRule) -> PoolResult<()> {
        self.validator.add_rule(rule)
    }

    /// 验证关系
    pub fn validate_relation(
        &self,
        source_node: &Node,
        target_node: &Node,
        relation: &Relation,
    ) -> PoolResult<ValidationResult> {
        self.validator.validate_relation(source_node, target_node, relation)
    }

    /// 缓存关系
    pub fn cache_relation(&mut self, key: String, relation: Relation) {
        let relations = self.relation_cache.get(&key)
            .cloned()
            .unwrap_or_else(ImVector::new);
        self.relation_cache = self.relation_cache.update(
            key,
            relations.push_back(relation)
        );
    }

    /// 获取缓存的关系
    pub fn get_cached_relations(&self, key: &str) -> Option<&ImVector<Relation>> {
        self.relation_cache.get(key)
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.relation_cache = ImHashMap::new();
    }

    /// 获取验证器
    pub fn validator(&self) -> &ContentRelationValidator {
        &self.validator
    }

    /// 获取可变验证器
    pub fn validator_mut(&mut self) -> &mut ContentRelationValidator {
        &mut self.validator
    }
}

impl Default for ContentRelationManager {
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
    fn test_content_relation_validator_creation() {
        let validator = ContentRelationValidator::new();
        assert!(validator.get_rules().is_empty());
    }

    #[test]
    fn test_add_rule() {
        let mut validator = ContentRelationValidator::new();
        let rule = ContentRelationRule {
            name: "test_rule".to_string(),
            description: Some("Test rule".to_string()),
            source_type: "document".to_string(),
            target_type: "paragraph".to_string(),
            relation_type: RelationType::ParentChild,
            constraints: RelationConstraints::new(),
            priority: 1,
            enabled: true,
        };
        
        validator.add_rule(rule).unwrap();
        assert_eq!(validator.get_rules().len(), 1);
    }

    #[test]
    fn test_validate_relation() {
        let mut validator = ContentRelationValidator::new();
        let rule = ContentRelationRule {
            name: "test_rule".to_string(),
            description: None,
            source_type: "document".to_string(),
            target_type: "paragraph".to_string(),
            relation_type: RelationType::ParentChild,
            constraints: RelationConstraints::new(),
            priority: 1,
            enabled: true,
        };
        
        validator.add_rule(rule).unwrap();
        
        let source_node = create_test_node("doc1", "document");
        let target_node = create_test_node("para1", "paragraph");
        let relation = Relation::new(RelationType::ParentChild);
        
        let result = validator.validate_relation(&source_node, &target_node, &relation).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_content_relation_manager() {
        let mut manager = ContentRelationManager::new();
        let rule = ContentRelationRule {
            name: "test_rule".to_string(),
            description: None,
            source_type: "document".to_string(),
            target_type: "paragraph".to_string(),
            relation_type: RelationType::ParentChild,
            constraints: RelationConstraints::new(),
            priority: 1,
            enabled: true,
        };
        
        manager.add_rule(rule).unwrap();
        assert_eq!(manager.validator().get_rules().len(), 1);
    }
}
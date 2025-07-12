use mf_model::{
    graph::{HybridGraph, GraphNode, Relation, RelationType},
    content_relation::{ContentRelationManager, ContentRelationRule, RelationConstraints},
    node_type_graph::{GraphNodeType, GraphNodeTypeManager, AttributeDefinition, AttributeType, NodeConstraints, NodeBehaviors},
    node::Node,
    types::NodeId,
    error::PoolResult,
};
use im::HashMap as ImHashMap;
use im::Vector as ImVector;
use serde_json::Value;

/// 示例：使用图结构模型创建复杂的文档结构
fn main() -> PoolResult<()> {
    println!("=== ModuForge-RS 图结构模型示例 ===\n");

    // 1. 创建节点类型管理器
    let mut type_manager = GraphNodeTypeManager::new();
    
    // 2. 定义文档节点类型
    let document_type = GraphNodeType::new("document".to_string())
        .with_description("文档根节点".to_string())
        .with_group("block".to_string())
        .with_attribute("title".to_string(), AttributeDefinition {
            name: "title".to_string(),
            attribute_type: AttributeType::String,
            required: true,
            default: Some(Value::String("Untitled Document".to_string())),
            validation: None,
            description: Some("文档标题".to_string()),
        })
        .with_constraints(NodeConstraints::new()
            .with_children_count(Some(1), None)
            .with_allowed_child("section".to_string())
            .with_allowed_child("paragraph".to_string()))
        .with_behaviors(NodeBehaviors::new()
            .with_editable(true)
            .with_deletable(false)
            .with_movable(false));

    // 3. 定义段落节点类型
    let paragraph_type = GraphNodeType::new("paragraph".to_string())
        .with_description("段落节点".to_string())
        .with_group("block".to_string())
        .with_attribute("align".to_string(), AttributeDefinition {
            name: "align".to_string(),
            attribute_type: AttributeType::String,
            required: false,
            default: Some(Value::String("left".to_string())),
            validation: Some("in:left,center,right".to_string()),
            description: Some("段落对齐方式".to_string()),
        })
        .with_constraints(NodeConstraints::new()
            .with_children_count(Some(0), None)
            .with_allowed_child("text".to_string())
            .with_allowed_child("link".to_string()))
        .with_behaviors(NodeBehaviors::new()
            .with_editable(true)
            .with_deletable(true)
            .with_movable(true));

    // 4. 定义文本节点类型
    let text_type = GraphNodeType::new("text".to_string())
        .with_description("文本节点".to_string())
        .with_group("inline".to_string())
        .with_attribute("content".to_string(), AttributeDefinition {
            name: "content".to_string(),
            attribute_type: AttributeType::String,
            required: true,
            default: None,
            validation: None,
            description: Some("文本内容".to_string()),
        })
        .with_constraints(NodeConstraints::new()
            .with_children_count(Some(0), Some(0)))
        .with_behaviors(NodeBehaviors::new()
            .with_editable(true)
            .with_deletable(true)
            .with_movable(true));

    // 5. 注册节点类型
    type_manager.register_node_type(document_type)?;
    type_manager.register_node_type(paragraph_type)?;
    type_manager.register_node_type(text_type)?;

    println!("✅ 节点类型注册完成");
    println!("注册的类型: {:?}", type_manager.get_all_node_types().iter().map(|t| &t.name).collect::<Vec<_>>());

    // 6. 创建内容关系管理器
    let mut relation_manager = ContentRelationManager::new();

    // 7. 定义关系规则
    let parent_child_rule = ContentRelationRule {
        name: "document_paragraph_rule".to_string(),
        description: Some("文档与段落的关系规则".to_string()),
        source_type: "document".to_string(),
        target_type: "paragraph".to_string(),
        relation_type: RelationType::ParentChild,
        constraints: RelationConstraints::new()
            .with_count(Some(1), None)
            .with_required_attr("title".to_string()),
        priority: 1,
        enabled: true,
    };

    let paragraph_text_rule = ContentRelationRule {
        name: "paragraph_text_rule".to_string(),
        description: Some("段落与文本的关系规则".to_string()),
        source_type: "paragraph".to_string(),
        target_type: "text".to_string(),
        relation_type: RelationType::Contains,
        constraints: RelationConstraints::new()
            .with_count(Some(1), None)
            .with_required_attr("content".to_string()),
        priority: 1,
        enabled: true,
    };

    // 8. 添加关系规则
    relation_manager.add_rule(parent_child_rule)?;
    relation_manager.add_rule(paragraph_text_rule)?;

    println!("✅ 关系规则添加完成");

    // 9. 创建混合图
    let mut graph = HybridGraph::new();

    // 10. 创建节点
    let document_node = type_manager.get_node_type("document").unwrap()
        .create_node(
            Some("doc_1".to_string()),
            Some(ImHashMap::from(vec![
                ("title".to_string(), Value::String("示例文档".to_string())),
            ])),
            ImVector::new(),
        )?;

    let paragraph_node = type_manager.get_node_type("paragraph").unwrap()
        .create_node(
            Some("para_1".to_string()),
            Some(ImHashMap::from(vec![
                ("align".to_string(), Value::String("left".to_string())),
            ])),
            ImVector::new(),
        )?;

    let text_node = type_manager.get_node_type("text").unwrap()
        .create_node(
            Some("text_1".to_string()),
            Some(ImHashMap::from(vec![
                ("content".to_string(), Value::String("这是一个示例段落。".to_string())),
            ])),
            ImVector::new(),
        )?;

    // 11. 添加节点到图
    let doc_index = graph.add_node(document_node)?;
    let para_index = graph.add_node(paragraph_node)?;
    let text_index = graph.add_node(text_node)?;

    println!("✅ 节点添加到图完成");
    println!("节点数量: {}", graph.node_count());

    // 12. 添加关系
    let parent_child_relation = Relation::new(RelationType::ParentChild)
        .with_description("文档包含段落".to_string())
        .with_weight(1.0);

    let contains_relation = Relation::new(RelationType::Contains)
        .with_description("段落包含文本".to_string())
        .with_weight(1.0);

    graph.add_relation(&"doc_1".into(), &"para_1".into(), parent_child_relation)?;
    graph.add_relation(&"para_1".into(), &"text_1".into(), contains_relation)?;

    println!("✅ 关系添加完成");
    println!("边数量: {}", graph.edge_count());

    // 13. 验证关系
    let doc_node = graph.get_node(&"doc_1".into()).unwrap();
    let para_node = graph.get_node(&"para_1".into()).unwrap();
    let text_node = graph.get_node(&"text_1".into()).unwrap();

    let parent_child_relation_for_validation = Relation::new(RelationType::ParentChild);
    let validation_result = relation_manager.validate_relation(
        &doc_node.node,
        &para_node.node,
        &parent_child_relation_for_validation,
    )?;

    println!("✅ 关系验证完成");
    println!("验证结果: {:?}", validation_result);

    // 14. 查询图结构
    println!("\n=== 图结构查询 ===");
    
    // 获取文档的子节点
    let children = graph.get_children(&"doc_1".into());
    println!("文档子节点数量: {}", children.len());
    for child in children {
        println!("  - 子节点: {} (类型: {})", child.id(), child.node_type());
    }

    // 获取段落的父节点
    if let Some(parent) = graph.get_parent(&"para_1".into()) {
        println!("段落父节点: {} (类型: {})", parent.id(), parent.node_type());
    }

    // 获取段落的邻居
    let neighbors = graph.get_neighbors(&"para_1".into());
    println!("段落邻居数量: {}", neighbors.len());
    for neighbor in neighbors {
        println!("  - 邻居: {} (类型: {})", neighbor.id(), neighbor.node_type());
    }

    // 15. 图算法示例
    println!("\n=== 图算法示例 ===");
    
    // 检查循环
    let has_cycles = graph.has_cycles();
    println!("图是否包含循环: {}", has_cycles);

    // 获取所有节点
    let all_nodes = graph.get_all_nodes();
    println!("所有节点:");
    for node in all_nodes {
        println!("  - {} (类型: {})", node.id(), node.node_type());
    }

    // 获取所有关系
    let all_relations = graph.get_all_relations();
    println!("所有关系:");
    for (source, target, relation) in all_relations {
        println!("  - {} -> {} ({})", 
            source.id(), target.id(), relation.relation_type.name());
    }

    // 16. 节点类型查询
    println!("\n=== 节点类型查询 ===");
    
    let block_types = type_manager.get_node_types_by_group("block");
    println!("块级节点类型:");
    for node_type in block_types {
        println!("  - {}: {}", node_type.name, 
            node_type.description.as_deref().unwrap_or("无描述"));
    }

    let inline_types = type_manager.get_node_types_by_group("inline");
    println!("行内节点类型:");
    for node_type in inline_types {
        println!("  - {}: {}", node_type.name, 
            node_type.description.as_deref().unwrap_or("无描述"));
    }

    // 17. 行为检查
    println!("\n=== 行为检查 ===");
    
    let document_type = type_manager.get_node_type("document").unwrap();
    println!("文档节点行为:");
    println!("  - 可编辑: {}", document_type.supports_behavior("editable"));
    println!("  - 可删除: {}", document_type.supports_behavior("deletable"));
    println!("  - 可移动: {}", document_type.supports_behavior("movable"));

    let paragraph_type = type_manager.get_node_type("paragraph").unwrap();
    println!("段落节点行为:");
    println!("  - 可编辑: {}", paragraph_type.supports_behavior("editable"));
    println!("  - 可删除: {}", paragraph_type.supports_behavior("deletable"));
    println!("  - 可移动: {}", paragraph_type.supports_behavior("movable"));

    println!("\n=== 示例完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_model_example() {
        assert!(main().is_ok());
    }
}
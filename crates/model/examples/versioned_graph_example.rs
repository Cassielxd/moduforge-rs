use mf_model::{
    versioned_graph::{VersionedGraph, GraphSnapshot},
    graph_node_type::{GraphNodeType, ChildConstraint, CreationRule, CreationCondition, CreationAction, ValidationRule, ValidationCondition},
    graph::{Relation, RelationType},
    node::Node,
    node_type::NodeSpec,
    attrs::Attrs,
    types::NodeId,
    id_generator::IdGenerator,
    schema::Schema,
};
use im::HashMap as ImHashMap;
use im::Vector as ImVector;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ModuForge-RS 版本化图模型示例");
    println!("=====================================");

    // 1. 创建版本化图
    let mut graph = VersionedGraph::new()
        .with_max_snapshots(50);

    println!("✅ 创建版本化图成功");

    // 2. 定义图节点类型
    let mut document_type = GraphNodeType::new("document".to_string(), NodeSpec::default());
    document_type.add_group("block".to_string());
    document_type.set_default_attribute("title".to_string(), Value::String("Untitled Document".to_string()));

    // 添加子节点约束
    let mut paragraph_constraint = ChildConstraint {
        allowed_types: ImVector::new().push_back("paragraph".to_string()),
        forbidden_types: ImVector::new(),
        min_count: 1,
        max_count: None,
        relation_type: RelationType::ParentChild,
        required: true,
    };
    document_type.add_child_constraint("paragraphs".to_string(), paragraph_constraint);

    // 添加创建规则：当文档没有段落时，自动创建一个段落
    let creation_rule = CreationRule {
        name: "auto_create_paragraph".to_string(),
        condition: CreationCondition::MissingChildType("paragraph".to_string()),
        action: CreationAction::CreateChild("paragraph".to_string()),
        priority: 1,
        enabled: true,
    };
    document_type.add_creation_rule(creation_rule);

    // 添加验证规则：确保文档至少有一个段落
    let validation_rule = ValidationRule {
        name: "require_paragraph".to_string(),
        condition: ValidationCondition::RequiredChildType("paragraph".to_string()),
        error_message: "Document must have at least one paragraph".to_string(),
        priority: 1,
        enabled: true,
    };
    document_type.add_validation_rule(validation_rule);

    let mut paragraph_type = GraphNodeType::new("paragraph".to_string(), NodeSpec::default());
    paragraph_type.add_group("block".to_string());
    paragraph_type.set_default_attribute("style".to_string(), Value::String("normal".to_string()));

    // 段落可以包含文本节点
    let mut text_constraint = ChildConstraint {
        allowed_types: ImVector::new().push_back("text".to_string()),
        forbidden_types: ImVector::new(),
        min_count: 0,
        max_count: None,
        relation_type: RelationType::ParentChild,
        required: false,
    };
    paragraph_type.add_child_constraint("text".to_string(), text_constraint);

    let mut text_type = GraphNodeType::new("text".to_string(), NodeSpec::default());
    text_type.add_group("inline".to_string());
    text_type.set_default_attribute("content".to_string(), Value::String("".to_string()));

    // 3. 创建节点类型映射
    let mut node_types = ImHashMap::new();
    node_types = node_types.update("document".to_string(), document_type);
    node_types = node_types.update("paragraph".to_string(), paragraph_type);
    node_types = node_types.update("text".to_string(), text_type);

    println!("✅ 定义图节点类型成功");

    // 4. 创建文档节点（递归创建）
    let schema = Schema::default();
    let document_id = node_types.get("document").unwrap().create_and_fill(
        &mut graph,
        Some(NodeId::from("doc_1")),
        Some(&ImHashMap::new().update("title".to_string(), Value::String("My Document".to_string()))),
        vec![], // 空内容，让创建规则自动填充
        None,
        &schema,
        &node_types,
    )?;

    println!("✅ 创建文档节点成功，ID: {}", document_id);

    // 5. 创建快照
    let snapshot = graph.create_snapshot(Some("Initial document creation".to_string()))?;
    println!("✅ 创建快照成功，版本: {}", snapshot.version());

    // 6. 添加段落节点
    let paragraph_id = node_types.get("paragraph").unwrap().create_and_fill(
        &mut graph,
        Some(NodeId::from("para_1")),
        Some(&ImHashMap::new().update("style".to_string(), Value::String("heading".to_string()))),
        vec![],
        None,
        &schema,
        &node_types,
    )?;

    // 建立父子关系
    let relation = Relation::new(RelationType::ParentChild);
    graph.add_relation(&document_id, &paragraph_id, relation)?;

    println!("✅ 添加段落节点成功，ID: {}", paragraph_id);

    // 7. 添加文本节点
    let text_id = node_types.get("text").unwrap().create_and_fill(
        &mut graph,
        Some(NodeId::from("text_1")),
        Some(&ImHashMap::new().update("content".to_string(), Value::String("Hello, World!".to_string()))),
        vec![],
        None,
        &schema,
        &node_types,
    )?;

    // 建立父子关系
    let relation = Relation::new(RelationType::ParentChild);
    graph.add_relation(&paragraph_id, &text_id, relation)?;

    println!("✅ 添加文本节点成功，ID: {}", text_id);

    // 8. 创建第二个快照
    let snapshot2 = graph.create_snapshot(Some("Added paragraph and text".to_string()))?;
    println!("✅ 创建第二个快照成功，版本: {}", snapshot2.version());

    // 9. 显示图信息
    println!("\n📊 图信息:");
    println!("  节点数量: {}", graph.node_count());
    println!("  边数量: {}", graph.edge_count());
    println!("  当前版本: {}", graph.current_version());
    println!("  快照数量: {}", graph.get_snapshots().len());

    // 10. 显示所有节点
    println!("\n📋 所有节点:");
    for node in graph.get_all_nodes() {
        println!("  - {} (类型: {})", node.id(), node.node_type());
    }

    // 11. 显示所有关系
    println!("\n🔗 所有关系:");
    for (source, target, relation) in graph.get_all_relations() {
        println!("  - {} -> {} ({})", source.id(), target.id(), relation.relation_type.name());
    }

    // 12. 验证节点
    println!("\n✅ 验证节点:");
    for node in graph.get_all_nodes() {
        if let Some(node_type) = node_types.get(node.node_type()) {
            match node_type.validate_node(&graph, node.id()) {
                Ok(_) => println!("  - {} 验证通过", node.id()),
                Err(e) => println!("  - {} 验证失败: {}", node.id(), e),
            }
        }
    }

    // 13. 恢复到第一个快照
    println!("\n⏪ 恢复到第一个快照...");
    graph.restore_snapshot(snapshot.version())?;
    println!("✅ 恢复成功");
    println!("  节点数量: {}", graph.node_count());
    println!("  边数量: {}", graph.edge_count());

    // 14. 显示版本历史
    println!("\n📜 版本历史:");
    for snapshot in graph.get_snapshots().iter() {
        println!("  - 版本 {}: {} ({})", 
            snapshot.version(), 
            snapshot.description().unwrap_or("无描述"),
            snapshot.timestamp().format("%Y-%m-%d %H:%M:%S")
        );
    }

    println!("\n🎉 示例完成！");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_graph_workflow() {
        let mut graph = VersionedGraph::new();
        
        // 创建节点类型
        let mut document_type = GraphNodeType::new("document".to_string(), NodeSpec::default());
        document_type.add_group("block".to_string());
        
        let mut node_types = ImHashMap::new();
        node_types = node_types.update("document".to_string(), document_type);
        
        // 创建文档节点
        let schema = Schema::default();
        let document_id = node_types.get("document").unwrap().create_and_fill(
            &mut graph,
            Some(NodeId::from("test_doc")),
            None,
            vec![],
            None,
            &schema,
            &node_types,
        ).unwrap();
        
        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains_node(&document_id));
        
        // 创建快照
        let snapshot = graph.create_snapshot(Some("Test snapshot".to_string())).unwrap();
        assert_eq!(snapshot.version(), 1);
        
        // 添加更多节点
        let paragraph_id = node_types.get("document").unwrap().create_and_fill(
            &mut graph,
            Some(NodeId::from("test_para")),
            None,
            vec![],
            None,
            &schema,
            &node_types,
        ).unwrap();
        
        assert_eq!(graph.node_count(), 2);
        
        // 恢复到快照
        graph.restore_snapshot(snapshot.version()).unwrap();
        assert_eq!(graph.node_count(), 1);
    }
}
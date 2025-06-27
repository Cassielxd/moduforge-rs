use moduforge_transform::{
    attr_step::AttrStep,
    mark_step::{AddMarkStep, RemoveMarkStep},
    node_step::{AddNodeStep, RemoveNodeStep},
    step::Step,
};
use moduforge_model::{mark::Mark, node::Node, tree::Tree};
use std::{any::TypeId, collections::HashMap};

use crate::{NodeData, MarkData, RoomSnapshot, StepResult};
use serde_json::Value as JsonValue;
use yrs::{
    types::{
        array::ArrayRef,
        map::MapRef,
        Value,
    },
    Array, ArrayPrelim, Map, MapPrelim, TransactionMut, WriteTxn,
};

/// Trait for converting a `Step` into a Yrs transaction.
/// This trait is dyn-safe.
pub trait StepConverter: Send + Sync {
    /// Applies the step to a Yrs document transaction.
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>>;

    /// Returns the name of the converter.
    fn name(&self) -> &'static str;

    /// Checks if this converter supports the given step type.
    fn supports(&self, step: &dyn Step) -> bool;

    /// Gets a description of the step's operation.
    fn get_description(&self, step: &dyn Step) -> String {
        format!("Executing operation: {} ({})", step.name(), self.name())
    }
}

/// Default Step converter for unsupported steps.
pub struct DefaultStepConverter;

impl StepConverter for DefaultStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        Ok(StepResult {
            step_id: uuid::Uuid::new_v4().to_string(),
            step_name: step.name().to_string(),
            description: format!("Default handler processed unknown step: {}", step.name()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            client_id: txn.origin().as_ref().map(|s| s.to_string()).unwrap_or_default(),
        })
    }

    fn name(&self) -> &'static str {
        "DefaultStepConverter"
    }

    fn supports(&self, _step: &dyn Step) -> bool {
        true // Default converter supports all types as a fallback.
    }
}

// --- Helper methods for converters ---

fn json_value_to_yrs_any(value: &JsonValue) -> yrs::Any {
    match value {
        JsonValue::Null => yrs::Any::Null,
        JsonValue::Bool(b) => yrs::Any::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                yrs::Any::BigInt(i)
            } else if let Some(f) = n.as_f64() {
                yrs::Any::Number(f)
            } else {
                yrs::Any::Null
            }
        }
        JsonValue::String(s) => yrs::Any::String(s.clone().into()),
        JsonValue::Array(arr) => {
            let yrs_array: Vec<yrs::Any> = arr.iter().map(json_value_to_yrs_any).collect();
            yrs::Any::Array(yrs_array.into())
        }
        JsonValue::Object(obj) => {
            let yrs_map: std::collections::HashMap<String, yrs::Any> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_yrs_any(v)))
                .collect();
            yrs::Any::Map(yrs_map.into())
        }
    }
}

fn add_mark_to_array(
    marks_array: &ArrayRef,
    txn: &mut TransactionMut,
    mark: &Mark,
) {
    let mark_map = MapPrelim::<yrs::Any>::from([
        ("type".to_string(), yrs::Any::String(mark.r#type.clone().into())),
        ("attrs".to_string(), {
            let attrs_map: std::collections::HashMap<String, yrs::Any> = mark.attrs.iter()
                .map(|(k, v)| (k.clone(), json_value_to_yrs_any(v)))
                .collect();
            yrs::Any::Map(attrs_map.into())
        }),
    ]);
    marks_array.push_back(txn, mark_map);
}

fn get_or_create_node_data_map(
    nodes_map: &MapRef,
    txn: &mut TransactionMut,
    node_id: &str,
) -> MapRef {
    if let Some(Value::YMap(map)) = nodes_map.get(txn, node_id) {
        map
    } else {
        nodes_map.insert(txn, node_id.to_string(), MapPrelim::<yrs::Any>::new())
    }
}

fn get_or_create_node_attrs_map(
    node_data_map: &MapRef,
    txn: &mut TransactionMut,
) -> MapRef {
    if let Some(Value::YMap(map)) = node_data_map.get(txn, "attrs") {
        map
    } else {
            node_data_map.insert(txn, "attrs", MapPrelim::<yrs::Any>::new())
    }
}

fn get_or_create_marks_array(
    node_data_map: &MapRef,
    txn: &mut TransactionMut,
) -> ArrayRef {
    if let Some(Value::YArray(array)) = node_data_map.get(txn, "marks") {
        array
    } else {
        node_data_map.insert(txn, "marks", ArrayPrelim::from(Vec::<yrs::Any>::new()))
    }
}


/// Converter for node-related steps.
pub struct NodeStepConverter;

impl NodeStepConverter {
    fn insert_node_data(&self, txn: &mut TransactionMut, nodes_map: &MapRef, node: &Node) {
        let node_data_map = get_or_create_node_data_map(nodes_map, txn, &node.id);
        node_data_map.insert(txn, "type", node.r#type.clone());

        let attrs_map = node_data_map.insert(txn, "attrs", MapPrelim::<yrs::Any>::new());
        for (key, value) in node.attrs.iter() {
            attrs_map.insert(txn, key.clone(), json_value_to_yrs_any(value));
        }

        let content_array = node_data_map.insert(txn, "content", ArrayPrelim::from(Vec::<yrs::Any>::new()));
        for child_id in &node.content {
            content_array.push_back(txn, yrs::Any::String(child_id.clone().into()));
        }

        let marks_array = node_data_map.insert(txn, "marks", ArrayPrelim::from(Vec::<yrs::Any>::new()));
        for mark in &node.marks {
            add_mark_to_array(&marks_array, txn, mark);
        }
    }
}

impl StepConverter for NodeStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        let client_id = txn.origin().as_ref().map(|s| s.to_string()).unwrap_or_default();

        if let Some(add_step) = step.downcast_ref::<AddNodeStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let all_nodes = &add_step.nodes;

            for node_enum in all_nodes {
                self.insert_node_data(txn, &nodes_map, &node_enum.0);
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("Added {} nodes", all_nodes.len()),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            });
        } else if let Some(remove_step) = step.downcast_ref::<RemoveNodeStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            for node_id in &remove_step.node_ids {
                nodes_map.remove(txn, &node_id.to_string());
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("Removed {} nodes", remove_step.node_ids.len()),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            });
        }

        Err("Unsupported node operation".into())
    }

    fn name(&self) -> &'static str {
        "NodeStepConverter"
    }

    fn supports(&self, step: &dyn Step) -> bool {
        step.type_id() == TypeId::of::<AddNodeStep>() || step.type_id() == TypeId::of::<RemoveNodeStep>()
    }
}

/// Converter for attribute-related steps.
pub struct AttrStepConverter;

impl StepConverter for AttrStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        let client_id = txn.origin().as_ref().map(|s| s.to_string()).unwrap_or_default();
        if let Some(attr_step) = step.downcast_ref::<AttrStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let node_data_map = get_or_create_node_data_map(&nodes_map, txn, &attr_step.id);
            let attrs_map = get_or_create_node_attrs_map(&node_data_map, txn);

            for (key, value) in attr_step.values.iter() {
                attrs_map.insert(txn, key.clone(), json_value_to_yrs_any(value));
            }

            Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!(
                    "Updated {} attributes for node {}",
                    attr_step.values.len(),
                    attr_step.id
                ),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            })
        } else {
            Err("Unsupported attribute operation".into())
        }
    }

    fn name(&self) -> &'static str {
        "AttrStepConverter"
    }

    fn supports(&self, step: &dyn Step) -> bool {
        step.type_id() == TypeId::of::<AttrStep>()
    }
}

/// Converter for mark-related steps.
pub struct MarkStepConverter;

impl MarkStepConverter {
    /// Removes a mark from a Yrs array by its type.
    fn remove_mark_from_array(
        &self,
        marks_array: &ArrayRef,
        txn: &mut TransactionMut,
        mark_type_to_remove: &str,
    ) {
        let len = marks_array.len(txn);
        for i in (0..len).rev() {
            if let Some(Value::YMap(mark_map)) = marks_array.get(txn, i) {
                if let Some(mark_type_value) = mark_map.get(txn, "type") {
                    let mark_type = match mark_type_value {
                        Value::YText(_text_ref) => {
                            // For now, skip text refs as we can't easily extract string
                            continue;
                        },
                        Value::Any(any) => any.to_string(),
                        _ => continue,
                    };
                    if mark_type == mark_type_to_remove {
                        marks_array.remove(txn, i);
                        return; 
                    }
                }
            }
        }
    }
}

impl StepConverter for MarkStepConverter {
    fn apply_to_yrs_txn(
        &self,
        step: &dyn Step,
        txn: &mut TransactionMut,
    ) -> Result<StepResult, Box<dyn std::error::Error>> {
        let client_id = txn.origin().as_ref().map(|s| s.to_string()).unwrap_or_default();

        if let Some(add_mark_step) = step.downcast_ref::<AddMarkStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let node_data_map =
                get_or_create_node_data_map(&nodes_map, txn, &add_mark_step.id);
            let marks_array = get_or_create_marks_array(&node_data_map, txn);
            
            for mark in &add_mark_step.marks {
                 add_mark_to_array(&marks_array, txn, mark);
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("Added {} marks to node {}", add_mark_step.marks.len(), add_mark_step.id),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            });
        } else if let Some(remove_mark_step) = step.downcast_ref::<RemoveMarkStep>() {
            let nodes_map = txn.get_or_insert_map("nodes");
            let node_data_map =
                get_or_create_node_data_map(&nodes_map, txn, &remove_mark_step.id);
            let marks_array = get_or_create_marks_array(&node_data_map, txn);
            
            for mark_type in &remove_mark_step.mark_types {
                self.remove_mark_from_array(&marks_array, txn, mark_type);
            }

            return Ok(StepResult {
                step_id: uuid::Uuid::new_v4().to_string(),
                step_name: step.name().to_string(),
                description: format!("Removed {} marks from node {}", remove_mark_step.mark_types.len(), remove_mark_step.id),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                client_id,
            });
        }

        Err("Unsupported mark operation".into())
    }

    fn name(&self) -> &'static str {
        "MarkStepConverter"
    }

    fn supports(&self, step: &dyn Step) -> bool {
        step.type_id() == TypeId::of::<AddMarkStep>() || step.type_id() == TypeId::of::<RemoveMarkStep>()
    }
}

/// Registry for all step converters.
pub struct StepConverterRegistry {
    converters: Vec<Box<dyn StepConverter>>,
}

impl Default for StepConverterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StepConverterRegistry {
    /// Creates a new registry with all default converters.
    pub fn new() -> Self {
        let mut registry = Self {
            converters: Vec::new(),
        };

        registry.register(Box::new(NodeStepConverter));
        registry.register(Box::new(AttrStepConverter));
        registry.register(Box::new(MarkStepConverter));
        registry.register(Box::new(DefaultStepConverter));

        registry
    }

    /// Registers a new converter.
    pub fn register(&mut self, converter: Box<dyn StepConverter>) {
        tracing::info!("Registering Step converter: {}", converter.name());
        self.converters.push(converter);
    }

    /// Finds a converter that supports the given step.
    pub fn find_converter(&self, step: &dyn Step) -> Option<&(dyn StepConverter)> {
        for converter in &self.converters {
            if converter.supports(step) {
                return Some(converter.as_ref());
            }
        }
        None
    }
}

/// Global mapper for handling conversions.
#[derive(Debug)]
pub struct Mapper;

impl Mapper {
    /// Gets the global singleton instance of the converter registry.
    pub fn global_registry() -> &'static StepConverterRegistry {
        use std::sync::OnceLock;
        static REGISTRY: OnceLock<StepConverterRegistry> = OnceLock::new();
        REGISTRY.get_or_init(StepConverterRegistry::new)
    }

    /// Converts a ModuForge `Tree` into a `RoomSnapshot`.
    pub fn tree_to_snapshot(tree: &Tree, room_id: String) -> RoomSnapshot {
        let mut nodes = HashMap::new();

        fn collect_nodes(tree: &Tree, node_id: &str, nodes: &mut HashMap<String, NodeData>) {
            if let Some(node) = tree.get_node(&node_id.to_string()) {
                let node_data = NodeData {
                    id: node.id.clone(),
                    node_type: node.r#type.clone(),
                    attrs: node.attrs.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    content: node.content.iter().cloned().collect(),
                    marks: node
                        .marks
                        .iter()
                        .map(|mark| MarkData {
                            mark_type: mark.r#type.clone(),
                            attrs: mark.attrs.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                        })
                        .collect(),
                };

                nodes.insert(node_id.to_string(), node_data);

                for child_id in &node.content {
                    collect_nodes(tree, child_id, nodes);
                }
            }
        }

        collect_nodes(tree, &tree.root_id, &mut nodes);

        RoomSnapshot {
            room_id,
            root_id: tree.root_id.clone(),
            nodes,
            version: 0, 
        }
    }
}

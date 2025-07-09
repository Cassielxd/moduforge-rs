# ModuForge-RS API Documentation

## Table of Contents

1. [Overview](#overview)
2. [Core Module (moduforge-core)](#core-module-moduforge-core)
3. [Model Module (moduforge-model)](#model-module-moduforge-model)
4. [State Module (moduforge-state)](#state-module-moduforge-state)
5. [Transform Module (moduforge-transform)](#transform-module-moduforge-transform)
6. [Rules Engine (moduforge-rules-engine)](#rules-engine-moduforge-rules-engine)
7. [Expression System (moduforge-rules-expression)](#expression-system-moduforge-rules-expression)
8. [Collaboration System (moduforge-collaboration)](#collaboration-system-moduforge-collaboration)
9. [Template System (moduforge-template)](#template-system-moduforge-template)
10. [Macro System (moduforge-macro)](#macro-system-moduforge-macro)
11. [Examples and Usage Patterns](#examples-and-usage-patterns)

## Overview

ModuForge-RS is a comprehensive state management and data transformation framework built in Rust. It provides a complete suite of tools for building sophisticated document editors, rule engines, and collaborative applications with immutable data structures and event-driven architecture.

### Key Features

- **Immutable Data Structures**: Built on `im-rs` for persistent, efficient data structures
- **Event-Driven Architecture**: All state changes emit events for reactive programming
- **Plugin System**: Extensible architecture with dynamic plugin loading
- **Transaction System**: ACID-compliant transactions for data consistency
- **Rules Engine**: Business rule evaluation based on GoRules JDM standard
- **Real-time Collaboration**: Built-in support for multi-user editing via Yrs
- **Type Safety**: Comprehensive type system with compile-time guarantees

---

## Core Module (moduforge-core)

The core module provides the foundational functionality for the ModuForge framework.

### Main Exports

```rust
pub use error::{ForgeResult, error_utils};
pub mod model {
    pub use mf_model::*;
}
pub mod state {
    pub use mf_state::*;
}
pub mod transform {
    pub use mf_transform::*;
}
```

### Key Components

#### 1. Async Runtime

```rust
use mf_core::async_runtime::{AsyncRuntime, RuntimeOptions};
use mf_core::async_processor::AsyncProcessor;
use mf_core::middleware::MiddlewareStack;

// Create runtime configuration
let mut options = RuntimeOptions::default();
options.set_middleware_stack(MiddlewareStack::new());

// Initialize async runtime
let runtime = AsyncRuntime::new(options).await?;

// Process tasks asynchronously
let processor = AsyncProcessor::new();
processor.process_task(|state| async move {
    // Your async processing logic here
    Ok(())
}).await?;
```

#### 2. Event System

```rust
use mf_core::event::{Event, EventHandler, EventBus};
use async_trait::async_trait;

#[derive(Debug)]
struct CustomEventHandler;

#[async_trait]
impl EventHandler<Event> for CustomEventHandler {
    async fn handle(&self, event: &Event) -> ForgeResult<()> {
        match event {
            Event::Create(state) => {
                println!("State created: version {}", state.version);
            }
            Event::TrApply(tr_id, transactions, state) => {
                println!("Transaction applied: {}", tr_id);
            }
            _ => {}
        }
        Ok(())
    }
}

// Create event bus and add handler
let event_bus = EventBus::<Event>::new();
event_bus.add_event_handler(Arc::new(CustomEventHandler))?;

// Start event loop
event_bus.start_event_loop();

// Broadcast events
event_bus.broadcast(Event::Create(Arc::new(state))).await?;
```

#### 3. Extension System

```rust
use mf_core::extension::{Extension, ExtensionManager};
use mf_core::extension_manager::ExtensionManager;

// Create extension manager
let mut extension_manager = ExtensionManager::new();

// Add extension
let mut extension = Extension::new();
extension.add_plugin(plugin); // Add your plugins

extension_manager.add_extension(extension);

// Process with extensions
extension_manager.process_transaction(&transaction).await?;
```

#### 4. Middleware System

```rust
use mf_core::middleware::{Middleware, MiddlewareStack};
use mf_state::{State, Transaction};
use async_trait::async_trait;

#[derive(Debug)]
struct LoggingMiddleware {
    name: String,
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn before_dispatch(&self, transaction: &mut Transaction) -> ForgeResult<()> {
        println!("Processing transaction: {}", transaction.id);
        Ok(())
    }

    async fn after_dispatch(
        &self, 
        state: Option<Arc<State>>, 
        transactions: &[Transaction]
    ) -> ForgeResult<Option<Transaction>> {
        println!("Transaction completed successfully");
        Ok(None)
    }
}

// Usage
let mut middleware_stack = MiddlewareStack::new();
middleware_stack.add(LoggingMiddleware { 
    name: "Logger".to_string() 
});
```

#### 5. Node System

```rust
use mf_core::node::Node;

// Create core node specification
let mut attrs = HashMap::new();
attrs.insert("align".to_string(), AttributeSpec { 
    default: Some(Value::String("left".to_string())) 
});

let spec = NodeSpec {
    content: Some("inline".to_string()),
    attrs: Some(attrs),
    desc: Some("Paragraph node".to_string()),
    ..Default::default()
};

let node = Node::create("paragraph", spec);
```

---

## Model Module (moduforge-model)

The model module defines the core data structures used throughout the framework.

### Key Components

#### 1. Node System

```rust
use mf_model::node::Node;
use mf_model::attrs::Attrs;
use mf_model::types::NodeId;

// Create a new node
let node = Node::new(
    "node_1",                    // id
    "paragraph".to_string(),     // node_type
    Attrs::default(),            // attributes
    vec![],                      // children IDs
    vec![]                       // marks
);

// Access node properties
println!("Node ID: {}", node.id);
println!("Node type: {}", node.node_type);
println!("Child count: {}", node.child_count());
```

#### 2. Node Types and Specs

```rust
use mf_model::node_type::{NodeType, NodeSpec, NodeEnum};
use mf_model::schema::{AttributeSpec, SchemaSpec};

// Define node specification
let spec = NodeSpec {
    content: Some("block+".to_string()),   // Content expression
    attrs: Some({
        let mut attrs = HashMap::new();
        attrs.insert("level".to_string(), AttributeSpec {
            default: Some(Value::Number(1.into()))
        });
        attrs
    }),
    marks: Some("_".to_string()),          // Mark expression
    desc: Some("Heading node".to_string()),
    ..Default::default()
};

// Create node type
let node_type = NodeType::new("heading", spec);

// Compile and validate
let compiled_type = NodeType::compile("heading", spec)?;
```

#### 3. Attributes System

```rust
use mf_model::attrs::{Attrs, FilteredAttrs};
use serde_json::{Value, json};

// Create attributes
let mut attrs = Attrs::default();
attrs.set("align", json!("center"));
attrs.set("indent", json!(2));

// Get typed value
let align: String = attrs.get_value("align")?;
let indent: i32 = attrs.get_value("indent")?;

// Update attributes
let new_attrs = attrs.update(json!({
    "color": "red",
    "bold": true
}));

// Filter attributes
let filtered = FilteredAttrs::new(&attrs, &["align", "color"]);
```

#### 4. Mark System

```rust
use mf_model::mark::{Mark, MarkType};
use mf_model::mark_type::MarkSpec;

// Create mark specification
let mark_spec = MarkSpec {
    attrs: Some({
        let mut attrs = HashMap::new();
        attrs.insert("href".to_string(), AttributeSpec { 
            default: None 
        });
        attrs
    }),
    inclusive: Some(false),
    ..Default::default()
};

// Create mark type
let mark_type = MarkType::create("link", mark_spec);

// Create mark instance
let mark = Mark::new("link", json!({"href": "https://example.com"}));
```

#### 5. Tree and Node Pool

```rust
use mf_model::tree::Tree;
use mf_model::node_pool::NodePool;
use mf_model::node_type::NodeEnum;

// Create tree from node hierarchy
let root_node = Node::new("root", "document".to_string(), Attrs::default(), vec![], vec![]);
let child_node = Node::new("child1", "paragraph".to_string(), Attrs::default(), vec![], vec![]);

let node_enum = NodeEnum(root_node, vec![
    NodeEnum(child_node, vec![])
]);

// Create tree and node pool
let tree = Tree::from(node_enum);
let node_pool = NodePool::new(Arc::new(tree));

// Query nodes
let root = node_pool.root();
let children = node_pool.children(&root.id);
let node_by_id = node_pool.get_node(&NodeId::from("child1"));

// Tree operations
let mut tree = Tree::new(root_node);
tree.add_node(parent_id, child_node, None)?;
tree.move_node(&node_id, &new_parent_id, Some(1))?;
tree.remove_node(&node_id)?;

// Update operations
tree.update_attr(&node_id, "color", json!("blue"))?;
tree.add_mark(&node_id, mark)?;
tree.remove_mark(&node_id, &mark_name)?;
```

#### 6. Schema System

```rust
use mf_model::schema::{Schema, SchemaSpec, AttributeSpec};

// Define schema specification
let schema_spec = SchemaSpec {
    nodes: {
        let mut nodes = HashMap::new();
        nodes.insert("paragraph".to_string(), NodeSpec {
            content: Some("inline*".to_string()),
            attrs: Some({
                let mut attrs = HashMap::new();
                attrs.insert("align".to_string(), AttributeSpec {
                    default: Some(json!("left"))
                });
                attrs
            }),
            ..Default::default()
        });
        nodes
    },
    marks: HashMap::new(),
    ..Default::default()
};

// Compile schema
let schema = Schema::compile(schema_spec)?;

// Validate content
let is_valid = schema.check_content(&node_type, &content)?;
```

---

## State Module (moduforge-state)

The state module provides state management, transaction processing, and plugin system.

### Key Components

#### 1. State Management

```rust
use mf_state::{State, StateConfig, Configuration};
use mf_core::extension::Extension;

// Create state configuration
let config = StateConfig {
    schema: Some(schema),
    extension: Some(extension),
    ..Default::default()
};

// Create initial state
let state = State::create(config).await?;

// Access state properties
println!("State version: {}", state.version);
println!("Node count: {}", state.node_pool.len());

// Get resource manager
let resource_manager = state.resource_manager();
```

#### 2. Transaction System

```rust
use mf_state::Transaction;
use mf_transform::step::Step;
use mf_transform::node_step::AddNodeStep;

// Create transaction
let mut transaction = Transaction::new();

// Add steps to transaction
let add_step = AddNodeStep::new(node, Some(parent_id));
transaction.add_step(add_step);

// Set transaction metadata
transaction.set_meta("user_id", "user_123");
transaction.set_meta("action", "add_paragraph");

// Apply transaction
let new_state = state.apply_transaction(transaction).await?;
```

#### 3. Plugin System

```rust
use mf_state::plugin::{Plugin, PluginSpec, PluginTrait, StateField};
use async_trait::async_trait;

#[derive(Debug)]
struct HistoryPlugin {
    max_history: usize,
}

#[async_trait]
impl PluginTrait for HistoryPlugin {
    async fn init(
        &self,
        config: &StateConfig,
        instance: Option<&State>,
    ) -> StateResult<PluginState> {
        // Initialize plugin state
        let state = Arc::new(HashMap::new());
        Ok(state)
    }

    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // Handle transaction completion
        Ok(None)
    }
}

// Create and register plugin
let plugin_spec = PluginSpec {
    key: ("history".to_string(), "v1".to_string()),
    tr: Some(Arc::new(HistoryPlugin { max_history: 50 })),
    priority: 10,
    state_field: Some(StateField::new("history_data")),
};

let plugin = Plugin::new(plugin_spec);
extension.add_plugin(Arc::new(plugin));
```

#### 4. Resource Management

```rust
use mf_state::resource::{Resource, ResourceManager};
use std::borrow::Cow;

// Define custom resource
#[derive(Debug)]
struct CacheManager {
    data: HashMap<String, String>,
}

impl Resource for CacheManager {
    fn name(&self) -> Cow<str> {
        "CacheManager".into()
    }
}

// Register resource
let resource_manager = state.resource_manager();
let mut rm = resource_manager.write().unwrap();
rm.resource_table.add(CacheManager {
    data: HashMap::new(),
});

// Access resource
let cache = rm.resource_table.get::<CacheManager>(0)?;
```

#### 5. Logging System

```rust
use mf_state::init_logging;
use tracing::{info, debug, warn, error};

// Initialize logging
init_logging("info", Some("logs/moduforge.log"))?;

// Use logging
info!("Application started");
debug!("Debug information");
warn!("Warning message");
error!("Error occurred");
```

---

## Transform Module (moduforge-transform)

The transform module handles document transformations through a step-based system.

### Key Components

#### 1. Transform System

```rust
use mf_transform::transform::Transform;
use mf_transform::step::Step;

// Create transform
let mut transform = Transform::new(node_pool);

// Apply single step
let step = AddNodeStep::new(node, parent_id);
transform.step(step)?;

// Apply multiple steps
let steps: Vec<Box<dyn Step>> = vec![
    Box::new(AddNodeStep::new(node1, parent_id)),
    Box::new(AddNodeStep::new(node2, parent_id)),
];
transform.apply_steps_batch(steps)?;

// Check if document changed
if transform.doc_changed() {
    println!("Document was modified");
}

// Commit or rollback changes
transform.commit();
// or
transform.rollback();
```

#### 2. Node Steps

```rust
use mf_transform::node_step::{AddNodeStep, RemoveNodeStep, MoveNodeStep};

// Add node step
let add_step = AddNodeStep::new(
    node,                        // Node to add
    Some(parent_id),            // Parent node ID (optional)
);

// Remove node step
let remove_step = RemoveNodeStep::new(node_id);

// Move node step  
let move_step = MoveNodeStep::new(
    node_id,                    // Node to move
    new_parent_id,              // New parent
    Some(2),                    // Position index (optional)
);

// Execute steps
transform.step(add_step)?;
transform.step(remove_step)?;
transform.step(move_step)?;
```

#### 3. Mark Steps

```rust
use mf_transform::mark_step::{AddMarkStep, RemoveMarkStep};

// Add mark step
let add_mark_step = AddMarkStep::new(
    node_id,                    // Target node
    mark,                       // Mark to add
);

// Remove mark step
let remove_mark_step = RemoveMarkStep::new(
    node_id,                    // Target node  
    mark_name,                  // Mark name to remove
);

transform.step(add_mark_step)?;
transform.step(remove_mark_step)?;
```

#### 4. Attribute Steps

```rust
use mf_transform::attr_step::AttrStep;

// Update attributes
let attr_step = AttrStep::new(
    node_id,                    // Target node
    json!({                     // New attributes
        "align": "center",
        "color": "red"
    }),
);

transform.step(attr_step)?;
```

#### 5. Patch System

```rust
use mf_transform::patch::Patch;

// Create patch from changes
let patch = Patch::from_changes(&old_tree, &new_tree);

// Apply patch
let patched_tree = patch.apply(&base_tree)?;

// Patches can represent:
enum Patch {
    AddNode { node: Node, parent: Option<NodeId>, position: Option<usize> },
    RemoveNode { node_id: NodeId },
    MoveNode { node_id: NodeId, new_parent: NodeId, position: Option<usize> },
    UpdateAttrs { node_id: NodeId, attrs: Value },
    AddMark { node_id: NodeId, mark: Mark },
    RemoveMark { node_id: NodeId, mark_name: String },
}
```

---

## Rules Engine (moduforge-rules-engine)

A business-friendly rules engine based on the GoRules JSON Decision Model (JDM) standard.

### Key Components

#### 1. Decision Engine

```rust
use mf_rules_engine::{DecisionEngine, EvaluationOptions};
use mf_rules_engine::loader::{FilesystemLoader, FilesystemLoaderOptions};
use serde_json::json;

// Create engine with filesystem loader
let engine = DecisionEngine::new(FilesystemLoader::new(FilesystemLoaderOptions {
    root: "/path/to/decisions",
    keep_in_memory: true,
}));

// Evaluate decision
let context = json!({ 
    "customer": { 
        "age": 25, 
        "income": 50000 
    } 
});

let result = engine.evaluate("eligibility.json", &context).await?;
println!("Decision result: {:?}", result);

// Get and cache decision for repeated use
let decision = engine.get_decision("eligibility.json").await?;
let result1 = decision.evaluate(&context).await?;
let result2 = decision.evaluate(&context).await?; // Cached
```

#### 2. Decision Creation

```rust
use mf_rules_engine::{Decision, DecisionEngine};
use mf_rules_engine::model::DecisionContent;

// Create decision from JSON content
let decision_content: DecisionContent = serde_json::from_str(r#"
{
  "key": "customer_eligibility",
  "name": "Customer Eligibility Check",
  "input": {
    "customer": {
      "age": "number",
      "income": "number"
    }
  },
  "output": {
    "eligible": "boolean",
    "reason": "string"
  },
  "rules": [
    {
      "when": "customer.age >= 18 && customer.income > 30000",
      "then": {
        "eligible": true,
        "reason": "Meets age and income requirements"
      }
    }
  ]
}
"#)?;

// Create decision directly
let decision = Decision::from(decision_content);

// Or create through engine
let engine = DecisionEngine::default();
let decision = engine.create_decision(decision_content.into());

// Evaluate
let result = decision.evaluate(&json!({
    "customer": { "age": 25, "income": 45000 }
})).await?;
```

#### 3. Loaders

```rust
use mf_rules_engine::loader::{
    FilesystemLoader, FilesystemLoaderOptions,
    MemoryLoader, ClosureLoader, NoopLoader
};

// Filesystem loader
let fs_loader = FilesystemLoader::new(FilesystemLoaderOptions {
    root: "/decisions",
    keep_in_memory: true,
});

// Memory loader
let memory_loader = MemoryLoader::default();
memory_loader.add("rule1", decision_content);

// Closure loader
let closure_loader = ClosureLoader::new(|key: &str| async move {
    // Custom loading logic
    load_decision_from_database(key).await
});

// Use with engine
let engine = DecisionEngine::new(fs_loader);
```

#### 4. Custom Functions

```rust
use mf_rules_engine::Variable;

// Define custom function
fn calculate_discount(args: &[Variable]) -> Result<Variable, String> {
    if let [Variable::Number(price), Variable::Number(discount_rate)] = args {
        let discount = price * discount_rate / 100.0;
        Ok(Variable::Number(discount.into()))
    } else {
        Err("Invalid arguments for calculate_discount".to_string())
    }
}

// Register and use in expressions
// This would be part of a custom function registry
```

#### 5. Error Handling

```rust
use mf_rules_engine::{EvaluationError, DecisionGraphValidationError};

match engine.evaluate("complex_decision.json", &context).await {
    Ok(result) => println!("Success: {:?}", result),
    Err(EvaluationError::LoaderError(err)) => {
        eprintln!("Failed to load decision: {}", err);
    },
    Err(EvaluationError::ValidationError(err)) => {
        eprintln!("Validation failed: {}", err);
    },
    Err(err) => eprintln!("Other error: {}", err),
}
```

This completes the first part of the API documentation. Would you like me to continue with the remaining modules (Expression System, Collaboration, Template, Macro, and Examples)?
# ModuForge-RS Macro System Development Guide

## Overview

ModuForge-RS provides a powerful procedural macro system that automatically generates code for nodes and marks through `#[derive(Node)]` and `#[derive(Mark)]`. The macro system is designed with strict adherence to SOLID principles, offering type-safe, flexible, and high-performance code generation capabilities.

## Core Features

### 🎯 Design Separation Principles
- **Node Definition vs Instance Creation**: `node_definition()` contains only schema definitions for `#[attr]` fields, while `from()` handles instance creation for all fields
- **Attribute Precision**: Only fields marked with `#[attr]` become part of the node's attribute definition
- **Type Safety**: Supports safe conversion of generic types and custom types, requiring custom types to implement `Default + Serialize` traits

### 🚀 Supported Features
- **Basic Types**: `String`, `i32`, `f64`, `bool`, etc.
- **Generic Types**: `Option<T>`, `Vec<T>`, `HashMap<K,V>`, etc.
- **Custom Types**: Support for constructor expressions like `CustomStruct::new()`
- **JSON Default Values**: Support for complex JSON structures as default values
- **Bidirectional Conversion**: Automatic generation of `From` trait implementations
- **Error Handling**: Type validation with graceful degradation

## Node Macro Usage Guide

### Basic Usage

```rust
use mf_derive::Node;

#[derive(Node)]
#[node_type = "paragraph"]
#[marks = "bold italic"]
#[content = "text*"]
struct ParagraphNode {
    #[attr]
    text: String,
    
    #[attr(default=1)]
    level: i32,
    
    // Non-attribute field, won't appear in node_definition()
    cache: String,
}
```

### Advanced Feature Examples

#### 1. Generic Type Support

```rust
#[derive(Node)]
#[node_type = "document"]
struct DocumentNode {
    #[attr]
    title: String,
    
    // Option type
    #[attr]
    subtitle: Option<String>,
    
    // Vec type
    #[attr]
    tags: Vec<String>,
    
    // Complex generic type
    #[attr]
    metadata: HashMap<String, String>,
}
```

#### 2. Custom Type Expressions

```rust
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Clone)]
struct DocumentConfig {
    pub auto_backup: bool,
    pub sync_enabled: bool,
}

impl DocumentConfig {
    pub fn new() -> Self {
        Self {
            auto_backup: true,
            sync_enabled: false,
        }
    }
}

#[derive(Node)]
#[node_type = "document"]
struct AdvancedDocumentNode {
    // Using custom constructor
    #[attr(default="DocumentConfig::new()")]
    config: DocumentConfig,
    
    // Using constructor with parameters
    #[attr(default="HashMap::with_capacity(10)")]
    cache: HashMap<String, String>,
    
    // Using method chaining
    #[attr(default="SettingsBuilder::new().with_defaults().build()")]
    settings: DocumentSettings,
}
```

#### 3. JSON Default Values

```rust
#[derive(Node)]
#[node_type = "ui_component"]
struct UIComponentNode {
    // JSON object default value
    #[attr(default={"theme": "light", "auto_save": true, "max_history": 50})]
    ui_config: serde_json::Value,
    
    // JSON array default value
    #[attr(default=["draft", "review", "published"])]
    workflow_states: serde_json::Value,
}
```

### Generated Methods

Each `#[derive(Node)]` struct automatically generates the following methods:

#### 1. `node_definition()` - Static Method
```rust
/// Get node definition (schema definition)
/// Contains only AttributeSpec for #[attr] marked fields
pub fn node_definition() -> mf_core::node::Node {
    // Auto-generated implementation
}
```

#### 2. `from()` - Instance Creation Method
```rust
/// Create struct instance from mf_model::node::Node
/// Handles all fields (including non-#[attr] fields)
pub fn from(node: &mf_model::node::Node) -> Result<Self, String> {
    // Auto-generated implementation
}
```

#### 3. `default_instance()` - Default Instance Method
```rust
/// Create default instance (fallback method for failures)
fn default_instance() -> Self {
    // Auto-generated implementation
}
```

#### 4. `From` Trait Implementations
```rust
// Bidirectional conversion support
impl From<MyStruct> for mf_core::node::Node { ... }
impl From<mf_model::node::Node> for MyStruct { ... }
```

## Mark Macro Usage Guide

### Basic Usage

```rust
use mf_derive::Mark;

#[derive(Mark)]
#[mark_type = "emphasis"]
struct EmphasisMark {
    #[attr]
    level: String,
    
    #[attr]
    color: Option<String>,
}
```

### Generated Methods

```rust
impl EmphasisMark {
    /// Convert struct to mf_core::mark::Mark instance
    pub fn to_mark(&self) -> mf_core::mark::Mark {
        // Auto-generated implementation
    }
}
```

## Complete Example: Document Node

Here's a comprehensive example showcasing all features:

```rust
use mf_derive::Node;
use uuid::Uuid;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Supporting type definitions
#[derive(Default, Serialize, Clone)]
struct DocumentConfig {
    pub auto_backup: bool,
    pub sync_enabled: bool,
}

impl DocumentConfig {
    pub fn new() -> Self {
        Self {
            auto_backup: true,
            sync_enabled: false,
        }
    }
}

#[derive(Default, Serialize, Clone)]
struct DocumentSettings {
    pub theme: String,
    pub font_size: i32,
    pub line_height: f32,
}

struct SettingsBuilder {
    settings: DocumentSettings,
}

impl SettingsBuilder {
    pub fn new() -> Self {
        Self {
            settings: DocumentSettings::default(),
        }
    }
    
    pub fn with_defaults(mut self) -> Self {
        self.settings.theme = "system".to_string();
        self.settings.font_size = 14;
        self.settings.line_height = 1.5;
        self
    }
    
    pub fn build(self) -> DocumentSettings {
        self.settings
    }
}

/// Complete feature document node
/// 
/// Demonstrates all Node derive macro supported features:
/// - Basic attributes and default values
/// - Generic type support (Option<T>, Vec<T>)
/// - Custom type expressions
/// - Complex JSON default values
/// - Non-attribute field handling
#[derive(Node)]
#[node_type = "document"]
#[marks = "bold italic underline strikethrough"]
#[content = "block+"]
struct DocumentNode {
    // === Basic attribute fields ===
    
    /// Document title (required attribute)
    #[attr]
    title: String,
    
    /// Document description (with string default value)
    #[attr(default="Untitled Document")]
    description: String,
    
    /// Document version (with numeric default value)
    #[attr(default=1)]
    version: i32,
    
    /// Is published (with boolean default value)
    #[attr(default=true)]
    is_published: bool,
    
    /// Weight score (with float default value)
    #[attr(default=5.0)]
    weight: f64,
    
    // === Optional type fields ===
    
    /// Optional subtitle
    #[attr]
    subtitle: Option<String>,
    
    /// Optional priority
    #[attr]
    priority: Option<i32>,
    
    /// Optional tag list (with null default value)
    #[attr(default=null)]
    tags: Option<Vec<String>>,
    
    // === Complex type fields ===
    
    /// Document unique identifier (UUID type)
    #[attr]
    document_id: Uuid,
    
    /// Binary data
    #[attr]
    binary_data: Vec<u8>,
    
    /// String vector
    #[attr]
    categories: Vec<String>,
    
    // === Custom type expressions ===
    
    /// Custom configuration (using constructor)
    #[attr(default="DocumentConfig::new()")]
    config: DocumentConfig,
    
    /// Metadata mapping (using constructor with parameters)
    #[attr(default="HashMap::with_capacity(10)")]
    metadata: HashMap<String, String>,
    
    /// Builder pattern (method chaining)
    #[attr(default="SettingsBuilder::new().with_defaults().build()")]
    settings: DocumentSettings,
    
    // === JSON default values ===
    
    /// Complex JSON configuration
    #[attr(default={"theme": "light", "auto_save": true, "max_history": 50})]
    ui_config: serde_json::Value,
    
    /// JSON array configuration
    #[attr(default=["draft", "review", "published"])]
    workflow_states: serde_json::Value,
    
    // === Non-attribute fields (won't appear in node_definition) ===
    
    /// Runtime computed field
    computed_hash: String,
    
    /// Cache data
    cache: Option<Vec<u8>>,
    
    /// Internal state marker
    _internal_state: std::marker::PhantomData<()>,
}
```

## Usage Examples

### 1. Getting Node Definition

```rust
// Get node definition (for schema definition)
let node_definition = DocumentNode::node_definition();
println!("Node type: {}", node_definition.name);
println!("Supported marks: {:?}", node_definition.spec.marks);
println!("Content expression: {:?}", node_definition.spec.content);
```

### 2. Creating Node Instances

```rust
// Create actual node instance data
let mut attrs = imbl::HashMap::new();
attrs.insert("title".to_string(), serde_json::json!("My Document"));
attrs.insert("version".to_string(), serde_json::json!(2));
attrs.insert("is_published".to_string(), serde_json::json!(false));

let node_instance = mf_model::node::Node {
    id: "doc_001".into(),
    r#type: "document".to_string(),
    attrs: mf_model::attrs::Attrs { attrs },
    content: imbl::Vector::new(),
    marks: imbl::Vector::new(),
};

// Convert from Node to struct (type-safe conversion)
match DocumentNode::from(&node_instance) {
    Ok(doc_struct) => {
        println!("Conversion successful:");
        println!("  Title: {}", doc_struct.title);
        println!("  Version: {}", doc_struct.version);
        println!("  Published: {}", doc_struct.is_published);
    },
    Err(e) => {
        println!("Conversion failed: {}", e);
    }
}
```

### 3. Bidirectional Conversion

```rust
// Using .into() method for conversion (auto-degradation)
let doc_struct: DocumentNode = node_instance.into(); // Falls back to default_instance() on failure

// Reverse conversion: from struct to Node definition
let definition: mf_core::node::Node = doc_struct.into();
```

## Type Requirements

### Custom Type Requirements

When using custom type expressions, types must satisfy the following trait requirements:

```rust
// Custom types must implement these two traits
#[derive(Default, Serialize)]
struct MyCustomType {
    // Field definitions
}

// Or implement manually
impl Default for MyCustomType {
    fn default() -> Self {
        // Default implementation
    }
}

impl Serialize for MyCustomType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // Serialization implementation
    }
}
```

### Supported Expression Patterns

The macro system can recognize the following custom type expression patterns:

#### Module Path + Constructor
- `CustomStruct::new()`
- `CustomStruct::default()`
- `some_module::CustomStruct::new()`
- `std::collections::HashMap::with_capacity(10)`

#### Common Constructor Patterns
- `::new()` - Most common constructor
- `::default()` - Default constructor
- `::with_default()` - Constructor with default parameters
- `::with_capacity()` - Constructor with capacity
- `::empty()` - Create empty instance
- `::create()` - Generic creation method

#### Method Chaining Patterns
- `Builder::new().build()`
- `Config::new().with_field("value").finalize()`
- `CustomStruct::builder().field(value).build()`

## Error Handling

### Type Validation Errors

```rust
// When node type doesn't match
let result = DocumentNode::from(&wrong_type_node);
match result {
    Ok(doc) => { /* Handle success */ },
    Err(e) => {
        // e contains detailed error information, such as:
        // "Node type mismatch: expected 'document', actual 'paragraph'"
        println!("Conversion failed: {}", e);
    }
}
```

### Graceful Degradation

```rust
// Using .into() method, failures automatically degrade to default instance
let doc_struct: DocumentNode = potentially_invalid_node.into();
// Even if conversion fails, you get a valid default instance
```

## Performance Optimization

### Memory Efficiency
- Uses immutable data structures (`im-rs`) for structural sharing
- `Arc` reference counting reduces memory allocation
- Type-safe zero-copy conversions

### Compile-time Optimization
- Macros generate code at compile time, zero runtime overhead
- Type checking completed at compile time
- Static dispatch, no virtual function calls

### Runtime Performance
- Efficient field access patterns
- Minimized JSON serialization/deserialization
- Optimized error handling paths

## Integration Examples

### Integration with State System

```rust
use mf_state::{Transaction, State};
use mf_transform::node_step::AddNodeStep;

async fn add_document_node(state: &State) -> Result<State, Box<dyn std::error::Error>> {
    // Create document node definition
    let node_def = DocumentNode::node_definition();
    
    // Create transaction
    let mut transaction = Transaction::new();
    transaction.add_step(AddNodeStep::new(node_def, None));
    
    // Apply transaction
    let new_state = state.apply_transaction(transaction).await?;
    Ok(new_state)
}
```

### Integration with Plugin System

```rust
use mf_state::plugin::{Plugin, PluginTrait};
use async_trait::async_trait;

#[derive(Debug)]
struct DocumentValidationPlugin;

#[async_trait]
impl PluginTrait for DocumentValidationPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> Result<Option<Transaction>> {
        // Validate document nodes
        for transaction in transactions {
            for step in &transaction.steps {
                if let Some(add_step) = step.as_any().downcast_ref::<AddNodeStep>() {
                    if add_step.node.name == "document" {
                        // Execute document-specific validation logic
                        validate_document_node(&add_step.node)?;
                    }
                }
            }
        }
        Ok(None)
    }
}

fn validate_document_node(node: &mf_core::node::Node) -> Result<(), ValidationError> {
    // Validation logic
    Ok(())
}
```

## Best Practices

### 1. Design Principles
- **Clear Separation**: Distinguish between node definition (schema) and instance creation
- **Type Safety**: Prefer compile-time type checking
- **Graceful Degradation**: Provide reasonable defaults and error handling

### 2. Naming Conventions
- Use descriptive node type names
- Attribute names use snake_case
- Custom types use PascalCase

### 3. Performance Recommendations
- For large structs, consider wrapping fields with `Arc<T>`
- Use custom expressions for complex default values instead of JSON
- Avoid frequent type conversions in hot paths

### 4. Debugging Tips
- Use `cargo expand` to view generated macro code
- Enable verbose error messages for debugging
- Use unit tests to verify generated macro code

## Troubleshooting

### Common Compilation Errors

#### 1. Custom Type Missing Trait Implementation
```
error: the trait `Default` is not implemented for `CustomType`
```
**Solution**: Implement `Default` and `Serialize` traits for custom types

#### 2. Invalid Default Value Expression
```
error: expected expression, found `invalid_syntax`
```
**Solution**: Check the syntax correctness of default value expressions

#### 3. Node Type Mismatch
```
Node type mismatch: expected 'document', actual 'paragraph'
```
**Solution**: Ensure the passed Node instance type matches the struct's `node_type`

### Debugging Steps

1. **Check Macro Attributes**: Ensure all required attributes are correctly set
2. **Verify Type Requirements**: Ensure custom types implement required traits
3. **Test Expressions**: Verify custom type expressions can compile and execute correctly
4. **Unit Testing**: Write comprehensive unit tests for generated code

## Related Documentation

- **[Macro Expansion Example](./macro-expansion-example.md)** - View complete macro expansion results and code generation examples
- **[Quick Start Guide](./quick-start.md)** - Learn how to start using the macro system in your project
- **[API Reference](./api-reference.md)** - Detailed macro API documentation

This macro system provides ModuForge-RS with powerful and flexible code generation capabilities. By following best practices and design principles, you can build high-performance, type-safe, and maintainable applications.
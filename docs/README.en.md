# ModuForge-RS

[Read this in Chinese](./index.md)

ModuForge is a Rust-based state management and data transformation framework focusing on immutable data structures and event-driven architecture. It provides a business-agnostic editor core implementation that can be customized and extended to support the needs of any business scenario.

### How does ModuForge work?

- **How it works:** Define basic nodes, marks, and constraints, then define extensions to add behavior.

  - **model:** Defines basic data, including Nodes, Marks, Schemas, etc.

  - **state:** Manages state, primarily responsible for state updates and plugin scheduling.

  - **transform:** Implements transactions, similar to database transactions, ensuring atomicity and data consistency. The smallest unit of operation can be extended.

  - **core:** Combines `model`, `state`, and `transform` to further implement the core editor functionality, adding and collecting extensions.

  - **rules:** The rule engine system, including expression parsing, backend execution, core engine, and a template system.

### Project Directory Structure

```
moduforge-rs/
├── core/           # Core functionality module
│   ├── src/
│   │   ├── lib.rs                 # Core library entry point
│   │   ├── async_processor.rs     # Async task processor
│   │   ├── async_runtime.rs       # Async runtime environment
│   │   ├── error.rs               # Error types and handling
│   │   ├── event.rs               # Event system
│   │   ├── extension.rs           # Extension mechanism
│   │   ├── extension_manager.rs   # Extension manager
│   │   ├── flow.rs                # Flow control
│   │   ├── helpers/               # Helper functions
│   │   ├── history_manager.rs     # History management
│   │   ├── mark.rs                # Mark system
│   │   ├── metrics.rs             # Metrics system
│   │   ├── middleware.rs          # Middleware support
│   │   ├── node.rs                # Node system
│   │   ├── runtime.rs             # Runtime environment
│   │   └── types.rs               # Core type definitions
│   └── Cargo.toml                 # Core module dependency configuration
│
├── model/          # Data model module
│   ├── src/
│   │   ├── lib.rs                 # Model definition entry point
│   │   ├── node.rs                # Node definition
│   │   ├── mark.rs                # Mark definition
│   │   ├── attrs.rs               # Attribute definition
│   │   ├── mark_type.rs           # Mark type definition
│   │   ├── node_type.rs           # Node type definition
│   │   ├── schema.rs              # Schema definition
│   │   ├── content.rs             # Content matching definition
│   │   ├── error.rs               # Error types and handling
│   │   ├── id_generator.rs        # ID generator
│   │   ├── node_pool.rs           # Node pool management
│   │   └── types.rs               # Common type definitions
│   └── Cargo.toml                 # Model module dependency configuration
│
├── transform/      # Data transformation module
│   ├── src/
│   │   ├── lib.rs                 # Transformation function entry point
│   │   ├── attr_step.rs           # Attribute step
│   │   ├── draft.rs               # Draft system
│   │   ├── mark_step.rs           # Mark step
│   │   ├── node_step.rs           # Node step
│   │   ├── patch.rs               # Patch system
│   │   ├── step.rs                # Step definition
│   │   └── transform.rs           # Transformation system
│   └── Cargo.toml                 # Transformation module dependency configuration
│
├── state/          # State management module
│   ├── src/
│   │   ├── lib.rs                 # State management entry point
│   │   ├── error.rs               # Error types and handling
│   │   ├── gotham_state.rs        # Gotham state management
│   │   ├── logging.rs             # Logging system
│   │   ├── ops.rs                 # Operation definitions
│   │   ├── plugin.rs              # Plugin system
│   │   ├── resource.rs            # Resource management
│   │   ├── resource_table.rs      # Resource table
│   │   ├── state.rs               # State management
│   │   └── transaction.rs         # Transaction handling
│   └── Cargo.toml                 # State module dependency configuration
│
├── rules/          # Rule engine module
│   ├── expression/  # Expression parsing and handling
│   ├── backend/     # Rule engine backend
│   ├── engine/      # Rule engine core
│   └── template/    # Template system
│
├── macros/         # Macro definition module
│   ├── src/
│   │   ├── lib.rs                 # Macro definition entry point
│   │   ├── command.rs             # Command macro
│   │   ├── extension.rs           # Extension macro
│   │   ├── mark.rs                # Mark macro
│   │   ├── node.rs                # Node macro
│   │   └── plugin.rs              # Plugin macro
│   └── Cargo.toml                 # Macro module dependency configuration
│
├── demo/           # Example and demo code
│   ├── src/
│   ├── Cargo.toml
│   └── README.md
│
├── docs/           # Project documentation
│   ├── node-budget-mapping.md    # Node model to business mapping
│   ├── architecture_use_cases.md # Architecture use case analysis
│   ├── plugin-development-guide.md # Plugin development guide
│   └── ...                       # Other analysis documents
│
├── test-data/      # Test data
├── Cargo.toml      # Workspace configuration file
├── Cargo.lock      # Dependency lock file
├── rustfmt.toml    # Rust code formatting configuration
├── release.toml    # Release configuration
└── .gitignore      # Git ignore file configuration
```

### Core Components Explained

#### State Management

`State` is the core state management component of the editor, responsible for maintaining the editor's overall state. It includes the following key features:

- **Configuration Management**: Manages editor configuration through the `Configuration` struct, including plugin lists, document structure definitions, etc.
- **Plugin State**: Manages the state data of all plugins through `fields_instances`.
- **Document Management**: Manages the document's node pool through `node_pool`.
- **Versioning**: Tracks state changes through the `version` field.
- **Resource Management**: Manages global resources through `resource_manager`.

`State` provides the following main functions:
- Creating and initializing new editor states
- Managing plugin states
- Handling transactions and state updates
- Reconfiguring the state to adapt to new requirements

#### GlobalResourceManager

`GlobalResourceManager` is the editor's runtime global resource manager, responsible for managing all registered resources and states. It includes the following key features:

- **Resource Table Management**: Manages all registered resources through `ResourceTable`.
- **Gotham State Management**: Manages state specific to the Gotham framework through `GothamState`.
- **Thread Safety**: Implements the `Send` and `Sync` traits, ensuring safe transfer and sharing between threads.
- **Resource Cleanup**: Provides a `clear` method to clean up all resources.

Primary use cases for `GlobalResourceManager`:
- Sharing resources between plugins
- Managing global state
- Handling data exchange across plugins
- Managing global configuration at runtime

`GlobalResourceManager` Usage Example

Here is a typical scenario for using `GlobalResourceManager`:

```rust
// 1. Define a custom resource type
#[derive(Debug)]
struct CacheManager {
    data: HashMap<String, String>,
}

impl Resource for CacheManager {
    fn name(&self) -> Cow<str> {
        "CacheManager".into()
    }
}

// 2. Register the resource during plugin initialization
async fn plugin_init(config: &StateConfig, instance: Option<&State>) -> PluginState {
    // Get the resource manager
    let resource_manager = instance.unwrap().resource_manager();
    let mut resource_manager = resource_manager.write().unwrap();
    
    // Create and register the cache manager
    let cache_manager = CacheManager {
        data: HashMap::new(),
    };
    resource_manager.resource_table.add(cache_manager);
    
    // Return the plugin state
    Arc::new(HashMap::new())
}

// 3. Use the shared resource in a plugin
async fn plugin_operation(state: &State) {
    // Get the resource manager
    let resource_manager = state.resource_manager();
    let resource_manager = resource_manager.read().unwrap();
    
    // Get the cache manager
    let cache_manager = resource_manager.resource_table.get::<CacheManager>(0).unwrap();
    
    // Use the cache manager
    cache_manager.data.insert("key".to_string(), "value".to_string());
}

// 4. Access the same resource from another plugin
async fn another_plugin_operation(state: &State) {
    let resource_manager = state.resource_manager();
    let resource_manager = resource_manager.read().unwrap();
    
    let cache_manager = resource_manager.resource_table.get::<CacheManager>(0).unwrap();
    let value = cache_manager.data.get("key").unwrap();
    println!("Retrieved value: {}", value);
}
```

This example demonstrates:
1. How to define a custom resource type
2. How to register a resource during plugin initialization
3. How to use a shared resource within a plugin
4. How to share and access the same resource across multiple plugins

With `GlobalResourceManager`, different plugins can safely share and access global resources without direct dependencies on each other.

#### Differences between State and GlobalResourceManager

Although `State` and `GlobalResourceManager` both involve state management, they have distinct responsibilities and use cases:

1. **Scope of Management**
   - `State` manages the overall state of the editor, including document content, plugin states, configurations, etc.
   - `GlobalResourceManager` focuses on managing runtime resources and globally shared state.

2. **Lifecycle**
   - The lifecycle of `State` is tied to an editor instance; it is created and destroyed with the editor.
   - The lifecycle of `GlobalResourceManager` is more flexible and can be shared across different `State` instances.

3. **Access Method**
   - `State` updates its state through transactions, ensuring atomicity and consistency.
   - `GlobalResourceManager` provides direct resource access interfaces, suitable for fast reads and writes of shared resources.

4. **Use Case**
   - `State` is used to manage the core state of the editor, such as document content and plugin configurations.
   - `GlobalResourceManager` is used to manage resources shared across plugins, such as caches and configurations.

5. **Thread Safety**
   - State updates in `State` are single-threaded, with consistency guaranteed by transactions.
   - `GlobalResourceManager` is thread-safe and can be accessed safely in a multi-threaded environment.

6. **Extensibility**
   - The state structure of `State` is relatively fixed, centered around documents and plugins.
   - `GlobalResourceManager` can dynamically register and manage any type of resource, offering greater extensibility.

### What Technology Stack Does ModuForge Use?

- **im-rs:** ModuForge uses `im-rs` for basic data definitions, ensuring data immutability.

- **tokio:** An asynchronous runtime that supports high-concurrency async operations.

- **serde:** Serialization and deserialization support for data persistence and transmission.

- **thiserror/anyhow:** Error handling frameworks providing type-safe error management.

- **zen:** A rule engine for decoupling business logic from hard-coded implementations (if used).

### ModuForge Framework Design Philosophy

- **Extensibility:** ModuForge is designed to be highly extensible, allowing developers to customize the editor's functionality and behavior. This includes a plugin system that simplifies adding new features, making any functionality extendable (e.g., history, undo, redo).

- **Modularity:** The entire framework is broken down into independent modules, each responsible for a specific aspect of the editor, such as the model, state management, command execution, etc. This design allows developers to selectively include modules as needed.

- **Immutable Data:** Uses `im-rs` to ensure the immutability of data structures, providing safe concurrent access and efficient structural sharing.

- **Event-Driven:** Based on an event-driven architecture, all state changes are handled through the event system, ensuring system responsiveness and predictability.

- **Command Pattern:** Uses the command pattern to handle editing operations. Each operation is encapsulated as a command object, which facilitates undo/redo operations and helps implement complex editing logic.

- **State Management:** The editor's state is centrally managed. All modifications to the document trigger state changes, helping to maintain data consistency and predictability.

## Suitability Analysis for Large Tree-like Editors

The ModuForge framework is particularly well-suited for developing large tree-like editors. Here is a detailed analysis:

### 1. Tree Structure Support

The framework provides comprehensive support for tree structures:

- **Node Definition**:
  - Each node has a unique ID
  - Supports node types (`type`)
  - Supports node attributes (`attrs`)
  - Supports a list of child nodes (`content`)
  - Supports node marks (`marks`)

- **Tree Operation API**:
  - Get list of child nodes
  - Recursively get all descendants (depth-first)
  - Get parent node
  - Get node depth
  - Get node path
  - Check if a node is a leaf
  - Get sibling nodes
  - Get subtree size

### 2. Editing Functionality Support

The framework provides complete editing functionality:

- **Node Operations**:
  - Add node
  - Replace node
  - Move node
  - Delete node
  - Recursively delete a subtree

- **Transaction Support**:
  - All operations are executed within a transaction
  - Supports atomicity of operations
  - Supports undo/redo
  - Supports patch recording

### 3. Statistics Functionality Support

The framework provides rich statistical functions:

- **Node Statistics**:
  - Get total number of nodes
  - Get subtree size
  - Get node depth
  - Supports custom filtering and searching

- **Performance Optimization**:
  - Uses immutable data structures (`im-rs`)
  - Uses `Arc` for reference counting
  - Supports concurrent access (`Send` + `Sync`)

### 4. Particularly Suitable Scenarios

1. **Large-Scale Tree Data Editing**:
   - Supports deeply nested tree structures
   - Efficient node lookup and traversal
   - Supports large-scale data operations

2. **Complex Data Statistics**:
   - Supports custom statistical rules
   - Supports node filtering and searching
   - Supports subtree statistics

3. **Real-time Editing and Updates**:
   - Supports transactional operations
   - Supports undo/redo
   - Supports incremental updates

### 5. Performance Considerations

1. **Memory Efficiency**:
   - Uses immutable data structures
   - Uses reference counting
   - Supports shared nodes

2. **Operational Efficiency**:
   - Efficient node lookup
   - Optimized tree traversal
   - Batch operation support

3. **Concurrency Support**:
   - Thread-safe design
   - Supports concurrent access
   - Supports resource management

### 6. Extensibility

1. **Custom Node Types**:
   - Supports custom node attributes
   - Supports custom node marks
   - Supports custom node content

2. **Plugin System**:
   - Supports custom editing operations
   - Supports custom statistical rules
   - Supports custom validation rules

### Conclusion

The ModuForge framework is very well-suited for developing large tree-like editors, especially in the following scenarios:

1. Editors that need to handle large amounts of tree-structured data
2. Applications requiring complex editing operations
3. Systems that need real-time statistics and updates
4. Applications requiring high performance and concurrency support
5. Editors that need to be highly customizable

The framework's design fully considers performance, extensibility, and ease of use, making it capable of supporting the development needs of large tree-like editors.

## About ModuForge

ModuForge is a general-purpose editor framework derived from current pricing software. Therefore, it is not tied to any specific pricing business; it is simply a large, general-purpose editor framework.

## License

For internal use by the pricing software team. Please do not distribute.

## 📚 Related Documents

This project includes several detailed analysis and design documents covering architecture, business applications, design patterns, and more:

### 🎋 Business Model Mapping

#### [Precise Mapping of the Node Model to Construction Budgets](./node-budget-mapping.md)
**Details how to precisely map ModuForge's Node model to the construction budget business.**

- **Core Content**:
  - Detailed mapping relationship between the ModuForge Node model and the construction budget business
  - Complete definition of the budget item hierarchy (Budget Document → Project → Unit Project → Division → Sub-item → Bill Item)
  - Concrete implementation code for `NodeSpec` business type specifications
  - Application of the `Mark` system in business state management
  - Code examples for actual business queries and statistical functions

- **Technical Highlights**:
  - The hierarchical structure naturally supports the organization of engineering budgets
  - The attribute system perfectly matches cost data (quantity, unit price, amount, etc.)
  - The mark system supports business state management (calculated, locked, quota applied, etc.)
  - Provides complete implementation code for a business analyzer

### 🚀 Architecture Use Cases

#### [Analysis of Business Scenarios for the Architecture](./architecture_use_cases.md)
**In-depth analysis of the applicability of the ModuForge architecture in different business scenarios.**

- **Business Scenario Categories**:
  - **Business Process Orchestration**: Workflow engines, data processing pipelines (ETL)
  - **Computation Orchestration**: Pricing engine systems, risk control decision engines  
  - **Content Management**: Collaborative editors, content publishing systems
  - **Rule Engines**: Business rule engines, A/B testing frameworks
  - **Intelligent Computing**: Recommendation systems, machine learning pipelines

- **Practical Application Examples**:
  - Insurance pricing engines, ride-hailing fare systems
  - Online document collaboration, collaborative code editing
  - Big data processing platforms, real-time data stream processing
  - Risk control systems, recommendation algorithm platforms

### 🔗 Business Dependency Design

#### [Pluggable Architecture Design for A-depends-on-B Business Dependencies](./business_dependency_design.md)
**A traditional business dependency manager solution.**

- **Design Features**:
  - Manages inter-business dependencies through a dedicated `BusinessDependencyManager`
  - Supports dependency type classification (computation, data, event dependencies)
  - Implements complete dependency checking and execution order management
  - Provides topological sorting to ensure correct dependency execution order

- **Core Components**:
  - `BusinessDependencyManager`: Centralized dependency management
  - `BusinessDependency`: Describes dependency relationships
  - Complete implementation examples for Business A and Business B plugins

#### [Decoupling Business Dependencies with Transaction Meta](./meta_based_dependency_design.md)
**A recommended lightweight business dependency solution.**

- **Design Advantages**:
  - Uses the `meta` field of a `Transaction` to pass business dependency information
  - More lightweight, no need for an additional dependency manager component
  - Implemented entirely based on the existing transaction system
  - Supports business degradation and fault tolerance

- **Technical Implementation**:
  - Structured design of the `meta` field (business type, status, dependencies)
  - Passing of business execution context
  - Dependency satisfaction checks and waiting mechanisms
  - Complete plugin implementation code examples

### 📈 Architecture Analysis

#### [Analysis of Architectural Limitations](./architecture_limitations_analysis.md)
**An objective analysis of the strengths and limitations of the ModuForge architecture.**

- **Analysis Dimensions**:
  - Performance characteristics (memory usage, concurrency, response time)
  - Extensibility (plugin system, business adaptation capabilities)
  - Complexity (development difficulty, learning curve, maintenance cost)
  - Applicability boundaries (suitable and unsuitable business scenarios)

#### [Simplified and Enhanced History Management](./simple_enhanced_history.md)
**Design and implementation of history management and undo/redo functionality.**

- **Core Features**:
  - Snapshot-based history management strategy
  - Efficient implementation of undo/redo operations
  - Mechanisms for compressing and clearing history records
  - Deep integration with the transaction system

---

### 📖 Documentation Usage Suggestions

1. **For Beginners**: It is recommended to first read this `README.md` to understand the overall architecture, then review `architecture_use_cases.md` to understand applicable scenarios.

2. **Business Modeling**: If you need to apply ModuForge to a specific business, focus on the mapping methods in `node-budget-mapping.md`.

3. **Complex Dependencies**: If there are complex inter-business dependencies, first consider the lightweight solution in `meta_based_dependency_design.md`.

4. **Architectural Decisions**: When making technology choices for a project, refer to the objective analysis in `architecture_limitations_analysis.md`.

5. **Feature Extensions**: When you need to add history management functionality, refer to the implementation方案 in `simple_enhanced_history.md`.

These documents together form the complete technical system of the ModuForge project, providing comprehensive guidance from conceptual understanding to concrete implementation for readers at all levels. 
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ModuForge-RS is a comprehensive Rust-based framework for state management and data transformation, featuring immutable data structures, event-driven architecture, collaboration capabilities, and a powerful rules engine. It's designed as a modular, extensible framework that can be adapted to various business scenarios through plugins and extensions.

## Common Development Commands

### Build and Test
```bash
# Build all workspace members
cargo build

# Build with all features
cargo build --all-features

# Run all tests
cargo test

# Run tests for specific crate
cargo test -p mf-core
cargo test -p mf-engine
cargo test -p mf-collaboration

# Run benchmarks (expression crate has extensive benchmarks)
cargo bench

# Format code according to project standards
cargo fmt

# Lint code with clippy
cargo clippy
cargo clippy --no-deps  # Only lint current crate, not dependencies
cargo clippy --fix      # Auto-fix suggestions
```

### Running Examples
```bash
# Core examples
cargo run --example config_usage -p mf-core
cargo run --example xml_schema_integration -p mf-core
cargo run --example improved_event_system -p mf-core

# Expression examples
cargo run --example compilation_demo -p mf-expression
cargo run --example custom_function_demo -p mf-expression

# File handling examples
cargo run --example export_zip_cbor -p mf-file
cargo run --example export_single -p mf-file

# Collaboration examples
cargo run --example client -p mf-collaboration-client

# Search examples
cargo run --example basic -p mf-search

# Persistence examples
cargo run --example 01_runtime_with_persistence -p mf-persistence
```

### Demo Application
```bash
# Run Tauri-based demo application
cd demo
npm install
npm run tauri dev
```

## Architecture Overview

The framework consists of multiple interconnected crates organized as follows:

### Core Architecture Crates
- **mf-core**: Framework foundation with async runtime, events, extensions, middleware
- **mf-model**: Data models - nodes, marks, attributes, schemas, tree structures
- **mf-state**: State management with transactions, plugins, resource management
- **mf-transform**: Data transformation operations and transaction steps
- **mf-macro**: Procedural macros for nodes, plugins, extensions

### Rules Engine System
- **mf-engine**: Business rules engine based on GoRules JDM standard
- **mf-expression**: High-performance expression language with WASM support
- **mf-template**: Template rendering system

### Collaboration & Data
- **mf-collaboration**: Real-time collaborative editing using Yrs CRDT
- **mf-collaboration-client**: Client-side collaboration utilities
- **mf-file**: Document serialization/deserialization (JSON, CBOR, MessagePack)
- **mf-search**: Search indexing and querying capabilities
- **mf-persistence**: Data persistence and recovery mechanisms

### Development Utilities
- **mf-derive**: Procedural macros for dependency injection
- **devtool-rules**: Development tooling

## Key Architectural Patterns

### Immutable Data Structures
The framework uses `im-rs` crate extensively for persistent, immutable data structures:
- All state changes are immutable transformations
- Efficient structural sharing reduces memory overhead
- Thread-safe concurrent access via `Arc` wrapping

### Event-Driven Architecture
- All state changes emit events through the event system
- Type-safe event handling with async support
- Event sourcing capabilities for state reconstruction

### Plugin System
- Dynamic plugin loading and lifecycle management
- Plugin isolation and dependency injection
- Extension points throughout the framework

### Transaction Model
- ACID-compliant transactions for state changes
- Rollback capabilities and transaction logging
- Atomic batch operations

### Middleware Chain
- Request/response processing pipeline
- Configurable middleware ordering
- Cross-cutting concerns handling

## Configuration Files

### Rust Configuration
- **rustfmt.toml**: Code formatting rules (80-char width, 4-space tabs, vertical fn params)
- **Cargo.toml**: Workspace configuration with edition = "2024"
- **release.toml**: Release configuration

### Cursor Rules
The project includes `.cursorrules` for external project integration with comprehensive coding guidelines and architectural patterns.

## Data Model Architecture

### Node System
- Hierarchical document structure with typed nodes
- Attribute-based properties and mark-based formatting
- Content validation and schema constraints
- Tree operations (traversal, manipulation, queries)

### State Management Flow
1. **Transaction Creation**: Bundle related changes
2. **Plugin Processing**: Middleware and plugin hooks
3. **State Application**: Immutable state transformation
4. **Event Emission**: Notify subscribers of changes
5. **Persistence**: Optional state serialization

### Collaboration Architecture
- **Real-time Sync**: Yrs CRDT for conflict-free collaboration
- **WebSocket Server**: High-performance real-time communication
- **Room Management**: Multi-user session handling
- **State Mapping**: Document state to collaboration state conversion

## Business Rule Engine

### Expression Language
- High-performance custom expression evaluator
- Type system with intelligent inference
- Custom function support and method chaining
- WASM compilation support for client-side execution

### Decision Engine
- GoRules JDM standard compliance
- Graph-based decision execution
- Multiple loader strategies (filesystem, memory, closure)
- Rule composition and chaining

## Performance Considerations

### Memory Management
- Immutable data structures with structural sharing
- Arc-based reference counting for shared ownership
- LRU caching for frequently accessed data
- Memory-mapped file operations for large datasets

### Async Processing
- Tokio-based async runtime
- Background task processing
- Connection pooling for external resources
- Backpressure handling

### Benchmarking
The expression crate includes comprehensive benchmarks:
- Lexer performance testing
- Standard expression evaluation
- Unary operation benchmarks
- Isolate execution benchmarks

## Development Workflow

### Testing Strategy
- Unit tests for all public APIs
- Integration tests for cross-crate functionality
- Property-based testing where appropriate
- Mock external dependencies
- Test error conditions and edge cases

### Code Organization
- Each crate has clear single responsibility
- Examples demonstrate key functionality
- Comprehensive documentation with examples
- Consistent error handling patterns

### Dependencies Management
- Workspace-level dependency management
- Minimal external dependencies
- Security-audited dependencies
- Version pinning in Cargo.lock

## Common Patterns

### Error Handling
```rust
use anyhow::{Result, Context};
use thiserror::Error;

// For library errors, use thiserror
#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("Node not found: {id}")]
    NodeNotFound { id: String },
}

// For application errors, use anyhow
fn process_document() -> Result<()> {
    let doc = load_document().context("Failed to load document")?;
    Ok(())
}
```

### State Updates
```rust
use mf_state::{Transaction, State};
use mf_transform::node_step::AddNodeStep;

// Always use transactions for state changes
let mut transaction = Transaction::new();
transaction.add_step(AddNodeStep::new(node, parent_id));
transaction.set_meta("action", "add_node");

let new_state = runtime.apply_transaction(transaction).await?;
```

### Plugin Development
```rust
use mf_state::plugin::{Plugin, PluginTrait};
use async_trait::async_trait;

#[derive(Debug)]
struct ValidationPlugin;

#[async_trait]
impl PluginTrait for ValidationPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> Result<Option<Transaction>> {
        // Plugin logic here
        Ok(None)
    }
}
```

## Debugging and Monitoring

### Logging
The framework uses `tracing` for structured logging:
```rust
use tracing::{info, debug, error, instrument};

#[instrument]
async fn process_transaction(tx: &Transaction) -> Result<()> {
    debug!("Processing transaction: {}", tx.id);
    // Process...
    info!("Transaction completed successfully");
    Ok(())
}
```

### Performance Monitoring
- Built-in metrics collection
- Transaction timing tracking
- Memory usage monitoring
- Plugin execution profiling

## Integration Notes

### Real-time Collaboration
For collaborative features, the framework provides seamless integration between document state and collaboration state through mapping layers.

### Rules Engine Integration
Business logic can be externalized through the rules engine, supporting dynamic rule evaluation without code changes.

### File Format Support
Documents can be serialized to multiple formats (JSON, CBOR, MessagePack) with compression support.

### Search Integration
Full-text search capabilities with indexing, query parsing, and result ranking.

This framework is particularly well-suited for applications requiring:
- Complex document editing with collaboration
- Dynamic business rule evaluation
- High-performance data transformation
- Extensible plugin architectures
- Real-time collaborative features
---
layout: home

hero:
  name: "ModuForge-RS"
  text: "Modular Application Framework"
  tagline: "High-performance Rust-based modular framework with state management, rules engine, real-time collaboration and plugin extensibility"
  image:
    src: /logo.svg
    alt: ModuForge-RS
  actions:
    - theme: brand
      text: Get Started
      link: /en/setup-external-project
    - theme: alt
      text: Plugin Development Guide
      link: /en/plugin-development-guide
    - theme: alt
      text: View on GitHub
      link: https://github.com/Cassielxd/moduforge-rs

features:
  - icon: 🏗️
    title: Modular Architecture
    details: Highly modular architecture with 14 independent crates for flexible composition and on-demand integration.
  
  - icon: 🚀
    title: High-Performance Runtime
    details: Tokio-based async runtime with immutable data structures and concurrent transaction processing.
  
  - icon: 🔧
    title: Plugin Ecosystem
    details: Complete plugin development framework with dependency management, lifecycle control, and hot-swapping.
  
  - icon: 📊
    title: Rules Engine
    details: Built-in GoRules JDM-compatible rules engine with high-performance expression language.
  
  - icon: 🤝
    title: Real-time Collaboration
    details: Conflict-free collaborative system based on Yrs CRDT with WebSocket real-time synchronization.
  
  - icon: 🎯
    title: Business Neutral
    details: Zero business logic coupling, adaptable to editors, pricing, workflow, and other scenarios through extensions.
---

## What is ModuForge-RS?

ModuForge-RS is a high-performance modular application framework built in Rust, specifically designed for constructing complex business applications. The framework consists of 14 independent crates, providing complete solutions from data modeling, state management, rules engine to real-time collaboration.

### Core Capabilities

🏗️ **Modular Design** - 14 specialized crates for flexible composition and extension  
⚡ **High-Performance Runtime** - Tokio-based async architecture with high concurrency support  
🔧 **Plugin Ecosystem** - Complete plugin development framework with hot-swapping and dependency management  
📊 **Rules Engine** - Built-in business rules engine with dynamic decision-making and expression computing  
🤝 **Real-time Collaboration** - CRDT-based conflict-free collaboration with multi-user real-time editing  
🎯 **Business Neutral** - Zero business logic coupling, adaptable to any domain application scenarios

### How ModuForge Works

- **model**: Defines basic data including Nodes, Marks, Schemas, etc.
- **state**: Manages state, primarily responsible for state updates and plugin scheduling.
- **transform**: Implements transactions similar to database transactions, ensuring atomicity and data consistency.
- **core**: Combines model, state, and transform to implement core runtime functionality.
- **rules**: Rule engine system including expression parsing, backend execution, and template system.

### Core Features

#### 🏗️ **Architecture Components**

- **Async Processor**: High-performance async task processing
- **Event System**: Type-safe event dispatch and handling
- **Extension Mechanism**: Flexible plugin and extension loading
- **Middleware Support**: Configurable request/response pipeline
- **Flow Control**: Sync and async flow management

#### 📊 **Data Model**

- **Node System**: Hierarchical document node structure
- **Mark System**: Document formatting and attribute marking
- **Attribute System**: Type-safe property management
- **Schema Definition**: Document structure validation
- **Content Matching**: Smart content validation and matching

#### 🔄 **State Management**

- **Immutable State**: Persistent data structures based on im-rs
- **Transaction Processing**: ACID-compatible transaction system
- **Resource Management**: Global resource table and lifecycle management
- **Plugin System**: Dynamic plugin loading with state isolation
- **Logging System**: Structured logging and performance monitoring

### Getting Started

1. **[Setup External Project](/en/setup-external-project)** - Learn how to integrate ModuForge-RS into your project
2. **[Plugin Development Guide](/en/plugin-development-guide)** - Build custom plugins for your use case
3. **[Architecture Use Cases](/en/architecture_use_cases)** - Explore different application scenarios
4. **[Feature Showcase](/en/demo-showcase)** - See ModuForge-RS in action

### Use Cases

ModuForge-RS is designed for scenarios requiring:

- **Collaborative Editors** - Real-time document collaboration with conflict resolution
- **Workflow Engines** - Complex business process orchestration
- **Rule Engines** - Dynamic business rule evaluation and execution
- **Data Processing Pipelines** - ETL and data transformation workflows
- **Content Management** - Version-controlled content with audit trails

### Community

- **GitHub**: [moduforge-rs](https://github.com/Cassielxd/moduforge-rs)
- **Documentation**: Comprehensive guides and API reference
- **Examples**: Real-world integration examples and demos 
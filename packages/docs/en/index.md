---
layout: home

hero:
  name: "ModuForge-RS"
  text: "Rust-based State Management Framework"
  tagline: "Immutable data structures and event-driven architecture for any business scenario"
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
  - icon: 🚀
    title: Event-Driven Architecture
    details: Built on immutable data structures with comprehensive event system for reliable state management.
  
  - icon: 🔧
    title: Plugin System
    details: Highly modular architecture with dynamic plugin loading and lifecycle management.
  
  - icon: ⚡
    title: Async Processing
    details: High-performance async runtime with middleware support and concurrent transaction processing.
  
  - icon: 🎯
    title: Business Agnostic
    details: No business logic binding - customize and extend for any use case through extensions.
  
  - icon: 📊
    title: Rules Engine
    details: Powerful rule engine with expression parsing, decision trees, and template system.
  
  - icon: 🔄
    title: Real-time Collaboration
    details: Built-in collaboration features with WebSocket support and conflict resolution.
---

## Quick Start

ModuForge-RS is a Rust-based state management and data transformation framework focusing on immutable data structures and event-driven architecture. It provides a business-agnostic editor core implementation that can be customized and extended to support the needs of any business scenario.

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
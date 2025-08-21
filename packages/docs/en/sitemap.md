# ModuForge-RS Documentation Sitemap

## 📚 Core Documentation

### Getting Started
- [Project Overview](/en/) - Introduction and key features
- [Quick Start](/en/quick-start) - Get started with the framework
- [Architecture Overview](/en/architecture-overview) - Overall framework architecture
- [API Reference](/en/api-reference) - Complete API documentation

### Development Guide
- [Plugin Development Guide](/en/plugin-development-guide) - Detailed plugin development tutorial
- [Macro System Guide](/en/macro-system-guide) - Node and Mark derive macro usage guide
- [Macro Expansion Example](/en/macro-expansion-example) - Complete macro expansion results showcase
- [Performance Guide](/en/performance-guide) - Performance optimization best practices
- [External Project Integration](/en/setup-external-project) - Integrate into existing projects
- [Integration Example](/en/example-integration-project) - Complete integration example

## 🏗️ Architecture Design

### Design Documents
- [Architecture Design Overview](/en/architecture-design) - Complete architecture design
- [Use Cases Analysis](/en/architecture_use_cases) - Applicable scenarios analysis
- [Architecture Limitations](/en/architecture_limitations_analysis) - Design constraints explanation
- [Business Dependency Design](/en/business_dependency_design) - Business layer dependencies
- [Meta-based Dependency Design](/en/meta_based_dependency_design) - Metadata-driven design

## 💼 Business Applications

### Application Examples
- [Node Budget Mapping](/en/node-budget-mapping) - Budget management application
- [Custom Function Development](/en/CUSTOM_FUNCTIONS) - Extend expression language
- [Enhanced History Features](/en/simple_enhanced_history) - History management

## 🤝 Collaboration & Deployment

### Collaboration System
- [Collaboration System](/en/collaboration-system) - Real-time collaboration features
- [WebSocket Troubleshooting](/en/websocket-error-troubleshooting) - Connection issue resolution

### Deployment Guide
- [Deployment Guide](/en/DEPLOYMENT) - Production environment deployment

## 🎯 Demos & Analysis

### Project Showcase
- [Feature Showcase](/en/demo-showcase) - Feature demonstrations
- [Project Analysis](/en/ANALYSIS) - Technical project analysis

## 🌐 Multi-language Versions

### Chinese Documentation
- [中文版本](/) - Complete Chinese documentation
- All English documents have corresponding Chinese versions in root directory

## 📁 Documentation Structure

```
packages/docs/
├── 📄 Core Documents (Chinese)
│   ├── index.md (Homepage)
│   ├── quick-start.md (Quick Start)
│   ├── api-reference.md (API Reference)
│   ├── architecture-overview.md (Architecture Overview)
│   └── performance-guide.md (Performance Guide)
├── 🔧 Development Guides
│   ├── plugin-development-guide.md
│   ├── setup-external-project.md
│   └── example-integration-project.md
├── 🏗️ Architecture Design
│   ├── architecture-design.md
│   ├── architecture_use_cases.md
│   ├── architecture_limitations_analysis.md
│   ├── business_dependency_design.md
│   └── meta_based_dependency_design.md
├── 💼 Business Applications
│   ├── node-budget-mapping.md
│   ├── CUSTOM_FUNCTIONS.md
│   └── simple_enhanced_history.md
├── 🤝 Collaboration & Deployment
│   ├── collaboration-system.md
│   ├── DEPLOYMENT.md
│   └── websocket-error-troubleshooting.md
├── 🎯 Demos & Analysis
│   ├── demo-showcase.md
│   └── ANALYSIS.md
└── 🌐 English Documentation
    └── en/ (English versions of all documents)
```

## 🔍 Quick Find

### By Role
- **New Users**: [Project Overview](/en/) → [Quick Start](/en/quick-start) → [Integration Example](/en/example-integration-project)
- **Developers**: [API Reference](/en/api-reference) → [Plugin Development](/en/plugin-development-guide) → [Performance Guide](/en/performance-guide)
- **Architects**: [Architecture Overview](/en/architecture-overview) → [Architecture Design](/en/architecture-design) → [Use Cases Analysis](/en/architecture_use_cases)
- **DevOps**: [Deployment Guide](/en/DEPLOYMENT) → [Troubleshooting](/en/websocket-error-troubleshooting)

### By Technical Domain
- **State Management**: [API Reference - mf-state](/en/api-reference#mf-state-api)
- **Rules Engine**: [API Reference - mf-engine](/en/api-reference#mf-engine-api)
- **Collaboration**: [Collaboration System](/en/collaboration-system) → [API Reference - mf-collaboration](/en/api-reference#mf-collaboration-api)
- **Performance**: [Performance Guide](/en/performance-guide) → [Architecture Limitations](/en/architecture_limitations_analysis)

## 🛠️ Documentation Tools

### Local Development
```bash
# Start documentation development server
npm run dev

# Build documentation
npm run build

# Preview build results
npm run preview
```

### Documentation Configuration
- VitePress configuration: `.vitepress/config.ts`
- Supports Mermaid chart rendering
- Supports multi-language switching
- Supports local search

---

> 💡 **Tip**: Use the documentation site's search function to quickly find relevant content, supports both Chinese and English search.
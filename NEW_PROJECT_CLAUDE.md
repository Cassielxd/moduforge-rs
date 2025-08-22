# CLAUDE.md - 使用 ModuForge-RS 框架

这个文件提供在新项目中使用 ModuForge-RS 框架的完整指导。

## 项目概述

本项目基于 ModuForge-RS 框架构建，这是一个全面的 Rust 状态管理和数据转换框架，具有不可变数据结构、事件驱动架构、协作功能和强大的规则引擎。

## 框架架构图

### 整体架构概览

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           ModuForge-RS 框架架构                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐              │
│  │ 前端层 (Frontend)│    │  协作层 (Collab) │    │ 规则引擎 (Rules) │              │
│  │                 │    │                 │    │                 │              │
│  │ • Tauri/Electron│    │ • 实时同步       │    │ • 表达式语言     │              │
│  │ • Vue/React     │    │ • CRDT (Yrs)    │    │ • 决策引擎       │              │
│  │ • IPC 通信      │    │ • WebSocket     │    │ • 模板渲染       │              │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘              │
│           │                       │                       │                     │
│           └───────────────────────┼───────────────────────┘                     │
│                                   │                                             │
│  ┌─────────────────────────────────┼─────────────────────────────────────────┐   │
│  │                      核心运行时 (Core Runtime)                           │   │
│  │                                 │                                       │   │
│  │  ┌─────────────┐    ┌──────────┼──────────┐    ┌─────────────────────┐  │   │
│  │  │ 扩展管理器   │    │   中间件  │  链      │    │   插件系统          │  │   │
│  │  │ Extension   │    │ Middleware│ Chain    │    │   Plugin System     │  │   │
│  │  │ Manager     │    │           │          │    │                     │  │   │
│  │  │             │    │ • 请求拦截│          │    │ • 状态字段          │  │   │
│  │  │ • 节点注册  │    │ • 响应处理│          │    │ • 生命周期管理      │  │   │
│  │  │ • 标记注册  │    │ • 日志记录│          │    │ • 依赖注入          │  │   │
│  │  │ • 模式验证  │    │ • 性能监控│          │    │                     │  │   │
│  │  └─────────────┘    └───────────┼──────────┘    └─────────────────────┘  │   │
│  │                                 │                                       │   │
│  │  ┌─────────────────────────────────────────────────────────────────────┐  │   │
│  │  │                    事务处理层 (Transaction Layer)                   │  │   │
│  │  │                                                                     │  │   │
│  │  │ ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │  │   │
│  │  │ │   事务管理   │  │   步骤执行   │  │   回滚机制   │  │   历史管理   │ │  │   │
│  │  │ │ Transaction │  │    Step     │  │   Rollback  │  │   History   │ │  │   │
│  │  │ │   Manager   │  │  Execution  │  │             │  │   Manager   │ │  │   │
│  │  │ │             │  │             │  │             │  │             │ │  │   │
│  │  │ │ • ACID 保证 │  │ • 原子操作  │  │ • 撤销/重做 │  │ • 版本控制  │ │  │   │
│  │  │ │ • 批量处理  │  │ • 顺序执行  │  │ • 状态恢复  │  │ • 分支管理  │ │  │   │
│  │  │ └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │  │   │
│  │  └─────────────────────────────────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                   │                                             │
│  ┌─────────────────────────────────┼─────────────────────────────────────────┐   │
│  │                        状态管理层 (State Layer)                         │   │
│  │                                 │                                       │   │
│  │  ┌─────────────┐    ┌──────────┼──────────┐    ┌─────────────────────┐  │   │
│  │  │  状态容器   │    │   资源   │   管理    │    │    事件系统         │  │   │
│  │  │    State    │    │ Resource │ Manager   │    │  Event System       │  │   │
│  │  │ Container   │    │          │           │    │                     │  │   │
│  │  │             │    │          │           │    │ • 事件发布          │  │   │
│  │  │ • 不可变性  │    │ • 缓存   │           │    │ • 事件订阅          │  │   │
│  │  │ • 结构共享  │    │ • 连接池 │           │    │ • 异步处理          │  │   │
│  │  │ • 并发安全  │    │ • 配置   │           │    │ • 类型安全          │  │   │
│  │  └─────────────┘    └───────────┼──────────┘    └─────────────────────┘  │   │
│  └─────────────────────────────────┼─────────────────────────────────────────┘   │
│                                   │                                             │
│  ┌─────────────────────────────────┼─────────────────────────────────────────┐   │
│  │                       数据模型层 (Model Layer)                          │   │
│  │                                 │                                       │   │
│  │  ┌─────────────┐    ┌──────────┼──────────┐    ┌─────────────────────┐  │   │
│  │  │   节点树    │    │   属性   │   标记    │    │      模式验证       │  │   │
│  │  │  Node Tree  │    │  Attrs   │  Marks    │    │  Schema Validation  │  │   │
│  │  │             │    │          │           │    │                     │  │   │
│  │  │ • 层次结构  │    │ • 键值对 │ • 格式化  │    │ • 类型检查          │  │   │
│  │  │ • 遍历操作  │    │ • 类型化 │ • 样式    │    │ • 约束验证          │  │   │
│  │  │ • 查询语法  │    │ • 序列化 │ • 语义    │    │ • 内容模式          │  │   │
│  │  └─────────────┘    └───────────┼──────────┘    └─────────────────────┘  │   │
│  └─────────────────────────────────┼─────────────────────────────────────────┘   │
│                                   │                                             │
│  ┌─────────────────────────────────┼─────────────────────────────────────────┐   │
│  │                      存储与 I/O 层 (Storage & I/O)                      │   │
│  │                                 │                                       │   │
│  │  ┌─────────────┐    ┌──────────┼──────────┐    ┌─────────────────────┐  │   │
│  │  │  文件系统   │    │   持久化 │   搜索    │    │     网络通信        │  │   │
│  │  │ File System │    │Persistence│  Search   │    │   Network Comm      │  │   │
│  │  │             │    │           │           │    │                     │  │   │
│  │  │ • 多格式    │    │ • SQLite  │ • 索引    │    │ • WebSocket         │  │   │
│  │  │ • 压缩      │    │ • 快照    │ • 查询    │    │ • HTTP API          │  │   │
│  │  │ • 加密      │    │ • 恢复    │ • 排序    │    │ • 消息队列          │  │   │
│  │  └─────────────┘    └───────────┼──────────┘    └─────────────────────┘  │   │
│  └─────────────────────────────────┼─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 数据流图

```
前端 UI (Frontend)                       Rust 后端 (Backend)
┌─────────────────────────────┐         ┌─────────────────────────────┐
│                             │         │                             │
│  ┌─────────────────────┐    │         │    ┌─────────────────────┐  │
│  │   前端框架           │    │         │    │   Tauri Commands    │  │
│  │ (Vue/React/Svelte)  │    │         │    │                     │  │
│  │                     │    │   IPC   │    │ • 业务逻辑处理       │  │
│  │ • 用户界面          │◀───┼─────────┼───▶│ • 状态管理          │  │
│  │ • 事件处理          │    │         │    │ • 数据验证          │  │
│  │ • 状态同步          │    │         │    │                     │  │
│  └─────────────────────┘    │         │    └─────────────────────┘  │
│                             │         │              │              │
│  ┌─────────────────────┐    │         │              ▼              │
│  │   Tauri API         │    │         │    ┌─────────────────────┐  │
│  │                     │    │         │    │   ModuForge Core    │  │
│  │ • invoke()          │    │         │    │                     │  │
│  │ • listen()          │    │         │    │ ┌─────────────────┐ │  │
│  │ • emit()            │    │         │    │ │   中间件链       │ │  │
│  └─────────────────────┘    │         │    │ │ Middleware Chain│ │  │
└─────────────────────────────┘         │    │ └─────────────────┘ │  │
                                        │    │          │          │  │
                                        │    │          ▼          │  │
┌─────────────────────────────┐         │    │ ┌─────────────────┐ │  │
│   协作服务 (可选)            │         │    │ │   事务层         │ │  │
│ Collaboration Service       │         │    │ │ Transaction     │ │  │
│                             │         │    │ │    Layer        │ │  │
│ • WebSocket 连接            │◀────────┼────┼─┤                 │ │  │
│ • CRDT 同步                 │         │    │ │ • ACID 保证     │ │  │
│ • 多用户协作                │         │    │ │ • 批量处理      │ │  │
└─────────────────────────────┘         │    │ └─────────────────┘ │  │
                                        │    │          │          │  │
                                        │    │          ▼          │  │
                                        │    │ ┌─────────────────┐ │  │
                                        │    │ │   状态层         │ │  │
                                        │    │ │  State Layer    │ │  │
                                        │    │ │                 │ │  │
                                        │    │ │ • 不可变状态    │ │  │
                                        │    │ │ • 事件发布      │ │  │
                                        │    │ │ • 插件系统      │ │  │
                                        │    │ └─────────────────┘ │  │
                                        │    └─────────────────────┘  │
                                        │              │              │
                                        │              ▼              │
                                        │    ┌─────────────────────┐  │
                                        │    │   存储层            │  │
                                        │    │  Storage Layer     │  │
                                        │    │                    │  │
                                        │    │ • 文件系统         │  │
                                        │    │ • 数据库           │  │
                                        │    │ • 搜索引擎         │  │
                                        │    └─────────────────────┘  │
                                        └─────────────────────────────┘

IPC 通信机制:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
• invoke(): 前端调用后端函数 (Command Pattern)
• listen(): 前端监听后端事件 (Event Pattern) 
• emit(): 后端向前端发送事件 (Publish Pattern)
```

### 组件交互图

```
应用层 Application Layer
│
├── 编辑器 Editor
│   ├── 配置 Configuration
│   ├── 扩展管理 Extension Management
│   └── 生命周期 Lifecycle
│
├── 中间件链 Middleware Chain
│   ├── 请求处理 Request Processing
│   ├── 响应拦截 Response Interception
│   └── 横切关注点 Cross-cutting Concerns
│
├── 事务系统 Transaction System
│   ├── 事务管理器 Transaction Manager
│   │   ├── 开始事务 Begin Transaction
│   │   ├── 提交事务 Commit Transaction
│   │   └── 回滚事务 Rollback Transaction
│   │
│   ├── 步骤执行器 Step Executor
│   │   ├── 节点操作 Node Operations
│   │   │   ├── 添加节点 Add Node
│   │   │   ├── 删除节点 Remove Node
│   │   │   └── 更新节点 Update Node
│   │   │
│   │   ├── 属性操作 Attribute Operations
│   │   │   ├── 设置属性 Set Attribute
│   │   │   └── 删除属性 Remove Attribute
│   │   │
│   │   └── 标记操作 Mark Operations
│   │       ├── 添加标记 Add Mark
│   │       └── 删除标记 Remove Mark
│   │
│   └── 历史管理器 History Manager
│       ├── 撤销 Undo
│       ├── 重做 Redo
│       └── 快照 Snapshot
│
├── 状态管理 State Management
│   ├── 状态容器 State Container
│   │   ├── 不可变状态 Immutable State
│   │   ├── 结构共享 Structural Sharing
│   │   └── 并发访问 Concurrent Access
│   │
│   ├── 资源管理器 Resource Manager
│   │   ├── 连接池 Connection Pool
│   │   ├── 缓存系统 Cache System
│   │   └── 配置管理 Configuration
│   │
│   └── 事件系统 Event System
│       ├── 事件发布器 Event Publisher
│       ├── 事件订阅器 Event Subscriber
│       └── 事件路由 Event Router
│
├── 数据模型 Data Model
│   ├── 节点树 Node Tree
│   │   ├── 层次结构 Hierarchical Structure
│   │   ├── 遍历算法 Traversal Algorithms
│   │   └── 查询接口 Query Interface
│   │
│   ├── 属性系统 Attribute System
│   │   ├── 类型系统 Type System
│   │   ├── 验证器 Validators
│   │   └── 序列化 Serialization
│   │
│   └── 模式管理 Schema Management
│       ├── 类型定义 Type Definition
│       ├── 约束验证 Constraint Validation
│       └── 迁移支持 Migration Support
│
└── 存储层 Storage Layer
    ├── 文件系统 File System
    │   ├── 多格式支持 Multi-format Support
    │   │   ├── JSON
    │   │   ├── CBOR
    │   │   └── MessagePack
    │   │
    │   ├── 压缩算法 Compression
    │   └── 加密支持 Encryption
    │
    ├── 持久化 Persistence
    │   ├── SQLite 数据库 SQLite Database
    │   ├── 快照管理 Snapshot Management
    │   └── 增量备份 Incremental Backup
    │
    └── 搜索引擎 Search Engine
        ├── 索引构建 Index Building
        ├── 查询解析 Query Parsing
        └── 结果排序 Result Ranking
```

### 核心设计模式

#### 1. CQRS (命令查询责任分离)
```
命令端 (Command Side)        查询端 (Query Side)
┌─────────────────────┐     ┌─────────────────────┐
│     命令处理器       │     │     查询处理器       │
│  Command Handler    │     │   Query Handler     │
│                     │     │                     │
│ • 事务操作          │     │ • 读取操作          │
│ • 状态变更          │     │ • 数据投影          │
│ • 事件发布          │     │ • 缓存优化          │
└─────────────────────┘     └─────────────────────┘
           │                           ▲
           │                           │
           ▼                           │
┌─────────────────────────────────────────────────┐
│              事件存储和状态                      │
│           Event Store & State                   │
│                                                 │
│ • 不可变事件日志                                │
│ • 状态快照                                      │
│ • 版本控制                                      │
└─────────────────────────────────────────────────┘
```

#### 2. 事件驱动架构 (Event-Driven Architecture)
```
事件生产者          事件总线          事件消费者
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   业务操作   │───▶│   事件路由   │───▶│   UI 更新    │
│ Business Op │    │Event Router │    │ UI Update   │
└─────────────┘    └─────────────┘    └─────────────┘
                          │
                          ├───▶┌─────────────┐
                          │    │   持久化     │
                          │    │Persistence  │
                          │    └─────────────┘
                          │
                          └───▶┌─────────────┐
                               │   协作同步   │
                               │Collaboration│
                               └─────────────┘
```

#### 3. 插件架构 (Plugin Architecture)
```
                    ┌─────────────────────────────────┐
                    │         插件管理器               │
                    │      Plugin Manager            │
                    │                                │
                    │ • 插件注册                      │
                    │ • 生命周期管理                   │
                    │ • 依赖解析                      │
                    │ • 优先级排序                    │
                    └─────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
┌───────▼───────┐        ┌─────────▼─────────┐        ┌───────▼───────┐
│   状态插件     │        │    中间件插件      │        │   扩展插件     │
│ State Plugin  │        │ Middleware Plugin │        │Extension Plugin│
│               │        │                   │        │               │
│ • 状态字段     │        │ • 请求拦截         │        │ • 节点类型     │
│ • 资源管理     │        │ • 响应处理         │        │ • 标记类型     │
│ • 事务处理     │        │ • 日志记录         │        │ • 模式验证     │
└───────────────┘        └───────────────────┘        └───────────────┘

插件间数据依赖流 (Plugin Data Dependency Flow):
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          插件间事务流转机制                                      │
│                                                                                 │
│  初始事务 → 插件 A → 新事务₁ → 插件 B → 新事务₂ → 插件 C                         │
│                                                                                 │
│  ┌─────────────┐ 1.计算完成  ┌─────────────┐ 2.验证完成  ┌─────────────┐          │
│  │  插件 A     │────────────▶│  插件 B     │────────────▶│  插件 C     │          │
│  │(计算插件)    │ 提交新事务   │ (验证插件)   │ 提交新事务   │ (汇总插件)   │          │
│  │Priority: 10 │             │Priority: 20 │             │Priority: 30 │          │
│  └─────────────┘             └─────────────┘             └─────────────┘          │
│        ▲                           ▲                           ▲                 │
│        │                           │                           │                 │
│  ┌──────────────┐          ┌──────────────┐          ┌──────────────┐            │
│  │ 原始事务      │          │ 通知事务₁     │          │ 通知事务₂     │            │
│  │ meta:        │          │ meta:        │          │ meta:        │            │
│  │"nodes_to_    │          │"price_calc_  │          │"validation_  │            │
│  │ calculate"   │          │ completed"   │          │ completed"   │            │
│  │              │          │"calculation_ │          │"validation_  │            │
│  │              │          │ results"     │          │ results"     │            │
│  └──────────────┘          └──────────────┘          └──────────────┘            │
│                                                                                 │
│  关键机制:                                                                       │
│  • 插件主动提交新事务: runtime.dispatch_flow(new_transaction)                     │
│  • Meta 传递计算结果: transaction.set_meta("results", data)                      │
│  • Resource Table 持久化: resource_manager.add(key, value)                     │
│  • 事务链式触发: A完成→通知B→B完成→通知C→C完成                                    │
│                                                                                 │
│  ┌─────────────────────────────────────────────────────────────────────────┐    │
│  │                    事务时序图                                            │    │
│  │                                                                         │    │
│  │  时间轴   插件A          插件B          插件C                              │    │
│  │    │      │              │              │                              │    │
│  │    ▼      │              │              │                              │    │
│  │   T1   ▶ 处理原始事务      │              │                              │    │
│  │   T2     │ ▶ 提交通知事务₁  │              │                              │    │
│  │   T3     │              ▶ 收到通知事务₁   │                              │    │
│  │   T4     │              │ ▶ 提交通知事务₂  │                              │    │
│  │   T5     │              │              ▶ 收到通知事务₂                   │    │
│  │   T6     │              │              │ ▶ 生成最终结果                  │    │
│  │                                                                         │    │
│  └─────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 关键架构原则

#### 1. 不可变性 (Immutability)
- **结构共享**: 使用 `im-rs` 实现高效的不可变数据结构
- **写时复制**: 只有变更部分才会创建新的数据结构
- **并发安全**: 天然支持多线程并发访问

#### 2. 单向数据流 (Unidirectional Data Flow)
```
前端 Frontend                     Rust 后端 Backend
┌─────────────┐    IPC invoke    ┌─────────────┐
│ User Action │ ──────────────▶  │ Transaction │
└─────────────┘                  └─────────────┘
      ▲                                 │
      │                                 ▼
      │                         ┌─────────────┐
      │                         │    State    │
      │                         └─────────────┘
      │                                 │
      │                                 ▼
      │          IPC emit/event  ┌─────────────┐
      └─────────────────────────  │   Events    │
                                 └─────────────┘
```

#### 3. 组合优于继承 (Composition over Inheritance)
- **特征组合**: 使用 Rust trait 系统实现行为组合
- **插件系统**: 通过插件动态扩展功能
- **中间件链**: 可组合的处理管道

#### 4. 依赖注入 (Dependency Injection)
- **资源管理**: 通过资源表管理依赖
- **插件依赖**: 自动解析插件间依赖关系
- **配置注入**: 运行时配置注入

## 依赖配置

### Cargo.toml 依赖声明

```toml
[package]
name = "your-project"
version = "0.1.0"
edition = "2021"

[dependencies]
# 核心框架
moduforge-core = "0.4.12"        # 异步运行时、事件系统、扩展管理
moduforge-model = "0.4.12"       # 数据模型 - 节点、标记、属性、模式
moduforge-state = "0.4.12"       # 状态管理、事务、插件、资源管理
moduforge-transform = "0.4.12"   # 数据转换操作和事务步骤

# 规则引擎系统
moduforge-engine = "0.4.12"      # 基于 GoRules JDM 标准的业务规则引擎
moduforge-expression = "0.4.12"  # 高性能表达式语言，支持 WASM
moduforge-template = "0.4.12"    # 模板渲染系统

# 协作与数据
moduforge-collaboration = "0.4.12"        # 使用 Yrs CRDT 的实时协作编辑
moduforge-collaboration-client = "0.4.12" # 客户端协作工具
moduforge-file = "0.4.12"                 # 文档序列化/反序列化 (JSON, CBOR, MessagePack)
moduforge-search = "0.4.12"               # 搜索索引和查询功能
moduforge-persistence = "0.4.12"          # 数据持久化和恢复机制

# 开发工具
moduforge-macro = "0.4.12"       # 节点、插件、扩展的过程宏
moduforge-derive = "0.4.12"      # 依赖注入的过程宏

# 其他必需依赖
tokio = { version = "1", features = ["full"] }
anyhow = "1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
lazy_static = "1.4"   # 用于静态节点和标记定义
async-trait = "0.1"   # 用于异步 trait 实现
tracing = "0.1"       # 日志系统
tracing-subscriber = "0.3"  # 日志订阅器

# Tauri 桌面应用开发 (可选)
tauri = { version = "1.0", features = ["api-all"] }
tauri-build = { version = "1.0", features = [] }
```

### 可选依赖组合

根据项目需求选择合适的依赖组合：

```toml
# 最小核心 - 基础文档处理
[dependencies]
moduforge-core = "0.4.12"
moduforge-model = "0.4.12"
moduforge-state = "0.4.12"

# 规则引擎 - 业务逻辑处理
[dependencies]
moduforge-engine = "0.4.12"
moduforge-expression = "0.4.12"

# 协作系统 - 实时多用户编辑
[dependencies]
moduforge-collaboration = "0.4.12"
moduforge-collaboration-client = "0.4.12"

# 数据处理 - 完整的数据管道
[dependencies]
moduforge-file = "0.4.12"
moduforge-search = "0.4.12"
moduforge-persistence = "0.4.12"

# 桌面应用 - Tauri 集成
[dependencies]
moduforge-core = "0.4.12"
moduforge-model = "0.4.12"
moduforge-state = "0.4.12"
tauri = { version = "1.0", features = ["api-all"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
tauri-build = { version = "1.0", features = [] }
```

## 常用导入

```rust
// 核心功能
use mf_core::{Runtime, Config, Event, Extension};
use mf_model::{Node, Mark, Attrs, Schema, Tree};
use mf_state::{State, Transaction, Plugin};
use mf_transform::{AddNodeStep, RemoveNodeStep, UpdateAttrsStep};

// 规则引擎
use mf_engine::{Engine, Decision};
use mf_expression::{Expression, Variable};

// 协作功能
use mf_collaboration::{SyncService, YrsManager};
use mf_collaboration_client::{CollaborationClient, Mapping};

// 文件和持久化
use mf_file::{Document, ZipDocWriter, ZipDocReader};
use mf_persistence::{SqlitePersistence, RecoveryManager};

// 搜索功能
use mf_search::{SearchService, Indexer, Query};

// 宏
use mf_macro::{Node, Plugin, Extension};

// 错误处理
use anyhow::{Result, Context};
```

## 基础使用模式

### 1. 节点定义 (使用宏简化定义)

```rust
use lazy_static::lazy_static;
use mf_core::node::Node;
use mf_macro::node;

// 使用 node! 宏定义节点
lazy_static! {
    // 定义单价构成节点
    pub static ref DJGC: Node = node!(
        "djgc",           // 节点名称
        "单价构成",        // 节点描述
        "",               // 节点内容模式
        "value" => "".into()  // 默认属性
    );
    
    // 定义单价构成行节点
    pub static ref DJGC_NODE: Node = node!(
        "djgcRowNode",
        "单价构成行节点",
        "",
        "qfCode" => "".into(),
        "type" => "".into(),
        "code" => "".into(),
        "caculateBase" => "".into(),
        "desc" => "".into(),
        "rate" => "".into(),
        "price" => 0.into()
    );
}

// 构建节点集合的函数
pub fn init_nodes() -> Vec<Node> {
    let mut nodes = vec![DJGC_NODE.clone()];
    let mut djgc = DJGC.clone();
    djgc.set_content("djgcRowNode+");  // 设置内容模式：一个或多个 djgcRowNode
    nodes.push(djgc);
    nodes
}
```

### 2. 标记定义 (Mark)

```rust
use lazy_static::lazy_static;
use mf_core::mark::Mark;
use mf_macro::mark;

pub const BG_COLOR_STR: &str = "bgColor";
pub const FOOTNOTE_STR: &str = "footnote";

lazy_static! {
    // 定义背景颜色标记
    pub static ref BG_COLOR: Mark = mark!(
        BG_COLOR_STR,
        "背景颜色",
        "value" => "#ffffff".into()
    );
    
    // 定义脚注标记
    pub static ref FOOTNOTE: Mark = mark!(
        FOOTNOTE_STR,
        "脚注",
        "value" => "".into()
    );
}
```

### 3. 中间件编写

```rust
use std::sync::Arc;
use async_trait::async_trait;
use mf_core::{middleware::Middleware, ForgeResult};
use mf_state::{State, Transaction};

/// 收集分部分项措施项目汇总中间件
/// 当编辑区分部分项措施项目节点更新后需要收集汇总
#[derive(Debug)]
pub struct CollectFbfxCsxmMiddleware;

#[async_trait]
impl Middleware for CollectFbfxCsxmMiddleware {
    /// 返回中间件的名称
    fn name(&self) -> String {
        "collect_fbfx_csxm".to_string()
    }

    /// 在核心分发之后处理结果
    /// 返回一个可能包含需要额外处理的事务的 MiddlewareResult
    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Transaction],
    ) -> ForgeResult<Option<Transaction>> {
        println!("分部分项措施项目汇总");
        
        for tr in transactions {
            // 检查事务元数据中是否包含定额 ID
            if let Some(de_ids) = tr.get_meta::<Vec<String>>("de_ids") {
                // 汇总对应的定额价格向上汇总
                // 这里可以实现具体的汇总逻辑
                println!("处理定额汇总: {:?}", de_ids);
            }
        }
        
        // 返回 None 表示不需要额外的事务
        // 返回 Some(transaction) 可以触发额外的状态更新
        Ok(None)
    }
}
```

### 4. 插件系统完整实现

#### 4.1 插件元数据和配置

```rust
use mf_state::plugin::{PluginMetadata, PluginConfig, PluginTrait, StateField, PluginSpec, Plugin};
use async_trait::async_trait;
use std::collections::HashMap;
use serde_json::Value;

/// 定义插件元数据
fn create_plugin_metadata() -> PluginMetadata {
    PluginMetadata {
        name: "price_calculator".to_string(),
        version: "1.0.0".to_string(),
        description: "价格计算插件，支持复杂的定价规则".to_string(),
        author: "ModuForge Team".to_string(),
        dependencies: vec!["base_calculator".to_string()], // 依赖基础计算插件
        conflicts: vec!["legacy_pricer".to_string()],      // 与旧版定价插件冲突
        state_fields: vec!["calculation_cache".to_string(), "price_history".to_string()],
        tags: vec!["calculation".to_string(), "pricing".to_string(), "core".to_string()],
    }
}

/// 定义插件配置
fn create_plugin_config() -> PluginConfig {
    let mut settings = HashMap::new();
    settings.insert("max_cache_size".to_string(), Value::Number(1000.into()));
    settings.insert("enable_history".to_string(), Value::Bool(true));
    settings.insert("calculation_timeout".to_string(), Value::Number(5000.into()));
    
    PluginConfig {
        enabled: true,
        priority: 10, // 高优先级，较早执行
        settings,
    }
}
```

#### 4.2 插件状态字段实现

```rust
use std::sync::Arc;
use async_trait::async_trait;
use mf_state::{plugin::StateField, resource::Resource, State, StateConfig, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 价格计算缓存数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationData {
    pub node_id: String,
    pub base_price: f64,
    pub calculated_price: f64,
    pub formula: String,
    pub calculated_at: DateTime<Utc>,
    pub is_valid: bool,
}

/// 插件状态：价格计算器缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculatorState {
    pub cache: HashMap<String, PriceCalculationData>,
    pub total_calculations: u64,
    pub cache_hits: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
    pub max_cache_size: usize,
}

impl Resource for PriceCalculatorState {}

/// 价格计算状态字段管理器
#[derive(Debug)]
pub struct PriceCalculatorStateField;

#[async_trait]
impl StateField for PriceCalculatorStateField {
    /// 初始化插件状态
    async fn init(
        &self,
        config: &StateConfig,
        _instance: &State,
    ) -> Arc<dyn Resource> {
        // 从配置中读取缓存大小
        let max_cache_size = config.get_plugin_setting("price_calculator", "max_cache_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as usize;
        
        Arc::new(PriceCalculatorState {
            cache: HashMap::new(),
            total_calculations: 0,
            cache_hits: 0,
            last_cleanup: None,
            max_cache_size,
        })
    }
    
    /// 应用状态变更
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut state = value.downcast_ref::<PriceCalculatorState>()
            .expect("状态类型错误")
            .clone();
        
        // 检查是否需要计算价格
        if let Some(node_ids) = tr.get_meta::<Vec<String>>("nodes_to_calculate") {
            for node_id in node_ids {
                if let Some(node) = new_state.doc().get_node(&node_id) {
                    // 检查缓存
                    if let Some(cached) = state.cache.get(&node_id) {
                        if cached.is_valid && self.is_cache_fresh(cached) {
                            state.cache_hits += 1;
                            continue;
                        }
                    }
                    
                    // 执行价格计算
                    let calculation_data = self.calculate_price(&node, &state).await;
                    state.cache.insert(node_id, calculation_data);
                    state.total_calculations += 1;
                    
                    // 清理过期缓存
                    if state.cache.len() > state.max_cache_size {
                        self.cleanup_cache(&mut state);
                    }
                }
            }
        }
        
        Arc::new(state)
    }
    
    /// 序列化插件状态
    fn serialize(&self, value: Arc<dyn Resource>) -> Option<Vec<u8>> {
        value.downcast_ref::<PriceCalculatorState>()
            .and_then(|state| serde_json::to_vec(state).ok())
    }
    
    /// 反序列化插件状态
    fn deserialize(&self, data: &Vec<u8>) -> Option<Arc<dyn Resource>> {
        serde_json::from_slice::<PriceCalculatorState>(data)
            .ok()
            .map(|state| Arc::new(state) as Arc<dyn Resource>)
    }
}

impl PriceCalculatorStateField {
    async fn calculate_price(
        &self, 
        node: Arc<Node>, 
        _state: &PriceCalculatorState
    ) -> PriceCalculationData {
        let base_price = node.attrs.get("base_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let quantity = node.attrs.get("quantity")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let rate = node.attrs.get("rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        
        let calculated_price = base_price * quantity * rate;
        
        PriceCalculationData {
            node_id: node.id.clone(),
            base_price,
            calculated_price,
            formula: "base_price * quantity * rate".to_string(),
            calculated_at: Utc::now(),
            is_valid: calculated_price > 0.0,
        }
    }
    
    fn is_cache_fresh(&self, data: &PriceCalculationData) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(data.calculated_at);
        age.num_minutes() < 30 // 30分钟内有效
    }
    
    fn cleanup_cache(&self, state: &mut PriceCalculatorState) {
        let now = Utc::now();
        state.cache.retain(|_, data| {
            let age = now.signed_duration_since(data.calculated_at);
            age.num_hours() < 24 // 保留24小时内的数据
        });
        state.last_cleanup = Some(now);
    }
}
```

#### 4.3 插件业务逻辑实现

```rust
use mf_state::{plugin::PluginTrait, error::StateResult, State, Transaction};

/// 价格计算插件
#[derive(Debug)]
pub struct PriceCalculatorPlugin;

#[async_trait]
impl PluginTrait for PriceCalculatorPlugin {
    /// 获取插件元数据
    fn metadata(&self) -> PluginMetadata {
        create_plugin_metadata()
    }
    
    /// 获取插件配置
    fn config(&self) -> PluginConfig {
        create_plugin_config()
    }
    
    /// 事务过滤：决定是否处理特定事务
    async fn filter_transaction(&self, tr: &Transaction, _state: &State) -> bool {
        // 只处理包含价格计算标识的事务
        tr.has_meta("calculate_prices") || tr.has_meta("nodes_to_calculate")
    }
    
    /// 追加事务：在主事务后生成额外的通知事务
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 检查是否有需要通知的计算完成事件
        for tr in transactions {
            if tr.has_meta("nodes_to_calculate") {
                // 从插件状态中获取计算结果
                if let Some(plugin_state) = new_state.get_field("price_calculator") {
                    if let Some(calc_state) = plugin_state.downcast_ref::<PriceCalculatorState>() {
                        // 检查是否有新的计算结果
                        let recent_calculations: Vec<&PriceCalculationData> = calc_state.cache
                            .values()
                            .filter(|data| {
                                let age = Utc::now().signed_duration_since(data.calculated_at);
                                age.num_seconds() < 60 // 最近1分钟的计算
                            })
                            .collect();
                        
                        if !recent_calculations.is_empty() {
                            // 创建通知事务
                            let mut notification_tx = new_state.tr();
                            notification_tx.set_meta("price_calculation_completed", true);
                            notification_tx.set_meta("calculated_nodes", 
                                recent_calculations.iter()
                                    .map(|data| data.node_id.clone())
                                    .collect::<Vec<String>>()
                            );
                            notification_tx.set_meta("total_calculations", calc_state.total_calculations);
                            notification_tx.set_meta("cache_hit_rate", 
                                if calc_state.total_calculations > 0 {
                                    calc_state.cache_hits as f64 / calc_state.total_calculations as f64
                                } else {
                                    0.0
                                }
                            );
                            
                            return Ok(Some(notification_tx));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
}
```

#### 4.4 插件注册和管理器使用

```rust
use mf_state::plugin::{PluginManager, Plugin, PluginSpec};
use anyhow::Result;

/// 插件注册和初始化
pub async fn setup_plugin_system() -> Result<PluginManager> {
    let plugin_manager = PluginManager::new();
    
    // 创建价格计算插件
    let price_plugin_spec = PluginSpec {
        state_field: Some(Arc::new(PriceCalculatorStateField)),
        tr: Arc::new(PriceCalculatorPlugin),
    };
    let price_plugin = Arc::new(Plugin::new(price_plugin_spec));
    
    // 注册插件
    plugin_manager.register_plugin(price_plugin).await?;
    
    // 创建验证插件（依赖于价格计算插件）
    let validation_plugin_spec = PluginSpec {
        state_field: Some(Arc::new(ValidationStateField)),
        tr: Arc::new(ValidationPlugin),
    };
    let validation_plugin = Arc::new(Plugin::new(validation_plugin_spec));
    plugin_manager.register_plugin(validation_plugin).await?;
    
    // 完成插件注册（会进行依赖检查、冲突检测等）
    plugin_manager.finalize_registration().await?;
    
    // 验证插件系统是否正确初始化
    assert!(plugin_manager.is_initialized().await);
    
    Ok(plugin_manager)
}

/// 获取插件执行顺序
pub async fn get_plugin_execution_order(plugin_manager: &PluginManager) {
    let sorted_plugins = plugin_manager.get_sorted_plugins().await;
    
    println!("插件执行顺序:");
    for (index, plugin) in sorted_plugins.iter().enumerate() {
        let metadata = plugin.get_metadata();
        println!("{}. {} v{} (优先级: {})", 
            index + 1, 
            metadata.name, 
            metadata.version,
            plugin.get_config().priority
        );
        
        if !metadata.dependencies.is_empty() {
            println!("   依赖: {:?}", metadata.dependencies);
        }
        if !metadata.conflicts.is_empty() {
            println!("   冲突: {:?}", metadata.conflicts);
        }
    }
}

/// 验证插件
#[derive(Debug)]
pub struct ValidationPlugin;

#[async_trait]
impl PluginTrait for ValidationPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "data_validator".to_string(),
            version: "1.0.0".to_string(),
            description: "数据验证插件，验证计算结果的合理性".to_string(),
            author: "ModuForge Team".to_string(),
            dependencies: vec!["price_calculator".to_string()], // 依赖价格计算插件
            conflicts: vec![],
            state_fields: vec!["validation_results".to_string()],
            tags: vec!["validation".to_string(), "quality".to_string()],
        }
    }
    
    fn config(&self) -> PluginConfig {
        let mut settings = HashMap::new();
        settings.insert("max_price_threshold".to_string(), Value::Number(1000000.into()));
        settings.insert("min_price_threshold".to_string(), Value::Number(0.into()));
        
        PluginConfig {
            enabled: true,
            priority: 20, // 较低优先级，在价格计算后执行
            settings,
        }
    }
    
    async fn filter_transaction(&self, tr: &Transaction, _state: &State) -> bool {
        // 只处理价格计算完成的事务
        tr.has_meta("price_calculation_completed")
    }
    
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        _old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        for tr in transactions {
            if tr.has_meta("price_calculation_completed") {
                if let Some(calculated_nodes) = tr.get_meta::<Vec<String>>("calculated_nodes") {
                    // 验证计算结果
                    let validation_results = self.validate_calculations(&calculated_nodes, new_state).await;
                    
                    if !validation_results.is_empty() {
                        let mut validation_tx = new_state.tr();
                        validation_tx.set_meta("validation_completed", true);
                        validation_tx.set_meta("validation_results", validation_results);
                        
                        return Ok(Some(validation_tx));
                    }
                }
            }
        }
        Ok(None)
    }
}

impl ValidationPlugin {
    async fn validate_calculations(
        &self, 
        node_ids: &[String], 
        state: &State
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        if let Some(calc_state) = state.get_field("price_calculator")
            .and_then(|s| s.downcast_ref::<PriceCalculatorState>()) {
            
            for node_id in node_ids {
                if let Some(calc_data) = calc_state.cache.get(node_id) {
                    let is_valid = calc_data.calculated_price >= 0.0 
                        && calc_data.calculated_price <= 1_000_000.0;
                    
                    results.push(ValidationResult {
                        node_id: node_id.clone(),
                        is_valid,
                        calculated_price: calc_data.calculated_price,
                        validation_message: if is_valid {
                            "价格计算结果正常".to_string()
                        } else {
                            "价格超出合理范围".to_string()
                        },
                    });
                }
            }
        }
        
        results
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub node_id: String,
    pub is_valid: bool,
    pub calculated_price: f64,
    pub validation_message: String,
}

/// 验证插件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationState {
    pub total_validations: u64,
    pub failed_validations: u64,
    pub last_validation: Option<DateTime<Utc>>,
}

impl Resource for ValidationState {}

#[derive(Debug)]
pub struct ValidationStateField;

#[async_trait]
impl StateField for ValidationStateField {
    async fn init(&self, _config: &StateConfig, _instance: &State) -> Arc<dyn Resource> {
        Arc::new(ValidationState {
            total_validations: 0,
            failed_validations: 0,
            last_validation: None,
        })
    }
    
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        _new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut state = value.downcast_ref::<ValidationState>()
            .expect("状态类型错误")
            .clone();
        
        if let Some(validation_results) = tr.get_meta::<Vec<ValidationResult>>("validation_results") {
            state.total_validations += validation_results.len() as u64;
            state.failed_validations += validation_results.iter()
                .filter(|r| !r.is_valid)
                .count() as u64;
            state.last_validation = Some(Utc::now());
        }
        
        Arc::new(state)
    }
}
```

### 4.5. 插件间数据依赖管理 (使用 Meta 流转)

当多个插件需要协作时，推荐使用事务的 meta 数据进行插件间的数据传递和依赖管理。

#### 插件依赖场景示例

```rust
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use mf_state::{plugin::StateField, resource::Resource, State, Transaction};

// 定义插件间传递的数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalculationResult {
    pub node_id: String,
    pub calculated_value: f64,
    pub formula_used: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationResult {
    pub node_id: String,
    pub is_valid: bool,
    pub error_messages: Vec<String>,
    pub validated_by: String,
}

// ================================================================================
// 插件 A: 价格计算插件
// ================================================================================

/// StateField: 专门为价格计算插件定义独立的状态存储
/// 目的: 管理缓存数据、统计信息等需要持久化的插件状态
#[derive(Debug)]
pub struct PriceCalculationStateField;

#[async_trait]
impl StateField for PriceCalculationStateField {
    /// 初始化插件的独立状态空间
    async fn init(&self, _config: &StateConfig, _instance: &State) -> Arc<dyn Resource> {
        Arc::new(PriceCalculationState {
            calculation_cache: HashMap::new(),
            total_calculations: 0,
            last_calculation_time: None,
        })
    }
    
    /// 处理事务，更新插件状态（纯状态管理，不产生新事务）
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut state = value.downcast_ref::<PriceCalculationState>()
            .expect("状态类型错误")
            .clone();
            
        // 只负责状态变更，更新统计信息和缓存
        if let Some(node_ids) = tr.get_meta::<Vec<String>>("nodes_to_calculate") {
            // 更新统计信息
            state.total_calculations += 1;
            state.last_calculation_time = Some(chrono::Utc::now());
            
            // 缓存计算结果（如果需要的话）
            for node_id in node_ids {
                if let Some(node) = new_state.doc().get_node(&node_id) {
                    let calculated_value = self.calculate_price(&node).await;
                    state.calculation_cache.insert(node_id.clone(), calculated_value);
                }
            }
        }
        
        Arc::new(state)
    }
}

impl PriceCalculationStateField {
    async fn calculate_price(&self, node: Arc<Node>) -> f64 {
        // 具体的价格计算逻辑
        let base_price = node.attrs.get("base_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let quantity = node.attrs.get("quantity")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let rate = node.attrs.get("rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        
        base_price * quantity * rate
    }
}

/// PluginTrait: 插件的业务逻辑层，负责决定何时产生新事务
/// 目的: 根据业务需求判断是否需要通知其他插件
#[derive(Debug)]
pub struct PriceCalculationPlugin;

#[async_trait]
impl PluginTrait for PriceCalculationPlugin {
    /// 业务逻辑：判断何时需要产生新事务
    /// ✅ 使用 Transaction Meta 方式 - 直接从事务中计算和传递数据
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 🔄 直接从 transactions 参数中查找触发条件
        for transaction in transactions {
            if let Some(node_ids) = transaction.get_meta::<Vec<String>>("nodes_to_calculate") {
                println!("插件 A 检测到计算请求: {:?}", node_ids);
                
                // 直接执行计算，不存储到状态中（除非需要缓存）
                let mut calculation_results = Vec::new();
                
                for node_id in node_ids {
                    if let Some(node) = new_state.doc().get_node(&node_id) {
                        let calculated_value = self.calculate_price(&node).await;
                        
                        let result = CalculationResult {
                            node_id: node_id.clone(),
                            calculated_value,
                            formula_used: "base_price * quantity * rate".to_string(),
                            timestamp: chrono::Utc::now(),
                        };
                        
                        calculation_results.push(result);
                    }
                }
                
                // 业务规则：只有当计算值在合理范围内才通知下游
                let valid_results: Vec<_> = calculation_results
                    .iter()
                    .filter(|result| result.calculated_value > 0.0 && result.calculated_value < 10000.0)
                    .cloned()
                    .collect();
                
                if !valid_results.is_empty() {
                    // ✅ 正确方式：使用 new_state 创建事务
                    let mut notification_transaction = new_state.tr();
                    
                    // 设置通知标识
                    notification_transaction.set_meta("price_calculation_completed", true);
                    
                    // 直接传递计算结果（推荐方式）
                    notification_transaction.set_meta("calculation_results", valid_results.clone());
                    
                    // 业务判断：如果计算量大，设置批量处理标识
                    if valid_results.len() > 10 {
                        notification_transaction.set_meta("batch_processing_required", true);
                    }
                    
                    println!("插件 A 完成计算，通过 meta 传递 {} 个结果", valid_results.len());
                    return Ok(Some(notification_transaction));
                }
            }
        }
        
        // 检查是否需要定期汇总（这种情况需要访问状态）
        if let Some(calc_state) = new_state.get_field("price_calculation")
            .and_then(|state| state.downcast_ref::<PriceCalculationState>()) 
        {
            if let Some(last_time) = calc_state.last_calculation_time {
                let duration = chrono::Utc::now().signed_duration_since(last_time);
                if duration.num_hours() >= 1 && calc_state.total_calculations >= 100 {
                    let mut summary_transaction = new_state.tr();
                    summary_transaction.set_meta("periodic_summary_required", true);
                    summary_transaction.set_meta("calculation_count", calc_state.total_calculations);
                    
                    println!("插件 A 触发定期汇总");
                    return Ok(Some(summary_transaction));
                }
            }
        }
        
        Ok(None)
    }
}

impl PriceCalculationPlugin {
    async fn calculate_price(&self, node: Arc<Node>) -> f64 {
        // 具体的价格计算逻辑
        let base_price = node.attrs.get("base_price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let quantity = node.attrs.get("quantity")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let rate = node.attrs.get("rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        
        base_price * quantity * rate
    }
}

/// 价格计算插件的独立状态结构
/// 专注于需要持久化的数据：缓存、统计信息等
#[derive(Debug, Clone)]
pub struct PriceCalculationState {
    pub calculation_cache: HashMap<String, f64>,
    pub total_calculations: u64,
    pub last_calculation_time: Option<chrono::DateTime<chrono::Utc>>,
}
impl Resource for PriceCalculationState {}

// ================================================================================
// 插件 B: 数据验证插件 (依赖于插件 A 的结果)
// ================================================================================

/// StateField: 专门为验证插件定义独立的状态存储
/// 只存储需要持久化的验证统计信息
#[derive(Debug)]
pub struct ValidationStateField;

#[async_trait]
impl StateField for ValidationStateField {
    async fn init(&self, _config: &StateConfig, _instance: &State) -> Arc<dyn Resource> {
        Arc::new(ValidationState { 
            total_validations: 0,
            failed_validations: 0,
            last_validation_time: None,
        })
    }
    
    /// 只负责更新验证统计信息，不处理业务逻辑
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut state = value.downcast_ref::<ValidationState>()
            .expect("状态类型错误")
            .clone();
            
        // 只更新统计信息
        if tr.has_meta("price_calculation_completed") {
            state.total_validations += 1;
            state.last_validation_time = Some(chrono::Utc::now());
            
            // 如果有验证失败，更新失败计数
            if let Some(validation_results) = tr.get_meta::<Vec<ValidationResult>>("validation_results") {
                let failures = validation_results.iter().filter(|r| !r.is_valid).count();
                state.failed_validations += failures as u64;
            }
        }
        
        Arc::new(state)
    }
}

/// PluginTrait: 验证插件的业务逻辑
/// ✅ 使用 Transaction Meta 方式 - 直接从事务中获取数据并验证
#[derive(Debug)]
pub struct ValidationPlugin;

#[async_trait]
impl PluginTrait for ValidationPlugin {
    /// 业务逻辑：从 transactions 参数中直接获取计算结果并进行验证
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 🔄 直接从 transactions 参数中查找计算完成的事务
        for transaction in transactions {
            if transaction.has_meta("price_calculation_completed") {
                println!("插件 B 检测到价格计算完成");
                
                // 直接从事务 meta 中获取计算结果
                if let Some(calculation_results) = transaction.get_meta::<Vec<CalculationResult>>("calculation_results") {
                    println!("插件 B 获取到 {} 个计算结果，开始验证", calculation_results.len());
                    
                    // 直接对计算结果进行验证，不需要存储到状态
                    let mut validation_results = Vec::new();
                    
                    for calc_result in calculation_results {
                        let validation_result = self.validate_calculation(&calc_result).await;
                        validation_results.push(validation_result);
                    }
                    
                    // ✅ 正确方式：使用 new_state 创建事务
                    let mut notification_transaction = new_state.tr();
                    
                    // 设置验证完成标识
                    notification_transaction.set_meta("validation_completed", true);
                    
                    // 直接传递验证结果（推荐方式）
                    notification_transaction.set_meta("validation_results", validation_results.clone());
                    
                    // 检查是否有验证失败
                    let has_failures = validation_results.iter().any(|v| !v.is_valid);
                    notification_transaction.set_meta("has_validation_failures", has_failures);
                    
                    if has_failures {
                        let failed_node_ids: Vec<String> = validation_results
                            .iter()
                            .filter(|r| !r.is_valid)
                            .map(|r| r.node_id.clone())
                            .collect();
                        notification_transaction.set_meta("failed_node_ids", failed_node_ids);
                    }
                    
                    println!("插件 B 完成验证，通过 meta 传递 {} 个验证结果", validation_results.len());
                    return Ok(Some(notification_transaction));
                }
            }
        }
        
        Ok(None)
    }
}

/// 验证插件的状态数据 - 只存储统计信息
#[derive(Debug, Clone)]
pub struct ValidationState {
    pub total_validations: u64,
    pub failed_validations: u64,
    pub last_validation_time: Option<chrono::DateTime<chrono::Utc>>,
}
impl Resource for ValidationState {}

impl ValidationPlugin {
    async fn validate_calculation(&self, calc_result: &CalculationResult) -> ValidationResult {
        let mut errors = Vec::new();
        
        // 验证计算结果的合理性
        if calc_result.calculated_value < 0.0 {
            errors.push("计算值不能为负数".to_string());
        }
        
        if calc_result.calculated_value > 1_000_000.0 {
            errors.push("计算值超出合理范围".to_string());
        }
        
        // 验证公式
        if calc_result.formula_used.is_empty() {
            errors.push("缺少计算公式".to_string());
        }
        
        ValidationResult {
            node_id: calc_result.node_id.clone(),
            is_valid: errors.is_empty(),
            error_messages: errors,
            validated_by: "ValidationPlugin".to_string(),
        }
    }
}

// ================================================================================
// 插件 C: 汇总插件 (依赖于插件 A 和 B 的结果)  
// ================================================================================

/// StateField: 汇总插件的状态存储
#[derive(Debug)]
pub struct SummaryStateField;

#[async_trait]
impl StateField for SummaryStateField {
    async fn init(&self, _config: &StateConfig, _instance: &State) -> Arc<dyn Resource> {
        Arc::new(SummaryState {
            total_summaries: 0,
            last_summary_time: None,
        })
    }
    
    /// 只负责更新汇总统计信息
    async fn apply(
        &self,
        tr: &Transaction,
        value: Arc<dyn Resource>,
        _old_state: &State,
        new_state: &State,
    ) -> Arc<dyn Resource> {
        let mut state = value.downcast_ref::<SummaryState>()
            .expect("状态类型错误")
            .clone();
            
        // 只更新统计信息
        if tr.has_meta("validation_completed") {
            state.total_summaries += 1;
            state.last_summary_time = Some(chrono::Utc::now());
        }
        
        Arc::new(state)
    }
}

/// PluginTrait: 汇总插件的业务逻辑
/// ✅ 使用 Transaction Meta 方式 - 直接从事务中获取所有必要数据
#[derive(Debug)]
pub struct SummaryPlugin;

#[async_trait]
impl PluginTrait for SummaryPlugin {
    /// 业务逻辑：从 transactions 参数中收集所有需要的数据并生成汇总
    async fn append_transaction(
        &self,
        transactions: &[Transaction],
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 🔄 直接从 transactions 参数中查找验证完成的事务
        let mut calculation_results = Vec::new();
        let mut validation_results = Vec::new();
        
        // 遍历所有事务，收集计算和验证结果
        for transaction in transactions {
            // 收集计算结果
            if transaction.has_meta("price_calculation_completed") {
                if let Some(calc_results) = transaction.get_meta::<Vec<CalculationResult>>("calculation_results") {
                    calculation_results.extend(calc_results);
                }
            }
            
            // 收集验证结果
            if transaction.has_meta("validation_completed") {
                println!("插件 C 检测到验证完成");
                
                if let Some(valid_results) = transaction.get_meta::<Vec<ValidationResult>>("validation_results") {
                    validation_results.extend(valid_results);
                    
                    // 如果有验证结果，生成汇总报告
                    if !validation_results.is_empty() {
                        println!("插件 C 开始生成汇总报告");
                        
                        // 直接生成汇总报告，不需要存储中间状态
                        let summary = self.create_summary_report(
                            &calculation_results, 
                            &validation_results
                        ).await;
                        
                        // ✅ 正确方式：使用 new_state 创建事务
                        let mut notification_transaction = new_state.tr();
                        
                        // 设置汇总完成标识
                        notification_transaction.set_meta("summary_completed", true);
                        
                        // 直接传递汇总报告（推荐方式）
                        notification_transaction.set_meta("summary_report", summary.clone());
                        notification_transaction.set_meta("workflow_completed", true);
                        
                        println!("插件 C 完成汇总，通过 meta 传递汇总报告");
                        return Ok(Some(notification_transaction));
                    }
                }
            }
        }
        
        Ok(None)
    }
}

/// 汇总插件的状态数据 - 只存储统计信息
#[derive(Debug, Clone)]
pub struct SummaryState {
    pub total_summaries: u64,
    pub last_summary_time: Option<chrono::DateTime<chrono::Utc>>,
}
impl Resource for SummaryState {}

impl SummaryPlugin {
    async fn create_summary_report(
        &self, 
        calculations: &[CalculationResult], 
        validations: &[ValidationResult]
    ) -> SummaryReport {
        SummaryReport {
            total_calculations: calculations.len(),
            valid_calculations: validations.iter().filter(|v| v.is_valid).count(),
            invalid_calculations: validations.iter().filter(|v| !v.is_valid).count(),
            total_value: calculations.iter().map(|c| c.calculated_value).sum(),
            generated_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummaryReport {
    pub total_calculations: usize,
    pub valid_calculations: usize,
    pub invalid_calculations: usize,
    pub total_value: f64,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}
impl Resource for SummaryReport {}
```

#### 插件注册和优先级配置

```rust
/// 配置插件的依赖关系和执行顺序 
/// ✅ 使用新的插件管理器 API
pub async fn init_dependent_extensions() -> Result<Vec<Extensions>> {
    let mut extensions = Vec::new();
    
    // 初始化插件管理器
    let plugin_manager = PluginManager::new();
    
    // 注册插件（插件管理器会自动处理依赖关系）
    let plugins = vec![
        create_price_calculator_plugin(),
        create_validation_plugin(), 
        create_report_plugin(),
    ];
    
    for plugin in plugins {
        plugin_manager.register_plugin(plugin).await?;
    }
    
    // 完成注册并验证依赖关系
    plugin_manager.finalize_registration().await?;
    
    // 获取排序后的插件
    let sorted_plugins = plugin_manager.get_sorted_plugins().await;
    
    // 创建扩展容器并添加插件
    let mut extension = Extension::new();
    for plugin in sorted_plugins {
        extension.add_plugin(plugin);
    }
    
    extensions.push(Extensions::E(extension));
    Ok(extensions)
}

/// 新插件系统的完整流程说明
/// 
/// 1. **插件注册阶段**：
///    - 调用 plugin_manager.register_plugin() 注册各个插件
///    - 系统收集插件元数据（名称、版本、依赖、冲突等）
///    - 构建插件依赖图
/// 
/// 2. **验证阶段**：
///    - 调用 plugin_manager.finalize_registration()
///    - 检查循环依赖（自动检测并报告）
///    - 检查缺失依赖（验证所有依赖是否满足）
///    - 检查插件冲突（防止冲突插件同时加载）
///    - 生成拓扑排序的执行顺序
/// 
/// 3. **执行阶段**：
///    - 按依赖关系排序后的顺序执行插件
///    - StateField.apply() 处理状态变更，更新插件私有数据
///    - PluginTrait.append_transaction() 产生新事务，触发下游插件
/// 
/// 4. **状态管理**：
///    - 每个插件拥有独立的状态空间
///    - 支持状态序列化/反序列化
///    - 插件间通过事务 Meta 进行轻量级通信
/// 
/// 关键优势：
/// - 🔍 智能依赖管理：自动检测循环依赖和缺失依赖
/// - ⚡ 冲突防护：自动验证并防止冲突插件加载
/// - 📋 元数据驱动：丰富的插件元信息支持
/// - 🔄 完整生命周期：从注册到执行的完整管理
/// - 💾 状态持久化：内置的序列化/反序列化支持
```

#### Transaction Meta vs State 存储的设计权衡

基于你的观察，确实存在一个重要的架构设计问题：**为什么需要同时使用 Transaction Meta 和 State 存储？**

```rust
// 关键问题：append_transaction 参数中已经有 transactions 数组
async fn append_transaction(
    &self,
    transactions: &[Transaction],  // 👈 可以直接访问所有事务和其 meta
    old_state: &State,
    new_state: &State,
) -> StateResult<Option<Transaction>>
```

**设计权衡分析**：

1. **Transaction Meta: 轻量级事务间通信**
   ```rust
   // ✅ 优势：直接访问，无需状态查找
   if let Some(results) = transaction.get_meta::<Vec<CalculationResult>>("calculation_results") {
       // 直接使用 meta 中的数据
   }
   
   // ❌ 限制：数据生命周期仅限于事务链执行期间
   ```

2. **State 存储: 持久化插件状态**
   ```rust
   // ✅ 优势：数据在整个应用生命周期内持久存在
   let calc_state = new_state.get_field("price_calculation")
       .and_then(|state| state.downcast_ref::<PriceCalculationState>());
   
   // ✅ 优势：支持复杂的状态管理，如缓存、统计、历史记录
   ```

**推荐的最佳实践**（基于实际框架行为）：

```rust
/// 优化的插件实现 - 充分利用 transactions 参数
#[async_trait]
impl PluginTrait for ValidationPlugin {
    async fn append_transaction(
        &self,
        transactions: &[Transaction],  // 👈 直接使用这个参数
        old_state: &State,
        new_state: &State,
    ) -> StateResult<Option<Transaction>> {
        // 🔄 策略1: 直接从 transactions 中获取数据（推荐）
        for transaction in transactions {
            if transaction.has_meta("price_calculation_completed") {
                // 直接从事务 meta 获取数据，无需状态查找
                if let Some(results) = transaction.get_meta::<Vec<CalculationResult>>("calculation_results") {
                    return self.process_calculation_results(results).await;
                }
            }
        }
        
        // 🔄 策略2: 从状态获取持久化数据（仅在必要时）
        // 只有当需要访问历史数据、缓存或跨事务状态时才使用
        if let Some(plugin_state) = new_state.get_field("validation") {
            // 访问持久化的插件状态
        }
        
        Ok(None)
    }
}

impl ValidationPlugin {
    async fn process_calculation_results(
        &self, 
        results: Vec<CalculationResult>
    ) -> StateResult<Option<Transaction>> {
        let mut notification_transaction = Transaction::new();
        
        // 验证逻辑...
        let validation_results = self.validate_results(&results).await;
        
        // 只传递必要的数据到下游
        notification_transaction.set_meta("validation_completed", true);
        notification_transaction.set_meta("validation_results", validation_results);
        
        Ok(Some(notification_transaction))
    }
}
```

**何时使用哪种方式**：

| 使用场景 | Transaction Meta | State 存储 |
|---------|------------------|------------|
| 插件间数据传递 | ✅ 首选 | ❌ 过度设计 |
| 临时计算结果 | ✅ 合适 | ❌ 不必要 |
| 缓存和性能优化 | ❌ 不合适 | ✅ 必要 |
| 历史记录和统计 | ❌ 不合适 | ✅ 必要 |
| 跨事务状态 | ❌ 不可能 | ✅ 必要 |
| 大型复杂对象 | ⚠️ 谨慎使用 | ✅ 推荐 |

#### 使用插件依赖的完整示例

```rust
async fn example_dependent_plugins_workflow() -> Result<()> {
    // 1. 创建编辑器配置
    let create_callback = Arc::new(SimpleNodePoolFn);
    let mut builder = EditorOptionsBuilder::new();
    builder = builder
        .content(Content::NodePoolFn(create_callback))
        .extensions(init_dependent_extensions()); // 使用依赖插件配置
    
    let options = builder.build();
    let mut editor = DemoEditor::create(DemoEditorOptions {
        editor_options: options
    }).await?;
    
    // 2. 创建需要计算的节点
    let mut transaction = Transaction::new();
    
    let calc_node = Node::new(
        "item_001".to_string(),
        NodeType::text("calculation_item"),
        Attrs::from([
            ("base_price".to_string(), 100.0.into()),
            ("quantity".to_string(), 5.0.into()),
            ("rate".to_string(), 1.2.into()),
        ]),
        None,
    );
    
    transaction.add_step(AddNodeStep::new_single(calc_node, None));
    
    // 3. 设置触发插件链的 meta 信息
    transaction.set_meta("nodes_to_calculate", vec!["item_001".to_string()]);
    transaction.set_meta("trigger_validation", true);
    transaction.set_meta("generate_summary", true);
    
    // 4. 执行事务，触发插件链
    editor.dispatch_flow_with_meta(
        transaction,
        "执行价格计算和验证".to_string(),
        serde_json::json!({
            "workflow": "price_calculation_validation",
            "initiator": "user"
        })
    ).await?;
    
    // 5. 获取最终结果
    let state = editor.get_state().await;
    let resource_manager = state.resource_manager();
    
    // 获取汇总报告
    if let Some(summary) = resource_manager.resource_table
        .get::<SummaryReport>("summary_report".to_string()) 
    {
        println!("汇总报告: {:?}", summary);
    }
    
    Ok(())
}
```

#### 最佳实践

基于真实框架的插件依赖管理要点：

1. **优先使用 Transaction Meta 进行插件间通信**：
   ```rust
   // ✅ 推荐：直接从 transactions 参数获取数据
   async fn append_transaction(
       &self,
       transactions: &[Transaction],
       old_state: &State,
       new_state: &State,
   ) -> StateResult<Option<Transaction>> {
       // 遍历所有事务，查找相关的 meta 数据
       for tx in transactions {
           if let Some(data) = tx.get_meta::<Vec<ResultData>>("calculation_results") {
               // ✅ 正确：使用 new_state 创建新事务
               let mut notification_tx = new_state.tr();
               notification_tx.set_meta("data_processed", true);
               notification_tx.set_meta("processed_data", processed_data);
               return Ok(Some(notification_tx));
           }
       }
       Ok(None)
   }
   
   // ❌ 错误：Transaction::new() 需要 State 参数
   // let mut tx = Transaction::new(); // 这样会编译错误
   
   // ✅ 正确：使用 State.tr() 方法创建事务
   // let mut tx = new_state.tr();
   ```

2. **双重实现模式的设计目的**：
   - `StateField`: 为每个插件定义单独的状态存储空间，管理插件私有数据
   - `PluginTrait`: 插件的业务逻辑层，根据业务需求产生新的事务
   
3. **职责分离原则**：
   ```rust
   // StateField: 纯粹的状态管理，不产生新事务
   impl StateField {
       async fn apply() -> Arc<dyn Resource> {
           // ✅ 只负责：更新插件状态、缓存数据、统计信息
           // ❌ 不负责：业务逻辑判断、产生新事务
           let updated_state = process_data(transaction_data);
           Arc::new(updated_state)
       }
   }
   
   // PluginTrait: 业务逻辑判断，决定是否产生新事务
   impl PluginTrait {
       async fn append_transaction(
           &self,
           transactions: &[Transaction],  // 👈 充分利用这个参数
           old_state: &State,
           new_state: &State,
       ) -> StateResult<Option<Transaction>> {
           // ✅ 只负责：业务逻辑判断、条件检查、产生新事务
           // ❌ 不负责：状态存储、数据缓存
           
           // 优先从 transactions 获取数据
           for tx in transactions {
               if tx.has_meta("trigger_condition") {
                   return Ok(Some(create_notification_transaction()));
               }
           }
           
           // 只有在需要持久化状态时才访问 State
           Ok(None)
       }
   }
   ```

3. **插件独立状态空间**：
   - 每个插件拥有完全独立的状态结构体
   - 状态包含：计算结果、缓存数据、统计信息、配置参数
   - 通过 `StateField` 管理，通过 `PluginTrait` 使用

4. **Transaction 创建的正确方式**：
   ```rust
   // ✅ 正确：必须使用 State 来创建 Transaction
   async fn append_transaction(
       &self,
       transactions: &[Transaction],
       old_state: &State,
       new_state: &State,
   ) -> StateResult<Option<Transaction>> {
       // 使用 new_state 创建新事务
       let mut new_transaction = new_state.tr();
       new_transaction.set_meta("plugin_completed", true);
       Ok(Some(new_transaction))
   }
   
   // ❌ 错误：这样会编译失败
   // let mut tx = Transaction::new(); // 缺少 State 参数
   
   // 📚 原因：Transaction::new(state: &State) 需要 State 参数
   // 这是因为 Transaction 需要访问文档状态和配置信息
   ```

5. **Meta 数据设计**：
   - 使用语义化的键名：`price_calculation_completed`、`validation_failures`
   - 传递轻量级标识，复杂数据存储在插件状态中
   - 避免在 meta 中存储大量数据

6. **优先级设置**：
   ```rust
   priority: 10,  // A 插件先执行
   priority: 20,  // B 插件后执行（依赖 A）
   priority: 30,  // C 插件最后执行（依赖 A、B）
   ```

7. **错误处理策略**：
   - `append_transaction` 返回 `StateResult<Option<Transaction>>`
   - 返回 `Ok(None)` 表示不需要额外事务
   - 返回 `Err()` 中断整个事务流程

8. **调试和监控**：
   ```rust
   println!("插件 {} 收到事务: {:?}", plugin_name, tr.meta);
   tracing::debug!("处理完成，提交通知事务");
   ```

9. **避免循环依赖**：
   - 明确定义插件间的依赖方向
   - 使用优先级确保单向数据流
   - 避免插件 A 依赖插件 B，同时插件 B 又依赖插件 A

10. **资源清理**：
    - 定期清理过期的计算结果
    - 在插件状态中管理数据生命周期
    - 避免在资源表中积累过多临时数据

11. **Transaction Meta 优先原则**：
    ```rust
    /// 插件间数据流转的最佳实践
    /// 
    /// ✅ 推荐方式：使用 Transaction Meta
    /// append_transaction(transactions: &[Transaction], ...) {
    ///     for tx in transactions {
    ///         if let Some(data) = tx.get_meta("calculation_results") {
    ///             // 直接处理数据，高效且简洁
    ///             let mut new_tx = new_state.tr(); // ✅ 正确创建方式
    ///             new_tx.set_meta("processed", true);
    ///             return Ok(Some(new_tx));
    ///         }
    ///     }
    /// }
    /// 
    /// ❌ 避免方式：不必要的状态存储
    /// // 只有在需要跨事务持久化时才使用 State 存储
    /// if let Some(plugin_state) = new_state.get_field("plugin_name") {
    ///     // 仅用于缓存、统计、历史记录等场景
    /// }
    ```

12. **文档化依赖关系**：
    ```rust
    /// 插件依赖图：
    /// 原始事务 → 插件A(计算) → 通知事务₁ → 插件B(验证) → 通知事务₂ → 插件C(汇总)
    /// 
    /// Meta 流转（推荐）：
    /// - "nodes_to_calculate" → 触发插件A
    /// - "price_calculation_completed" → 触发插件B  
    /// - "validation_completed" → 触发插件C
    /// 
    /// 数据访问方式：
    /// - 临时数据：transactions[].get_meta() （推荐）
    /// - 持久化数据：state.get_field() （仅在必要时）
    /// 
    /// Transaction 创建：
    /// - 必须使用：new_state.tr() （正确）
    /// - 避免使用：Transaction::new() （错误）
    ```

### 5. 编辑器创建和配置

```rust
use std::sync::Arc;
use mf_core::{
    extension::Extension,
    types::{Content, EditorOptionsBuilder, Extensions, NodePoolFnTrait},
    runtime::async_runtime::ForgeAsyncRuntime,
};
use mf_state::plugin::{Plugin, PluginSpec};
use anyhow::Result;

pub struct DemoEditorOptions {
    pub editor_options: RuntimeOptions,
}

pub struct DemoEditor {
    editor: ForgeAsyncRuntime,
    options: DemoEditorOptions,
}

impl DemoEditor {
    pub async fn create(options: DemoEditorOptions) -> Result<Self> {
        let editor = ForgeAsyncRuntime::create(options.editor_options.clone()).await?;
        Ok(Self { editor, options })
    }
}

/// 获取编辑器配置
pub async fn init_options(
    create_callback: Arc<dyn NodePoolFnTrait>
) -> DemoEditorOptions {
    let mut builder = EditorOptionsBuilder::new();
    builder = builder
        .content(Content::NodePoolFn(create_callback))
        // 设置历史记录限制
        .history_limit(20)
        // 添加扩展
        .extensions(init_extension())
        // 添加中间件
        .add_middleware(CollectFbfxCsxmMiddleware);
        
    let options = builder.build();
    DemoEditorOptions { editor_options: options }
}

/// 获取扩展配置
pub fn init_extension() -> Vec<Extensions> {
    let mut extensions = vec![
        // 添加标记扩展
        Extensions::M(BG_COLOR.clone()),
        Extensions::M(FOOTNOTE.clone()),
    ];
    
    // 添加节点扩展
    let nodes = init_nodes();
    for mut node in nodes {
        // 可以在这里设置节点的内容模式
        if node.get_name() == "djgc" {
            node.set_content("djgcRowNode+");
        }
        extensions.push(Extensions::N(node));
    }
    
    // 添加插件扩展
    let mut extension = Extension::new();
    let inc_plugin = Plugin::new(PluginSpec {
        key: ("inc_plugin".to_string(), "增量数据插件".to_string()),
        state_field: Some(Arc::new(IncStateField)),
        tr: None,
        priority: 10,
    });
    extension.add_plugin(Arc::new(inc_plugin));
    extensions.push(Extensions::E(extension));
    
    extensions
}

/// 创建编辑器
pub async fn init_editor(options: DemoEditorOptions) -> DemoEditor {
    DemoEditor::create(options).await.unwrap()
}
```

### 6. 状态管理和事务

```rust
use mf_state::{Transaction, State, transaction::Command};
use mf_transform::{AddNodeStep, AttrStep};
use mf_model::{Node, Attrs};
use std::sync::Arc;

// 实现命令模式进行状态更新
async fn update_node_example(editor: &mut DemoEditor, node_id: String, new_attrs: Attrs) -> Result<()> {
    // 创建事务
    let mut transaction = Transaction::new();
    transaction.add_step(AttrStep::new(node_id, new_attrs));
    transaction.set_meta("action", "update_node_attrs");
    
    // 应用事务
    editor.dispatch_flow_with_meta(
        transaction,
        "更新节点属性".to_string(),
        serde_json::json!({"type": "attr_update"})
    ).await?;
    
    Ok(())
}

// 添加节点
async fn add_node_example(editor: &mut DemoEditor, parent_id: Option<String>) -> Result<()> {
    // 创建新节点
    let node = DJGC_NODE.clone();
    
    // 创建事务
    let mut transaction = Transaction::new();
    transaction.add_step(AddNodeStep::new_single(node, parent_id));
    transaction.set_meta("action", "add_node");
    
    // 应用事务
    editor.dispatch_flow_with_meta(
        transaction,
        "添加节点".to_string(),
        serde_json::json!({"type": "node_add"})
    ).await?;
    
    Ok(())
}
```

### 7. Tauri 集成 (桌面应用开发)

#### 前端 JavaScript/TypeScript

```typescript
// src/api/editor.ts
import { invoke, listen } from '@tauri-apps/api/tauri';

export interface NodeData {
  id: string;
  type: string;
  attrs: Record<string, any>;
  content?: string;
}

export class EditorAPI {
  // 调用后端命令
  static async createNode(parentId: string | null, nodeData: NodeData): Promise<void> {
    await invoke('create_node', { parentId, nodeData });
  }

  static async updateNodeAttrs(nodeId: string, attrs: Record<string, any>): Promise<void> {
    await invoke('update_node_attrs', { nodeId, attrs });
  }

  static async deleteNode(nodeId: string): Promise<void> {
    await invoke('delete_node', { nodeId });
  }

  static async getDocumentState(): Promise<any> {
    return await invoke('get_document_state');
  }

  // 监听后端事件
  static async listenToStateChanges(callback: (state: any) => void): Promise<void> {
    await listen('state-changed', (event) => {
      callback(event.payload);
    });
  }

  static async listenToErrors(callback: (error: string) => void): Promise<void> {
    await listen('editor-error', (event) => {
      callback(event.payload as string);
    });
  }
}

// 使用示例
async function setupEditor() {
  // 监听状态变化
  await EditorAPI.listenToStateChanges((newState) => {
    console.log('状态已更新:', newState);
    // 更新 UI
  });

  // 监听错误
  await EditorAPI.listenToErrors((error) => {
    console.error('编辑器错误:', error);
  });

  // 创建节点
  await EditorAPI.createNode(null, {
    id: 'root',
    type: 'document',
    attrs: { title: '新文档' }
  });
}
```

#### 后端 Rust Commands

```rust
// src/commands/editor.rs
use tauri::{command, State, Window};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use crate::editor::{DemoEditor, EditorAPI};

#[derive(Debug, Deserialize)]
pub struct NodeData {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    attrs: serde_json::Value,
    content: Option<String>,
}

// 全局编辑器状态
pub type EditorState = Mutex<Option<DemoEditor>>;

#[command]
pub async fn create_node(
    parent_id: Option<String>,
    node_data: NodeData,
    editor_state: State<'_, EditorState>,
    window: Window,
) -> Result<(), String> {
    let mut editor_guard = editor_state.lock().map_err(|e| e.to_string())?;
    let editor = editor_guard.as_mut().ok_or("编辑器未初始化")?;

    // 创建节点
    match EditorAPI::create_node(editor, parent_id, node_data).await {
        Ok(_) => {
            // 发送状态更新事件到前端
            let new_state = editor.get_state().await;
            window.emit("state-changed", &*new_state).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            // 发送错误事件到前端
            window.emit("editor-error", e.to_string()).map_err(|e| e.to_string())?;
            Err(e.to_string())
        }
    }
}

#[command]
pub async fn update_node_attrs(
    node_id: String,
    attrs: serde_json::Value,
    editor_state: State<'_, EditorState>,
    window: Window,
) -> Result<(), String> {
    let mut editor_guard = editor_state.lock().map_err(|e| e.to_string())?;
    let editor = editor_guard.as_mut().ok_or("编辑器未初始化")?;

    match EditorAPI::update_node_attrs(editor, node_id, attrs).await {
        Ok(_) => {
            let new_state = editor.get_state().await;
            window.emit("state-changed", &*new_state).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            window.emit("editor-error", e.to_string()).map_err(|e| e.to_string())?;
            Err(e.to_string())
        }
    }
}

#[command]
pub async fn delete_node(
    node_id: String,
    editor_state: State<'_, EditorState>,
    window: Window,
) -> Result<(), String> {
    let mut editor_guard = editor_state.lock().map_err(|e| e.to_string())?;
    let editor = editor_guard.as_mut().ok_or("编辑器未初始化")?;

    match EditorAPI::delete_node(editor, node_id).await {
        Ok(_) => {
            let new_state = editor.get_state().await;
            window.emit("state-changed", &*new_state).map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(e) => {
            window.emit("editor-error", e.to_string()).map_err(|e| e.to_string())?;
            Err(e.to_string())
        }
    }
}

#[command]
pub async fn get_document_state(
    editor_state: State<'_, EditorState>,
) -> Result<serde_json::Value, String> {
    let editor_guard = editor_state.lock().map_err(|e| e.to_string())?;
    let editor = editor_guard.as_ref().ok_or("编辑器未初始化")?;

    let state = editor.get_state().await;
    // 序列化状态为 JSON
    serde_json::to_value(&*state).map_err(|e| e.to_string())
}

#[command]
pub async fn initialize_editor(
    editor_state: State<'_, EditorState>,
    window: Window,
) -> Result<(), String> {
    // 创建编辑器实例
    let create_callback = Arc::new(SimpleNodePoolFn);
    let options = init_options(create_callback).await;
    let editor = init_editor(options).await;

    // 存储到全局状态
    let mut editor_guard = editor_state.lock().map_err(|e| e.to_string())?;
    *editor_guard = Some(editor);

    // 发送初始化完成事件
    window.emit("editor-initialized", ()).map_err(|e| e.to_string())?;
    
    Ok(())
}
```

#### main.rs 配置

```rust
// src/main.rs
use tauri::Manager;
use std::sync::Mutex;

mod commands;
mod editor;
mod nodes;
mod marks;
mod middleware;
mod plugins;

use commands::editor::*;

fn main() {
    tauri::Builder::default()
        .manage(EditorState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            initialize_editor,
            create_node,
            update_node_attrs,
            delete_node,
            get_document_state,
        ])
        .setup(|app| {
            // 应用启动时的初始化逻辑
            let window = app.get_window("main").unwrap();
            
            // 可以在这里进行一些初始化工作
            tracing_subscriber::fmt::init();
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
```

#### Tauri 配置 (tauri.conf.json)

```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:3000",
    "distDir": "../dist"
  },
  "package": {
    "productName": "ModuForge App",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "window": {
        "all": false,
        "close": true,
        "hide": true,
        "show": true,
        "maximize": true,
        "minimize": true,
        "unmaximize": true,
        "unminimize": true,
        "startDragging": true
      }
    },
    "bundle": {
      "active": true,
      "category": "DeveloperTool",
      "copyright": "",
      "deb": {
        "depends": []
      },
      "externalBin": [],
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "identifier": "com.moduforge.app",
      "longDescription": "",
      "macOS": {
        "entitlements": null,
        "exceptionDomain": "",
        "frameworks": [],
        "providerShortName": null,
        "signingIdentity": null
      },
      "resources": [],
      "shortDescription": "",
      "targets": "all",
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    },
    "security": {
      "csp": null
    },
    "updater": {
      "active": false
    },
    "windows": [
      {
        "fullscreen": false,
        "height": 600,
        "resizable": true,
        "title": "ModuForge App",
        "width": 800
      }
    ]
  }
}
```

### 8. 规则引擎使用

```rust
use mf_engine::{Engine, loader::MemoryLoader};
use mf_expression::Variable;
use serde_json::json;

async fn rules_example() -> Result<()> {
    // 创建规则引擎
    let loader = MemoryLoader::default();
    let engine = Engine::new(loader);
    
    // 准备输入数据
    let input = Variable::from(json!({
        "user": {
            "age": 25,
            "membership": "premium"
        }
    }));
    
    // 执行规则
    let result = engine.evaluate("user_rules", &input).await?;
    println!("规则执行结果: {:?}", result);
    
    Ok(())
}
```

### 9. 表达式语言

```rust
use mf_expression::{Expression, Variable};
use serde_json::json;

fn expression_example() -> Result<()> {
    // 编译表达式
    let expr = Expression::compile("user.age >= 18 && user.status == 'active'")?;
    
    // 准备数据
    let data = Variable::from(json!({
        "user": {
            "age": 25,
            "status": "active"
        }
    }));
    
    // 执行表达式
    let result = expr.execute(&data)?;
    println!("表达式结果: {:?}", result.to_bool());
    
    Ok(())
}
```

### 10. 协作功能

```rust
use mf_collaboration::{SyncService, types::RoomConfig};
use tokio_tungstenite::tungstenite::protocol::Message;

async fn collaboration_example() -> Result<()> {
    // 创建协作服务
    let mut sync_service = SyncService::new();
    
    // 创建房间
    let room_config = RoomConfig {
        room_id: "doc_123".to_string(),
        max_clients: 10,
    };
    
    sync_service.create_room(room_config).await?;
    
    // 处理客户端消息
    // let message = Message::text("sync_data");
    // sync_service.handle_message("doc_123", "client_1", message).await?;
    
    Ok(())
}
```

### 11. 文件操作

```rust
use mf_file::{ZipDocWriter, formats::JsonFormat};
use std::path::Path;

async fn file_export_example(state: &State) -> Result<()> {
    // 创建 ZIP 文档写入器
    let mut writer = ZipDocWriter::new();
    
    // 设置格式
    writer.set_format(Box::new(JsonFormat::new()));
    
    // 导出文档
    let output_path = Path::new("output.zip");
    writer.export_document(state, output_path).await?;
    
    println!("文档已导出到: {:?}", output_path);
    Ok(())
}
```

### 12. 搜索功能

```rust
use mf_search::{SearchService, model::{IndexRequest, SearchRequest}};

async fn search_example() -> Result<()> {
    // 创建搜索服务
    let mut search_service = SearchService::new();
    
    // 创建索引
    let index_req = IndexRequest {
        document_id: "doc_1".to_string(),
        content: "这是要索引的文档内容".to_string(),
        metadata: Default::default(),
    };
    
    search_service.index_document(index_req).await?;
    
    // 执行搜索
    let search_req = SearchRequest {
        query: "文档内容".to_string(),
        limit: 10,
        offset: 0,
    };
    
    let results = search_service.search(search_req).await?;
    println!("搜索结果: {:?}", results);
    
    Ok(())
}
```

## 开发环境配置

### 系统要求
- **Rust**: 1.70+ (建议使用最新稳定版)
- **Node.js**: 16+ (用于 Tauri 前端开发)
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 20.04+)

### 开发工具推荐
```bash
# Rust 工具链
rustup update stable
rustup component add clippy rustfmt rust-analyzer

# 开发工具
cargo install cargo-edit    # 依赖管理
cargo install cargo-watch   # 自动重新编译
cargo install cargo-audit   # 安全审计
cargo install tauri-cli     # Tauri CLI (如果使用桌面应用)

# IDE 插件推荐
# - VS Code: rust-analyzer, Tauri, Error Lens
# - IntelliJ: Rust Plugin
# - Vim/Neovim: rust.vim, coc-rust-analyzer
```

### 环境变量配置
```bash
# .env 文件示例
RUST_LOG=debug                           # 日志级别
MODUFORGE_DATA_DIR=./data                # 数据目录
MODUFORGE_CACHE_SIZE=1000               # 缓存大小
MODUFORGE_COLLABORATION_URL=ws://localhost:8080  # 协作服务器
DATABASE_URL=sqlite:./app.db            # 数据库连接
```

## 常用开发命令

### 构建和测试
```bash
# 基础构建
cargo build
cargo build --release

# 功能特定构建
cargo build --features collaboration
cargo build --all-features

# 测试
cargo test                              # 所有测试
cargo test --lib                        # 单元测试
cargo test --test integration           # 集成测试
cargo test -p mf-core                   # 特定包测试
cargo test collaboration -- --nocapture # 带输出的测试

# 代码质量
cargo fmt                               # 格式化
cargo clippy                            # 静态分析
cargo clippy --fix                      # 自动修复
cargo audit                             # 安全审计

# 文档
cargo doc --open                        # 生成并打开文档
cargo doc --document-private-items      # 包含私有项的文档
```

### Tauri 开发命令
```bash
# 开发模式
cargo tauri dev                         # 启动开发服务器
cargo tauri dev -- --features debug    # 带调试功能

# 构建
cargo tauri build                       # 构建生产版本
cargo tauri build --debug              # 构建调试版本

# 图标生成
cargo tauri icon path/to/icon.png      # 生成应用图标
```

### 示例和演示
```bash
# 核心功能示例
cargo run --example basic_usage
cargo run --example node_operations
cargo run --example state_management

# 集成示例
cargo run --example collaboration_demo
cargo run --example rules_engine_demo
cargo run --example file_operations
cargo run --example tauri_integration

# 性能测试
cargo bench                             # 运行基准测试
cargo bench --features collaboration    # 协作功能基准测试
```

## 错误处理和调试

### 常见错误及解决方案

#### 1. 编译错误
```bash
# 依赖版本冲突
error: failed to select a version for `serde`
# 解决方案：
cargo update
cargo clean && cargo build

# 特征冲突
error: the trait bound is not satisfied
# 解决方案：检查 Cargo.toml 中的 features 配置
```

#### 2. 运行时错误
```rust
// 状态初始化失败
Error: "编辑器未初始化"
// 解决方案：确保在使用前调用初始化
let editor = init_editor(options).await?;

// 事务失败
Error: "Transaction validation failed"
// 解决方案：检查事务步骤的有效性和顺序
```

#### 3. Tauri 集成错误
```typescript
// IPC 调用失败
Error: "Command not found"
// 解决方案：检查命令注册和参数类型匹配

// 事件监听失败
Error: "Event listener not working"
// 解决方案：确保事件名称匹配和监听器正确注册
```

### 调试技巧

#### 1. 日志配置
```rust
// 详细日志配置
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "moduforge=debug,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// 在代码中使用
use tracing::{debug, info, warn, error, instrument};

#[instrument]
async fn process_transaction(transaction: &Transaction) -> Result<()> {
    debug!("开始处理事务: {}", transaction.id);
    // 处理逻辑
    info!("事务处理完成");
    Ok(())
}
```

#### 2. 开发模式调试
```rust
#[cfg(debug_assertions)]
fn debug_state(state: &State) {
    println!("状态调试信息:");
    println!("- 节点数量: {}", state.node_count());
    println!("- 内存使用: {:?}", state.memory_usage());
}
```

#### 3. 性能监控
```rust
use std::time::Instant;

async fn measured_operation() -> Result<()> {
    let start = Instant::now();
    
    // 执行操作
    perform_operation().await?;
    
    let duration = start.elapsed();
    if duration.as_millis() > 100 {
        warn!("操作耗时过长: {:?}", duration);
    }
    
    Ok(())
}
```

## 测试策略

### 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_node_creation() {
        let mut editor = create_test_editor().await;
        let node_id = "test_node";
        
        let result = create_node(&mut editor, node_id, test_node_data()).await;
        assert!(result.is_ok());
        
        let state = editor.get_state().await;
        assert!(state.doc().get_node(node_id).is_some());
    }

    #[test]
    fn test_expression_evaluation() {
        let expr = Expression::compile("user.age >= 18").unwrap();
        let data = test_user_data();
        let result = expr.execute(&data).unwrap();
        assert_eq!(result.to_bool(), true);
    }
}
```

### 集成测试
```rust
// tests/integration_test.rs
use moduforge::*;

#[tokio::test]
async fn test_complete_workflow() {
    // 1. 初始化编辑器
    let mut editor = setup_test_editor().await;
    
    // 2. 创建文档结构
    create_test_document(&mut editor).await;
    
    // 3. 执行业务操作
    let result = perform_complex_operation(&mut editor).await;
    
    // 4. 验证结果
    assert!(result.is_ok());
    validate_final_state(&editor).await;
}
```

### 性能测试
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_node_creation(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let editor = rt.block_on(create_test_editor());
    
    c.bench_function("create 1000 nodes", |b| {
        b.to_async(&rt).iter(|| async {
            for i in 0..1000 {
                create_test_node(&editor, &format!("node_{}", i)).await;
            }
        });
    });
}

criterion_group!(benches, bench_node_creation);
criterion_main!(benches);
```

## 生产环境部署

### 构建优化
```toml
# Cargo.toml 生产配置
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

# 针对特定架构优化
[profile.release-with-debug]
inherits = "release"
debug = true
```

### Docker 部署 (后端服务)
```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建生产版本
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/your-app /usr/local/bin/your-app

# 创建非 root 用户
RUN useradd -r -s /bin/false appuser
USER appuser

EXPOSE 8080
CMD ["your-app"]
```

### 系统服务配置 (Linux)
```ini
# /etc/systemd/system/moduforge-app.service
[Unit]
Description=ModuForge Application
After=network.target

[Service]
Type=simple
User=moduforge
Group=moduforge
WorkingDirectory=/opt/moduforge
ExecStart=/opt/moduforge/bin/app
Restart=always
RestartSec=10

# 环境变量
Environment=RUST_LOG=info
Environment=DATABASE_URL=sqlite:/opt/moduforge/data/app.db
Environment=MODUFORGE_DATA_DIR=/opt/moduforge/data

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/opt/moduforge/data

[Install]
WantedBy=multi-user.target
```

### 监控和指标

#### 1. 应用指标收集
```rust
use metrics::{counter, histogram, gauge};

pub struct AppMetrics;

impl AppMetrics {
    pub fn record_transaction(&self, duration: Duration) {
        histogram!("transaction_duration_seconds", duration);
        counter!("transactions_total", 1);
    }
    
    pub fn record_active_users(&self, count: u64) {
        gauge!("active_users", count as f64);
    }
    
    pub fn record_error(&self, error_type: &str) {
        counter!("errors_total", 1, "type" => error_type.to_string());
    }
}

// 集成到应用中
#[instrument]
async fn process_transaction(tx: Transaction) -> Result<()> {
    let start = Instant::now();
    
    match execute_transaction(tx).await {
        Ok(result) => {
            APP_METRICS.record_transaction(start.elapsed());
            Ok(result)
        }
        Err(e) => {
            APP_METRICS.record_error("transaction_failed");
            Err(e)
        }
    }
}
```

#### 2. 健康检查端点
```rust
use axum::{response::Json, http::StatusCode};
use serde_json::json;

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    // 检查数据库连接
    if !database_healthy().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    
    // 检查关键服务
    if !core_services_healthy().await {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}
```

## 性能优化指南

### 1. 内存优化
```rust
// 使用对象池减少分配
use object_pool::Pool;

lazy_static! {
    static ref NODE_POOL: Pool<Node> = Pool::new(100, || Node::default());
}

fn create_optimized_node() -> PooledNode {
    let mut node = NODE_POOL.try_pull().unwrap_or_else(|| Node::default());
    // 重置节点状态
    node.reset();
    node
}

// 批量操作优化
async fn batch_create_nodes(nodes: Vec<NodeData>) -> Result<()> {
    let mut transaction = Transaction::new();
    
    // 批量添加步骤，减少事务开销
    for node_data in nodes {
        transaction.add_step(AddNodeStep::new_single(
            node_data.into(), 
            node_data.parent_id
        ));
    }
    
    // 一次性提交
    editor.dispatch_flow(transaction).await
}
```

### 2. 并发优化
```rust
use rayon::prelude::*;
use dashmap::DashMap;

// 并行节点处理
fn process_nodes_parallel(nodes: Vec<Node>) -> Vec<ProcessedNode> {
    nodes
        .into_par_iter()
        .map(|node| process_single_node(node))
        .collect()
}

// 使用无锁数据结构
lazy_static! {
    static ref CACHE: DashMap<String, CachedValue> = DashMap::new();
}

fn get_cached_or_compute(key: &str) -> CachedValue {
    CACHE.entry(key.to_string())
        .or_insert_with(|| expensive_computation(key))
        .clone()
}
```

### 3. I/O 优化
```rust
// 异步批量文件操作
async fn batch_save_documents(docs: Vec<Document>) -> Result<()> {
    let futures: Vec<_> = docs
        .into_iter()
        .map(|doc| save_document_async(doc))
        .collect();
    
    // 并发执行，但限制并发数
    use futures::stream::{iter, StreamExt};
    iter(futures)
        .buffer_unordered(10) // 最多10个并发
        .try_collect()
        .await
}

// 连接池配置
#[derive(Clone)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        }
    }
}
```

## 故障排除指南

### 常见问题诊断

#### 1. 内存泄漏
```bash
# 使用 valgrind 检测内存问题
valgrind --tool=memcheck --leak-check=full cargo run

# 使用 heaptrack 进行堆分析
heaptrack cargo run
heaptrack_gui heaptrack.your-app.PID.gz
```

#### 2. 性能问题
```bash
# 使用 perf 进行性能分析
perf record --call-graph=dwarf cargo run
perf report

# 使用 flamegraph 生成火焰图
cargo install flamegraph
cargo flamegraph --bin your-app
```

#### 3. 日志分析
```bash
# 结构化日志查询
grep "ERROR" app.log | jq '.timestamp, .message'

# 性能相关日志
grep "duration" app.log | jq '.duration' | sort -n | tail -10
```

### 配置调优

#### 1. 运行时参数
```bash
# JVM 风格的运行时调优（针对大型应用）
export RUST_MIN_STACK=8388608  # 8MB 栈大小
export RUST_BACKTRACE=1        # 启用回溯
export RAYON_NUM_THREADS=8     # 限制并行线程数
```

#### 2. 系统级优化
```bash
# Linux 系统调优
echo 'vm.max_map_count=262144' >> /etc/sysctl.conf
echo 'fs.file-max=2097152' >> /etc/sysctl.conf
sysctl -p

# 增加文件描述符限制
echo '* soft nofile 65536' >> /etc/security/limits.conf
echo '* hard nofile 65536' >> /etc/security/limits.conf
```

## 版本管理和升级

### 版本兼容性

#### 支持的版本范围
```toml
# 推荐的版本配置
[dependencies]
# 使用语义化版本控制
moduforge-core = "^0.4.12"    # 兼容 0.4.x 系列
moduforge-model = "~0.4.12"   # 只允许补丁级别更新

# 检查版本兼容性
[dependencies.moduforge-engine]
version = "0.4.12"
default-features = false
features = ["expression", "decision"]
```

#### 升级检查清单
```bash
# 1. 检查当前版本
cargo tree | grep moduforge

# 2. 检查可用更新
cargo outdated

# 3. 安全审计
cargo audit

# 4. 测试兼容性
cargo test --all-features

# 5. 检查破坏性变更
cargo doc --open
```

### 数据迁移

#### 1. 状态数据迁移
```rust
use semver::Version;

pub struct DataMigrator {
    from_version: Version,
    to_version: Version,
}

impl DataMigrator {
    pub async fn migrate(&self, data_path: &Path) -> Result<()> {
        match (self.from_version.major, self.to_version.major) {
            (0, 0) => self.migrate_patch().await,
            (0, 1) => self.migrate_major().await,
            _ => Err(anyhow!("不支持的版本迁移")),
        }
    }

    async fn migrate_patch(&self) -> Result<()> {
        // 补丁版本迁移，通常只需要数据格式调整
        info!("执行补丁级别迁移");
        Ok(())
    }

    async fn migrate_major(&self) -> Result<()> {
        // 主版本迁移，可能需要重新构建数据
        info!("执行主版本迁移");
        
        // 1. 备份原数据
        self.backup_data().await?;
        
        // 2. 转换数据格式
        self.convert_data_format().await?;
        
        // 3. 验证迁移结果
        self.validate_migration().await?;
        
        Ok(())
    }
}
```

#### 2. 配置文件迁移
```rust
#[derive(Serialize, Deserialize)]
struct ConfigV1 {
    database_url: String,
    cache_size: usize,
}

#[derive(Serialize, Deserialize)]
struct ConfigV2 {
    database: DatabaseConfig,
    cache: CacheConfig,
    // 新增功能配置
    collaboration: Option<CollaborationConfig>,
}

fn migrate_config(old_config: ConfigV1) -> ConfigV2 {
    ConfigV2 {
        database: DatabaseConfig {
            url: old_config.database_url,
            pool_size: 10, // 默认值
        },
        cache: CacheConfig {
            size: old_config.cache_size,
            ttl: Duration::from_secs(300), // 新增默认值
        },
        collaboration: None, // 新功能，默认关闭
    }
}
```

### 最佳实践总结

#### 1. 项目组织
- 使用工作空间组织多个相关 crate
- 遵循 Rust 的命名约定和目录结构
- 编写全面的文档和示例

#### 2. 依赖管理
- 优先使用语义化版本
- 定期更新依赖并进行安全审计
- 使用特性门控减少编译时间

#### 3. 错误处理
- 使用 `thiserror` 定义库错误类型
- 使用 `anyhow` 处理应用级错误
- 提供有意义的错误消息和上下文

#### 4. 测试策略
- 编写单元测试覆盖所有公共 API
- 使用集成测试验证端到端功能
- 进行性能基准测试

#### 5. 性能考虑
- 使用不可变数据结构和结构共享
- 合理使用异步编程和并发
- 监控内存使用和 GC 压力

#### 6. 插件依赖管理
- **使用 Meta 进行插件间通信**：通过事务 meta 传递轻量级状态标识
- **Resource Table 存储复杂数据**：将计算结果等复杂对象存储在资源表中
- **合理设置插件优先级**：确保依赖插件按正确顺序执行
- **明确数据生命周期**：及时清理临时数据，避免内存泄漏
- **设计回退机制**：当依赖数据不可用时的处理策略
- **文档化依赖关系**：清楚记录插件间的依赖关系和数据流

## 架构特性

### 不可变数据结构
- 所有状态变更都是不可变转换
- 高效的结构共享减少内存开销
- 通过 `Arc` 包装实现线程安全的并发访问

### 事件驱动架构
- 所有状态变更通过事件系统发出
- 类型安全的事件处理，支持异步
- 事件溯源功能用于状态重建

### 插件系统
- 动态插件加载和生命周期管理
- 插件隔离和依赖注入
- 框架中的扩展点

### 事务模型
- 符合 ACID 的状态变更事务
- 回滚功能和事务日志
- 原子批量操作

## 错误处理模式

```rust
use anyhow::{Result, Context};
use thiserror::Error;

// 自定义错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("文档未找到: {id}")]
    DocumentNotFound { id: String },
    
    #[error("规则执行失败: {rule_name}")]
    RuleExecutionFailed { rule_name: String },
}

// 使用 anyhow 进行错误处理
fn process_document(doc_id: &str) -> Result<()> {
    let doc = load_document(doc_id)
        .context(format!("加载文档失败: {}", doc_id))?;
    
    let result = process_rules(&doc)
        .context("规则处理失败")?;
    
    save_result(result)
        .context("保存结果失败")?;
    
    Ok(())
}
```

## 性能优化建议

### 内存管理
- 使用结构共享的不可变数据结构
- Arc 引用计数用于共享所有权
- LRU 缓存用于频繁访问的数据

### 异步处理
- 基于 Tokio 的异步运行时
- 后台任务处理
- 外部资源的连接池

### 批量操作
- 使用事务进行批量状态更新
- 批量索引更新
- 批量文件操作

## 集成注意事项

### 实时协作
框架通过映射层在文档状态和协作状态之间提供无缝集成。

### 规则引擎集成
业务逻辑可以通过规则引擎外部化，支持动态规则评估而无需代码更改。

### 文件格式支持
文档可以序列化为多种格式（JSON、CBOR、MessagePack），支持压缩。

### 搜索集成
全文搜索功能，包括索引、查询解析和结果排名。

这个框架特别适合需要以下功能的应用：
- 具有协作功能的复杂文档编辑
- 动态业务规则评估
- 高性能数据转换
- 可扩展插件架构
- 实时协作功能

## 完整项目结构示例

### 标准 Rust 项目结构
```
your-project/
├── Cargo.toml
├── CLAUDE.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── nodes/
│   │   ├── mod.rs
│   │   └── your_nodes.rs
│   ├── marks/
│   │   ├── mod.rs
│   │   └── your_marks.rs
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── your_middleware.rs
│   ├── plugins/
│   │   ├── mod.rs
│   │   └── your_plugin.rs
│   ├── editor/
│   │   ├── mod.rs
│   │   └── editor_config.rs
│   └── types.rs
└── examples/
    └── basic_usage.rs
```

### Tauri 桌面应用项目结构
```
your-tauri-app/
├── Cargo.toml
├── CLAUDE.md
├── tauri.conf.json
├── build.rs
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── commands/          # Tauri 命令
│   │   ├── mod.rs
│   │   └── editor.rs
│   ├── nodes/
│   │   ├── mod.rs
│   │   └── your_nodes.rs
│   ├── marks/
│   │   ├── mod.rs
│   │   └── your_marks.rs
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── your_middleware.rs
│   ├── plugins/
│   │   ├── mod.rs
│   │   └── your_plugin.rs
│   ├── editor/
│   │   ├── mod.rs
│   │   └── editor_config.rs
│   └── types.rs
├── src-ui/               # 前端代码
│   ├── package.json
│   ├── index.html
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── api/
│   │   │   └── editor.ts
│   │   ├── components/
│   │   │   ├── Editor.vue
│   │   │   └── NodeTree.vue
│   │   └── stores/
│   │       └── editor.ts
│   └── dist/             # 构建输出
└── icons/                # 应用图标
    ├── 32x32.png
    ├── 128x128.png
    ├── icon.icns
    └── icon.ico
```

### lib.rs
```rust
pub mod nodes;
pub mod marks;
pub mod middleware;
pub mod plugins;
pub mod editor;
pub mod types;

// 重新导出常用类型
pub use mf_core::*;
pub use mf_model::*;
pub use mf_state::*;
pub use mf_transform::*;
pub use editor::{MyEditor, MyEditorOptions, init_editor, init_options};
```

### main.rs
```rust
use anyhow::Result;
use your_project::{init_editor, init_options};
use std::sync::Arc;
use mf_core::types::NodePoolFnTrait;

// 定义一个简单的节点池创建函数
struct SimpleNodePoolFn;

impl NodePoolFnTrait for SimpleNodePoolFn {
    fn call(&self) -> mf_model::node_pool::NodePool {
        mf_model::node_pool::NodePool::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建节点池回调
    let create_callback: Arc<dyn NodePoolFnTrait> = Arc::new(SimpleNodePoolFn);
    
    // 获取编辑器配置
    let options = init_options(create_callback).await;
    
    // 创建编辑器
    let mut editor = init_editor(options).await;
    
    // 获取当前状态
    let state = editor.get_state().await;
    println!("编辑器创建成功，文档 ID: {}", state.id());
    
    // 在这里添加你的应用逻辑
    
    Ok(())
}
```

## 快速启动模板

```rust
use mf_core::{
    runtime::async_runtime::ForgeAsyncRuntime,
    types::{RuntimeOptions, EditorOptionsBuilder, Content, NodePoolFnTrait}
};
use mf_model::node_pool::NodePool;
use anyhow::Result;
use std::sync::Arc;

// 简单的节点池创建函数
struct DefaultNodePoolFn;

impl NodePoolFnTrait for DefaultNodePoolFn {
    fn call(&self) -> NodePool {
        NodePool::default()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建编辑器配置
    let create_callback: Arc<dyn NodePoolFnTrait> = Arc::new(DefaultNodePoolFn);
    let mut builder = EditorOptionsBuilder::new();
    builder = builder
        .content(Content::NodePoolFn(create_callback))
        .history_limit(20);
    
    let options = builder.build();
    
    // 创建运行时
    let runtime = ForgeAsyncRuntime::create(options).await?;
    
    // 获取当前状态
    let state = runtime.get_state();
    println!("ModuForge-RS 应用启动成功！文档 ID: {}", state.id());
    
    // 你的应用逻辑
    
    Ok(())
}
```
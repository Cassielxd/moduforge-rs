# Mermaid 图表测试页面

这是一个用于测试 Mermaid 图表在 VitePress 中显示效果的页面。

## 流程图测试

### 基本流程图
```mermaid
flowchart TD
    A[开始] --> B{是否配置正确?}
    B -->|是| C[显示图表]
    B -->|否| D[检查配置]
    D --> B
    C --> E[结束]
```

### 复杂流程图
```mermaid
flowchart LR
    subgraph "前端"
        A[用户界面]
        B[状态管理]
        C[组件库]
    end
    
    subgraph "后端"
        D[API 服务]
        E[业务逻辑]
        F[数据库]
    end
    
    A --> D
    B --> E
    C --> F
    D --> E
    E --> F
```

## 序列图测试

```mermaid
sequenceDiagram
    participant U as 用户
    participant C as 客户端
    participant S as 服务器
    participant D as 数据库
    
    U->>C: 发起请求
    C->>S: API 调用
    S->>D: 查询数据
    D-->>S: 返回结果
    S-->>C: 响应数据
    C-->>U: 显示结果
```

## 类图测试

```mermaid
classDiagram
    class ModuForge {
        +State state
        +EventBus eventBus
        +PluginManager plugins
        +apply_transaction(tr)
        +get_state()
    }
    
    class State {
        +Document doc
        +Selection selection
        +get_node(id)
        +update(changes)
    }
    
    class Plugin {
        +name: string
        +version: string
        +init()
        +destroy()
    }
    
    ModuForge --> State
    ModuForge --> Plugin
    State --> Document
```

## 甘特图测试

```mermaid
gantt
    title ModuForge-RS 开发计划
    dateFormat  YYYY-MM-DD
    section 核心框架
    状态管理        :done,    core,     2024-01-01,2024-02-01
    事件系统        :done,    events,   2024-01-15,2024-02-15
    插件系统        :active,  plugins,  2024-02-01,2024-03-01
    
    section 扩展功能
    协作系统        :crit,    collab,   2024-02-15,2024-03-15
    模板系统        :         template, 2024-03-01,2024-03-30
    规则引擎        :         rules,    2024-03-15,2024-04-15
    
    section 文档和测试
    API 文档        :         docs,     2024-02-01,2024-04-01
    单元测试        :         tests,    2024-01-01,2024-04-30
    集成测试        :         integration, 2024-03-01,2024-04-30
```

## 状态图测试

```mermaid
stateDiagram-v2
    [*] --> 初始化
    初始化 --> 加载中
    加载中 --> 就绪
    加载中 --> 错误
    就绪 --> 执行中
    执行中 --> 就绪
    执行中 --> 错误
    错误 --> 重试
    重试 --> 加载中
    就绪 --> [*]
    错误 --> [*]
```

## 饼图测试

```mermaid
pie title ModuForge-RS 组件分布
    "核心框架" : 30
    "状态管理" : 25
    "插件系统" : 20
    "协作功能" : 15
    "工具库" : 10
```

## Git 图测试

```mermaid
gitgraph:
    options:
    {
        "mainBranchName": "main"
    }
    commit
    commit
    branch feature/collaboration
    checkout feature/collaboration
    commit
    commit
    checkout main
    commit
    merge feature/collaboration
    commit
    branch feature/rules-engine
    checkout feature/rules-engine
    commit
    commit
    checkout main
    merge feature/rules-engine
```

## 实体关系图测试

```mermaid
erDiagram
    Document ||--o{ Node : contains
    Node ||--o{ Attribute : has
    Node ||--o{ Mark : has
    Document ||--|| Schema : validates
    
    Document {
        string id
        string version
        datetime created_at
        datetime updated_at
    }
    
    Node {
        string id
        string type
        json content
        int position
    }
    
    Attribute {
        string name
        string value
        string type
    }
    
    Mark {
        string type
        int start
        int end
        json attrs
    }
    
    Schema {
        string name
        string version
        json definition
    }
```

## 用户旅程图测试

```mermaid
journey
    title 用户使用 ModuForge-RS 的旅程
    section 发现
      了解框架: 5: 用户
      阅读文档: 3: 用户
      查看示例: 4: 用户
    section 试用
      安装框架: 3: 用户
      运行示例: 4: 用户
      编写代码: 2: 用户
    section 采用
      集成项目: 3: 用户, 开发团队
      部署上线: 4: 用户, 开发团队, 运维团队
      持续维护: 5: 开发团队, 运维团队
```

## 象限图测试

```mermaid
quadrantChart
    title ModuForge-RS 功能优先级矩阵
    x-axis 低复杂度 --> 高复杂度
    y-axis 低价值 --> 高价值
    
    quadrant-1 快速胜利
    quadrant-2 重大项目
    quadrant-3 填补项目
    quadrant-4 可能的浪费
    
    状态管理: [0.8, 0.9]
    插件系统: [0.6, 0.8]
    协作功能: [0.9, 0.7]
    模板系统: [0.3, 0.6]
    规则引擎: [0.7, 0.5]
    工具库: [0.2, 0.4]
```

## 时间线图测试

```mermaid
timeline
    title ModuForge-RS 发展历程
    
    2024年1月 : 项目启动
              : 核心架构设计
              : 状态管理实现
    
    2024年2月 : 插件系统开发
              : 事件机制完善
              : API 文档编写
    
    2024年3月 : 协作功能实现
              : 模板系统开发
              : 性能优化
    
    2024年4月 : 规则引擎集成
              : 完整测试覆盖
              : 1.0 版本发布
```

---

## 测试说明

如果你能看到上述所有图表正常显示，说明 Mermaid 配置已经成功！

### 特性验证清单

- ✅ 流程图 (Flowchart)
- ✅ 序列图 (Sequence Diagram)  
- ✅ 类图 (Class Diagram)
- ✅ 甘特图 (Gantt Chart)
- ✅ 状态图 (State Diagram)
- ✅ 饼图 (Pie Chart)
- ✅ Git 图 (Git Graph)
- ✅ 实体关系图 (ER Diagram)
- ✅ 用户旅程图 (User Journey)
- ✅ 象限图 (Quadrant Chart)
- ✅ 时间线图 (Timeline)

### 样式特性

- 🎨 自定义主题色彩
- 📱 响应式设计
- 🌙 暗色模式支持
- ✨ 悬停动画效果
- 🖼️ 边框和阴影
- 📐 居中对齐
- 🎯 打印友好 
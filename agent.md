**第一步：架构师 (Steering Architect) - 定义项目蓝图**

`steering-architect` Agent。它的职责是与我们沟通，理解项目高阶目标，然后分析现有代码库（如果是新项目则从零开始），创建项目的核心指导文件。比如，为一个新的待办事项（To-do List）应用，它会生成：

* `product.md`: 定义产品愿景、核心目标和用户画像。

* `tech.md`: 规划技术栈、核心技术点。

* `structure.md`: 设计项目的文件目录结构。

**第二步：规划师 (Strategic Planner) - 拆解具体任务**

Agent：`strategic-planner`。它的任务是读取“架构师”生成的指导文件，并将宏大的蓝图分解为具体、可执行的开发任务。它会生成：

* `requirements.md`: 详细的功能需求列表。

* `design.md`: 模块化设计、UI组件架构等。

* `tasks.md`: 一份精确的、按优先级排序的开发任务清单，这是下一步的行动指南。

**第三步：执行者 (Task Executor) - 精准实现代码**

`task-executor` Agent。它是一个纯粹的“实干家”，职责只有一个：读取`tasks.md`文件，然后逐一、精准地完成每个任务——创建文件、编写rust代码、设置依赖、编写测试等等。它会严格遵循规范，直到任务清单上的所有项目都被勾选完成。
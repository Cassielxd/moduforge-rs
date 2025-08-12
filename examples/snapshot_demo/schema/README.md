# ModuForge Schema 配置说明

这个目录包含了 ModuForge 项目的 schema 配置文件。

## 文件结构

```
schema/
├── main.xml             # 主schema配置文件
├── moduforge-schema.xsd # XSD验证文件（提供编辑器智能提示）
└── README.md           # 本说明文件
```

## 自动加载机制

ModuForge 运行时会自动查找并加载项目根目录下的 `schema/main.xml` 文件：

```rust
use mf_core::{ForgeRuntime, ForgeAsyncRuntime};

// 自动加载 schema/main.xml（如果存在）
let runtime = ForgeRuntime::from_xml_schema(None, None).await?;
let async_runtime = ForgeAsyncRuntime::from_xml_schema(None, None).await?;
```

## 配置文件说明

### main.xml

主schema配置文件，定义了文档的结构和行为：

- **节点定义** (`<nodes>`)：定义文档中可用的节点类型
- **标记定义** (`<marks>`)：定义文档中可用的标记类型
- **全局属性** (`<global_attributes>`)：定义适用于多种节点的通用属性

### moduforge-schema.xsd

XSD验证文件，为编辑器提供智能提示功能：

- **自动完成**：输入时提示可用的元素和属性
- **语法验证**：实时检查XML语法错误
- **文档提示**：显示元素和属性的说明信息

## 编辑器配置

### VS Code

1. 安装 "XML" 扩展 (Red Hat)
2. 打开 `schema/main.xml` 文件
3. 享受智能提示和语法验证

### IntelliJ IDEA / WebStorm

1. 内置XML支持，无需额外配置
2. 打开 `schema/main.xml` 文件
3. IDE会自动识别XSD并提供智能提示

### 其他编辑器

大多数现代编辑器都支持XML Schema验证，请参考编辑器文档配置XML支持。

## 快速开始

1. **复制示例文件**：
   ```bash
   cp schema/main.xml your-project/schema/main.xml
   cp schema/moduforge-schema.xsd your-project/schema/moduforge-schema.xsd
   ```

2. **编辑schema配置**：
   - 修改 `main.xml` 中的节点和标记定义
   - 根据需要添加或删除节点类型
   - 配置全局属性

3. **在代码中使用**：
   ```rust
   use mf_core::ForgeRuntime;
   
   // 自动加载schema配置
   let runtime = ForgeRuntime::from_xml_schema(None, None).await?;
   ```

## 节点定义示例

```xml
<node name="paragraph" group="block">
  <desc>段落节点，包含内联内容</desc>
  <content>inline*</content>
  <marks>strong em link</marks>
  <attrs>
    <attr name="align" default="left"/>
    <attr name="indent" default="0"/>
  </attrs>
</node>
```

## 标记定义示例

```xml
<mark name="strong" group="formatting">
  <desc>粗体文本标记</desc>
  <spanning>true</spanning>
  <attrs>
    <attr name="weight" default="bold"/>
  </attrs>
</mark>
```

## 全局属性示例

```xml
<global_attributes>
  <global_attribute types="paragraph heading">
    <attr name="id"/>
    <attr name="class"/>
    <attr name="style"/>
  </global_attribute>
  <global_attribute types="*">
    <attr name="data-custom"/>
  </global_attribute>
</global_attributes>
```

## 高级功能

### 多文件引用

支持通过 `<imports>` 和 `<includes>` 引用其他schema文件：

```xml
<schema top_node="doc">
  <imports>
    <import src="./base-nodes.xml"/>
  </imports>
  <includes>
    <include src="./extensions.xml"/>
  </includes>
  <!-- 本文件的定义 -->
</schema>
```

### 内容规则

支持丰富的内容规则语法：

- `paragraph+` - 一个或多个段落
- `text*` - 零个或多个文本节点
- `heading?` - 零个或一个标题
- `_` - 不允许任何内容

### 属性类型

支持多种属性默认值类型：

- 字符串：`default="text"`
- 数字：`default="42"`
- 布尔值：`default="true"`
- JSON：`default='{"key": "value"}'`

## 故障排除

### schema/main.xml 不存在

如果文件不存在，运行时会使用默认配置。创建该文件后重启应用即可。

### 智能提示不工作

1. 确认XSD文件路径正确
2. 检查编辑器是否支持XML Schema
3. 重新加载编辑器或项目

### 语法错误

1. 检查XML语法是否正确
2. 验证元素顺序是否符合XSD定义
3. 确认必需属性是否存在

## 参考资料

- [ModuForge 文档](../packages/docs/)
- [XML Schema 标准](https://www.w3.org/XML/Schema)
- [示例项目](../demo/)

有问题请参考项目文档或提交issue。 
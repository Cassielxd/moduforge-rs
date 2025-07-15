# ModuForge XML Schema 智能提示配置

这个目录包含了 ModuForge XML Schema 的 XSD 定义文件，可以为编辑器提供智能提示功能。

## 文件说明

- `moduforge-schema.xsd` - ModuForge XML Schema 的 XSD 定义文件
- `basic-document.xml` - 使用 XSD 的示例文档
- `multi-file/` - 多文件引用的示例

## 如何使用

### 1. 在 XML 文件中引用 XSD

在您的 ModuForge XML schema 文件的根元素中添加以下属性：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<schema top_node="doc" 
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="moduforge-schema.xsd">
  <!-- 您的 schema 内容 -->
</schema>
```

### 2. 编辑器配置

#### VS Code
1. 安装 "XML" 扩展 (Red Hat)
2. 确保 `moduforge-schema.xsd` 文件在您的项目目录中
3. 在 XML 文件中添加 XSD 引用后，编辑器会自动提供智能提示

#### IntelliJ IDEA / WebStorm
1. 内置 XML 支持
2. 在 XML 文件中添加 XSD 引用后，IDE 会自动识别并提供提示

#### Eclipse
1. 内置 XML 编辑器支持
2. 在 XML 文件中添加 XSD 引用后，编辑器会提供智能提示

### 3. 智能提示功能

使用 XSD 后，编辑器会提供以下智能提示功能：

#### 元素提示
- 在 `<schema>` 内输入 `<` 时，会提示可用的子元素（nodes, marks, imports, includes, global_attributes）
- 在 `<nodes>` 内输入 `<` 时，会提示 `<node>` 元素
- 在 `<marks>` 内输入 `<` 时，会提示 `<mark>` 元素

#### 属性提示
- 在 `<schema>` 标签内会提示 `top_node` 属性
- 在 `<node>` 标签内会提示 `name` 和 `group` 属性
- 在 `<mark>` 标签内会提示 `name` 和 `group` 属性
- 在 `<attr>` 标签内会提示 `name` 和 `default` 属性

#### 属性值提示
- `group` 属性会提示预定义的分组类型（block, inline, formatting, link 等）
- `spanning` 属性会提示 true/false 值

#### 验证功能
- 检查必需属性是否存在
- 验证属性值是否符合定义的类型
- 检查元素结构是否正确

### 4. 完整示例

```xml
<?xml version="1.0" encoding="UTF-8"?>
<schema top_node="doc" 
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="moduforge-schema.xsd">
  
  <global_attributes>
    <global_attribute types="paragraph heading">
      <attr name="id"/>
      <attr name="class"/>
    </global_attribute>
  </global_attributes>
  
  <nodes>
    <node name="doc" group="block">
      <desc>文档根节点</desc>
      <content>paragraph+</content>
      <attrs>
        <attr name="title" default="Untitled Document"/>
        <attr name="version" default="1.0"/>
      </attrs>
    </node>
    
    <node name="paragraph" group="block">
      <desc>段落节点</desc>
      <content>text*</content>
      <marks>strong em</marks>
      <attrs>
        <attr name="align" default="left"/>
      </attrs>
    </node>
    
    <node name="text">
      <desc>文本节点</desc>
    </node>
  </nodes>
  
  <marks>
    <mark name="strong" group="formatting">
      <desc>粗体标记</desc>
      <spanning>true</spanning>
    </mark>
    
    <mark name="em" group="formatting">
      <desc>斜体标记</desc>
      <spanning>true</spanning>
    </mark>
  </marks>
  
</schema>
```

### 5. 故障排除

#### 智能提示不工作？
1. 确认 XSD 文件路径正确
2. 确认 XML 文件中正确引用了 XSD
3. 重启编辑器或重新加载项目
4. 检查编辑器是否安装了 XML 扩展

#### 路径问题？
- 使用相对路径：`xsi:noNamespaceSchemaLocation="./moduforge-schema.xsd"`
- 使用绝对路径：`xsi:noNamespaceSchemaLocation="file:///path/to/moduforge-schema.xsd"`
- 使用 HTTP 路径：`xsi:noNamespaceSchemaLocation="http://example.com/moduforge-schema.xsd"`

#### 验证错误？
- 检查 XML 结构是否符合 XSD 定义
- 确认必需属性是否存在
- 检查属性值是否正确

### 6. 高级功能

#### 多文件引用
XSD 支持 import 和 include 元素：

```xml
<schema top_node="doc" 
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="moduforge-schema.xsd">
  
  <imports>
    <import src="./base-nodes.xml"/>
  </imports>
  
  <includes>
    <include src="./extensions.xml"/>
  </includes>
  
  <!-- 本文件的定义 -->
</schema>
```

#### 自定义验证
可以扩展 XSD 文件来添加更多验证规则：

```xml
<!-- 添加更多的枚举值 -->
<xs:simpleType name="CustomNodeGroupType">
  <xs:restriction base="xs:string">
    <xs:enumeration value="block"/>
    <xs:enumeration value="inline"/>
    <xs:enumeration value="custom"/>
  </xs:restriction>
</xs:simpleType>
```

## 支持的编辑器

- ✅ Visual Studio Code (需要 XML 扩展)
- ✅ IntelliJ IDEA / WebStorm
- ✅ Eclipse
- ✅ Sublime Text (需要 XML 插件)
- ✅ Atom (需要 XML 插件)
- ✅ Vim/Neovim (需要 XML 插件)

## 其他解决方案

如果您需要更高级的功能，可以考虑：

1. **Language Server Protocol (LSP)** - 为 ModuForge XML 开发专门的语言服务器
2. **Monaco Editor** - 在 Web 应用中集成智能提示
3. **自定义编辑器插件** - 为特定编辑器开发专门的插件

这个 XSD 文件提供了基础但功能完整的智能提示支持，应该能满足大多数编辑需求。 
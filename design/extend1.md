### **扩展支持模块**

#### **moduforge-macros & moduforge-macros-derive**

* **用途**：简化 模型mf-core 中的 Node 和Mark 的定义映射  ，并提供转换成 Node 定义的 toNode 方法

* **应用场景**：

  * 节点定义的时候 根据 宏 自动生成

  * 样板代码减少

  * 编译时验证

* **使用示例**：

```rust
#[derive(Node, Serialize, Deserialize)]
#[node_type = "GCXM" , marks="color",content="DCXM"]
pub struct ConstructProject {
    // 自动生成Node包装器代码
    #[attr]
    name:String,
    #[attr]
    age:Option<i32>
}
#[derive(Mark, Serialize, Deserialize)]
#[mark_type = "color"]
pub struct Color {
  // 自动生成Node包装器代码
  #[attr]
  name:String,
  #[attr]
  age:Option<i32>
}
///  
```
### 严格执行原则
* 在现有 moduforge-macros 和moduforge-macros-derive库上进行扩展
# ModuForge 示例集合

本目录包含了ModuForge依赖注入框架的各种功能演示。

## 🚀 示例列表

### 📦 `comprehensive_demo.rs` - **综合功能演示**
**推荐首先运行此示例！**

展示ModuForge的所有核心功能：
- ✨ 依赖注入容器 - 组件注册、生命周期管理、依赖解析
- 🎯 自动AOP代理 - 通过`auto_proxy = true`实现零手动操作
- 🔍 AOP切面编程 - 前置、后置、环绕、异常处理切面
- ⚙️ 配置管理 - 配置注入和环境变量支持
- 🏭 Bean工厂 - 复杂对象创建
- 📊 系统监控 - 性能监控和安全审计
- ❌ 错误处理 - 完整的异常处理机制

```bash
cargo run --example comprehensive_demo
```

### 📚 基础示例

#### `basic_usage.rs` - **基础用法**
最简单的ModuForge使用示例，适合初学者：
- 基本的组件定义和注册
- 简单的依赖注入
- 容器的基本操作

```bash
cargo run --example basic_usage
```

#### `macro_usage.rs` - **宏系统使用**
展示ModuForge的宏系统：
- `#[derive(Component)]` 自动实现
- `#[service]` 属性宏
- `#[bean]` 工厂方法

```bash
cargo run --example macro_usage
```

### 🎯 AOP专题示例

#### `aop_macro_demo.rs` - **AOP宏注册演示**
展示AOP切面的宏化自动注册：
- 5种切面类型的derive宏
- 自动切面注册机制
- 切点表达式配置

```bash
cargo run --example aop_macro_demo
```

#### `auto_aop_demo.rs` - **自动AOP演示**
展示AOP代理的使用：
- 手动代理创建方式
- AOP代理包装器
- 方法拦截和切面应用

```bash
cargo run --example auto_aop_demo
```

## 🎓 学习路径建议

### 初学者路径
1. **`basic_usage.rs`** - 了解基本概念
2. **`macro_usage.rs`** - 学习宏系统
3. **`comprehensive_demo.rs`** - 掌握完整功能

### 进阶用户路径
1. **`comprehensive_demo.rs`** - 快速了解全部功能
2. **`aop_macro_demo.rs`** - 深入理解AOP宏系统
3. **`auto_aop_demo.rs`** - 掌握AOP代理机制

## 🔧 运行要求

确保您的Rust版本支持以下特性：
- `async/await` 语法
- 过程宏 (procedural macros)
- `tokio` 异步运行时

## 📖 关键概念

### 依赖注入
- **组件**: 实现`Component` trait的结构体
- **生命周期**: Singleton（单例）、Transient（瞬态）、Scoped（作用域）
- **自动解析**: 容器自动管理依赖关系

### AOP切面编程
- **切面类型**: Before（前置）、After（后置）、Around（环绕）、AfterThrowing（异常后）
- **自动代理**: 通过`auto_proxy = true`启用
- **切点表达式**: 支持通配符匹配

### 宏系统
- **Derive宏**: 自动实现trait
- **属性宏**: 组件和服务标注
- **自动注册**: 通过`ctor`实现启动时注册

## 🚀 快速开始

```rust
use mf_contex::*;

#[derive(Debug, Default, Component)]
#[component(name = "my_service", auto_proxy = true)]
pub struct MyService;

impl MyService {
    pub async fn hello(&self) -> String {
        "Hello, ModuForge!".to_string()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_container().await?;
    
    let container = global_container();
    let service: Arc<MyService> = container.resolve().await?;
    
    println!("{}", service.hello().await);
    Ok(())
}
```

## 🎉 特色功能

- **零配置**: 组件自动注册，无需手动配置
- **类型安全**: 编译时依赖检查
- **异步支持**: 完整的async/await支持
- **性能优越**: 零运行时开销的依赖注入
- **功能丰富**: 企业级特性完备

立即运行 `cargo run --example comprehensive_demo` 体验完整功能！
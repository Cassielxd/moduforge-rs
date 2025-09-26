# Dynamic Extension Loader Design

本文档说明 `DynamicExtensionLoader` 的体系架构、插件 ABI 约定、运行时加载流程、快照/配置集成以及测试规划，用于支撑以 DLL 形式动态引入节点定义、标记与插件扩展。

---

## 1. 设计目标

- **运行时扩展**：无需重启即可加载/卸载扩展，支持热更新。
- **跨平台**：兼容 Windows（DLL）、Linux（SO）、macOS（DYLIB）。
- **安全可靠**：对版本、签名、依赖与回滚策略进行约束。
- **可持久化**：快照与配置能够记录并恢复动态扩展。
- **易于扩展**：提供稳定的 ABI 与开发模板，降低第三方接入门槛。

---

## 2. 模块结构

```
crates/core/src/extension_manager/
  ├── mod.rs
  ├── dynamic/
  │     ├── mod.rs            // 对外导出
  │     ├── loader.rs         // DynamicExtensionLoader 核心逻辑
  │     ├── handle.rs         // DynamicHandle / ExtensionId
  │     ├── descriptor.rs     // ExtensionDescriptor 解析
  │     ├── ffi.rs            // 插件 ABI 定义
  │     └── tests.rs          // 单元与组件测试
  └── ...
```

- `DynamicExtensionLoader`：管理动态扩展的加载、卸载、重载，向 `ExtensionManager` 提供扩展列表。
- `DynamicHandle`：封装单个 DLL 及其注册结果、资源句柄、插件实例。
- `ExtensionDescriptor`：描述插件提供的节点、标记、插件、操作函数和元数据。
- `ffi`：跨 DLL 边界的数据结构、函数签名及宿主上下文。

---

## 3. 核心类型

### 3.1 ExtensionId
- 推荐格式：`"{manifest.name}@{manifest.version}"` 或 `Uuid`。
- 在快照、日志、运行时命令中唯一标识动态扩展。

### 3.2 DynamicHandle
- 字段示例：
  - `library`: 跨平台库句柄（`libloading::Library` / Windows `HMODULE`）
  - `manifest`: 插件 `Manifest`
  - `descriptor`: 解析后的 `ExtensionDescriptor`
  - `host_handle`: 插件返回的卸载句柄
  - `extensions`: `Vec<Extensions>`
  - `plugins`: `Vec<Arc<Plugin>>`
  - `op_fns`: `Vec<Arc<dyn Fn(&GlobalResourceManager) -> ForgeResult<()>>>`
- `Drop` 时若插件提供 `mf_unregister_extension`，自动调用。

### 3.3 ExtensionDescriptor
- 插件声明的节点、标记、插件、操作函数集合，支持 JSON/MessagePack 格式。
- 关键字段：`nodes`, `marks`, `plugins`, `operations`, `metadata`, `capabilities`, `dependencies`, `version`。
- 由 `descriptor.rs` 提供解析、校验和转换为内部类型的工具函数。

---

## 4. 插件 ABI 约定

插件必须使用 C ABI 导出以下符号：

```c
extern "C" const char* mf_extension_manifest(void);

extern "C" MfResult mf_register_extension(
    const MfHostContext* ctx,
    MfRegistration* out_registration
);

extern "C" void mf_free_registration(MfRegistration* registration);

// 可选：若存在则在卸载时调用
extern "C" void mf_unregister_extension(MfExtensionHandle handle);
```

### 4.1 Manifest JSON
- 插件利用 `mf_extension_manifest` 返回 UTF-8 JSON 字符串。
- 结构示例：
```json
{
  "name": "sample.richtext",
  "version": "1.2.0",
  "min_host_version": "0.4.0",
  "target_platform": ["windows-x86_64", "linux-x86_64"],
  "capabilities": ["nodes", "marks", "plugins"],
  "dependencies": [],
  "integrity_hash": "sha256:..."
}
```
- 宿主需验证：宿主版本、目标平台、所需能力、哈希/签名等。

### 4.2 MfRegistration
- 插件在 `mf_register_extension` 中填充：
  - `nodes`: `MfNodeDescriptor` 数组（名称、内容、属性等）
  - `marks`: `MfMarkDescriptor` 数组
  - `plugins`: 插件工厂或实例描述
  - `operations`: 操作函数描述，供扩展注册资源
  - `handle`: 插件定义的 opaque handle（供卸载使用）
  - `userdata`: 插件上下文
- 插件必须实现 `mf_free_registration` 释放注册对象，防止内存泄漏。

### 4.3 MfHostContext
- 宿主提供给插件的上下文，包含：
  - 版本信息：`host_version`, `api_version`
  - 宿主能力：`runtime_capabilities`
  - 工具函数：`log_fn(level, message)`, `alloc_fn`, `dealloc_fn`（可选）
  - 未来可扩展更多回调（如事件、配置访问）

---

## 5. 动态加载流程

1. **安全校验**
   - 路径白名单、签名检测、哈希比对。
   - 使用 `tokio::task::spawn_blocking` 加载库，失败返回 `ForgeError::Extension`。

2. **读取 Manifest**
   - 调用 `mf_extension_manifest`，解析 JSON。
   - 校验宿主版本/平台/能力，失败立即卸载库。

3. **注册扩展**
   - 构造 `MfHostContext`
   - 调用 `mf_register_extension`
   - 将 `MfRegistration` 转换为 `ExtensionDescriptor`
   - 进一步构建内部 `Node`、`Mark`、`Plugin`、`op_fns`
   - 任一步失败调用 `mf_free_registration` 并卸载库。

4. **提交给 ExtensionManager**
   - 获取写锁，将扩展合并入 `ExtensionManager`
   - 更新 schema、插件列表、操作函数
   - 广播 `Event::ExtensionLoaded`
   - 存储 `DynamicHandle`

5. **记录快照信息**
   - `ExtensionId`、路径、版本、hash、加载时间等，方便快照恢复。

---

## 6. 卸载与热更新

### 6.1 卸载流程

1. 查找 `DynamicHandle`
2. 从 `ExtensionManager` 移除关联节点/标记/插件/操作
3. 广播 `Event::ExtensionUnloaded`
4. 调用 `mf_unregister_extension(handle)`（若存在）
5. 从内部映射移除 handle

若过程中出现错误（如插件拒绝卸载），返回 `ForgeError::Extension` 并保留现有 handle，避免状态半更新。

### 6.2 热更新流程

1. 先加载新 DLL（完整执行加载流程但暂不提交）
2. 构造新 `DynamicHandle`
3. 获取写锁：
   - 卸载旧 handle
   - 注册新 handle
4. 注册成功后广播 `Event::ExtensionReloaded { old, new }`

若新版本加载失败，保持旧版本不变。

---

## 7. 配置与启动

`ForgeConfig.extension.dynamic` 建议结构：

```rust
pub struct DynamicExtensionConfig {
    pub search_paths: Vec<PathBuf>,
    pub auto_load: Vec<DynamicSpec>, // { path, integrity, mode }
    pub signature_required: bool,
    pub reload_policy: ReloadPolicy, // OnDemand | Auto
}
```

- 在 `ForgeRuntime::create_with_config` 中实例化 `DynamicExtensionLoader`
- 遍历 `auto_load` 列表自动加载；失败行为受 `reload_policy` 控制
- 提供运行时 API：
  - `register_dynamic_extension(path: impl AsRef<Path>)`
  - `unregister_extension(id: &ExtensionId)`
  - `reload_extension(id: &ExtensionId, path: impl AsRef<Path>)`

---

## 8. 快照集成

`CoreSnapshot` 新增：

```rust
pub struct DynamicSnapshot {
    pub id: ExtensionId,
    pub path: String,
    pub version: String,
    pub manifest: serde_json::Value,
    pub hash: String,
    pub loaded_at: u64,
}

pub struct CoreSnapshot {
    // ...
    pub dynamic_extensions: Vec<DynamicSnapshot>,
}
```

- 保存快照：遍历 loader 中所有 handle，生成 `DynamicSnapshot`
- 恢复快照：读取 `dynamic_extensions`，按顺序调用 `load_from_path` 并校验 hash
- 失败策略：
  - `strict`：若任意 DLL 加载失败则恢复失败
  - `warn`：记录 warning，跳过该扩展
  - `ignore`：完全忽略（不推荐）

策略由配置驱动。

---

## 9. 并发与线程安全

- loader 内部使用 `DashMap<ExtensionId, DynamicHandle>` 管理 handle
- 访问 `ExtensionManager` 时使用 `RwLock`，确保 schema 更新原子性
- 所有阻塞操作（`LoadLibrary`, `dlopen`）均通过 `spawn_blocking`
- 插件回调宿主（如 `log_fn`）时必须捕获 `panic`，转换为 `ForgeError::ExternalDependency`

---

## 10. 监控与事件

### 10.1 Metrics
- `core.extension.dynamic.load.duration`
- `core.extension.dynamic.load.failures`
- `core.extension.dynamic.unload.failures`
- `core.extension.dynamic.active_count`

### 10.2 Events
- `Event::ExtensionLoaded(ExtensionId, Manifest)`
- `Event::ExtensionUnloaded(ExtensionId)`
- `Event::ExtensionReloaded { old: ExtensionId, new: ExtensionId }`

### 10.3 Logging
- 加载/卸载成功：info
- 兼容性/校验警告：warn
- 安全/加载失败：error

---

## 11. 测试计划

1. **单元测试**
   - manifest 解析及错误分支
   - descriptor→内部结构的转换校验
   - 模拟 `mf_register_extension` 返回异常字段

2. **组件测试**
   - 使用 `test-data/dlls` 示例插件加载→验证 schema/插件→卸载
   - 熱更新流程验证（v1→v2→回滚）

3. **快照测试**
   - 加载动态扩展→生成快照→清理→恢复快照→验证扩展重新加载成功

4. **跨平台验证**
   - CI on Windows/Linux/macOS 运行加载/卸载脚本，确保 ABI 正确

---

## 12. 开发者指南

- 提供模板项目 `examples/dynamic-plugin`：
  - 包含 manifest 生成、节点/标记/插件示例、`mf_register_extension` 实现。
- 构建脚本：
  - `scripts/build_dynamic_plugin.sh / .ps1`：编译、生成 manifest、可选签名。
- 文档：
  - 常见错误说明（版本不兼容、符号缺失、签名失败）
  - Host API 版本升级的迁移指南

---

## 13. 未来扩展

- 插件隔离：引入多进程或 WebAssembly 沙箱。
- 远程仓库：支持从服务端下载并缓存扩展（需增强签名策略）。
- 调试工具：运行时列出扩展、查看 manifest、命令行热更新。
- 脚本扩展：在现有 Loader 框架下统一管理 wasm/python 等插件形态。

---

## 14. 参考资料

- [`libloading` crate 文档](https://docs.rs/libloading)
- Windows `LoadLibraryExW` / POSIX `dlopen`
- Rust FFI 与 ABI 兼容指南
- 本仓库 `ExtensionManager`, `SnapshotManager`, `ForgeRuntime` 代码基线

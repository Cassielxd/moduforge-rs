# 改造文档

## 目标
- 去除资源表中的未定义行为，保证类型安全。
- 降低文档状态流转时的内存复制成本，改善编辑延迟。
- 修正 `NodePool` 查询缓存与并行归约的正确性问题，并收敛调试输出。
- 降低深层文档输入对递归遍历的栈风险。

## 现状与问题
- `crates/state/src/resource.rs` 的 `downcast_arc/downcast` 使用 `unsafe` 指针转换，`ResourceTable` 调用时存在内存破坏风险。
- `State::doc()`、`State::apply_inner()` 与 `Transaction::doc()`（`crates/state`）多次整树 `clone`，对大文档是 O(n) 开销。
- `NodePool::parallel_query_reduce`（`crates/model/src/node_pool.rs`）在 reduce 阶段丢弃分片结果，同时多个 `println!` 泄露内部信息；缓存键对无法序列化的属性退化为空字符串。
- `NodePool::_collect_descendants` 与 `Tree::remove_subtree` 为递归实现，深度内容可触发栈溢出。

## 改造方案
1. **资源类型安全**  
   - 将 `Resource` trait 的 `downcast_*` 改为基于 `Arc<dyn Any + Send + Sync>` 的 `Arc::downcast`。  
   - `ResourceTable::get/take` 等接口配合调整，必要时新增辅助函数，确保失败路径返回 `None`。  
   - 补充单元测试覆盖正确类型与错误类型的场景。
2. **状态/事务共享树**  
   - 为 `NodePool` 增加从现有 `Arc<Tree>` 构建的快速路径，避免重新分配。  
   - `Transaction::doc()` 若已存在步骤，优先返回共享 `Arc<RwLock<Tree>>` 包装的 `NodePool`，仅在需要独占修改时复制。  
   - `State::apply_inner` 直接沿用 `Transaction` 提供的共享树实例，不再 `clone`。  
   - 对现有调用链做基准测试（构建含 1e5 节点文档）验证延迟下降。
3. **查询缓存与日志治理**  
   - 修正 `parallel_query_reduce` 的归约函数为真实组合，移除占位 `dummy_node`。  
   - 缓存键序列化失败时返回错误（或记录为 `tracing::warn!` 并跳过缓存），保证键唯一。  
   - 将 `println!` 改为 `tracing::{debug, info}`，并通过 `cfg!(debug_assertions)` 控制默认关闭。  
   - 追加单元测试覆盖 reduce 逻辑与缓存键的错误分支。
4. **递归改写**  
   - 将 `_collect_descendants` 与 `remove_subtree` 改为显式栈循环（如 `VecDeque`），并在遍历时限制最大深度。  
   - 对极端深度（>10k）输入添加回归测试，验证不会栈溢出。
5. **序列化优化（可选并行）**  
   - `State::serialize` 直接将 `NodePool` 写入 `Vec<u8>`（使用 `serde_json::to_writer`），避免中间 `String`。

## 实施步骤
- **第 1 周**：完成资源表安全化与相关测试；重构 `NodePool` 构造函数并迁移调用。
- **第 2 周**：改写 `Transaction`/`State` 的树共享策略，添加性能基准。
- **第 3 周**：修复 `NodePool` 缓存逻辑、日志和 reduce，实现递归迭代化，补充深度测试。
- **第 4 周**：联调并执行 `cargo fmt && cargo clippy && cargo test --workspace`，准备 benchmark 报告。

## 测试计划
- **单元测试**：覆盖资源表 downcast、reduce 正确性、缓存键错误分支、深度遍历和状态序列化。
- **集成测试**：模拟 10k+ 步骤编辑流程验证无 panic/性能下降。
- **基准测试**：使用 `cargo bench` 对比改造前后事务提交耗时与内存峰值。
- **日志验收**：在 release 构建中确保无多余 stdout 输出。

## 风险与回滚
- 树共享可能引入并发借用冲突；需保证 `Transform`/`NodePool` 接口保持 `Send + Sync`。若触发线程安全问题，可在 `Transaction::doc()` 内保留按需复制的后备路径。
- downcast 逻辑调整可能影响现有插件；提供迁移指南并暂时保留带 `deprecated` 标记的包装函数。
- 若迭代遍历导致性能回退，可通过编译特性临时回退到旧实现，便于快速恢复。

## 交付物
- 更新后的实现代码与单元/基准测试。
- 面向插件开发者的 downcast 迁移指南。
- 性能对比数据与主站回归验证报告。

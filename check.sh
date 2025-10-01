#!/bin/bash
echo "=== 1. 代码格式化 ==="
cargo fmt

echo ""
echo "=== 2. 快速类型检查 ==="
cargo check --all-targets

echo ""
echo "=== 3. Clippy 代码质量检查 ==="
cargo clippy --all-targets

echo ""
echo "=== 4. 运行测试 ==="
cargo test --lib

echo ""
echo "=== 完成 ==="

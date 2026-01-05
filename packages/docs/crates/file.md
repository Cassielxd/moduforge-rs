# moduforge-file 文档

`moduforge-file` 提供持久化文件格式、追加式日志、CRC 校验和 ZIP 封装。

## 概述

File 层提供高效的文件存储格式,支持增量写入、校验和压缩。

## 核心功能

- **追加式日志**：只追加的文件格式
- **CRC 校验**：数据完整性验证
- **Blake3 哈希**：快速哈希计算
- **ZIP 封装**：文档压缩和归档
- **内存映射**：高效文件读取

## 使用示例

```rust
use mf_file::{AppendOnlyFile, FileFormat};

// 创建追加式文件
let mut file = AppendOnlyFile::create("document.mf")?;

// 写入数据
file.append(&data)?;

// 读取数据
let entries = file.read_all()?;

// 压缩归档
file.compress_to_zip("archive.zip")?;
```

## 文件格式

ModuForge 使用自定义的二进制格式:

```
[CRC32(4字节)][数据长度(8字节)][数据]
```

## 下一步

- 查看 [moduforge-persistence](./persistence.md)

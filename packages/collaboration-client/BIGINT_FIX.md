# BigInt 序列化问题修复

## 问题描述

在使用 Yjs 协作客户端时，可能会遇到以下错误：

```
TypeError: Do not know how to serialize a BigInt
    at JSON.stringify (<anonymous>)
    at App.updateDataDisplay (main.ts:153:40)
```

## 问题原因

1. **Yjs 数据结构**：Yjs 的 `toJSON()` 方法可能返回包含 BigInt 值的数据结构
2. **JavaScript 限制**：原生的 `JSON.stringify()` 无法直接序列化 BigInt 类型
3. **触发场景**：当 Yjs 文档中包含大整数时，`toJSON()` 会返回 BigInt 对象

## 解决方案

### 1. 自定义序列化函数

创建了 `safeStringify` 方法来处理 BigInt：

```typescript
private safeStringify(obj: any, space?: number): string {
  try {
    return JSON.stringify(obj, (key, value) => {
      if (typeof value === 'bigint') {
        return value.toString() + 'n'; // 添加 'n' 后缀表示 BigInt
      }
      return value;
    }, space);
  } catch (error) {
    console.error('JSON 序列化失败:', error);
    return JSON.stringify({
      error: '序列化失败',
      message: error instanceof Error ? error.message : '未知错误',
      originalData: String(obj)
    }, null, 2);
  }
}
```

### 2. 替换所有 JSON.stringify 调用

将以下方法中的 `JSON.stringify` 替换为 `safeStringify`：

- `updateDataDisplay()` - 显示文档快照
- `logDetailedChanges()` - 记录详细变更
- `updateCursorDisplay()` - 更新光标显示
- `showDataDifference()` - 显示数据对比

### 3. 错误处理

添加了 try-catch 块来捕获序列化错误，并提供友好的错误信息。

## 修复的文件

- `packages/collaboration-client/example/main.ts`

## 修改的方法

1. **新增方法**：
   - `safeStringify()` - 安全的 JSON 序列化函数

2. **修改的方法**：
   - `updateDataDisplay()` - 使用 `safeStringify` 替代 `JSON.stringify`
   - `logDetailedChanges()` - 使用 `safeStringify` 替代 `JSON.stringify`
   - `updateCursorDisplay()` - 使用 `safeStringify` 替代 `JSON.stringify`
   - `showDataDifference()` - 使用 `safeStringify` 替代 `JSON.stringify`

## 测试建议

1. **基本功能测试**：
   - 连接协作房间
   - 添加节点
   - 更新属性
   - 检查数据是否正确显示

2. **BigInt 测试**：
   - 在节点属性中添加大整数
   - 验证序列化是否成功
   - 检查控制台是否有错误

3. **错误恢复测试**：
   - 模拟序列化失败场景
   - 验证错误处理是否正常工作

## 注意事项

1. **BigInt 表示**：BigInt 值会被转换为字符串形式，添加 `n` 后缀
2. **性能影响**：自定义序列化函数可能比原生 `JSON.stringify` 稍慢
3. **兼容性**：确保目标环境支持 BigInt 类型

## 相关链接

- [Yjs 文档](https://docs.yjs.dev/)
- [BigInt MDN 文档](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt)
- [JSON.stringify MDN 文档](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON/stringify) 
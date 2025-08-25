# 工作台工具函数说明

## 概述

该目录提供了工作台应用的统一工具函数和组合式函数，包括：

- **窗体管理**: 通用窗体管理器和相关功能
- **通用工具**: 格式化、防抖节流等常用工具函数
- **正则表达式**: 常用的验证正则表达式

## 快速开始

```javascript
// 导入所有工具函数
import * as utils from '@/utils'

// 或按需导入
import { 
  useUniversalWindowManager, 
  formatCurrency, 
  formatDateTime,
  debounce 
} from '@/utils'
```

## 工具函数

### 格式化函数

#### `formatFileSize(bytes)`
格式化文件大小显示。

```javascript
import { formatFileSize } from '@/utils'

console.log(formatFileSize(1024))     // '1 KB'
console.log(formatFileSize(1048576))  // '1 MB'
```

#### `formatCurrency(amount, currency?)`
格式化货币显示。

```javascript
import { formatCurrency } from '@/utils'

console.log(formatCurrency(1234.56))        // '¥1,234.56'
console.log(formatCurrency(1234.56, '$'))   // '$1,234.56'
```

#### `formatDateTime(date, format?)`
格式化日期时间显示。

```javascript
import { formatDateTime } from '@/utils'

const now = new Date()
console.log(formatDateTime(now))                    // '2024-01-01 12:00:00'
console.log(formatDateTime(now, 'YYYY-MM-DD'))     // '2024-01-01'
console.log(formatDateTime(now, 'HH:mm:ss'))       // '12:00:00'
```

### 性能优化函数

#### `debounce(func, wait)`
防抖函数，限制函数执行频率。

```javascript
import { debounce } from '@/utils'

const debouncedSearch = debounce((keyword) => {
  console.log('搜索:', keyword)
}, 300)

// 只有在停止输入300ms后才会执行
debouncedSearch('vue')
```

#### `throttle(func, limit)`
节流函数，限制函数执行频率。

```javascript
import { throttle } from '@/utils'

const throttledScroll = throttle(() => {
  console.log('滚动事件')
}, 100)

// 每100ms最多执行一次
window.addEventListener('scroll', throttledScroll)
```

### 数据处理函数

#### `deepClone(obj)`
深度克隆对象。

```javascript
import { deepClone } from '@/utils'

const original = { a: 1, b: { c: 2 } }
const cloned = deepClone(original)
cloned.b.c = 3
console.log(original.b.c)  // 2 (原对象不受影响)
```

#### `isEmpty(value)`
检查值是否为空。

```javascript
import { isEmpty } from '@/utils'

console.log(isEmpty(''))        // true
console.log(isEmpty([]))        // true
console.log(isEmpty({}))        // true
console.log(isEmpty(null))      // true
console.log(isEmpty('hello'))   // false
```

### 工具函数

#### `generateId(prefix?)`
生成唯一ID。

```javascript
import { generateId } from '@/utils'

console.log(generateId())           // 'id-1704096000000-abc123def'
console.log(generateId('window'))   // 'window-1704096000000-xyz789'
```

#### `sleep(ms)`
异步等待指定时间。

```javascript
import { sleep } from '@/utils'

async function example() {
  console.log('开始')
  await sleep(1000)  // 等待1秒
  console.log('结束')
}
```

### 正则表达式

#### `REGEX`
常用的正则表达式常量。

```javascript
import { REGEX } from '@/utils'

// 验证邮箱
console.log(REGEX.EMAIL.test('user@example.com'))  // true

// 验证手机号
console.log(REGEX.PHONE.test('13800138000'))       // true

// 验证身份证
console.log(REGEX.ID_CARD.test('110101199001011234'))  // true

// 验证强密码（至少8位，包含大小写字母和数字）
console.log(REGEX.PASSWORD.test('Password123'))    // true

// 验证URL
console.log(REGEX.URL.test('https://example.com')) // true

// 验证数字
console.log(REGEX.NUMBER.test('123.45'))           // true

// 验证整数
console.log(REGEX.INTEGER.test('123'))             // true

// 验证中文
console.log(REGEX.CHINESE.test('中文'))            // true
```

## 窗体管理器

窗体管理功能已迁移到 `@cost-app/shared-components`，通过工具函数统一导出：

### 基本使用

```javascript
import { useUniversalWindowManager } from '@/utils'

const windowManager = useUniversalWindowManager()

// 打开窗口
await windowManager.quick.estimateDemo()
await windowManager.quick.settings()

// 关闭所有窗口
await windowManager.quick.closeAll()
```

### 不同环境使用

```javascript
import { 
  useMainAppWindowManager,
  useChildAppWindowManager,
  useGlobalWindowManager 
} from '@/utils'

// 主应用使用
const mainManager = useMainAppWindowManager()

// 子应用使用
const childManager = useChildAppWindowManager()

// 全局单例模式
const globalManager = useGlobalWindowManager()
```

## 最佳实践

### 1. 统一导入
建议通过工具函数统一导入，避免重复导入：

```javascript
// ✅ 推荐
import { useUniversalWindowManager, formatCurrency } from '@/utils'

// ❌ 不推荐
import { useUniversalWindowManager } from '@cost-app/shared-components'
import { formatCurrency } from '@/utils/formatters'
```

### 2. 按需使用
根据实际需要选择合适的工具函数：

```javascript
// 轻量级使用
import { debounce, formatDateTime } from '@/utils'

// 完整功能使用
import * as utils from '@/utils'
```

### 3. 类型安全
工具函数都包含了类型检查，建议在 TypeScript 项目中使用：

```typescript
import { formatCurrency, REGEX } from '@/utils'

// 类型安全
const price: string = formatCurrency(100.5)
const isEmail: boolean = REGEX.EMAIL.test('user@example.com')
```

## 使用方法

### 1. 基本使用（工作台和子应用通用）

```javascript
import { useUniversalWindowManager } from '@cost-app/shared-components'

export default {
  setup() {
    const windowManager = useUniversalWindowManager()
    
    return {
      windowManager
    }
  },
  async mounted() {
    // 如果没有开启自动初始化，需要手动初始化
    if (!this.windowManager.initialized.value) {
      await this.windowManager.init()
    }
  }
}
```

### 2. 打开预定义窗口

```javascript
// 打开概算演示窗口
const windowId = await windowManager.openWindow('estimate-demo')

// 打开表格测试窗口（带自定义选项）
await windowManager.openWindow('table-test', {
  width: 1200,
  height: 800
})

// 打开数据查看器窗口
await windowManager.openWindow('data-viewer')

// 打开表单页面窗口
await windowManager.openWindow('form-page')

// 打开系统设置窗口（模态）
await windowManager.openWindow('settings')

// 打开窗体管理演示窗口
await windowManager.openWindow('window-manager-demo')

// 打开子应用 - 概算管理
await windowManager.openWindow('rough-estimate-main')

// 打开子应用 - 主工作台
await windowManager.openWindow('main-shell-dashboard')
```

### 3. 使用快捷方法

```javascript
// 工作台相关窗口快捷方法
await windowManager.quick.estimateDemo()
await windowManager.quick.tableTest()
await windowManager.quick.dataViewer()
await windowManager.quick.formPage()
await windowManager.quick.settings()
await windowManager.quick.windowDemo()

// 子应用快捷方法
await windowManager.quick.roughEstimate()
await windowManager.quick.mainShell()

// 管理快捷方法
await windowManager.quick.closeAll()
const count = windowManager.quick.getOpenCount()
const list = windowManager.quick.getOpenList()
```

### 4. 打开自定义窗口

```javascript
await windowManager.openCustomWindow({
  windowId: 'my-custom-window',
  title: '我的自定义窗口',
  url: '#/my-custom-page',
  width: 900,
  height: 600,
  modal: false,
  type: 'custom-type'
})
```

### 5. 窗口管理

```javascript
// 关闭指定窗口
await windowManager.closeWindow(windowId)

// 关闭所有窗口
await windowManager.closeAllWindows()

// 获取当前打开的窗口列表
const openWindows = windowManager.openWindowList.value
console.log(openWindows)

// 检查指定类型的窗口是否已打开
const isEstimateOpen = windowManager.isWindowTypeOpen('estimate-demo')

// 获取指定类型的窗口列表
const estimateWindows = windowManager.getWindowsByType('estimate-demo')

// 获取窗口数量
const count = windowManager.windowCount.value

// 检查是否有打开的窗口
const hasOpen = windowManager.hasOpenWindows.value
```

## 不同使用场景

### 主应用（工作台）中使用

```javascript
import { useMainAppWindowManager } from '@cost-app/shared-components'

export default {
  setup() {
    const windowManager = useMainAppWindowManager()
    return { windowManager }
  }
}
```

### 子应用中使用

```javascript
import { useChildAppWindowManager } from '@cost-app/shared-components'

export default {
  setup() {
    const windowManager = useChildAppWindowManager()
    return { windowManager }
  }
}
```

### 全局单例模式

```javascript
import { useGlobalWindowManager } from '@cost-app/shared-components'

// 在应用入口处初始化
const globalWindowManager = useGlobalWindowManager()
await globalWindowManager.init()

// 在其他地方使用
const windowManager = useGlobalWindowManager() // 返回同一个实例
```

## API 参考

### 状态属性

- `initialized` (Ref<boolean>): 是否已初始化
- `windowCount` (ComputedRef<number>): 当前打开的窗口数量
- `hasOpenWindows` (ComputedRef<boolean>): 是否有打开的窗口
- `openWindowList` (ComputedRef<Array>): 打开的窗口列表

### 核心方法

#### `init()`
初始化窗体管理器。

#### `openWindow(type, customOptions?)`
打开预定义类型的窗口。

**参数:**
- `type` (string): 窗口类型
- `customOptions` (Object, 可选): 自定义配置

#### `openCustomWindow(options)`
打开自定义窗口。

#### `closeWindow(windowId)`
关闭指定窗口。

#### `closeAllWindows()`
关闭所有窗口。

### 查询方法

#### `getWindowInfo(windowId)`
获取指定窗口的信息。

#### `isWindowOpen(windowId)`
检查指定窗口是否打开。

#### `getWindowsByType(type)`
获取指定类型的所有窗口。

#### `isWindowTypeOpen(type)`
检查指定类型的窗口是否已打开。

### 模板管理

#### `addWindowTemplate(type, template)`
添加窗口模板。

#### `addWindowTemplates(templates)`
批量添加窗口模板。

## 预定义窗口类型

### 工作台窗口
- `estimate-demo`: 概算演示
- `table-test`: 表格测试  
- `data-viewer`: 数据查看器
- `form-page`: 表单页面
- `settings`: 系统设置（模态）
- `window-manager-demo`: 窗体管理演示

### 子应用窗口
- `rough-estimate-main`: 概算管理
- `main-shell-dashboard`: 主工作台

## 自定义窗口模板

可以添加自己的窗口模板：

```javascript
// 添加单个模板
windowManager.addWindowTemplate('my-module', {
  title: '我的模块',
  url: '#/my-module',
  width: 1000,
  height: 700,
  modal: false
})

// 批量添加模板
windowManager.addWindowTemplates({
  'report-viewer': {
    title: '报表查看器',
    url: '#/reports',
    width: 1200,
    height: 800,
    modal: false
  },
  'user-settings': {
    title: '用户设置',
    url: '#/user-settings',
    width: 600,
    height: 500,
    modal: true
  }
})

// 使用自定义模板
await windowManager.openWindow('my-module')
await windowManager.openWindow('report-viewer')
```

## 完整使用示例

```vue
<template>
  <div class="window-control-demo">
    <h3>窗口管理演示</h3>
    <div class="window-stats">
      <p>当前打开窗口数: {{ windowManager.windowCount.value }}</p>
      <p>是否有打开窗口: {{ windowManager.hasOpenWindows.value ? '是' : '否' }}</p>
    </div>
    
    <div class="button-group">
      <a-button @click="openEstimate">打开概算演示</a-button>
      <a-button @click="openTable">打开表格测试</a-button>
      <a-button @click="openSettings">打开设置</a-button>
      <a-button @click="openSubApp">打开概算子应用</a-button>
      <a-button @click="closeAll" danger>关闭所有窗口</a-button>
    </div>

    <div class="window-list">
      <h4>打开的窗口列表：</h4>
      <ul>
        <li v-for="window in windowManager.openWindowList.value" :key="window.id">
          {{ window.title }} ({{ window.type }})
          <a-button size="small" @click="closeWindow(window.id)">关闭</a-button>
        </li>
      </ul>
    </div>
  </div>
</template>

<script>
import { useUniversalWindowManager } from '@cost-app/shared-components'

export default {
  name: 'WindowControlDemo',
  setup() {
    const windowManager = useUniversalWindowManager()
    
    const openEstimate = async () => {
      try {
        await windowManager.quick.estimateDemo({
          width: 1400,
          height: 900
        })
      } catch (error) {
        console.error('打开概算演示失败:', error)
      }
    }
    
    const openTable = async () => {
      try {
        await windowManager.quick.tableTest()
      } catch (error) {
        console.error('打开表格测试失败:', error)
      }
    }
    
    const openSettings = async () => {
      try {
        await windowManager.quick.settings()
      } catch (error) {
        console.error('打开设置失败:', error)
      }
    }
    
    const openSubApp = async () => {
      try {
        await windowManager.quick.roughEstimate()
      } catch (error) {
        console.error('打开概算子应用失败:', error)
      }
    }
    
    const closeAll = async () => {
      try {
        await windowManager.quick.closeAll()
      } catch (error) {
        console.error('关闭窗口失败:', error)
      }
    }
    
    const closeWindow = async (windowId) => {
      try {
        await windowManager.closeWindow(windowId)
      } catch (error) {
        console.error('关闭窗口失败:', error)
      }
    }
    
    return {
      windowManager,
      openEstimate,
      openTable,
      openSettings,
      openSubApp,
      closeAll,
      closeWindow
    }
  }
}
</script>

<style scoped>
.window-control-demo {
  padding: 20px;
}

.window-stats {
  background: #f5f5f5;
  padding: 10px;
  margin: 10px 0;
  border-radius: 4px;
}

.button-group {
  margin: 20px 0;
}

.button-group button {
  margin-right: 10px;
  margin-bottom: 10px;
}

.window-list ul {
  list-style-type: none;
  padding: 0;
}

.window-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  border: 1px solid #d9d9d9;
  margin-bottom: 5px;
  border-radius: 4px;
}
</style>
```

## 注意事项

1. **自动初始化**: 默认开启，可通过 `autoInitialize: false` 关闭
2. **自动检测**: 默认自动检测是否为子窗口，可通过 `autoDetect: false` 关闭
3. **全局单例**: 使用 `useGlobalWindowManager` 可以在应用中共享状态
4. **错误处理**: 所有异步方法都会抛出错误，建议使用 try-catch
5. **资源清理**: 组件销毁时会自动清理资源
# 🚀 窗口创建和数据交互完整指南

## 🔧 已修复的问题

### 1. 窗口创建问题
**修复内容**:
- ✅ 临时启用系统标题栏进行调试 (`decorations: true`)
- ✅ 添加强制显示和聚焦逻辑
- ✅ 增强错误处理和日志记录
- ✅ 添加窗口状态检查

### 2. 数据交互系统
**新增功能**:
- ✅ 完整的数据交互 API
- ✅ 模块间数据传递
- ✅ 共享数据管理
- ✅ 事件监听和广播

## 🏗️ 数据交互架构

### 数据流向图
```
主应用 (Dashboard)
    ↓ 发送初始数据
概算模块 ← → 预算模块
    ↓ 共享数据
结算模块 ← → 审核模块
    ↑
共享数据存储 (Tauri 后端)
```

### 数据交互 API

#### 1. 发送数据到指定模块
```javascript
import { DataExchange } from '@/utils/dataExchange.js'

// 发送数据到概算模块
await DataExchange.sendEstimateData({
  projectId: 'proj_001',
  estimateItems: [...],
  totalAmount: 5000000
})
```

#### 2. 广播数据到所有模块
```javascript
// 通知所有模块数据变更
await DataExchange.notifyDataChange('project_update', {
  projectId: 'proj_001',
  status: 'updated',
  timestamp: Date.now()
})
```

#### 3. 共享数据管理
```javascript
// 设置项目信息
await DataExchange.setProjectInfo({
  id: 'proj_001',
  name: '办公楼建设项目',
  budget: 5000000
})

// 获取项目信息
const projectInfo = await DataExchange.getProjectInfo()
```

#### 4. 监听数据变化
```javascript
// 监听数据变化
DataExchange.onDataChange((data) => {
  console.log('数据变更:', data)
  // 更新 UI
})

// 监听共享数据变化
DataExchange.onSharedDataChange((data) => {
  console.log('共享数据变更:', data.key, data.data)
})
```

## 🎯 使用场景示例

### 场景1: 从主应用打开概算模块并传递项目数据
```javascript
// 在 Dashboard.vue 中
const openModule = async (module) => {
  // 1. 创建窗口
  await invoke('create_module_window', {
    moduleKey: module.key,
    title: module.title,
    url: `http://localhost:${module.port}`
  })
  
  // 2. 发送初始数据
  await DataExchange.sendToModule(module.key, {
    type: 'module_init',
    projectInfo: await DataExchange.getProjectInfo(),
    userInfo: await DataExchange.getUserInfo()
  })
}
```

### 场景2: 概算模块接收和处理数据
```javascript
// 在概算模块中
const handleDataReceived = (data) => {
  switch (data.type) {
    case 'module_init':
      // 初始化模块数据
      projectInfo.value = data.projectInfo
      userInfo.value = data.userInfo
      break
      
    case 'estimate_data':
      // 更新概算数据
      updateEstimateList(data.data)
      break
  }
}
```

### 场景3: 模块间数据同步
```javascript
// 概算模块更新数据后通知其他模块
const updateEstimate = async (estimateData) => {
  // 1. 更新本地数据
  estimateList.value = estimateData
  
  // 2. 广播数据变更
  await DataExchange.notifyDataChange('estimate_update', {
    moduleKey: 'rough-estimate',
    data: estimateData,
    timestamp: Date.now()
  })
}
```

## 🔍 调试和测试

### 1. 检查窗口创建
查看 Tauri 控制台输出：
```
创建模块窗口: rough-estimate - 概算管理系统 - URL: http://localhost:5174
URL解析成功: Url { ... }
开始创建窗口，标签: module-rough-estimate
模块窗口创建成功: module-rough-estimate
窗口显示和聚焦完成
```

### 2. 检查数据传递
查看浏览器控制台：
```javascript
// 主应用日志
"初始数据已发送到概算模块"

// 概算模块日志
"概算模块收到数据: { type: 'module_init', ... }"
"模块初始化完成"
```

### 3. 测试数据交互
```javascript
// 在浏览器控制台中测试
// 发送测试数据
await window.__TAURI__.core.invoke('send_data_to_module', {
  moduleKey: 'rough-estimate',
  data: { type: 'test', message: 'Hello from console!' }
})
```

## 📋 启动步骤

### 1. 启动概算模块
```bash
cd packages/rough-estimate
npm run dev
# 等待启动完成: http://localhost:5174/
```

### 2. 启动主应用
```bash
npm run tauri:dev
# 等待编译完成并启动 Tauri 应用
```

### 3. 测试功能
1. 点击概算模块卡片
2. 观察是否出现新窗口（现在有系统标题栏）
3. 检查控制台日志确认数据传递

## 🎨 界面特性

### 主应用
- 紫色渐变标题栏
- 6个业务模块卡片
- 系统状态显示
- 自定义窗口控制

### 概算模块
- 蓝色渐变标题栏
- 完整的概算管理界面
- 数据表格和操作按钮
- 自定义窗口控制

## 🔄 数据流程

### 模块启动流程
1. 用户点击模块卡片
2. 主应用创建新窗口
3. 窗口加载完成后发送初始数据
4. 模块接收数据并初始化界面

### 数据同步流程
1. 模块A更新数据
2. 调用数据交互API广播变更
3. 其他模块接收变更通知
4. 更新各自的界面显示

## 🚨 故障排除

### 问题1: 窗口不出现
**解决方案**:
- 检查概算模块是否在 5174 端口运行
- 查看 Tauri 控制台错误信息
- 确认 URL 可以在浏览器中访问

### 问题2: 数据传递失败
**解决方案**:
- 检查事件监听器是否正确设置
- 确认模块窗口已完全加载
- 查看控制台错误信息

### 问题3: 窗口控制不工作
**解决方案**:
- 确认 Tauri API 正确初始化
- 检查窗口对象是否获取成功
- 查看权限配置是否正确

## 🎉 预期结果

成功运行后你应该看到：
1. ✅ 主应用正常启动，显示工作台界面
2. ✅ 点击概算卡片后出现新窗口（带系统标题栏）
3. ✅ 概算模块正常显示，接收到初始数据
4. ✅ 控制台显示数据传递日志
5. ✅ 窗口控制功能正常工作

这个系统现在具备了完整的微前端架构和数据交互能力！🚀

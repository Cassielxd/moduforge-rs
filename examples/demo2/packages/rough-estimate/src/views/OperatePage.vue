<template>
  <div class="operate-page">
    <!-- 加载骨架屏 - 直接显示骨架屏，不要外层的 spin -->
    <div v-if="loading" class="loading-container">
      <div class="loading-skeleton">
        <a-skeleton active :paragraph="{ rows: 8 }" />
      </div>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="error-container">
      <a-result
        status="error"
        :title="error.title || '加载失败'"
        :sub-title="error.message || '无法加载操作组件'"
      >
        <template #extra>
          <a-button type="primary" @click="retryLoad">重新加载</a-button>
          <a-button @click="closeWindow">关闭窗口</a-button>
        </template>
      </a-result>
    </div>

    <!-- 动态组件 -->
    <component
      v-else-if="currentComponent"
      :is="currentComponent"
      v-bind="componentProps"
      :operate-type="operateType"
      :operate-label="operateLabel"
      :data="data"
      :table-data="tableData"
      :parent-window="parentWindow"
      :app-id="appId"
      @submit="handleSubmit"
      @cancel="handleCancel"
      @close="handleClose"
      @update="handleUpdate"
    />

    <!-- 默认内容 -->
    <div v-else class="default-content">
      <a-empty description="未找到对应的操作组件" />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, defineAsyncComponent } from 'vue'
import { useRoute } from 'vue-router'
import { message } from 'ant-design-vue'
import { invoke } from '@tauri-apps/api/core'

// 路由参数
const route = useRoute()

// 响应式数据
const loading = ref(true)
const error = ref(null)
const currentComponent = ref(null)

// 从URL参数获取配置
const operateType = computed(() => route.query.operate || route.params.operate)
const operateLabel = computed(() => route.query.operateLabel || operateType.value)
const componentName = computed(() => route.query.component || operateType.value)
const parentWindow = computed(() => route.query.parentWindow)
const appId = computed(() => route.query.appId)

// 数据参数
const data = computed(() => {
  try {
    return route.query.data ? JSON.parse(route.query.data) : null
  } catch (e) {
    console.warn('解析data参数失败:', e)
    return null
  }
})

const tableData = computed(() => {
  try {
    return route.query.tableData ? JSON.parse(route.query.tableData) : []
  } catch (e) {
    console.warn('解析tableData参数失败:', e)
    return []
  }
})

// 组件属性
const componentProps = computed(() => ({
  selectedCount: parseInt(route.query.selectedCount) || 0,
  totalCount: parseInt(route.query.totalCount) || 0,
  hasSelection: route.query.hasSelection === 'true'
}))

// 组件映射 - 从公共组件库导入
const componentMap = {
  // 表单窗口
  'FormWindow': () => import('@cost-app/shared-components').then(m => m.FormWindow),
  'form-window': () => import('@cost-app/shared-components').then(m => m.FormWindow),

  // 数据导入
  'import-data': () => import('@cost-app/shared-components').then(m => m.DataImportWindow),
  'DataImportWindow': () => import('@cost-app/shared-components').then(m => m.DataImportWindow),

  // 数据导出
  'export-table': () => import('@cost-app/shared-components').then(m => m.DataExportWindow),
  'export-data': () => import('@cost-app/shared-components').then(m => m.DataExportWindow),
  'DataExportWindow': () => import('@cost-app/shared-components').then(m => m.DataExportWindow),

  // 批量操作
  'batch-operation': () => import('@cost-app/shared-components').then(m => m.BatchOperationWindow),
  'BatchOperationWindow': () => import('@cost-app/shared-components').then(m => m.BatchOperationWindow),

  // 系统设置 - 暂时使用导入组件作为示例
  'system-settings': () => import('@cost-app/shared-components').then(m => m.DataImportWindow),
  'SystemSettingsWindow': () => import('@cost-app/shared-components').then(m => m.DataImportWindow),

  // 数据分析 - 暂时使用导出组件作为示例
  'data-analysis': () => import('@cost-app/shared-components').then(m => m.DataExportWindow),
  'DataAnalysisWindow': () => import('@cost-app/shared-components').then(m => m.DataExportWindow),

  // 模板管理 - 暂时使用导入组件作为示例
  'template-manage': () => import('@cost-app/shared-components').then(m => m.DataImportWindow),
  'TemplateManageWindow': () => import('@cost-app/shared-components').then(m => m.DataImportWindow),

  // 计税设置 - 暂时使用导入组件作为示例
  'tax-settings': () => import('@cost-app/shared-components').then(m => m.DataImportWindow),
  'TaxSettingsWindow': () => import('@cost-app/shared-components').then(m => m.DataImportWindow)
}

// 加载组件
const loadComponent = async () => {
  try {
    loading.value = true
    error.value = null

    const componentKey = componentName.value || operateType.value
    const componentLoader = componentMap[componentKey]

    if (!componentLoader) {
      throw new Error(`未找到组件: ${componentKey}`)
    }

    console.log('加载操作组件:', {
      operateType: operateType.value,
      componentName: componentName.value,
      componentKey
    })

    // 模拟加载延迟 - 减少到200ms，让骨架屏显示时间更短
    await new Promise(resolve => setTimeout(resolve, 200))

    // 动态加载组件
    const component = defineAsyncComponent({
      loader: componentLoader,
      loadingComponent: () => null,
      errorComponent: () => null,
      delay: 200,
      timeout: 10000
    })

    currentComponent.value = component
    
    console.log('组件加载成功:', componentKey)
  } catch (err) {
    console.error('加载组件失败:', err)
    error.value = {
      title: '组件加载失败',
      message: err.message || '未知错误'
    }
  } finally {
    loading.value = false
  }
}

// 重新加载
const retryLoad = () => {
  loadComponent()
}

// 事件处理
const handleSubmit = (data) => {
  console.log('操作提交:', data)
  // 向父窗口发送消息
  if (parentWindow.value) {
    invoke('send_window_message', {
      targetWindow: parentWindow.value,
      message: {
        type: 'operate-submit',
        operate: operateType.value,
        data
      }
    }).catch(console.error)
  }
  message.success('操作完成')
  closeWindow()
}

const handleCancel = () => {
  console.log('操作取消')
  closeWindow()
}

const handleClose = () => {
  console.log('操作关闭')
  closeWindow()
}

const handleUpdate = (data) => {
  console.log('操作更新:', data)
  // 向父窗口发送更新消息
  if (parentWindow.value) {
    invoke('send_window_message', {
      targetWindow: parentWindow.value,
      message: {
        type: 'operate-update',
        operate: operateType.value,
        data
      }
    }).catch(console.error)
  }
}

// 关闭窗口
const closeWindow = async () => {
  try {
    await invoke('close_current_window')
  } catch (error) {
    console.error('关闭窗口失败:', error)
    // 如果是开发环境，可能没有Tauri API，直接关闭
    if (window.close) {
      window.close()
    }
  }
}

// 组件挂载时加载
onMounted(() => {
  console.log('OperatePage 挂载:', {
    route: route.fullPath,
    query: route.query,
    params: route.params
  })
  
  loadComponent()
})
</script>

<style scoped>
.operate-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.loading-container {
  height: 100vh;
  background: #fff;
}

.loading-skeleton {
  padding: 24px;
  height: 100%;
}

.error-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  min-height: 400px;
}

.default-content {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  min-height: 400px;
}
</style>

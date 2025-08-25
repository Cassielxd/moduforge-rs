<template>
  <div class="form-page">
    <!-- 使用简单的表单窗口组件进行测试 -->
    <SimpleFormWindow
      :mode="formMode"
      :form-type="formType"
      :is-modal="isModal"
      :parent-window="parentWindow"
      :app-id="appId"
      :app-port="appPort"
      :initial-data="initialData"
    />
  </div>
</template>
<script setup>
import { ref, computed, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { SimpleFormWindow } from '@cost-app/shared-components'

// 从URL参数获取表单配置
const urlParams = new URLSearchParams(window.location.search)
const formMode = ref(urlParams.get('mode') || 'create')
const formType = ref(urlParams.get('formType') || 'estimate')
const isModal = ref(urlParams.get('modal') === 'true')
const parentWindow = ref(urlParams.get('parentWindow') || '')
const appId = ref(urlParams.get('appId') || '')
const appPort = ref(urlParams.get('appPort') || '')
const initialData = ref({})

// 解析初始数据
try {
  const dataParam = urlParams.get('data')
  if (dataParam) {
    initialData.value = JSON.parse(dataParam)
  }
} catch (error) {
  console.error('解析初始数据失败:', error)
}

// FormWindow 组件会处理所有表单逻辑

onMounted(() => {
  console.log('表单页面已加载', {
    mode: formMode.value,
    type: formType.value,
    isModal: isModal.value,
    parentWindow: parentWindow.value,
    appId: appId.value,
    appPort: appPort.value,
    data: initialData.value,
    currentUrl: window.location.href
  })
  
  // 如果有应用标识，可以根据不同应用加载不同的样式或组件
  if (appId.value) {
    console.log(`表单窗口属于应用: ${appId.value} (端口: ${appPort.value})`)
  }
})
</script>

<style scoped>
.form-page {
  background: #fff;
  min-height: 100vh;
}
</style>

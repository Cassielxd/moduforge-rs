<template>
  <div class="form-page">
    <div v-if="!componentLoaded" class="loading">
      <h2>正在加载表单组件...</h2>
      <p>Debug Info:</p>
      <ul>
        <li>Mode: {{ mode }}</li>
        <li>Form Type: {{ formType }}</li>
        <li>Modal: {{ isModal }}</li>
        <li>Parent: {{ parentWindow }}</li>
        <li>App ID: {{ appId }}</li>
        <li>App Port: {{ appPort }}</li>
        <li>Current URL: {{ currentURL }}</li>
      </ul>
    </div>
    
    <SimpleFormWindow 
      v-if="componentLoaded"
      :mode="mode"
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
import { ref, onMounted } from 'vue'
import { SimpleFormWindow } from '@cost-app/shared-components'

// 组件加载状态
const componentLoaded = ref(false)

// 从URL参数获取配置
const urlParams = new URLSearchParams(window.location.search)
const mode = ref(urlParams.get('mode') || 'create')
const formType = ref(urlParams.get('formType') || 'estimate')
const isModal = ref(urlParams.get('modal') === 'true')
const parentWindow = ref(urlParams.get('parentWindow') || '')
const appId = ref(urlParams.get('appId') || 'rough-estimate')
const appPort = ref(urlParams.get('appPort') || '5176')
const initialData = ref(null)
const currentURL = ref(window.location.href)

// 解析初始数据
const dataParam = urlParams.get('data')
if (dataParam) {
  try {
    initialData.value = JSON.parse(dataParam)
  } catch (error) {
    console.error('解析初始数据失败:', error)
    initialData.value = {}
  }
}

onMounted(() => {
  console.log('FormPage mounted with params:', {
    mode: mode.value,
    formType: formType.value,
    isModal: isModal.value,
    parentWindow: parentWindow.value,
    appId: appId.value,
    appPort: appPort.value,
    initialData: initialData.value,
    currentURL: currentURL.value
  })
  
  // 延迟加载组件
  setTimeout(() => {
    componentLoaded.value = true
    console.log('SimpleFormWindow component loaded')
  }, 1000)
})
</script>

<style scoped>
.form-page {
  width: 100%;
  height: 100vh;
  background: white;
}

.loading {
  padding: 20px;
  background: #f0f8ff;
  border: 2px solid #4169e1;
  margin: 20px;
  border-radius: 8px;
}

.loading h2 {
  color: #4169e1;
  margin-bottom: 16px;
}

.loading p {
  margin: 8px 0;
  font-weight: bold;
}

.loading ul {
  margin: 8px 0;
  padding-left: 20px;
}

.loading li {
  margin: 4px 0;
  color: #333;
}
</style>
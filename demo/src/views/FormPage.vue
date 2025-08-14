<template>
  <div class="form-page">
    <!-- 使用共享的表单组件 -->
    <CostForm
      :mode="formMode"
      :initial-data="initialData"
      :form-type="formType"
      @submit="handleFormSubmit"
      @cancel="handleFormCancel"
      @save-draft="handleSaveDraft"
    />
  </div>
</template>
<script setup>
import { ref, computed, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { CostForm } from '@cost-app/shared-components'

// 从URL参数获取表单配置
const urlParams = new URLSearchParams(window.location.search)
const formMode = ref(urlParams.get('mode') || 'create')
const formType = ref(urlParams.get('formType') || 'estimate')
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

// 表单事件处理
const handleFormSubmit = async (formData) => {
  try {
    console.log('提交表单数据:', formData)

    // 这里可以调用API保存数据
    // await api.saveEstimate(formData)

    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 1000))

    message.success('数据保存成功！')

    // 可以通过事件通知父窗口数据已更新
    // window.parent?.postMessage({ type: 'form-submitted', data: formData }, '*')

    // 延迟关闭窗口
    setTimeout(() => {
      closeWindow()
    }, 1500)

  } catch (error) {
    console.error('保存数据失败:', error)
    message.error('保存失败，请重试')
  }
}

const handleSaveDraft = async (formData) => {
  try {
    console.log('保存草稿:', formData)

    // 这里可以调用API保存草稿
    // await api.saveDraft(formData)

    // 模拟API调用
    await new Promise(resolve => setTimeout(resolve, 500))

    message.success('草稿保存成功！')

  } catch (error) {
    console.error('保存草稿失败:', error)
    message.error('保存草稿失败')
  }
}

const handleFormCancel = () => {
  closeWindow()
}

const closeWindow = async () => {
  try {
    const currentWindow = getCurrentWebviewWindow()
    await currentWindow.close()
  } catch (error) {
    console.error('关闭窗口失败:', error)
  }
}

onMounted(() => {
  console.log('表单页面已加载', {
    mode: formMode.value,
    type: formType.value,
    data: initialData.value
  })

  // 监听窗口事件
  const currentWindow = getCurrentWebviewWindow()

  currentWindow.listen('window-disabled', () => {
    console.log('窗口被禁用')
  })

  currentWindow.listen('window-enabled', () => {
    console.log('窗口被启用')
  })
})
</script>

<style scoped>
.form-page {
  background: #fff;
  min-height: 100vh;
}
</style>

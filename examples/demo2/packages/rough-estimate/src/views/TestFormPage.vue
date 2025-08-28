<template>
  <div class="test-form-page">
    <h1>表单页面测试</h1>
    <div class="test-info">
      <h2>当前信息</h2>
      <p>URL: {{ currentURL }}</p>
      <p>端口: {{ currentPort }}</p>
      <p>时间: {{ currentTime }}</p>
    </div>
    
    <div class="test-section">
      <h2>简单表单测试</h2>
      <div class="simple-form">
        <div class="form-group">
          <label>项目名称:</label>
          <input v-model="testData.name" placeholder="输入项目名称">
        </div>
        <div class="form-group">
          <label>金额:</label>
          <input v-model="testData.amount" type="number" placeholder="输入金额">
        </div>
        <div class="form-actions">
          <button @click="handleSave">保存</button>
          <button @click="handleCancel">取消</button>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
const currentURL = ref('')
const currentPort = ref('')
const currentTime = ref('')
const testData = ref({
  name: '测试项目',
  amount: 1000000
})

const handleSave = () => {
  console.log('保存数据:', testData.value)
  alert('数据已保存: ' + JSON.stringify(testData.value))
}

const handleCancel = () => {
  console.log('取消操作')
  alert('已取消')
}

onMounted(() => {
  currentURL.value = window.location.href
  currentPort.value = window.location.port || '未知'
  currentTime.value = new Date().toLocaleString()
  
  console.log('TestFormPage mounted:', {
    url: currentURL.value,
    port: currentPort.value,
    time: currentTime.value
  })
})
</script>

<style scoped>
.test-form-page {
  padding: 20px;
  background: white;
  min-height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.test-form-page h1 {
  color: #333;
  border-bottom: 2px solid #4169e1;
  padding-bottom: 10px;
  margin-bottom: 20px;
}

.test-info, .test-section {
  margin-bottom: 30px;
  padding: 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background: #f9f9f9;
}

.test-info h2, .test-section h2 {
  margin: 0 0 16px 0;
  color: #4169e1;
  font-size: 18px;
}

.test-info p {
  margin: 8px 0;
  color: #666;
}

.simple-form {
  background: white;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 4px;
  color: #495057;
  font-size: 14px;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  max-width: 300px;
  padding: 8px 12px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  font-size: 14px;
}

.form-group input:focus {
  outline: none;
  border-color: #4169e1;
  box-shadow: 0 0 0 2px rgba(65, 105, 225, 0.2);
}

.form-actions {
  display: flex;
  gap: 8px;
  margin-top: 20px;
}

.form-actions button {
  padding: 8px 16px;
  border: 1px solid #ced4da;
  border-radius: 4px;
  background: white;
  color: #495057;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.form-actions button:first-child {
  background: #4169e1;
  color: white;
  border-color: #4169e1;
}

.form-actions button:hover {
  opacity: 0.8;
}
</style>
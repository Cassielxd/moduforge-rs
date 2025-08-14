<template>
  <div class="settings-page">
    <div class="settings-header">
      <h2>⚙️ 系统设置</h2>
      <p>配置应用程序的各项参数</p>
    </div>

    <a-tabs v-model:activeKey="activeTab" type="card">
      <a-tab-pane key="general" tab="常规设置">
        <a-form layout="vertical">
          <a-form-item label="应用主题">
            <a-radio-group v-model:value="settings.theme">
              <a-radio value="light">浅色主题</a-radio>
              <a-radio value="dark">深色主题</a-radio>
              <a-radio value="auto">跟随系统</a-radio>
            </a-radio-group>
          </a-form-item>

          <a-form-item label="语言设置">
            <a-select v-model:value="settings.language" style="width: 200px">
              <a-select-option value="zh-CN">简体中文</a-select-option>
              <a-select-option value="en-US">English</a-select-option>
              <a-select-option value="ja-JP">日本語</a-select-option>
            </a-select>
          </a-form-item>

          <a-form-item label="自动保存">
            <a-switch v-model:checked="settings.autoSave" />
            <span style="margin-left: 8px;">启用自动保存功能</span>
          </a-form-item>

          <a-form-item label="保存间隔">
            <a-slider
              v-model:value="settings.saveInterval"
              :min="1"
              :max="10"
              :marks="{ 1: '1分钟', 5: '5分钟', 10: '10分钟' }"
              :disabled="!settings.autoSave"
            />
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="calculation" tab="计算设置">
        <a-form layout="vertical">
          <a-form-item label="默认货币">
            <a-select v-model:value="settings.currency" style="width: 200px">
              <a-select-option value="CNY">人民币 (¥)</a-select-option>
              <a-select-option value="USD">美元 ($)</a-select-option>
              <a-select-option value="EUR">欧元 (€)</a-select-option>
            </a-select>
          </a-form-item>

          <a-form-item label="小数位数">
            <a-input-number
              v-model:value="settings.decimalPlaces"
              :min="0"
              :max="6"
              style="width: 200px"
            />
          </a-form-item>

          <a-form-item label="税率设置">
            <a-input-number
              v-model:value="settings.taxRate"
              :min="0"
              :max="100"
              :step="0.1"
              addon-after="%"
              style="width: 200px"
            />
          </a-form-item>

          <a-form-item label="计算精度">
            <a-radio-group v-model:value="settings.precision">
              <a-radio value="normal">普通精度</a-radio>
              <a-radio value="high">高精度</a-radio>
              <a-radio value="financial">财务精度</a-radio>
            </a-radio-group>
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="export" tab="导出设置">
        <a-form layout="vertical">
          <a-form-item label="默认导出格式">
            <a-checkbox-group v-model:value="settings.exportFormats">
              <a-checkbox value="excel">Excel (.xlsx)</a-checkbox>
              <a-checkbox value="pdf">PDF (.pdf)</a-checkbox>
              <a-checkbox value="csv">CSV (.csv)</a-checkbox>
              <a-checkbox value="json">JSON (.json)</a-checkbox>
            </a-checkbox-group>
          </a-form-item>

          <a-form-item label="导出路径">
            <a-input
              v-model:value="settings.exportPath"
              placeholder="选择导出文件夹"
              readonly
            >
              <template #suffix>
                <a-button type="link" @click="selectExportPath">
                  选择文件夹
                </a-button>
              </template>
            </a-input>
          </a-form-item>

          <a-form-item label="文件命名规则">
            <a-input
              v-model:value="settings.fileNamePattern"
              placeholder="例如：项目名称_日期"
            />
            <div style="margin-top: 4px; font-size: 12px; color: #666;">
              可用变量：{project} - 项目名称，{date} - 日期，{time} - 时间
            </div>
          </a-form-item>
        </a-form>
      </a-tab-pane>

      <a-tab-pane key="advanced" tab="高级设置">
        <a-form layout="vertical">
          <a-form-item label="调试模式">
            <a-switch v-model:checked="settings.debugMode" />
            <span style="margin-left: 8px;">启用调试信息输出</span>
          </a-form-item>

          <a-form-item label="性能监控">
            <a-switch v-model:checked="settings.performanceMonitor" />
            <span style="margin-left: 8px;">监控应用性能</span>
          </a-form-item>

          <a-form-item label="缓存大小">
            <a-slider
              v-model:value="settings.cacheSize"
              :min="50"
              :max="500"
              :marks="{ 50: '50MB', 250: '250MB', 500: '500MB' }"
            />
          </a-form-item>

          <a-form-item label="数据备份">
            <a-radio-group v-model:value="settings.backupFrequency">
              <a-radio value="daily">每日备份</a-radio>
              <a-radio value="weekly">每周备份</a-radio>
              <a-radio value="monthly">每月备份</a-radio>
              <a-radio value="manual">手动备份</a-radio>
            </a-radio-group>
          </a-form-item>
        </a-form>
      </a-tab-pane>
    </a-tabs>

    <div class="settings-actions">
      <a-button @click="resetSettings">重置为默认</a-button>
      <a-button @click="closeWindow">取消</a-button>
      <a-button type="primary" @click="saveSettings" :loading="saving">
        保存设置
      </a-button>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

const activeTab = ref('general')
const saving = ref(false)

const settings = reactive({
  // 常规设置
  theme: 'light',
  language: 'zh-CN',
  autoSave: true,
  saveInterval: 5,
  
  // 计算设置
  currency: 'CNY',
  decimalPlaces: 2,
  taxRate: 13.0,
  precision: 'normal',
  
  // 导出设置
  exportFormats: ['excel', 'pdf'],
  exportPath: '',
  fileNamePattern: '{project}_{date}',
  
  // 高级设置
  debugMode: false,
  performanceMonitor: false,
  cacheSize: 250,
  backupFrequency: 'weekly'
})

const defaultSettings = { ...settings }

const selectExportPath = async () => {
  try {
    // 这里可以调用 Tauri 的文件选择对话框
    message.info('文件夹选择功能需要集成 Tauri 文件对话框')
  } catch (error) {
    console.error('选择文件夹失败:', error)
  }
}

const saveSettings = async () => {
  try {
    saving.value = true
    
    // 这里可以调用后端API保存设置
    console.log('保存设置:', settings)
    
    // 模拟保存过程
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    message.success('设置保存成功！')
    
    // 保存成功后可以选择关闭窗口
    setTimeout(() => {
      closeWindow()
    }, 1000)
    
  } catch (error) {
    console.error('保存设置失败:', error)
    message.error('保存设置失败，请重试')
  } finally {
    saving.value = false
  }
}

const resetSettings = () => {
  Object.assign(settings, defaultSettings)
  message.success('设置已重置为默认值')
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
  // 加载保存的设置
  console.log('加载设置页面')
})
</script>

<style scoped>
.settings-page {
  padding: 24px;
  background: #fff;
  min-height: 100vh;
}

.settings-header {
  text-align: center;
  margin-bottom: 32px;
  padding-bottom: 16px;
  border-bottom: 1px solid #f0f0f0;
}

.settings-header h2 {
  margin-bottom: 8px;
  color: #1890ff;
}

.settings-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 32px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}
</style>

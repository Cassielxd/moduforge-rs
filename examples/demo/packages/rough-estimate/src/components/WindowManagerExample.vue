<template>
  <div class="window-manager-example">
    <a-card title="子应用窗体管理演示" style="margin: 20px 0;">
      <div class="info-section">
        <p><strong>当前应用:</strong> 概算管理子应用</p>
        <p><strong>打开窗口数:</strong> {{ windowManager.windowCount.value }}</p>
        <p><strong>是否有打开窗口:</strong> {{ windowManager.hasOpenWindows.value ? '是' : '否' }}</p>
      </div>

      <a-divider />

      <div class="button-section">
        <h4>工作台窗口操作</h4>
        <a-space wrap>
          <a-button type="primary" @click="openEstimateDemo">
            打开数据查看器
          </a-button>
          <a-button @click="openTableTest">
            打开表格测试
          </a-button>
          <a-button @click="openDataViewer">
            打开数据查看器
          </a-button>
          <a-button @click="openSettings">
            打开设置（模态）
          </a-button>
        </a-space>

        <h4 style="margin-top: 20px;">子应用窗口操作</h4>
        <a-space wrap>
          <a-button type="primary" @click="openMainShell">
            打开主工作台
          </a-button>
          <a-button @click="openCustomWindow">
            打开自定义窗口
          </a-button>
        </a-space>

        <h4 style="margin-top: 20px;">窗口管理操作</h4>
        <a-space wrap>
          <a-button danger @click="closeAllWindows">
            关闭所有窗口
          </a-button>
          <a-button @click="showWindowList">
            显示窗口列表
          </a-button>
        </a-space>
      </div>

      <a-divider />

      <div v-if="windowManager.hasOpenWindows.value" class="window-list-section">
        <h4>当前打开的窗口</h4>
        <a-list 
          :data-source="windowManager.openWindowList.value" 
          size="small"
          :split="false"
        >
          <template #renderItem="{ item }">
            <a-list-item>
              <div class="window-item">
                <div class="window-info">
                  <strong>{{ item.title }}</strong>
                  <span class="window-type">{{ item.type }}</span>
                  <span class="window-time">{{ formatTime(item.openTime) }}</span>
                </div>
                <a-button 
                  size="small" 
                  danger 
                  @click="closeWindow(item.id)"
                >
                  关闭
                </a-button>
              </div>
            </a-list-item>
          </template>
        </a-list>
      </div>
    </a-card>
  </div>
</template>

<script>
import { useChildAppWindowManager } from '@cost-app/shared-components'
import { message } from 'ant-design-vue'

export default {
  name: 'WindowManagerExample',
  setup() {
    // 使用子应用专用的窗体管理器
    const windowManager = useChildAppWindowManager()

    // 打开工作台相关窗口
    const openEstimateDemo = async () => {
      try {
        await windowManager.quick.dataPage({
          width: 1400,
          height: 900
        })
        message.success('数据查看器窗口已打开')
      } catch (error) {
        message.error('打开数据查看器窗口失败')
        console.error(error)
      }
    }

    const openTableTest = async () => {
      try {
        await windowManager.quick.tableTest()
        message.success('表格测试窗口已打开')
      } catch (error) {
        message.error('打开表格测试窗口失败')
        console.error(error)
      }
    }

    const openDataViewer = async () => {
      try {
        await windowManager.quick.dataViewer()
        message.success('数据查看器窗口已打开')
      } catch (error) {
        message.error('打开数据查看器窗口失败')
        console.error(error)
      }
    }

    const openSettings = async () => {
      try {
        await windowManager.quick.settings()
        message.success('设置窗口已打开')
      } catch (error) {
        message.error('打开设置窗口失败')
        console.error(error)
      }
    }

    // 打开子应用相关窗口
    const openMainShell = async () => {
      try {
        await windowManager.quick.mainShell()
        message.success('主工作台窗口已打开')
      } catch (error) {
        message.error('打开主工作台窗口失败')
        console.error(error)
      }
    }

    const openCustomWindow = async () => {
      try {
        await windowManager.openCustomWindow({
          windowId: 'custom-from-estimate',
          title: '从概算应用打开的自定义窗口',
          url: '#/test-form',
          width: 800,
          height: 600,
          modal: false,
          type: 'custom-estimate'
        })
        message.success('自定义窗口已打开')
      } catch (error) {
        message.error('打开自定义窗口失败')
        console.error(error)
      }
    }

    // 窗口管理操作
    const closeAllWindows = async () => {
      try {
        await windowManager.closeAllWindows()
        message.success('所有窗口已关闭')
      } catch (error) {
        message.error('关闭窗口失败')
        console.error(error)
      }
    }

    const closeWindow = async (windowId) => {
      try {
        await windowManager.closeWindow(windowId)
        message.success('窗口已关闭')
      } catch (error) {
        message.error('关闭窗口失败')
        console.error(error)
      }
    }

    const showWindowList = () => {
      const windows = windowManager.openWindowList.value
      if (windows.length === 0) {
        message.info('当前没有打开的窗口')
      } else {
        const windowInfo = windows.map(w => `${w.title} (${w.type})`).join('\n')
        message.info(`当前打开的窗口:\n${windowInfo}`)
      }
    }

    // 时间格式化
    const formatTime = (time) => {
      if (!time) return ''
      return new Date(time).toLocaleTimeString()
    }

    return {
      windowManager,
      openEstimateDemo,
      openTableTest,
      openDataViewer,
      openSettings,
      openMainShell,
      openCustomWindow,
      closeAllWindows,
      closeWindow,
      showWindowList,
      formatTime
    }
  }
}
</script>

<style scoped>
.window-manager-example {
  padding: 20px;
}

.info-section {
  background: #f5f5f5;
  padding: 15px;
  border-radius: 6px;
  margin-bottom: 20px;
}

.info-section p {
  margin: 5px 0;
}

.button-section h4 {
  margin: 15px 0 10px 0;
  color: #1890ff;
}

.window-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  padding: 8px 0;
}

.window-info {
  display: flex;
  flex-direction: column;
  flex: 1;
}

.window-type {
  color: #666;
  font-size: 12px;
  margin-top: 2px;
}

.window-time {
  color: #999;
  font-size: 11px;
  margin-top: 2px;
}

.window-list-section {
  margin-top: 20px;
}

.window-list-section h4 {
  color: #1890ff;
  margin-bottom: 10px;
}
</style>
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Antd from 'ant-design-vue'
import 'ant-design-vue/dist/reset.css'

// 导入STable配置和样式
import { setupSTable } from '@cost-app/shared-components'
import '@surely-vue/table/dist/index.less'

// 导入共享组件样式 - 必须导入才能显示样式
import '@cost-app/shared-components/dist/shared-components.css'

import App from './App.vue'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(Antd)

// 注册STable
setupSTable(app)

app.mount('#app')

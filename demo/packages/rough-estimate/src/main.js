import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import Antd from 'ant-design-vue'
import 'ant-design-vue/dist/reset.css'

// 导入STable配置和样式
import { setupSTable } from '@cost-app/shared-components'
import '@surely-vue/table/dist/index.less'

// 导入共享组件样式 - 必须导入才能显示样式
import '@cost-app/shared-components/dist/shared-components.css'

import App from './App.vue'

// 创建路由 - 概算应用专用路由
const routes = [
  {
    path: '/',
    name: 'EstimateMain',
    component: () => import('./views/EstimateMain.vue'),
    meta: { title: '概算管理' }
  },
  {
    path: '/form-page',
    name: 'FormPage',
    component: () => import('./views/FormPage.vue'),
    meta: { title: '表单页面' }
  },
  {
    path: '/test-form',
    name: 'TestFormPage',
    component: () => import('./views/TestFormPage.vue'),
    meta: { title: '表单测试页面' }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫
router.beforeEach((to, from, next) => {
  if (to.meta.title) {
    document.title = `${to.meta.title} - 概算管理系统`
  }
  next()
})

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.use(Antd)

// 注册STable
setupSTable(app)

app.mount('#app')

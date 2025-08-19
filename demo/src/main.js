import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import Antd from 'ant-design-vue'
import 'ant-design-vue/dist/reset.css'

// 导入STable配置和样式
import { setupSTable } from '@cost-app/shared-components'
import '@surely-vue/table/dist/index.less'

import App from './App.vue'

// 创建路由
const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('./views/Dashboard.vue'),
    meta: { title: '工作台' }
  },
  {
    path: '/form-page',
    name: 'FormPage',
    component: () => import('./views/FormPage.vue'),
    meta: { title: '表单页面' }
  },
  {
    path: '/settings-page',
    name: 'SettingsPage',
    component: () => import('./views/SettingsPage.vue'),
    meta: { title: '系统设置' }
  },
  {
    path: '/data-page',
    name: 'DataPage',
    component: () => import('./views/DataPage.vue'),
    meta: { title: '数据查看器' }
  },
  {
    path: '/estimate-demo',
    name: 'EstimateDemo',
    component: () => import('./views/EstimateDemo.vue'),
    meta: { title: '概算演示' }
  },
  {
    path: '/table-test',
    name: 'TableTest',
    component: () => import('./views/TableTest.vue'),
    meta: { title: '表格测试' }
  },
  {
    path: '/window-manager',
    name: 'WindowManager',
    component: () => import('./components/WindowManagerDemo.vue'),
    meta: { title: '窗体管理演示' }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫
router.beforeEach((to, from, next) => {
  if (to.meta.title) {
    document.title = `${to.meta.title} - 造价管理系统`
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

import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'

import App from './App.vue'

// 路由配置
const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('./views/Dashboard.vue'),
    meta: { title: '工作台' }
  },
  {
    path: '/rough-estimate',
    name: 'RoughEstimate',
    component: () => import('./views/RoughEstimate.vue'),
    meta: { title: '概算' }
  },
  {
    path: '/budget',
    name: 'Budget',
    component: () => import('./views/Budget.vue'),
    meta: { title: '预算' }
  },
  {
    path: '/budget-review',
    name: 'BudgetReview',
    component: () => import('./views/BudgetReview.vue'),
    meta: { title: '预算审核' }
  },
  {
    path: '/settlement',
    name: 'Settlement',
    component: () => import('./views/Settlement.vue'),
    meta: { title: '结算' }
  },
  {
    path: '/settlement-review',
    name: 'SettlementReview',
    component: () => import('./views/SettlementReview.vue'),
    meta: { title: '结算审核' }
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

app.use(router)
app.use(pinia)
app.use(ElementPlus)

app.mount('#app')

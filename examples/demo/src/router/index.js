import { createRouter, createWebHistory } from 'vue-router'

// 路由配置
const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('../views/Dashboard.vue'),
    meta: { title: '工作台' }
  },
  {
    path: '/form-page',
    name: 'FormPage',
    component: () => import('../views/FormPage.vue'),
    meta: { title: '表单页面' }
  },
  {
    path: '/data-page',
    name: 'DataPage',
    component: () => import('../views/DataPage.vue'),
    meta: { title: '数据查看器' }
  },
  {
    path: '/table-test',
    name: 'TableTest',
    component: () => import('../views/TableTest.vue'),
    meta: { title: '表格测试' }
  },
]

// 创建路由实例
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

export default router
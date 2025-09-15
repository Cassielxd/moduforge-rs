import { createRouter, createWebHashHistory } from 'vue-router'

// 概算应用专用路由配置
const routes = [
  {
    path: '/',
    name: 'EstimateMain',
    component: () => import('../views/EstimateMain.vue'),
    meta: { title: '概算管理' }
  },
  {
    path: '/form-page',
    name: 'FormPage',
    component: () => import('../views/FormPage.vue'),
    meta: { title: '表单页面' }
  },
  {
    path: '/test-form',
    name: 'TestFormPage',
    component: () => import('../views/TestFormPage.vue'),
    meta: { title: '表单测试页面' }
  },
  {
    path: '/operate-page/:operate?',
    name: 'OperatePage',
    component: () => import('../views/OperatePage.vue'),
    meta: { title: '操作页面' }
  }
]

// 创建路由实例 - 使用 hash 模式以支持打包后的环境
const router = createRouter({
  history: createWebHashHistory(),
  routes
})

// 路由守卫
router.beforeEach((to, from, next) => {
  if (to.meta.title) {
    document.title = `${to.meta.title} - 概算管理系统`
  }
  next()
})

export default router
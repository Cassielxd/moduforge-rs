import { createRouter, createWebHashHistory } from 'vue-router'
import { defineAsyncComponent } from 'vue'

// 懒加载微前端模块
const RoughEstimateApp = defineAsyncComponent(() => import('roughEstimate/RoughEstimateApp'))
const BudgetApp = defineAsyncComponent(() => import('budget/BudgetApp'))
const BudgetReviewApp = defineAsyncComponent(() => import('budgetReview/BudgetReviewApp'))
const SettlementApp = defineAsyncComponent(() => import('settlement/SettlementApp'))
const SettlementReviewApp = defineAsyncComponent(() => import('settlementReview/SettlementReviewApp'))

const routes = [
  {
    path: '/',
    name: 'Dashboard',
    component: () => import('../views/Dashboard.vue'),
    meta: { title: '工作台' }
  },
  {
    path: '/rough-estimate',
    name: 'RoughEstimate',
    component: RoughEstimateApp,
    meta: { title: '概算' }
  },
  {
    path: '/budget',
    name: 'Budget', 
    component: BudgetApp,
    meta: { title: '预算' }
  },
  {
    path: '/budget-review',
    name: 'BudgetReview',
    component: BudgetReviewApp,
    meta: { title: '预算审核' }
  },
  {
    path: '/settlement',
    name: 'Settlement',
    component: SettlementApp,
    meta: { title: '结算' }
  },
  {
    path: '/settlement-review',
    name: 'SettlementReview',
    component: SettlementReviewApp,
    meta: { title: '结算审核' }
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

// 路由守卫
router.beforeEach((to, from, next) => {
  // 设置页面标题
  if (to.meta.title) {
    document.title = `${to.meta.title} - 造价管理系统`
  }
  next()
})

export default router

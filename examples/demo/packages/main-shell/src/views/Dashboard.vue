<template>
  <div class="dashboard">
    <el-row :gutter="20">
      <el-col :span="24">
        <el-card class="welcome-card">
          <h1>欢迎使用造价管理系统</h1>
          <p>这是一个基于微前端架构的造价管理系统演示</p>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" style="margin-top: 20px;">
      <el-col :span="6" v-for="module in modules" :key="module.name">
        <el-card class="module-card" @click="navigateTo(module.path)">
          <div class="module-content">
            <el-icon :size="40" class="module-icon">
              <component :is="module.icon" />
            </el-icon>
            <h3>{{ module.title }}</h3>
            <p>{{ module.description }}</p>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" style="margin-top: 20px;">
      <el-col :span="24">
        <el-card>
          <template #header>
            <span>系统状态</span>
          </template>
          <div class="status-grid">
            <div class="status-item">
              <span class="status-label">微前端模块:</span>
              <span class="status-value">{{ moduleStatus.total }} 个</span>
            </div>
            <div class="status-item">
              <span class="status-label">已加载模块:</span>
              <span class="status-value">{{ moduleStatus.loaded }} 个</span>
            </div>
            <div class="status-item">
              <span class="status-label">共享组件:</span>
              <span class="status-value">可用</span>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { 
  Document, 
  Money, 
  Check, 
  Files, 
  CircleCheck 
} from '@element-plus/icons-vue'

const router = useRouter()

const modules = ref([
  {
    name: 'rough-estimate',
    title: '概算',
    description: '项目概算管理',
    path: '/rough-estimate',
    icon: Document
  },
  {
    name: 'budget',
    title: '预算',
    description: '项目预算编制',
    path: '/budget',
    icon: Money
  },
  {
    name: 'budget-review',
    title: '预算审核',
    description: '预算审核流程',
    path: '/budget-review',
    icon: Check
  },
  {
    name: 'settlement',
    title: '结算',
    description: '项目结算管理',
    path: '/settlement',
    icon: Files
  },
  {
    name: 'settlement-review',
    title: '结算审核',
    description: '结算审核流程',
    path: '/settlement-review',
    icon: CircleCheck
  }
])

const moduleStatus = ref({
  total: 6,
  loaded: 1
})

const navigateTo = (path) => {
  router.push(path)
}
</script>

<style scoped>
.dashboard {
  max-width: 1200px;
  margin: 0 auto;
}

.welcome-card {
  text-align: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.welcome-card :deep(.el-card__body) {
  padding: 40px;
}

.module-card {
  cursor: pointer;
  transition: all 0.3s ease;
  height: 200px;
}

.module-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 25px rgba(0,0,0,0.15);
}

.module-content {
  text-align: center;
  padding: 20px;
}

.module-icon {
  color: #409eff;
  margin-bottom: 15px;
}

.module-content h3 {
  margin: 10px 0;
  color: #303133;
}

.module-content p {
  color: #909399;
  font-size: 14px;
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 20px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  background: #f5f7fa;
  border-radius: 4px;
}

.status-label {
  color: #606266;
}

.status-value {
  color: #409eff;
  font-weight: 600;
}
</style>

<template>
  <div class="budget-review-module">
    <el-card>
      <template #header>
        <span>预算审核</span>
      </template>
      <el-alert
        title="预算审核模块"
        type="warning"
        description="这里将是预算审核的独立微前端模块"
        show-icon
        :closable="false"
      />
    </el-card>
  </div>
</template>

<script setup>
console.log('✅ 预算审核模块已加载')
</script>

<style scoped>
.budget-review-module {
  max-width: 1200px;
  margin: 0 auto;
}
</style>

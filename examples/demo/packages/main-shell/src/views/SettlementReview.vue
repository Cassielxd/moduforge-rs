<template>
  <div class="settlement-review-module">
    <el-card>
      <template #header>
        <span>结算审核</span>
      </template>
      <el-alert
        title="结算审核模块"
        type="error"
        description="这里将是结算审核的独立微前端模块"
        show-icon
        :closable="false"
      />
    </el-card>
  </div>
</template>

<script setup>
console.log('🔍 结算审核模块已加载')
</script>

<style scoped>
.settlement-review-module {
  max-width: 1200px;
  margin: 0 auto;
}
</style>

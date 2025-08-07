<template>
  <div class="settlement-module">
    <el-card>
      <template #header>
        <span>结算管理</span>
      </template>
      <el-alert
        title="结算模块"
        type="info"
        description="这里将是结算管理的独立微前端模块"
        show-icon
        :closable="false"
      />
    </el-card>
  </div>
</template>

<script setup>
console.log('💰 结算模块已加载')
</script>

<style scoped>
.settlement-module {
  max-width: 1200px;
  margin: 0 auto;
}
</style>

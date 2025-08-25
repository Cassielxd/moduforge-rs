<template>
  <div class="rough-estimate-module">
    <el-card>
      <template #header>
        <span>概算管理</span>
      </template>
      <el-alert
        title="概算模块"
        type="success"
        description="这里将是概算管理的独立微前端模块"
        show-icon
        :closable="false"
      />
    </el-card>
  </div>
</template>

<script setup>
console.log('📊 概算模块已加载')
</script>

<style scoped>
.rough-estimate-module {
  max-width: 1200px;
  margin: 0 auto;
}
</style>

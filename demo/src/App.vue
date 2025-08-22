<script setup>
import { ref } from 'vue'

const theme = ref({
  token: {
    colorPrimary: '#1890ff',
    borderRadius: 6,
  },
})
</script>

<template>
  <div id="app">
    <a-config-provider :theme="theme">
      <router-view />
    </a-config-provider>
  </div>
</template>

<style>
body,
html {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

#app {
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>

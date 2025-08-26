import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'

export default defineConfig({
  plugins: [
    vue(),
    AutoImport({
      resolvers: [ElementPlusResolver()],
    }),
    Components({
      resolvers: [ElementPlusResolver()],
    }),
  ],
  server: {
    port: 5173,
    host: true
  },
  base: './',  // 使用相对路径
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: true,  // 开启 sourcemap 便于调试
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html')
      }
      // 移除 external 配置，让 Tauri API 正确打包
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  }
})

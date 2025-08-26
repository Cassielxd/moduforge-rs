import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { AntDesignVueResolver } from 'unplugin-vue-components/resolvers'

export default defineConfig({
  plugins: [
    vue(),
    // vueDevTools(), // 暂时注释掉以避免导入错误
    AutoImport({
      resolvers: [AntDesignVueResolver()],
    }),
    Components({
      resolvers: [AntDesignVueResolver({ importStyle: false })],
    }),
  ],
  server: {
    port: 5174,
    host: true,
    cors: true
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
      // CSS预处理器配置
  css: {
    preprocessorOptions: {
      less: {
        javascriptEnabled: true,
        modifyVars: {
          // 可以在这里添加Less变量覆盖
        }
      }
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  }
})

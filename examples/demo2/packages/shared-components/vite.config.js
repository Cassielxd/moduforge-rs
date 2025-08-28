import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig(({ mode }) => ({
  plugins: [vue()],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.js'),
      name: 'SharedComponents',
      fileName: (format) => `index.${format}.js`,
      formats: ['es', 'umd']
    },
    rollupOptions: {
      external: [
        'vue', 
        'ant-design-vue', 
        'dayjs', 
        '@surely-vue/table',
        '@tauri-apps/api',
        '@tauri-apps/api/webviewWindow',
        '@tauri-apps/api/core',
        '@tauri-apps/api/event'
      ],
      output: {
        globals: {
          vue: 'Vue',
          'ant-design-vue': 'antd',
          'dayjs': 'dayjs',
          '@surely-vue/table': 'STable',
          '@tauri-apps/api': 'TauriApi',
          '@tauri-apps/api/webviewWindow': 'TauriWebviewWindow',
          '@tauri-apps/api/core': 'TauriCore',
          '@tauri-apps/api/event': 'TauriEvent'
        }
      }
    },
    // 开发模式下不压缩，加快构建速度
    minify: mode === 'production',
    sourcemap: true,
    // 监听文件变化
    watch: mode === 'development' ? {} : null
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
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
  // 开发服务器配置
  server: {
    port: 5175,
    cors: true
  },
  // 优化依赖预构建
  optimizeDeps: {
    include: ['vue', 'ant-design-vue', 'dayjs']
  }
}))

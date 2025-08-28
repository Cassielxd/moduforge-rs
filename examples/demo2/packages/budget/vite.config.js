import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'
import { federation } from '@originjs/vite-plugin-federation'

export default defineConfig({
  plugins: [
    vue(),
    federation({
      name: 'budget',
      filename: 'remoteEntry.js',
      exposes: {
        './BudgetApp': './src/App.vue',
        './BudgetRoutes': './src/router/index.js',
        './BudgetStore': './src/store/index.js'
      },
      shared: {
        vue: {
          singleton: true,
          requiredVersion: '^3.5.13'
        },
        'element-plus': {
          singleton: true,
          requiredVersion: '^2.10.2'
        },
        '@cost-app/shared-components': {
          singleton: true
        }
      }
    })
  ],
  server: {
    port: 5175,
    cors: true
  },
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
    rollupOptions: {
      external: ['vue', 'element-plus']
    }
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@shared': resolve(__dirname, '../shared-components/src')
    }
  }
})

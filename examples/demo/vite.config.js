import { fileURLToPath, URL } from "node:url";
import { resolve } from "path";
import { copyFileSync, existsSync, mkdirSync, readdirSync, statSync } from "fs";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueDevTools from "vite-plugin-vue-devtools";

import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import { AntDesignVueResolver } from "unplugin-vue-components/resolvers";

// 递归复制目录
const copyDir = (src, dest) => {
  if (!existsSync(dest)) {
    mkdirSync(dest, { recursive: true });
  }

  const files = readdirSync(src);

  for (const file of files) {
    const srcPath = resolve(src, file);
    const destPath = resolve(dest, file);

    if (statSync(srcPath).isDirectory()) {
      copyDir(srcPath, destPath);
    } else {
      copyFileSync(srcPath, destPath);
    }
  }
};

// 自定义插件：复制 splashscreen.html 到 dist 目录
const copySplashscreenPlugin = () => {
  return {
    name: 'copy-splashscreen',
    writeBundle() {
      try {
        copyFileSync(
          resolve(__dirname, 'splashscreen.html'),
          resolve(__dirname, 'dist/splashscreen.html')
        );
        console.log('✓ splashscreen.html 已复制到 dist 目录');
      } catch (error) {
        console.error('复制 splashscreen.html 失败:', error);
      }
    }
  };
};

// 自定义插件：复制子模块构建产物
const copyPackagesPlugin = () => {
  return {
    name: 'copy-packages',
    writeBundle() {
      const packages = ['rough-estimate'];

      packages.forEach(pkg => {
        try {
          const srcPath = resolve(__dirname, `packages/${pkg}/dist`);
          const destPath = resolve(__dirname, `dist/${pkg}`);

          if (existsSync(srcPath)) {
            console.log(`📦 复制 ${pkg} 构建产物...`);
            copyDir(srcPath, destPath);
            console.log(`✅ ${pkg} 构建产物复制完成`);
          } else {
            console.log(`⚠️  ${pkg} dist 目录不存在，跳过复制`);
          }
        } catch (error) {
          console.error(`❌ 复制 ${pkg} 构建产物失败:`, error);
        }
      });
    }
  };
};

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueDevTools(),
    AutoImport({
      resolvers: [AntDesignVueResolver()],
    }),
    Components({
      resolvers: [AntDesignVueResolver({ importStyle: false })],
    }),
    copySplashscreenPlugin(),
    copyPackagesPlugin(),
  ],
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
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
  server: {
    port: 5173,
  },
  define: {
    "process.env": {
      VITE_APP_TITLE: JSON.stringify("造价管理系统"),
      VITE_API_BASE_URL: JSON.stringify("http://localhost:20008/api"),
      VITE_APP_ENV: JSON.stringify(process.env.NODE_ENV),
    },
  },
});

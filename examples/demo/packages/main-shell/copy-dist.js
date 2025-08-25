import { copyFileSync, mkdirSync, existsSync, readdirSync, statSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// 递归复制目录
function copyDir(src, dest) {
  if (!existsSync(dest)) {
    mkdirSync(dest, { recursive: true });
  }
  
  const files = readdirSync(src);
  
  for (const file of files) {
    const srcPath = join(src, file);
    const destPath = join(dest, file);
    
    if (statSync(srcPath).isDirectory()) {
      copyDir(srcPath, destPath);
    } else {
      copyFileSync(srcPath, destPath);
    }
  }
}

try {
  const distSrc = join(__dirname, 'dist');
  const distDest = join(__dirname, '../../dist/main-shell');
  
  if (existsSync(distSrc)) {
    console.log('📦 复制 main-shell 构建产物到主应用...');
    copyDir(distSrc, distDest);
    console.log('✅ main-shell 构建产物复制完成');
  } else {
    console.log('⚠️  main-shell dist 目录不存在，请先运行 build');
  }
} catch (error) {
  console.error('❌ 复制 main-shell 构建产物失败:', error);
  process.exit(1);
}

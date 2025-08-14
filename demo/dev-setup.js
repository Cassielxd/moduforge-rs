#!/usr/bin/env node

/**
 * 开发环境设置脚本
 * 用于启动 shared-components 的热更新和主应用
 */

import { spawn } from 'child_process'
import { fileURLToPath } from 'url'
import { dirname, join } from 'path'
import fs from 'fs'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m'
}

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`)
}

function logSection(title) {
  log(`\n${'='.repeat(50)}`, 'cyan')
  log(`  ${title}`, 'bright')
  log(`${'='.repeat(50)}`, 'cyan')
}

// 检查依赖是否安装
function checkDependencies() {
  logSection('检查依赖')
  
  const sharedComponentsPath = join(__dirname, 'packages/shared-components')
  const nodeModulesPath = join(sharedComponentsPath, 'node_modules')
  
  if (!fs.existsSync(nodeModulesPath)) {
    log('shared-components 依赖未安装，正在安装...', 'yellow')
    
    return new Promise((resolve, reject) => {
      const install = spawn('npm', ['install'], {
        cwd: sharedComponentsPath,
        stdio: 'inherit',
        shell: true
      })
      
      install.on('close', (code) => {
        if (code === 0) {
          log('依赖安装完成', 'green')
          resolve()
        } else {
          log('依赖安装失败', 'red')
          reject(new Error('依赖安装失败'))
        }
      })
    })
  } else {
    log('依赖已安装', 'green')
    return Promise.resolve()
  }
}

// 启动 shared-components 开发模式
function startSharedComponents() {
  logSection('启动 Shared Components 热更新')
  
  const sharedComponentsPath = join(__dirname, 'packages/shared-components')
  
  const sharedDev = spawn('npm', ['run', 'dev'], {
    cwd: sharedComponentsPath,
    stdio: 'pipe',
    shell: true
  })
  
  sharedDev.stdout.on('data', (data) => {
    log(`[Shared] ${data.toString().trim()}`, 'blue')
  })
  
  sharedDev.stderr.on('data', (data) => {
    log(`[Shared Error] ${data.toString().trim()}`, 'red')
  })
  
  sharedDev.on('close', (code) => {
    log(`Shared Components 进程退出，代码: ${code}`, 'yellow')
  })
  
  return sharedDev
}

// 启动主应用
function startMainApp() {
  logSection('启动主应用')
  
  const mainApp = spawn('npm', ['run', 'dev'], {
    cwd: __dirname,
    stdio: 'pipe',
    shell: true
  })
  
  mainApp.stdout.on('data', (data) => {
    log(`[Main] ${data.toString().trim()}`, 'green')
  })
  
  mainApp.stderr.on('data', (data) => {
    log(`[Main Error] ${data.toString().trim()}`, 'red')
  })
  
  mainApp.on('close', (code) => {
    log(`主应用进程退出，代码: ${code}`, 'yellow')
  })
  
  return mainApp
}

// 启动概算模块
function startEstimateModule() {
  logSection('启动概算模块')
  
  const estimatePath = join(__dirname, 'packages/rough-estimate')
  
  const estimateApp = spawn('npm', ['run', 'dev'], {
    cwd: estimatePath,
    stdio: 'pipe',
    shell: true
  })
  
  estimateApp.stdout.on('data', (data) => {
    log(`[Estimate] ${data.toString().trim()}`, 'magenta')
  })
  
  estimateApp.stderr.on('data', (data) => {
    log(`[Estimate Error] ${data.toString().trim()}`, 'red')
  })
  
  estimateApp.on('close', (code) => {
    log(`概算模块进程退出，代码: ${code}`, 'yellow')
  })
  
  return estimateApp
}

// 主函数
async function main() {
  log('🚀 启动开发环境', 'bright')
  
  try {
    // 检查并安装依赖
    await checkDependencies()
    
    // 启动所有服务
    const processes = []
    
    // 启动 shared-components 热更新
    processes.push(startSharedComponents())
    
    // 等待一下让 shared-components 先启动
    await new Promise(resolve => setTimeout(resolve, 3000))
    
    // 启动主应用
    processes.push(startMainApp())
    
    // 启动概算模块
    processes.push(startEstimateModule())
    
    logSection('所有服务已启动')
    log('📦 Shared Components: http://localhost:5175', 'blue')
    log('🏠 主应用: http://localhost:5173', 'green')
    log('📊 概算模块: http://localhost:5174', 'magenta')
    log('\n按 Ctrl+C 停止所有服务', 'yellow')
    
    // 处理退出信号
    process.on('SIGINT', () => {
      log('\n正在停止所有服务...', 'yellow')
      
      processes.forEach((proc, index) => {
        if (proc && !proc.killed) {
          proc.kill('SIGINT')
          log(`已停止进程 ${index + 1}`, 'yellow')
        }
      })
      
      setTimeout(() => {
        log('所有服务已停止', 'green')
        process.exit(0)
      }, 1000)
    })
    
    // 保持进程运行
    process.stdin.resume()
    
  } catch (error) {
    log(`启动失败: ${error.message}`, 'red')
    process.exit(1)
  }
}

// 运行主函数
main().catch(error => {
  log(`未处理的错误: ${error.message}`, 'red')
  process.exit(1)
})

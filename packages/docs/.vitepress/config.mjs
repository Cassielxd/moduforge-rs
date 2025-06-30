import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'ModuForge-RS',
  description: '基于 Rust 的状态管理和数据转换框架',
  
  // 基础配置
  base: '/',
  outDir: 'dist',
  cleanUrls: true,
  
  // 多语言配置
  locales: {
    root: {
      label: '中文',
      lang: 'zh-CN',
      title: 'ModuForge-RS',
      description: '基于 Rust 的状态管理和数据转换框架',
      themeConfig: {
        nav: [
          { text: '首页', link: '/' },
          { text: '指南', link: '/plugin-development-guide' },
          { text: '架构', link: '/architecture_use_cases' },
          { text: '示例', link: '/demo-showcase' },
          {
            text: '更多',
            items: [
              { text: 'English', link: '/en/' },
              { text: 'GitHub', link: 'https://github.com/Cassielxd/moduforge-rs' }
            ]
          }
        ],
        
        sidebar: {
          '/': [
            {
              text: '开始使用',
              collapsed: false,
              items: [
                { text: '项目概述', link: '/' },
                { text: '外部项目集成', link: '/setup-external-project' },
                { text: '集成示例', link: '/example-integration-project' }
              ]
            },
            {
              text: '开发指南',
              collapsed: false,
              items: [
                { text: '插件开发指南', link: '/plugin-development-guide' },
                { text: '自定义函数', link: '/CUSTOM_FUNCTIONS' },
                { text: '节点预算映射', link: '/node-budget-mapping' }
              ]
            },
            {
              text: '架构设计',
              collapsed: false,
              items: [
                { text: '应用场景分析', link: '/architecture_use_cases' },
                { text: '架构限制分析', link: '/architecture_limitations_analysis' },
                { text: '业务依赖设计', link: '/business_dependency_design' },
                { text: '元数据依赖设计', link: '/meta_based_dependency_design' }
              ]
            },
            {
              text: '示例和演示',
              collapsed: false,
              items: [
                { text: '功能演示', link: '/demo-showcase' },
                { text: '历史增强', link: '/simple_enhanced_history' }
              ]
            },
            {
              text: '故障排查',
              collapsed: false,
              items: [
                { text: 'WebSocket 错误排查', link: '/websocket-error-troubleshooting' },
                { text: '项目分析', link: '/ANALYSIS' }
              ]
            }
          ]
        },
        
        footer: {
          message: '基于 MIT 许可发布',
          copyright: 'Copyright © 2024 ModuForge Team'
        },
        
        search: {
          provider: 'local'
        },
        
        editLink: {
          pattern: 'https://github.com/Cassielxd/moduforge-rs/edit/main/packages/docs/:path',
          text: '在 GitHub 上编辑此页'
        },
        
        lastUpdated: {
          text: '最后更新于',
          formatOptions: {
            dateStyle: 'short',
            timeStyle: 'medium'
          }
        }
      }
    },
    
    en: {
      label: 'English',
      lang: 'en-US',
      title: 'ModuForge-RS',
      description: 'Rust-based state management and data transformation framework',
      themeConfig: {
        nav: [
          { text: 'Home', link: '/en/' },
          { text: 'Guide', link: '/en/plugin-development-guide' },
          { text: 'Architecture', link: '/en/architecture_use_cases' },
          { text: 'Examples', link: '/en/demo-showcase' },
          {
            text: 'More',
            items: [
              { text: '中文', link: '/' },
              { text: 'GitHub', link: 'https://github.com/Cassielxd/moduforge-rs' }
            ]
          }
        ],
        
        sidebar: {
          '/en/': [
            {
              text: 'Getting Started',
              collapsed: false,
              items: [
                { text: 'Overview', link: '/en/' },
                { text: 'External Project Setup', link: '/en/setup-external-project' },
                { text: 'Integration Example', link: '/en/example-integration-project' }
              ]
            },
            {
              text: 'Development Guide',
              collapsed: false,
              items: [
                { text: 'Plugin Development', link: '/en/plugin-development-guide' },
                { text: 'Custom Functions', link: '/en/CUSTOM_FUNCTIONS' },
                { text: 'Node Budget Mapping', link: '/en/node-budget-mapping' }
              ]
            },
            {
              text: 'Architecture',
              collapsed: false,
              items: [
                { text: 'Use Cases Analysis', link: '/en/architecture_use_cases' },
                { text: 'Limitations Analysis', link: '/en/architecture_limitations_analysis' },
                { text: 'Business Dependency Design', link: '/en/business_dependency_design' },
                { text: 'Meta-based Dependency Design', link: '/en/meta_based_dependency_design' }
              ]
            },
            {
              text: 'Examples & Demos',
              collapsed: false,
              items: [
                { text: 'Feature Showcase', link: '/en/demo-showcase' },
                { text: 'Enhanced History', link: '/en/simple_enhanced_history' }
              ]
            },
            {
              text: 'Troubleshooting',
              collapsed: false,
              items: [
                { text: 'WebSocket Error Troubleshooting', link: '/en/websocket-error-troubleshooting' },
                { text: 'Project Analysis', link: '/en/ANALYSIS' }
              ]
            }
          ]
        },
        
        footer: {
          message: 'Released under the MIT License',
          copyright: 'Copyright © 2024 ModuForge Team'
        },
        
        search: {
          provider: 'local'
        },
        
        editLink: {
          pattern: 'https://github.com/Cassielxd/moduforge-rs/edit/main/packages/docs/:path',
          text: 'Edit this page on GitHub'
        },
        
        lastUpdated: {
          text: 'Last updated',
          formatOptions: {
            dateStyle: 'short',
            timeStyle: 'medium'
          }
        }
      }
    }
  },
  
  // 主题配置
  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'ModuForge-RS',
    
    socialLinks: [
      { icon: 'github', link: 'https://github.com/Cassielxd/moduforge-rs' }
    ]
  },
  
  // Markdown 配置
  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    },
    lineNumbers: true,
    config: (md) => {
      // 可以在这里添加 markdown-it 插件
    }
  },
  
  // Vite 配置
  vite: {
    server: {
      port: 3000,
      host: true
    }
  }
}) 
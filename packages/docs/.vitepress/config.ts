import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

export default withMermaid(defineConfig({
  title: 'ModuForge-RS',
  description: '基于 Rust 的状态管理和数据转换框架',
  
  // 基础配置
  base: '/',
  outDir: 'dist',
  cleanUrls: true,
  
  // 🔧 修复死链接问题 - 忽略开发环境链接
  ignoreDeadLinks: [
    // 忽略本地开发环境链接
    /^http:\/\/localhost/,
    /^http:\/\/127\.0\.0\.1/,
    // 明确指定要忽略的链接
    'http://localhost:3000',
    'http://localhost:3000/',
    'http://localhost:3000/index',
    'http://localhost:3000/en/index',
  ],
  
  // 🎨 Mermaid 配置
  mermaid: {
    // 主题配置
    theme: 'default',
    // 自定义配置
    themeConfig: {
      // 亮色主题配置
      primaryColor: '#3b82f6',
      primaryTextColor: '#1f2937',
      primaryBorderColor: '#e5e7eb',
      lineColor: '#6b7280',
      secondaryColor: '#f3f4f6',
      tertiaryColor: '#ffffff',
      // 流程图配置
      background: '#ffffff',
      mainBkg: '#ffffff',
      secondBkg: '#f8fafc',
      tertiaryBkg: '#f1f5f9',
    },
    // 图表配置
    flowchart: {
      useMaxWidth: true,
      htmlLabels: true,
      curve: 'basis'
    },
    sequence: {
      useMaxWidth: true,
      wrap: true,
      width: 200
    },
    gantt: {
      useMaxWidth: true,
      leftPadding: 75,
      gridLineStartPadding: 35
    }
  },
  
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
          { text: '快速入门', link: '/quick-start' },
          { text: 'API 参考', link: '/api-reference' },
          { text: '架构概览', link: '/architecture-overview' },
          { text: '站点地图', link: '/sitemap' },
          { 
            text: '开发指南',
            items: [
              { text: '插件开发', link: '/plugin-development-guide' },
              { text: '性能优化', link: '/performance-guide' },
              { text: '外部项目集成', link: '/setup-external-project' }
            ]
          },
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
              text: '📚 开始使用',
              collapsed: false,
              items: [
                { text: '项目概述', link: '/' },
                { text: '快速入门', link: '/quick-start' },
                { text: '架构概览', link: '/architecture-overview' },
                { text: 'API 参考', link: '/api-reference' }
              ]
            },
            {
              text: '🔧 开发指南',
              collapsed: false,
              items: [
                { text: '插件开发指南', link: '/plugin-development-guide' },
                { text: '性能优化指南', link: '/performance-guide' },
                { text: '外部项目集成', link: '/setup-external-project' },
                { text: '集成示例项目', link: '/example-integration-project' }
              ]
            },
            {
              text: '🏗️ 架构设计',
              collapsed: false,
              items: [
                { text: '架构设计总览', link: '/architecture-design' },
                { text: '应用场景分析', link: '/architecture_use_cases' },
                { text: '架构限制分析', link: '/architecture_limitations_analysis' },
                { text: '业务依赖设计', link: '/business_dependency_design' },
                { text: '元数据依赖设计', link: '/meta_based_dependency_design' }
              ]
            },
            {
              text: '💼 业务应用',
              collapsed: false,
              items: [
                { text: '节点预算映射', link: '/node-budget-mapping' },
                { text: '自定义函数开发', link: '/CUSTOM_FUNCTIONS' },
                { text: '历史增强功能', link: '/simple_enhanced_history' }
              ]
            },
            {
              text: '🤝 协作与部署',
              collapsed: false,
              items: [
                { text: '协作系统', link: '/collaboration-system' },
                { text: '部署指南', link: '/DEPLOYMENT' },
                { text: 'WebSocket 故障排查', link: '/websocket-error-troubleshooting' }
              ]
            },
            {
              text: '🎯 演示与分析',
              collapsed: false,
              items: [
                { text: '功能演示', link: '/demo-showcase' },
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
          { text: 'Quick Start', link: '/en/quick-start' },
          { text: 'API Reference', link: '/en/api-reference' },
          { text: 'Architecture', link: '/en/architecture-overview' },
          { text: 'Sitemap', link: '/en/sitemap' },
          { 
            text: 'Development',
            items: [
              { text: 'Plugin Development', link: '/en/plugin-development-guide' },
              { text: 'Performance Guide', link: '/en/performance-guide' },
              { text: 'External Integration', link: '/en/setup-external-project' }
            ]
          },
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
              text: '📚 Getting Started',
              collapsed: false,
              items: [
                { text: 'Project Overview', link: '/en/' },
                { text: 'Quick Start', link: '/en/quick-start' },
                { text: 'Architecture Overview', link: '/en/architecture-overview' },
                { text: 'API Reference', link: '/en/api-reference' }
              ]
            },
            {
              text: '🔧 Development Guide',
              collapsed: false,
              items: [
                { text: 'Plugin Development Guide', link: '/en/plugin-development-guide' },
                { text: 'Performance Guide', link: '/en/performance-guide' },
                { text: 'External Project Setup', link: '/en/setup-external-project' },
                { text: 'Integration Example', link: '/en/example-integration-project' }
              ]
            },
            {
              text: '🏗️ Architecture Design',
              collapsed: false,
              items: [
                { text: 'Architecture Design Overview', link: '/en/architecture-design' },
                { text: 'Use Cases Analysis', link: '/en/architecture_use_cases' },
                { text: 'Limitations Analysis', link: '/en/architecture_limitations_analysis' },
                { text: 'Business Dependency Design', link: '/en/business_dependency_design' },
                { text: 'Meta-based Dependency Design', link: '/en/meta_based_dependency_design' }
              ]
            },
            {
              text: '💼 Business Applications',
              collapsed: false,
              items: [
                { text: 'Node Budget Mapping', link: '/en/node-budget-mapping' },
                { text: 'Custom Functions', link: '/en/CUSTOM_FUNCTIONS' },
                { text: 'Enhanced History', link: '/en/simple_enhanced_history' }
              ]
            },
            {
              text: '🤝 Collaboration & Deployment',
              collapsed: false,
              items: [
                { text: 'Collaboration System', link: '/en/collaboration-system' },
                { text: 'Deployment Guide', link: '/en/DEPLOYMENT' },
                { text: 'WebSocket Troubleshooting', link: '/en/websocket-error-troubleshooting' }
              ]
            },
            {
              text: '🎯 Demos & Analysis',
              collapsed: false,
              items: [
                { text: 'Feature Showcase', link: '/en/demo-showcase' },
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
      // Markdown-it 插件配置
      // Mermaid 插件会自动处理 mermaid 代码块
    }
  },
  
  // Vite 配置
  vite: {
    server: {
      port: 3000,
      host: true
    },
    // 优化配置
    optimizeDeps: {
      include: ['mermaid']
    }
  }
})) 
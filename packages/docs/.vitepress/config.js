import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

// https://vitepress.dev/reference/site-config
export default withMermaid(defineConfig({
  title: "ModuForge-RS",
  description: "Modern Rust Framework for High-Performance Document Editors",
  lang: 'zh-CN',
  base: '/moduforge-rs/',

  head: [
    ['link', { rel: 'icon', href: '/favicon.svg' }],
    ['meta', { name: 'theme-color', content: '#3c8772' }]
  ],

  themeConfig: {
    logo: '/logo.svg',

    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: '指南', link: '/guide/introduction' },
      { text: '示例', link: '/examples/price-rs-complete-project' },
      {
        text: '核心库',
        items: [
          { text: 'Model 数据模型', link: '/crates/model' },
          { text: 'State 状态管理', link: '/crates/state' },
          { text: 'Transform 文档转换', link: '/crates/transform' },
          { text: 'Core 运行时', link: '/crates/core' },
          { text: 'File 文件系统', link: '/crates/file' },
          { text: 'Search 搜索引擎', link: '/crates/search' },
          { text: 'Collaboration 协作', link: '/crates/collaboration' }
        ]
      },
      {
        text: 'v0.7.0',
        items: [
          // { text: '更新日志', link: '/changelog' },
          { text: 'v0.6.x', link: 'https://github.com/moduforge/moduforge-rs/tree/v0.6' }
        ]
      }
    ],

    sidebar: {
      '/guide/': [
        {
          text: '入门',
          collapsed: false,
          items: [
            { text: '介绍', link: '/guide/introduction' },
            { text: '快速开始', link: '/guide/quick-start' },
            { text: '核心概念', link: '/guide/core-concepts' },
            { text: '架构设计', link: '/guide/architecture' }
          ]
        },
        {
          text: '基础',
          collapsed: false,
          items: [
            { text: '创建编辑器', link: '/guide/creating-editor' },
            { text: '文档模型', link: '/guide/document-model' },
            { text: '状态管理', link: '/guide/state-management' },
            { text: '文档转换', link: '/guide/transformations' },
            { text: '插件系统', link: '/guide/plugins' }
          ]
        },
        {
          text: '进阶',
          collapsed: false,
          items: [
            { text: '自定义节点', link: '/guide/custom-nodes' },
            { text: '自定义标记', link: '/guide/custom-marks' },
            { text: '命令系统', link: '/guide/commands' },
            { text: '中间件', link: '/guide/middleware' },
            { text: '事件系统', link: '/guide/events' }
          ]
        },
        // {
        //   text: '协作',
        //   collapsed: false,
        //   items: [
        //     { text: '实时协作', link: '/guide/collaboration' },
        //     { text: 'CRDT 原理', link: '/guide/crdt' },
        //     { text: '冲突解决', link: '/guide/conflict-resolution' },
        //     { text: '部署指南', link: '/guide/deployment' }
        //   ]
        // },
        {
          text: '性能',
          collapsed: false,
          items: [
            { text: '性能优化', link: '/guide/performance' }
            // { text: '内存管理', link: '/guide/memory' },
            // { text: '基准测试', link: '/guide/benchmarks' }
          ]
        }
      ],

      '/examples/': [
        {
          text: 'Price-RS 完整案例',
          items: [
            { text: '项目概述与架构', link: '/examples/price-rs-complete-project' },
            { text: '扩展开发实战', link: '/examples/price-rs-extension-development' },
            { text: '运行时启动与引导', link: '/examples/price-rs-runtime-bootstrap' }
          ]
        }
      ],

      '/crates/': [
        {
          text: '核心库文档',
          items: [
            { text: 'Model - 数据模型', link: '/crates/model' },
            { text: 'State - 状态管理', link: '/crates/state' },
            { text: 'Transform - 文档转换', link: '/crates/transform' },
            { text: 'Core - 运行时框架', link: '/crates/core' },
            { text: 'File - 文件系统', link: '/crates/file' },
            { text: 'Persistence - 持久化', link: '/crates/persistence' },
            { text: 'Search - 搜索引擎', link: '/crates/search' },
            { text: 'Collaboration - 协作服务', link: '/crates/collaboration' },
            { text: 'Collaboration Client - 协作客户端', link: '/crates/collaboration-client' },
            { text: 'Macros Derive - 过程宏', link: '/crates/macros-derive' },
            { text: 'Macros - 声明宏', link: '/crates/macros' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/moduforge/moduforge-rs' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present ModuForge Team'
    },

    editLink: {
      pattern: 'https://github.com/moduforge/moduforge-rs/edit/main/packages/docs/:path',
      text: '在 GitHub 上编辑此页面'
    },

    search: {
      provider: 'local',
      options: {
        locales: {
          'zh-CN': {
            translations: {
              button: {
                buttonText: '搜索文档',
                buttonAriaLabel: '搜索文档'
              },
              modal: {
                noResultsText: '无法找到相关结果',
                resetButtonTitle: '清除查询条件',
                footer: {
                  selectText: '选择',
                  navigateText: '切换'
                }
              }
            }
          }
        }
      }
    },

    outline: {
      label: '页面导航',
      level: [2, 3]
    },

    lastUpdated: {
      text: '最后更新于',
      formatOptions: {
        dateStyle: 'short',
        timeStyle: 'medium'
      }
    },

    docFooter: {
      prev: '上一页',
      next: '下一页'
    },

    returnToTopLabel: '回到顶部'
  },

  markdown: {
    lineNumbers: true,
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    }
  },

  // Mermaid 配置
  mermaid: {
    theme: 'default',
    startOnLoad: true,
    securityLevel: 'loose'
  }
}))
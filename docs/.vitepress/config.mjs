import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "ModuForge-RS",
  description: "A Rust-based state management and data transformation framework.",
  themeConfig: {
    nav: [
      { text: '主页', link: '/' },
      { text: '文档', link: '/plugin-development-guide' }
    ],

    sidebar: [
      {
        text: '介绍',
        items: [
          { text: 'ModuForge 是什么？', link: '/' },
        ]
      },
      {
        text: '开发指南',
        items: [
          { text: '插件开发指南', link: '/plugin-development-guide' },
          { text: '自定义函数', link: '/CUSTOM_FUNCTIONS' },
        ]
      },
      {
        text: '架构设计',
        items: [
          { text: '架构应用场景', link: '/architecture_use_cases' },
          { text: '架构限制性分析', link: '/architecture_limitations_analysis' },
          { text: '业务依赖设计', link: '/business_dependency_design' },
          { text: '元数据依赖设计', link: '/meta_based_dependency_design' },
        ]
      },
      {
        text: '业务实践',
        items: [
          { text: 'Node模型映射', link: '/node-budget-mapping' },
          { text: '项目集成示例', link: '/example-integration-project' },
          { text: '外部项目设置', link: '/setup-external-project' },
          { text: '完整演示案例', link: '/demo-showcase' }
        ]
      }
    ],

    socialLinks: [
      // You can add links to your social media here
      // { icon: 'github', link: 'https://github.com/vuejs/vitepress' }
    ]
  }
}) 
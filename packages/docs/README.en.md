# ModuForge-RS 文档站点

该目录存放基于 VitePress 的文档。中文页面位于根目录，英文页面位于 en/ 目录。建议在修改时保持中英文同步。

## 快速开始
`ash
cd packages/docs
pnpm install
pnpm dev
`
访问 <http://localhost:3000> 以开发模式预览。

## 构建与预览
`ash
pnpm build
pnpm preview
`
构建产物输出到 dist/，可部署到任意静态站点。

## 目录结构
- .vitepress/：站点配置与自定义主题
- en/：英文文档
- public/：静态资源
- 其它 .md 文件：中文页面

## 维护建议
1. 更新中文文档后同步翻译英文版本。
2. 新增页面时，记得调整 .vitepress/config.ts 中的导航与侧边栏。
3. 如果需要团队知识库，可在此基础上扩展业务手册或 FAQ。

文档遵循 MIT 许可，可自由复用。

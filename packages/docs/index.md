---
layout: home

hero:
  name: "ModuForge-RS"
  text: "模块化运行时框架"
  tagline: "基于 Rust 的高性能运行时，覆盖不可变数据、事务、协作、检索与持久化"
  image:
    src: /logo.svg
    alt: ModuForge-RS
  actions:
    - theme: brand
      text: 快速开始
      link: /quick-start
    - theme: alt
      text: 插件开发指南
      link: /plugin-development-guide
    - theme: alt
      text: 查看 GitHub
      link: https://github.com/Cassielxd/moduforge-rs

features:
  - icon: 🏗️
    title: 模块化架构
    details: 按层拆分 11 个核心 crate，可自由组合模型、状态、事务与运行时。
  - icon: 🚀
    title: 高性能运行时
    details: 支持同步、异步、Actor 三种执行模型，并带有资源自适应配置。
  - icon: 🔧
    title: 插件生态
    details: 统一的扩展、插件、资源系统，方便封装业务逻辑与横切能力。
  - icon: 💾
    title: 事件持久化
    details: SQLite WAL 事件存储与快照策略，可按需调整一致性与压缩。
  - icon: 🔍
    title: 全文检索
    details: 基于 Tantivy 的增量索引服务，事务完成即可刷新搜索结果。
  - icon: 🤝
    title: 实时协作
    details: Warp + Yrs WebSocket 服务，内置房间管理、健康检查与断线恢复。
---

## 什么是 ModuForge-RS？

ModuForge-RS 是一个围绕不可变树形数据构建的运行时工作区。通过插件和事务管线，可以在不耦合业务代码的情况下快速搭建编辑器内核或领域运行时，并根据需要接入持久化、全文检索和协作能力。

## 推荐阅读
- [快速入门](./quick-start.md)
- [外部项目接入](./setup-external-project.md)
- [集成示例](./example-integration-project.md)
- [架构概览](./architecture-overview.md)
- [API 参考](./api-reference.md)

# frontend

This template should help get you started developing with Vue 3 in Vite.

## Recommended IDE Setup

[VSCode](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur).

## Type Support for `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) to make the TypeScript language service aware of `.vue` types.

## Customize configuration

See [Vite Configuration Reference](https://vite.dev/config/).

## Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

# ModuForge Demo

## 全局状态管理

### rootId 全局访问

项目中的 `rootId` 已经设置为全局状态，所有组件都可以访问。使用方式如下：

#### 1. 在组件中使用

```typescript
import { useRootStore } from '@/stores/root';

// 在 setup 中
const rootStore = useRootStore();

// 获取 rootId
const currentRootId = rootStore.getRootId;

// 设置 rootId
rootStore.setRootId('new-root-id');

// 清除 rootId
rootStore.clearRootId();
```

#### 2. 在模板中使用

```vue
<template>
  <div v-if="rootStore.getRootId">
    当前项目ID: {{ rootStore.getRootId }}
  </div>
</template>
```

#### 3. 响应式访问

由于使用了 Pinia，`rootId` 的变化会自动触发相关组件的重新渲染。

### Store 文件位置

- 全局 rootId store: `src/stores/root.ts`
- 历史记录 store: `src/stores/history.ts`

## 开发说明

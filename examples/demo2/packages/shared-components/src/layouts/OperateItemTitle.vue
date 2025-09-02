<!--
 * @Descripttion: 操作按钮标题组件
 * @Author: Claude
 * @Date: 2025-09-01
 * @LastEditors: Claude
 * @LastEditTime: 2025-09-01
-->
<template>
  <a-badge :dot="item.badgeDot">
    <icon-font
      :type="item.iconType"
      class="iconType"
      :style="item.iconStyle ?? {}"
    />
    <div
      class="label"
      v-if="isExpanded"
      :style="item.labelStyle ?? {}"
    >
      <slot name="label">{{ item.label }}</slot>
    </div>
  </a-badge>
</template>

<script setup>
import { inject, ref } from 'vue'
import { createFromIconfontCN } from '@ant-design/icons-vue'

// 创建 IconFont 组件
const IconFont = createFromIconfontCN({
  scriptUrl: '//at.alicdn.com/t/c/font_4199803_ualaqaqnkv.js',
})

const props = defineProps({
  item: {
    type: Object,
    default: () => {},
  },
})

// 尝试从父组件注入展开状态，如果没有则默认为true
const isExpanded = inject('isExpanded', ref(true))
</script>

<style lang="scss" scoped>
.iconType {
  font-size: 26px;
}
.icon {
  width: 28px;
  img {
    width: 100%;
  }
}
.label {
  font-size: 12px;
  margin-top: 2px;
  white-space: nowrap;
}
</style>
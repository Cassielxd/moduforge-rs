<template>
  <div>
    <a-tooltip
      placement="bottom"
      v-model:open="item.decVisible"
      @openChange="val => infoVisibleChange(val, item)"
    >
      <template #title>
        <span style="font-size: 12px; text-decoration: underline">{{
          item.label
        }}</span>
        <p v-if="item.infoDec" style="font-size: 10px">
          {{ item.infoDec }}
        </p>
      </template>
      <template v-if="['select', 'selectRadio'].includes(item.type)">
        <a-dropdown trigger="click" @openChange="visibleChange(item)">
          <div class="select-radio" v-if="['selectRadio'].includes(item.type)">
            <div class="select-head">
              <icon-font
                :type="item.iconType"
                class="iconType"
                :style="item.iconStyle ?? {}"
              />
              <div
                v-if="isExpanded"
                class="label"
                :style="item.labelStyle ?? {}"
              >
                {{ item.label }}
              </div>
              <icon-font
                v-if="isExpanded"
                type="icon-xiala"
                style="color: rgba(51, 51, 51, 0.39)"
              />
            </div>
            <div class="sub-name" v-if="isExpanded">
              {{
                item.options.find(opt => opt.kind === item.value)?.name || ''
              }}
            </div>
          </div>
          <div v-else>
            <OperateItemTitle :item="item">
              <template #label>
                {{ item.label }}
                <icon-font
                  type="icon-xiala"
                  style="color: rgba(51, 51, 51, 0.39)"
                />
              </template>
            </OperateItemTitle>
          </div>
          <template #overlay>
            <a-menu>
              <a-menu-item
                v-for="selectitem in item.options"
                :key="selectitem.kind"
                :disabled="!selectitem.isValid"
                @click="setSelectEmit(selectitem, item)"
              >
                <a-checkbox
                  v-if="['selectCheck'].includes(item.type)"
                  :checked="selectitem.kind == checkedIndex"
                ></a-checkbox>
                <a-radio
                  v-if="['selectRadio'].includes(item.type)"
                  :checked="selectitem.kind == item.value"
                ></a-radio>
                <span
                  v-if="['select-color'].includes(item.name)"
                  class="color-border"
                  :class="`${selectitem.kind}`"
                ></span>
                {{ selectitem.name }}
              </a-menu-item>
            </a-menu>
          </template>
        </a-dropdown>
      </template>
      <template v-else>
        <div>
          <OperateItemTitle :item="item"></OperateItemTitle>
        </div>
      </template>
    </a-tooltip>
  </div>
</template>

<script setup>
import { ref, inject } from 'vue'
import { createFromIconfontCN } from '@ant-design/icons-vue'
import OperateItemTitle from './OperateItemTitle.vue'

// 创建 IconFont 组件
const IconFont = createFromIconfontCN({
  scriptUrl: '//at.alicdn.com/t/c/font_4199803_ualaqaqnkv.js',
})

const props = defineProps({
  item: {
    type: Object,
    default: () => {},
  },
  parentItem: {
    type: Object,
    default: () => null,
  },
})

const emit = defineEmits(['setSelectEmit', 'setEmit'])

// 尝试从父组件注入展开状态，如果没有则默认为true
const isExpanded = inject('isExpanded', ref(true))
const checkedIndex = inject('checkedIndex', ref(null))

const setSelectEmit = (item, data) => {
  emit('setSelectEmit', { item, data })
}

const visibleChange = item => {
  emit('setEmit', item)
}

const infoVisibleChange = (val, item) => {
  if (val) {
    item.decVisible = true
  } else {
    item.decVisible = false
  }
}
</script>

<style lang="scss" scoped>
.select-radio {
  display: flex;
  flex-wrap: wrap;
  flex-direction: column;
  .select-head {
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
  }
  .sub-name {
    width: 100%;
    font-size: 12px;
    text-align: center;
  }
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
  }
}
</style>
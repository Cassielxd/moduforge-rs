<!--
 * @Descripttion: 操作栏组件 - 与原有框架operate.vue保持高度一致
 * @Author: Claude
 * @Date: 2025-09-01 
 * @LastEditors: Claude
 * @LastEditTime: 2025-09-01
-->
<template>
  <div class="operate-container hover-scrollbar-thumb">
    <div class="operate" :style="operateStyle">
      <div class="operate-scroll">
        <template v-for="(item, index) in filterOperateList" :key="index">
          <div
            class="operate-item"
            :class="{ disabled: item.disabled }"
            v-if="!item.isPolymerizeShow"
            @click="!item.disabled && !item.type ? handleOperateClick(item) : ''"
          >
            <a-tooltip
              placement="bottom"
              v-model:visible="item.decVisible"
              @visibleChange="val => infoVisibleChange(val, item)"
            >
              <template #title>
                <span style="font-size: 12px; text-decoration: underline">{{ item.label }}</span>
                <p v-if="item.infoDec" style="font-size: 10px">
                  {{ item.infoDec }}
                </p>
              </template>

              <!-- 下拉选择框类型 -->
              <template v-if="['selectCheck'].includes(item.type)">
                <a-dropdown
                  @visibleChange="() => handleOperateClick(item)"
                  trigger="click"
                  v-model:visible="item.dropdownVisible"
                >
                  <div>
                    <icon-font
                      :type="item.iconType"
                      class="iconType"
                      :style="item.iconStyle || {}"
                    />
                    <div
                      v-if="isExpanded"
                      class="label"
                      :style="item.labelStyle || {}"
                    >
                      {{ item.label }}
                    </div>
                  </div>
                  <template #overlay>
                    <a-menu>
                      <a-menu-item
                        v-for="selectitem in item.options"
                        :key="selectitem.kind"
                        :disabled="!selectitem.isValid"
                        @click="handleSelectClick(selectitem, item)"
                      >
                        <a-checkbox
                          v-if="['selectCheck'].includes(item.type)"
                          :checked="selectitem.kind == item.value"
                        ></a-checkbox>
                        <span
                          v-if="selectitem.colorClass"
                          class="color-border"
                          :class="selectitem.colorClass"
                        ></span>
                        {{ selectitem.name }}
                      </a-menu-item>
                    </a-menu>
                  </template>
                </a-dropdown>
              </template>
              
              <!-- 单选下拉类型 -->
              <template v-else-if="['selectRadio'].includes(item.type)">
                <a-dropdown
                  @visibleChange="() => handleOperateClick(item)"
                  trigger="click"
                  v-model:visible="item.dropdownVisible"
                >
                  <div class="select-radio">
                    <div class="select-head">
                      <icon-font
                        :type="item.iconType"
                        class="iconType"
                        :style="item.iconStyle || {}"
                      />
                      <div class="label" :style="item.labelStyle || {}">
                        {{ item.label }}
                      </div>
                      <icon-font
                        v-if="isExpanded"
                        type="icon-xiala"
                        style="color: rgba(51, 51, 51, 0.39)"
                      />
                    </div>
                    <div class="sub-name" v-if="isExpanded">
                      {{ item.options.find(opt => opt.kind === item.value)?.name || '' }}
                    </div>
                  </div>
                  <template #overlay>
                    <a-menu>
                      <a-menu-item
                        v-for="selectitem in item.options"
                        :key="selectitem.kind"
                        :disabled="!selectitem.isValid"
                        @click="handleSelectClick(selectitem, item)"
                      >
                        <a-radio
                          :checked="selectitem.kind == item.value"
                        ></a-radio>
                        {{ selectitem.name }}
                      </a-menu-item>
                    </a-menu>
                  </template>
                </a-dropdown>
              </template>
              
              <!-- 菜单列表类型 -->
              <template v-else-if="item.type === 'menuList'">
                <a-dropdown
                  trigger="click"
                  v-model:visible="item.dropdownVisible"
                >
                  <div class="menu-list-btn">
                    <div class="menu-list-head">
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
                  </div>
                  <template #overlay>
                    <div class="menu-list-content">
                      <div
                        :class="{ disabled: subItem.disabled, 'menu-list': true }"
                        @click="!subItem.disabled ? handleOperateClick(subItem, item) : ''"
                        v-for="subItem in item.menuInfoList || []"
                        :key="subItem.name"
                      >
                        <OperateItem
                          :item="subItem"
                          @setSelectEmit="itemInfo => handleSelectClick(itemInfo.item, itemInfo.data, item)"
                        ></OperateItem>
                      </div>
                    </div>
                  </template>
                </a-dropdown>
              </template>
              
              <!-- 普通按钮类型 -->
              <template v-else>
                <OperateItem
                  :item="item"
                  @setSelectEmit="itemInfo => handleSelectClick(itemInfo.item, itemInfo.data)"
                  @setEmit="handleOperateClick"
                ></OperateItem>
              </template>
            </a-tooltip>
          </div>
        </template>
      </div>
    </div>
    <div class="contract-btn" @click="toggleExpanded()">
      <up-outlined v-if="isExpanded" />
      <down-outlined v-else />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, provide } from 'vue'
import { DownOutlined, UpOutlined } from '@ant-design/icons-vue'
import OperateItem from './OperateItem.vue'
import OperateItemTitle from './OperateItemTitle.vue'
import { createFromIconfontCN } from '@ant-design/icons-vue'

// 创建 IconFont 组件（使用原框架的图标库）
const IconFont = createFromIconfontCN({
  scriptUrl: '//at.alicdn.com/t/c/font_4199803_ualaqaqnkv.js',
})

// 在 components 中注册 icon-font
const components = {
  'icon-font': IconFont
}

// Props
const props = defineProps({
  // 操作列表数据
  operateList: {
    type: Array,
    default: () => [],
    required: true
  },
  // 当前组件类型  
  componentType: {
    type: String,
    default: ''
  },
  // 当前层级类型
  levelType: {
    type: [String, Number],
    default: ''
  },
  // 窗口类型
  windowType: {
    type: String,
    default: 'parentPage'
  },
  // 项目类型
  projectType: {
    type: String,
    default: ''
  },
  // 高度配置
  height: {
    type: String,
    default: '60px'
  },
  // 是否默认展开
  defaultExpanded: {
    type: Boolean,
    default: true
  }
})

// Emits
const emit = defineEmits([
  'operateClick',
  'selectClick',
  'expandChange'
])

// State
const isExpanded = ref(props.defaultExpanded)

// 提供展开状态给子组件
provide('isExpanded', isExpanded)

// Computed  
const operateStyle = computed(() => {
  return {
    height: props.height,
    lineHeight: props.height,
  }
})

const filterOperateList = computed(() => {
  if (!props.operateList || props.operateList.length === 0) {
    return []
  }
  
  return props.operateList.filter(item => {
    // 基本显示条件
    if (item.hidden) return false
    
    // 组件类型过滤
    if (item.components && item.components.length > 0) {
      if (!item.components.includes(props.componentType)) return false
    }
    
    // 层级类型过滤 
    if (item.levelType && item.levelType.length > 0) {
      if (!item.levelType.includes(Number(props.levelType))) return false
    }
    
    // 窗口类型过滤
    if (item.windows && item.windows.length > 0) {
      if (!item.windows.includes(props.windowType)) return false
    }
    
    // 项目类型过滤
    if (item.showProjectType && item.showProjectType.length > 0) {
      if (!item.showProjectType.includes(props.projectType)) return false
    }
    
    return true
  })
})

// Event handlers
const handleOperateClick = (item, parentItem = null) => {
  item.decVisible = false
  
  // 关闭父级菜单下拉
  if (parentItem && parentItem.type === 'menuList' && !['select', 'selectRadio'].includes(item.type)) {
    parentItem.dropdownVisible = false
  }
  
  emit('operateClick', item, parentItem)
}

const handleSelectClick = (selectItem, item, parentItem = null) => {
  // 关闭父级菜单下拉
  if (parentItem && parentItem.type === 'menuList' && !['select', 'selectRadio'].includes(item.type)) {
    parentItem.dropdownVisible = false
  }
  
  if (['selectRadio', 'selectCheck'].includes(item.type)) {
    item.value = selectItem.kind
  }
  
  emit('selectClick', selectItem, item, parentItem)
}

const toggleExpanded = () => {
  isExpanded.value = !isExpanded.value
  emit('expandChange', isExpanded.value)
}

const infoVisibleChange = (val, item) => {
  item.decVisible = !!val
}

// Watch
watch(() => props.defaultExpanded, (newVal) => {
  isExpanded.value = newVal
})

// Expose
defineExpose({
  isExpanded,
  filterOperateList
})
</script>

<style lang="scss" scoped>
.menu-list-content {
  padding: 5px 10px;
  display: flex;
  background-color: #fff;
  .menu-list {
    margin: 0 4px;
    text-align: center;
    cursor: pointer;
    &.disabled {
      opacity: 0.5;
    }
  }
}
.menu-list-btn {
  display: flex;
  align-items: center;
}
.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.select-radio {
  display: flex;
  flex-wrap: wrap;
  flex-direction: column;
  .select-head {
    display: flex;
    align-items: center;
    font-size: 14px;
  }
  .sub-name {
    margin-top: 4px;
    width: 100%;
    font-size: 12px;
    text-align: center;
  }
}
.operate {
  box-sizing: border-box;
  background: #f3f6f9;
  width: calc(100% - 30px);
  overflow-y: hidden;
  height: v-bind(height);
  line-height: v-bind(height);
  user-select: none;
  overflow-x: auto;
  &-scroll {
    display: flex;
    min-width: fit-content;
    padding: 0 5px;
  }
  &-item {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-width: 50px;
    padding: 0 4px;
    text-align: center;
    height: v-bind(height);
    cursor: pointer;
    div {
      height: auto;
      line-height: initial;
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
      margin-top: 2px;
    }
  }
}
.operate-container {
  position: relative;
  border-bottom: 1px solid #d6d6d6;
}
.contract-btn {
  position: absolute;
  top: 50%;
  right: 0;
  width: 30px;
  height: 100%;
  display: flex;
  align-items: center;
  transform: translateY(-50%);
  justify-content: center;
  cursor: pointer;
}
.color-border {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 1px solid #eeeeee;
  border-radius: 3px;
  margin-right: 5px;
}
.none {
  background: none;
}
.red {
  background: #ef7c77 !important;
}
.green {
  background: #e3fada !important;
}
.orange {
  background: #e59665 !important;
}
.yellow {
  background: #fdfdac !important;
}
.blue {
  background: #8fa9fa !important;
}
.purple {
  background: #cfaadd !important;
}
.lightBlue {
  background: #a5d7f0 !important;
}
.deepYellow {
  background: #fbdf89 !important;
}
</style>
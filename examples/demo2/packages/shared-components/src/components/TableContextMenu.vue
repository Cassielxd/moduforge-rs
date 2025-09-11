<template>
  <teleport to="body">
    <div
      class="table-context-menu"
      ref="contextMenuRef"
      :style="{
        position: 'fixed',
        zIndex: '9998',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
        maxHeight: menuStyle.maxHeight,
        overflowY: 'auto',
        left: menuStyle.left,
        top: menuStyle.top,
      }"
      v-if="visible"
      @click.stop
    >
      <a-menu
        style="width: 180px; font-size: 12px; border: none;"
        mode="vertical"
      >
        <template v-for="(item, index) in menuOptions" :key="item.code">
          <!-- 子菜单 -->
          <a-sub-menu v-if="item.children?.length > 0 && item.visible" :key="item.code">
            <template #title>{{ item.name }}</template>
            <a-menu-item
              v-for="(cItem, cIndex) in item.children"
              :key="cItem.code"
              :style="{ display: cItem.visible ? 'block' : 'none' }"
              @click="handleMenuClick(cItem)"
              :disabled="cItem.disabled"
            >
              {{ cItem.name }}
            </a-menu-item>
          </a-sub-menu>
          
          <!-- 普通菜单项 -->
          <a-menu-item
            v-else-if="!item.children && item.visible"
            :key="item.code"
            :disabled="item.disabled"
            @click="handleMenuClick(item)"
          >
            {{ item.name }}
          </a-menu-item>
        </template>
      </a-menu>
    </div>
  </teleport>
</template>

<script setup>
import { ref, reactive, nextTick, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  position: {
    type: Object,
    default: () => ({ x: 0, y: 0 })
  },
  record: {
    type: Object,
    default: () => ({})
  },
  menuOptions: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['menu-click', 'hide'])

const contextMenuRef = ref()
const menuStyle = reactive({
  left: 'auto',
  top: 'auto',
  maxHeight: '85vh',
})

const SAFE_DISTANCE = 185 // 头部加底部安全距离
const SAFE_DISTANCE_BOTTOM = 33 // 底部安全距离

const hideMenu = () => {
  emit('hide')
  document.removeEventListener('click', hideMenu)
  document.removeEventListener('scroll', hideMenu, true)
}

const handleMenuClick = (menuItem) => {
  emit('menu-click', {
    menu: menuItem,
    record: props.record
  })
  hideMenu()
}

const updatePosition = () => {
  if (!contextMenuRef.value) return
  
  const rect = contextMenuRef.value.getBoundingClientRect()
  const viewportBottom = window.innerHeight
  const elementHeight = rect.height
  const x = props.position.x
  const y = props.position.y
  const yViewportDistance = viewportBottom - y // 鼠标y点距离可视底部距离

  // 处理X轴位置
  const viewportRight = window.innerWidth
  const elementWidth = rect.width
  if (x + elementWidth > viewportRight) {
    menuStyle.left = (x - elementWidth) + 'px'
  } else {
    menuStyle.left = x + 'px'
  }

  // 处理Y轴位置
  const viewportHeight = viewportBottom - elementHeight - SAFE_DISTANCE
  if (yViewportDistance >= elementHeight) {
    menuStyle.top = y + 'px'
  } else if (yViewportDistance < elementHeight && viewportHeight > 0) {
    // y轴以下的可视区域小于菜单元素高度 && 页面整体可用高度大于菜单元素时，将菜单top往上移动
    menuStyle.top = (y - (elementHeight - yViewportDistance + SAFE_DISTANCE_BOTTOM)) + 'px'
  } else {
    menuStyle.maxHeight = viewportHeight + 'px'
    menuStyle.top = SAFE_DISTANCE + 'px'
  }
}

onMounted(() => {
  if (props.visible) {
    document.addEventListener('click', hideMenu)
    document.addEventListener('scroll', hideMenu, true)
    
    // 监听表格滚动
    const tableEl = document.querySelector('.surely-table-body-viewport-container')
    if (tableEl) {
      tableEl.addEventListener('scroll', hideMenu)
    }
    
    nextTick(() => {
      updatePosition()
    })
  }
})

onUnmounted(() => {
  document.removeEventListener('click', hideMenu)
  document.removeEventListener('scroll', hideMenu, true)
})

// 监听位置变化
watch(() => props.position, () => {
  if (props.visible) {
    nextTick(() => {
      updatePosition()
    })
  }
}, { deep: true })

// 监听显示状态变化
watch(() => props.visible, (newVal) => {
  if (newVal) {
    document.addEventListener('click', hideMenu)
    document.addEventListener('scroll', hideMenu, true)
    nextTick(() => {
      updatePosition()
    })
  } else {
    document.removeEventListener('click', hideMenu)
    document.removeEventListener('scroll', hideMenu, true)
  }
})
</script>

<style scoped>
.table-context-menu {
  background: white;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border: 1px solid #f0f0f0;
}

:deep(.ant-menu-item) {
  font-size: 11px;
  height: 24px;
  line-height: 24px;
  margin: 0;
  padding: 0 12px;
}

:deep(.ant-menu-submenu-title) {
  font-size: 11px;
  height: 24px !important;
  line-height: 24px !important;
  margin: 0;
  padding: 0 12px;
}

:deep(.ant-menu-item:hover) {
  background-color: #f5f5f5;
}

:deep(.ant-menu-item-disabled) {
  color: #bfbfbf;
}

:deep(.ant-menu-submenu-title:hover) {
  background-color: #f5f5f5;
}
</style>

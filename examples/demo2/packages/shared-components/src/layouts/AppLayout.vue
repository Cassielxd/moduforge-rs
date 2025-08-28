<template>
  <div class="app-layout">
    <a-spin
      :spinning="loading"
      :tip="loadingTip"
      wrapper-class-name="spin-layout">
      
      <!-- Header 区域 -->
      <header class="layout-header">
        <slot name="header">
          <SimpleHeader
            :title="title"
            :show-window-controls="showWindowControls"
            :is-maximized="isMaximized"
            @minimize="$emit('minimize')"
            @maximize="$emit('maximize')"
            @close="$emit('close')">
            <template #left>
              <slot name="header-left">
                <div class="logo">
                  <h2>{{ title }}</h2>
                </div>
              </slot>
            </template>
            <template #right>
              <slot name="header-right"></slot>
            </template>
          </SimpleHeader>
        </slot>
      </header>

      <!-- 操作条区域 -->
      <section class="layout-operate" v-if="showOperate">
        <slot name="operate"></slot>
      </section>

      <!-- 主要内容区域 -->
      <section class="layout-main">
        <div class="layout-split" :style="{ height: '100%' }">
          
          <!-- 左侧侧边栏 -->
          <aside 
            class="layout-aside"
            :class="{ 'aside-collapsed': asideCollapsed }"
            :style="asideStyle"
            v-if="showAside">
            
            <!-- 展开/收起按钮 -->
            <div class="aside-toggle" v-if="allowAsideToggle">
              <a-tooltip>
                <template #title>{{ asideCollapsed ? "展开" : "收起" }}操作栏</template>
                <div class="toggle-btn" @click="toggleAside">
                  <img 
                    :src="asideCollapsed ? expandIcon : collapseIcon" 
                    :alt="asideCollapsed ? '展开' : '收起'" />
                </div>
              </a-tooltip>
            </div>

            <!-- 侧边栏内容 -->
            <div class="aside-content">
              <slot name="aside">
                <!-- 默认侧边栏内容 -->
              </slot>
            </div>
          </aside>

          <!-- 主内容区域 -->
          <main class="layout-content" :style="contentStyle">
            <slot name="content">
              <!-- 主要内容区域 -->
            </slot>
          </main>
        </div>
      </section>

      <!-- Footer 区域 -->
      <footer class="layout-footer" v-if="showFooter">
        <slot name="footer">
          <!-- 默认footer内容 -->
        </slot>
      </footer>
    </a-spin>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import SimpleHeader from './SimpleHeader.vue'

// Props
const props = defineProps({
  title: {
    type: String,
    default: '应用系统'
  },
  loading: {
    type: Boolean,
    default: false
  },
  loadingTip: {
    type: String,
    default: ''
  },
  showWindowControls: {
    type: Boolean,
    default: true
  },
  isMaximized: {
    type: Boolean,
    default: false
  },
  showOperate: {
    type: Boolean,
    default: true
  },
  showAside: {
    type: Boolean,
    default: true
  },
  showFooter: {
    type: Boolean,
    default: true
  },
  allowAsideToggle: {
    type: Boolean,
    default: true
  },
  asideWidth: {
    type: [String, Number],
    default: 225
  },
  asideCollapsedWidth: {
    type: [String, Number],
    default: 47
  },
  expandIcon: {
    type: String,
    default: '' // 可以传入展开图标路径
  },
  collapseIcon: {
    type: String,
    default: '' // 可以传入收起图标路径
  }
})

// Emits
const emit = defineEmits(['minimize', 'maximize', 'close', 'aside-toggle'])

// State
const asideCollapsed = ref(false)

// Computed
const asideStyle = computed(() => {
  if (!props.showAside) return { display: 'none' }
  
  const width = asideCollapsed.value 
    ? (typeof props.asideCollapsedWidth === 'number' ? `${props.asideCollapsedWidth}px` : props.asideCollapsedWidth)
    : (typeof props.asideWidth === 'number' ? `${props.asideWidth}px` : props.asideWidth)
  
  return {
    width,
    minWidth: width,
    transition: 'width 0.3s linear'
  }
})

const contentStyle = computed(() => {
  if (!props.showAside) return { width: '100%' }
  
  const asideWidth = asideCollapsed.value 
    ? (typeof props.asideCollapsedWidth === 'number' ? props.asideCollapsedWidth : parseInt(props.asideCollapsedWidth))
    : (typeof props.asideWidth === 'number' ? props.asideWidth : parseInt(props.asideWidth))
  
  return {
    width: `calc(100% - ${asideWidth}px)`,
    flex: 1
  }
})

// Methods
const toggleAside = () => {
  asideCollapsed.value = !asideCollapsed.value
  emit('aside-toggle', asideCollapsed.value)
}

// Expose methods
defineExpose({
  toggleAside,
  asideCollapsed
})
</script>

<style scoped>
.app-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f0f2f5;
}

.spin-layout {
  height: 100%;
}

.layout-header {
  flex-shrink: 0;
  z-index: 1000;
}

.layout-operate {
  flex-shrink: 0;
  background: #fff;
  border-bottom: 1px solid #e8e8e8;
  padding: 4px 16px;
}

.layout-main {
  flex: 1;
  display: flex;
  overflow: auto;
  min-height: 0;
}

.layout-split {
  display: flex;
  width: 100%;
}

.layout-aside {
  background: #f8fbff;
  border-right: 2px solid #dcdfe6;
  position: relative;
  transition: width 0.3s linear;
  display: flex;
  flex-direction: column;
}

.aside-collapsed {
  /* 收起状态的样式 */
}

.aside-toggle {
  position: absolute;
  right: 6px;
  top: 50%;
  transform: translateY(-25%);
  width: 14px;
  font-size: 12px;
  height: 80px;
  z-index: 9;
  text-align: center;
  transition: all 0.1s linear;
  cursor: pointer;
  user-select: none;
}

.toggle-btn {
  display: none;
}

.aside-toggle:hover .toggle-btn {
  display: block;
}

.aside-content {
  flex: 1;
  overflow: auto;
}

.layout-content {
  flex: 1;
  background: #fff;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.layout-footer {
  flex-shrink: 0;
  background-color: #d9e1ef;
  border-top: 1px solid #e8e8e8;
  padding: 6px 16px;
  display: flex;
  align-items: center;
}

.logo h2 {
  margin: 0;
  color: white;
  font-weight: 600;
  font-size: 18px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .layout-aside {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    z-index: 999;
    box-shadow: 2px 0 6px rgba(0, 0, 0, 0.1);
  }
  
  .layout-content {
    width: 100% !important;
  }
  
  .aside-collapsed {
    left: -100%;
  }
}
</style>
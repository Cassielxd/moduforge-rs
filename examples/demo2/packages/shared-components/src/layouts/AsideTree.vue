<template>
  <div class="aside-tree" :class="{ 'tree-collapsed': collapsed }">
    <!-- 树形结构内容 -->
    <div class="tree-content" v-if="!collapsed">
      <slot name="tree">
        <a-tree
          v-if="treeData && treeData.length"
          :tree-data="treeData"
          :selected-keys="selectedKeys"
          :expanded-keys="expandedKeys"
          :show-icon="showIcon"
          :show-line="showLine"
          :checkable="checkable"
          :draggable="draggable"
          :block-node="blockNode"
          @select="handleSelect"
          @expand="handleExpand"
          @drop="handleDrop"
          @dragstart="handleDragStart"
          @dragenter="handleDragEnter"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
          @dragend="handleDragEnd">
          <template #title="{ title, key, dataRef }">
            <slot name="title" :title="title" :key="key" :dataRef="dataRef">
              {{ title }}
            </slot>
          </template>
          <template #icon="{ key, dataRef }">
            <slot name="icon" :key="key" :dataRef="dataRef"></slot>
          </template>
        </a-tree>
        <div v-else class="tree-empty">
          <slot name="empty">
            <a-empty description="暂无数据" />
          </slot>
        </div>
      </slot>
    </div>

    <!-- 收起状态显示的内容 -->
    <div class="tree-collapsed-content" v-else>
      <slot name="collapsed">
        <!-- 可以显示一些图标或简化信息 -->
        <div class="collapsed-icons">
          <div 
            v-for="item in collapsedItems" 
            :key="item.key"
            class="collapsed-item"
            :class="{ active: selectedKeys.includes(item.key) }"
            @click="handleCollapsedItemClick(item)">
            <a-tooltip :title="item.title" placement="right">
              <div class="item-icon">
                <slot name="collapsed-icon" :item="item">
                  {{ item.title.charAt(0) }}
                </slot>
              </div>
            </a-tooltip>
          </div>
        </div>
      </slot>
    </div>

    <!-- 操作按钮区域 -->
    <div class="tree-actions" v-if="showActions">
      <slot name="actions">
        <div class="action-buttons">
          <a-tooltip title="刷新">
            <a-button 
              type="text" 
              size="small" 
              @click="handleRefresh"
              :icon="h(ReloadOutlined)" />
          </a-tooltip>
          <a-tooltip title="展开全部">
            <a-button 
              type="text" 
              size="small" 
              @click="handleExpandAll"
              :icon="h(ExpandOutlined)" />
          </a-tooltip>
          <a-tooltip title="收起全部">
            <a-button 
              type="text" 
              size="small" 
              @click="handleCollapseAll"
              :icon="h(ShrinkOutlined)" />
          </a-tooltip>
        </div>
      </slot>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, h } from 'vue'
import { 
  ReloadOutlined, 
  ExpandOutlined, 
  ShrinkOutlined 
} from '@ant-design/icons-vue'

// Props
const props = defineProps({
  treeData: {
    type: Array,
    default: () => []
  },
  selectedKeys: {
    type: Array,
    default: () => []
  },
  expandedKeys: {
    type: Array,
    default: () => []
  },
  collapsed: {
    type: Boolean,
    default: false
  },
  showIcon: {
    type: Boolean,
    default: true
  },
  showLine: {
    type: Boolean,
    default: false
  },
  checkable: {
    type: Boolean,
    default: false
  },
  draggable: {
    type: Boolean,
    default: false
  },
  blockNode: {
    type: Boolean,
    default: true
  },
  showActions: {
    type: Boolean,
    default: true
  }
})

// Emits
const emit = defineEmits([
  'select', 
  'expand', 
  'drop', 
  'dragstart', 
  'dragenter', 
  'dragover', 
  'dragleave', 
  'dragend',
  'refresh',
  'expandAll',
  'collapseAll'
])

// Computed
const collapsedItems = computed(() => {
  // 当收起时，显示的简化项目列表
  return props.treeData
    .filter(item => !item.children || item.level === 0) // 只显示顶级项目
    .slice(0, 8) // 最多显示8个
})

// Event handlers
const handleSelect = (selectedKeys, info) => {
  emit('select', selectedKeys, info)
}

const handleExpand = (expandedKeys, info) => {
  emit('expand', expandedKeys, info)
}

const handleDrop = (info) => {
  emit('drop', info)
}

const handleDragStart = (info) => {
  emit('dragstart', info)
}

const handleDragEnter = (info) => {
  emit('dragenter', info)
}

const handleDragOver = (info) => {
  emit('dragover', info)
}

const handleDragLeave = (info) => {
  emit('dragleave', info)
}

const handleDragEnd = (info) => {
  emit('dragend', info)
}

const handleCollapsedItemClick = (item) => {
  emit('select', [item.key], { node: item, selected: true })
}

const handleRefresh = () => {
  emit('refresh')
}

const handleExpandAll = () => {
  const allKeys = getAllKeys(props.treeData)
  emit('expandAll', allKeys)
}

const handleCollapseAll = () => {
  emit('collapseAll', [])
}

// Helper functions
const getAllKeys = (data, keys = []) => {
  data.forEach(item => {
    keys.push(item.key)
    if (item.children && item.children.length) {
      getAllKeys(item.children, keys)
    }
  })
  return keys
}

// Expose methods
defineExpose({
  handleRefresh,
  handleExpandAll,
  handleCollapseAll
})
</script>

<style scoped>
.aside-tree {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f8fbff;
}

.tree-content {
  flex: 1;
  overflow: auto;
  padding: 8px;
}

.tree-collapsed-content {
  flex: 1;
  padding: 8px 4px;
  overflow: hidden;
  min-height: 0;
}

.collapsed-icons {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.collapsed-item {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(255, 255, 255, 0.6);
  margin: 0 auto;
}

.collapsed-item:hover {
  background: rgba(24, 144, 255, 0.1);
  color: #1890ff;
}

.collapsed-item.active {
  background: #1890ff;
  color: white;
}

.item-icon {
  font-size: 12px;
  font-weight: 600;
  text-align: center;
}

.tree-actions {
  flex-shrink: 0;
  padding: 8px;
  border-top: 1px solid #e8e8e8;
  background: rgba(255, 255, 255, 0.8);
}

.action-buttons {
  display: flex;
  justify-content: center;
  gap: 4px;
}

.tree-collapsed .action-buttons {
  flex-direction: column;
  gap: 2px;
}

.tree-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #999;
}

/* Tree 样式覆盖 */
:deep(.ant-tree) {
  background: transparent;
  height: 100%;
  overflow: auto;
}

:deep(.ant-tree .ant-tree-node-content-wrapper) {
  border-radius: 4px;
  transition: all 0.2s;
}

:deep(.ant-tree .ant-tree-node-content-wrapper:hover) {
  background: rgba(24, 144, 255, 0.1);
}

:deep(.ant-tree .ant-tree-node-selected .ant-tree-node-content-wrapper) {
  background: #1890ff;
  color: white;
}

:deep(.ant-tree .ant-tree-treenode) {
  padding: 2px 0;
}

/* 滚动条样式 */
.tree-content::-webkit-scrollbar,
.tree-collapsed-content::-webkit-scrollbar {
  width: 4px;
}

.tree-content::-webkit-scrollbar-thumb,
.tree-collapsed-content::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 2px;
}

.tree-content::-webkit-scrollbar-track,
.tree-collapsed-content::-webkit-scrollbar-track {
  background: transparent;
}
</style>
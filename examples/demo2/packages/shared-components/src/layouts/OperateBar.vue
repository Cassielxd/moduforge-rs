<template>
  <div class="operate-bar">
    <div class="operate-content">
      <!-- 左侧操作按钮组 -->
      <div class="operate-left">
        <slot name="left">
          <a-space>
            <a-button-group v-if="showDefaultActions">
              <a-tooltip title="新建">
                <a-button 
                  type="primary" 
                  @click="handleAction('create')"
                  :icon="h(PlusOutlined)" />
              </a-tooltip>
              <a-tooltip title="编辑">
                <a-button 
                  @click="handleAction('edit')"
                  :disabled="!hasSelection"
                  :icon="h(EditOutlined)" />
              </a-tooltip>
              <a-tooltip title="删除">
                <a-button 
                  danger
                  @click="handleAction('delete')"
                  :disabled="!hasSelection"
                  :icon="h(DeleteOutlined)" />
              </a-tooltip>
            </a-button-group>

            <a-divider type="vertical" v-if="showDefaultActions" />

            <!-- 自定义操作按钮 -->
            <template v-for="action in actions" :key="action.key">
              <a-tooltip :title="action.tooltip || action.label">
                <a-button
                  :type="action.type || 'default'"
                  :danger="action.danger"
                  :disabled="action.disabled"
                  :loading="action.loading"
                  :icon="action.icon ? h(action.icon) : undefined"
                  @click="handleAction(action.key, action)"
                >
                  {{ action.hideLabel ? '' : action.label }}
                </a-button>
              </a-tooltip>
              <a-divider type="vertical" v-if="action.divider" />
            </template>
          </a-space>
        </slot>
      </div>

      <!-- 中间操作区域 -->
      <div class="operate-center">
        <slot name="center">
          <!-- 搜索框 -->
          <div class="search-area" v-if="showSearch">
            <a-input-search
              v-model:value="searchValue"
              :placeholder="searchPlaceholder"
              :style="{ width: searchWidth }"
              @search="handleSearch"
              @change="handleSearchChange"
            />
          </div>

          <!-- 筛选器 -->
          <div class="filter-area" v-if="showFilter">
            <a-space>
              <a-select
                v-for="filter in filters"
                :key="filter.key"
                v-model:value="filter.value"
                :placeholder="filter.placeholder"
                :style="{ width: filter.width || '120px' }"
                :options="filter.options"
                @change="handleFilterChange(filter.key, $event)"
                allowClear
              />
            </a-space>
          </div>
        </slot>
      </div>

      <!-- 右侧操作区域 -->
      <div class="operate-right">
        <slot name="right">
          <a-space>
            <!-- 视图切换 -->
            <div class="view-switcher" v-if="showViewSwitcher">
              <a-radio-group 
                v-model:value="currentView" 
                button-style="solid"
                size="small"
                @change="handleViewChange">
                <a-radio-button 
                  v-for="view in viewOptions" 
                  :key="view.value" 
                  :value="view.value">
                  <component :is="view.icon" v-if="view.icon" />
                  {{ view.label }}
                </a-radio-button>
              </a-radio-group>
            </div>

            <!-- 更多操作 -->
            <a-dropdown v-if="moreActions && moreActions.length">
              <a-button :icon="h(MoreOutlined)">
                更多
                <DownOutlined />
              </a-button>
              <template #overlay>
                <a-menu @click="handleMoreAction">
                  <a-menu-item 
                    v-for="action in moreActions" 
                    :key="action.key"
                    :disabled="action.disabled">
                    <component :is="action.icon" v-if="action.icon" />
                    {{ action.label }}
                  </a-menu-item>
                </a-menu>
              </template>
            </a-dropdown>

            <!-- 刷新按钮 -->
            <a-tooltip title="刷新" v-if="showRefresh">
              <a-button 
                @click="handleAction('refresh')"
                :loading="refreshLoading"
                :icon="h(ReloadOutlined)" />
            </a-tooltip>
          </a-space>
        </slot>
      </div>
    </div>

    <!-- 工具提示或状态信息 -->
    <div class="operate-status" v-if="statusText">
      <a-typography-text type="secondary" :style="{ fontSize: '12px' }">
        {{ statusText }}
      </a-typography-text>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, h } from 'vue'
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  SearchOutlined,
  ReloadOutlined,
  MoreOutlined,
  DownOutlined,
  TableOutlined,
  UnorderedListOutlined
} from '@ant-design/icons-vue'

// Props
const props = defineProps({
  // 基本配置
  showDefaultActions: {
    type: Boolean,
    default: true
  },
  hasSelection: {
    type: Boolean,
    default: false
  },
  actions: {
    type: Array,
    default: () => []
  },
  moreActions: {
    type: Array,
    default: () => []
  },

  // 搜索相关
  showSearch: {
    type: Boolean,
    default: true
  },
  searchPlaceholder: {
    type: String,
    default: '请输入搜索内容'
  },
  searchWidth: {
    type: String,
    default: '200px'
  },

  // 筛选相关
  showFilter: {
    type: Boolean,
    default: false
  },
  filters: {
    type: Array,
    default: () => []
  },

  // 视图切换
  showViewSwitcher: {
    type: Boolean,
    default: false
  },
  viewOptions: {
    type: Array,
    default: () => [
      { label: '表格', value: 'table', icon: TableOutlined },
      { label: '列表', value: 'list', icon: UnorderedListOutlined }
    ]
  },
  defaultView: {
    type: String,
    default: 'table'
  },

  // 其他
  showRefresh: {
    type: Boolean,
    default: true
  },
  refreshLoading: {
    type: Boolean,
    default: false
  },
  statusText: {
    type: String,
    default: ''
  }
})

// Emits
const emit = defineEmits([
  'action',
  'search', 
  'searchChange',
  'filterChange',
  'viewChange',
  'moreAction'
])

// State
const searchValue = ref('')
const currentView = ref(props.defaultView)

// Event handlers
const handleAction = (actionKey, actionData = null) => {
  emit('action', actionKey, actionData)
}

const handleSearch = (value) => {
  emit('search', value)
}

const handleSearchChange = (e) => {
  const value = e.target.value
  searchValue.value = value
  emit('searchChange', value)
}

const handleFilterChange = (filterKey, value) => {
  emit('filterChange', filterKey, value)
}

const handleViewChange = (e) => {
  const value = e.target.value
  currentView.value = value
  emit('viewChange', value)
}

const handleMoreAction = ({ key }) => {
  const action = props.moreActions.find(a => a.key === key)
  emit('moreAction', key, action)
}

// Watch
watch(() => props.defaultView, (newVal) => {
  currentView.value = newVal
})

// Expose
defineExpose({
  searchValue,
  currentView
})
</script>

<style scoped>
.operate-bar {
  background: #fff;
  border-bottom: 1px solid #e8e8e8;
  padding: 12px 16px 8px;
}

.operate-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.operate-left {
  flex-shrink: 0;
}

.operate-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
}

.operate-right {
  flex-shrink: 0;
}

.search-area,
.filter-area {
  display: flex;
  align-items: center;
}

.view-switcher {
  display: flex;
  align-items: center;
}

.operate-status {
  margin-top: 4px;
  padding-top: 4px;
  border-top: 1px solid #f0f0f0;
  text-align: center;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .operate-content {
    flex-direction: column;
    gap: 8px;
  }
  
  .operate-left,
  .operate-center,
  .operate-right {
    width: 100%;
  }
  
  .operate-center {
    justify-content: flex-start;
  }
  
  .search-area input {
    width: 100% !important;
  }
}

@media (max-width: 576px) {
  .operate-bar {
    padding: 8px 12px 4px;
  }
  
  .view-switcher :deep(.ant-radio-button-wrapper) {
    padding: 0 8px;
    font-size: 12px;
  }
  
  .search-area {
    width: 100%;
  }
}
</style>
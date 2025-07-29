<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { ElButton, ElTag, ElScrollbar } from 'element-plus';
import { Loading } from '@element-plus/icons-vue';

const props = defineProps();

const emit = defineEmits(['update:modelValue']);

const dropdownRef = ref(null);



const historyList = ref([]);
const loading = ref(false);

// 处理点击外部区域关闭菜单
const handleClickOutside = (event) => {
    if (dropdownRef.value && !dropdownRef.value.contains(event.target)) {
        emit('update:modelValue', false);
    }
};

// 模拟获取历史记录数据
const fetchHistory = async () => {
    loading.value = true;
    try {
        // TODO: 这里替换为实际的 IPC 调用
        // const response = await ipcRequest<HistoryItem[]>('get_history');
        // if (response.success) {
        //   historyList.value = response.data;
        // }

        // 临时使用模拟数据
        historyList.value = [
            { id: 1, title: '项目 A', timestamp: '2024-03-20 10:00:00', type: '创建' },
            { id: 2, title: '项目 B', timestamp: '2024-03-20 09:30:00', type: '修改' },
            { id: 3, title: '项目 C', timestamp: '2024-03-19 16:45:00', type: '删除' },
        ];
    } catch (error) {
        console.error('获取历史记录失败:', error);
    } finally {
        loading.value = false;
    }
};

// 监听 modelValue 变化
watch(() => props.modelValue, (newValue) => {
    if (newValue) {
        fetchHistory();
    }
}, { immediate: true });

onMounted(() => {
    document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside);
});

const handleClearHistory = async () => {
    try {
        // TODO: 实现清空历史记录的功能
        // await ipcRequest('clear_history');
        historyList.value = [];
    } catch (error) {
        console.error('清空历史记录失败:', error);
    }
};
</script>

<template>
    <div class="history-dropdown" ref="dropdownRef">
        <div class="history-dropdown-menu" v-show="modelValue" @click.stop>
            <div class="history-menu-header">
                <span class="title">历史记录</span>
                <el-button type="danger" size="small" @click="handleClearHistory">清空</el-button>
            </div>

            <div class="history-menu-content">
                <el-scrollbar max-height="300px">
                    <div v-if="loading" class="loading-container">
                        <el-icon class="is-loading">
                            <Loading />
                        </el-icon>
                        <span>加载中...</span>
                    </div>
                    <template v-else>
                        <div v-if="historyList.length === 0" class="empty-tip">
                            暂无历史记录
                        </div>
                        <div v-else class="history-list">
                            <div v-for="item in historyList" :key="item.id" class="history-item">
                                <div class="history-item-header">
                                    <span class="history-title">{{ item.title }}</span>
                                    <el-tag size="small"
                                        :type="item.type === '创建' ? 'success' : item.type === '修改' ? 'warning' : 'danger'">
                                        {{ item.type }}
                                    </el-tag>
                                </div>
                                <div class="history-time">{{ item.timestamp }}</div>
                            </div>
                        </div>
                    </template>
                </el-scrollbar>
            </div>
        </div>
    </div>
</template>

<style scoped>
.history-dropdown {
    position: relative;
    display: inline-block;
}

.history-dropdown-menu {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    width: 300px;
    background: white;
    border-radius: 4px;
    box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
    z-index: 1000;
}

.history-menu-header {
    padding: 12px 16px;
    border-bottom: 1px solid #e4e7ed;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.history-menu-header .title {
    font-size: 14px;
    font-weight: 500;
    color: #303133;
}

.history-menu-content {
    padding: 8px 0;
}

.loading-container {
    padding: 20px;
    text-align: center;
    color: #909399;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
}

.empty-tip {
    padding: 20px;
    text-align: center;
    color: #909399;
}

.history-list {
    padding: 0 12px;
}

.history-item {
    padding: 8px 0;
    border-bottom: 1px solid #f0f0f0;
}

.history-item:last-child {
    border-bottom: none;
}

.history-item-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
}

.history-title {
    font-size: 13px;
    color: #303133;
    flex: 1;
    margin-right: 8px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.history-time {
    font-size: 12px;
    color: #909399;
}
</style>
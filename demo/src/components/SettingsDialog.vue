<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage } from 'element-plus';
import { Delete } from '@element-plus/icons-vue';
import { open } from '@tauri-apps/plugin-dialog';
import { GlobalShortcut, useGlobalShortcutsStore } from '../stores/globalShortcuts';

const props = defineProps<{
    modelValue: boolean
}>();

const emit = defineEmits(['update:modelValue']);

const activeTab = ref('shortcuts');
const savePath = ref('');
const shortcutsStore = useGlobalShortcutsStore();
const editingShortcut = ref<string | null>(null);
const editingValue = ref('');
const newShortcut = ref({
    name: '',
    shortcut: '',
    enabled: true
});

onMounted(async () => {
    if (!shortcutsStore.isInitialized) {
        await shortcutsStore.initialize();
    }
});

const handleClose = () => {
    emit('update:modelValue', false);
};

const selectSavePath = async () => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: savePath.value,
        });
        if (selected) {
            savePath.value = selected as string;
        }
    } catch (error) {
        ElMessage.error('选择目录失败');
    }
};

const startEditing = (shortcut: any) => {
    editingShortcut.value = shortcut.id;
    editingValue.value = shortcut.shortcut;
};

const handleKeyPress = (event: Event) => {
    const keyboardEvent = event as KeyboardEvent;
    keyboardEvent.preventDefault();
    const keys = [];
    if (keyboardEvent.ctrlKey) keys.push('Ctrl');
    if (keyboardEvent.altKey) keys.push('Alt');
    if (keyboardEvent.shiftKey) keys.push('Shift');
    if (keyboardEvent.key !== 'Control' && keyboardEvent.key !== 'Alt' && keyboardEvent.key !== 'Shift') {
        keys.push(keyboardEvent.key.toUpperCase());
    }
    if (keys.length > 0) {
        editingValue.value = keys.join('+');
    }
};

const saveEdit = async (row: any) => {
    if (!editingValue.value) {
        ElMessage.warning('请输入快捷键组合');
        return;
    }
    await shortcutsStore.updateShortcut(row.id, editingValue.value);
    editingShortcut.value = null;
    editingValue.value = '';
    ElMessage.success('修改成功');
};

const cancelEdit = (row: any) => {
    editingShortcut.value = null;
    editingValue.value = '';
};

const handleBlur = (row: any) => {
    // 可选：失焦时自动取消编辑
    // cancelEdit(row);
};

const addNewShortcut = async () => {
    if (!newShortcut.value.name || !newShortcut.value.shortcut) {
        ElMessage.warning('请填写功能名称和快捷键组合');
        return;
    }
    try {
        await shortcutsStore.addShortcut({
            id: Date.now().toString(),
            name: newShortcut.value.name,
            shortcut: newShortcut.value.shortcut,
            enabled: newShortcut.value.enabled,
            handler: () => { }
        });
        newShortcut.value = {
            name: '',
            shortcut: '',
            enabled: true
        };
        ElMessage.success('添加成功');
    } catch (error) {
        console.error(error);
        ElMessage.error('添加失败');
    }
};

const handleNewShortcutKeyPress = (event: Event) => {
    const keyboardEvent = event as KeyboardEvent;
    keyboardEvent.preventDefault();
    const keys = [];
    if (keyboardEvent.ctrlKey) keys.push('Ctrl');
    if (keyboardEvent.altKey) keys.push('Alt');
    if (keyboardEvent.shiftKey) keys.push('Shift');
    if (keyboardEvent.key !== 'Control' && keyboardEvent.key !== 'Alt' && keyboardEvent.key !== 'Shift') {
        keys.push(keyboardEvent.key.toUpperCase());
    }
    if (keys.length > 0) {
        newShortcut.value.shortcut = keys.join('+');
    }
};

const deleteShortcut = async (id: string) => {
    try {
        await shortcutsStore.removeShortcut(id);
        ElMessage.success('删除成功');
    } catch (error) {
        ElMessage.error('删除失败');
    }
};

defineExpose({
    activeTab,
    savePath,
});
</script>

<template>
    <el-dialog :model-value="modelValue" @update:model-value="emit('update:modelValue', $event)" title="设置"
        width="600px" @close="handleClose">
        <el-tabs v-model="activeTab">
            <el-tab-pane label="快捷键" name="shortcuts">
                <div class="add-shortcut-form">
                    <el-form :model="newShortcut" label-width="80px">
                        <el-form-item label="功能名称">
                            <el-input v-model="newShortcut.name" placeholder="请输入功能名称" />
                        </el-form-item>
                        <el-form-item label="快捷键">
                            <el-input v-model="newShortcut.shortcut" @keydown="handleNewShortcutKeyPress"
                                placeholder="按下快捷键组合" />
                            <div class="shortcut-tip">仅支持 Windows，Mac 下请手动适配</div>
                        </el-form-item>
                        <el-form-item>
                            <el-button type="primary" @click="addNewShortcut">添加快捷键</el-button>
                        </el-form-item>
                    </el-form>
                </div>

                <el-table :data="shortcutsStore.shortcuts" style="width: 100%">
                    <el-table-column prop="name" label="功能" width="120" />
                    <el-table-column label="快捷键" width="320">
                        <template #default="{ row }">
                            <div class="shortcut-cell">
                                <template v-if="editingShortcut === row.id">
                                    <el-input v-model="editingValue" @keydown="handleKeyPress" placeholder="按下快捷键组合"
                                        style="width: 200px;" />
                                    <el-button type="primary" size="small" @click="() => saveEdit(row)">保存</el-button>
                                    <el-button size="small" @click="() => cancelEdit(row)">取消</el-button>
                                </template>
                                <template v-else>
                                    <div class="shortcut-display" @click="startEditing(row)">
                                        <span>{{ row.shortcut }}</span>
                                    </div>
                                </template>
                                <el-switch v-model="row.enabled" @change="() => shortcutsStore.toggleShortcut(row.id)"
                                    class="shortcut-switch" />
                            </div>
                        </template>
                    </el-table-column>
                    <el-table-column label="操作" width="60" align="center">
                        <template #default="{ row }">
                            <el-button type="danger" :icon="Delete" circle size="small" class="delete-icon-btn"
                                @click="deleteShortcut(row.id)" />
                        </template>
                    </el-table-column>
                </el-table>
            </el-tab-pane>

            <el-tab-pane label="文件设置" name="files">
                <div class="settings-item">
                    <span class="label">默认保存路径：</span>
                    <div class="path-input">
                        <el-input v-model="savePath" placeholder="请选择默认保存路径" readonly />
                        <el-button @click="selectSavePath">选择目录</el-button>
                    </div>
                </div>
            </el-tab-pane>
        </el-tabs>

        <template #footer>
            <span class="dialog-footer">
                <el-button @click="handleClose">取消</el-button>
                <el-button type="primary" @click="handleClose">确定</el-button>
            </span>
        </template>
    </el-dialog>
</template>

<style scoped>
.settings-item {
    margin-bottom: 20px;
}

.label {
    display: block;
    margin-bottom: 8px;
    color: #606266;
}

.path-input {
    display: flex;
    gap: 8px;
}

.path-input .el-input {
    flex: 1;
}

.shortcut-cell {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 44px;
    padding: 6px 0;
}

.shortcut-display {
    flex: 1;
    padding: 6px 12px;
    border: 1px solid #dcdfe6;
    border-radius: 4px;
    cursor: pointer;
    min-height: 36px;
    display: flex;
    align-items: center;
    background: #fafbfc;
}

.shortcut-display:hover {
    border-color: #409eff;
}

.shortcut-switch {
    flex-shrink: 0;
}

.add-shortcut-form {
    margin-bottom: 20px;
    padding: 20px;
    border: 1px solid #dcdfe6;
    border-radius: 4px;
    background-color: #f5f7fa;
}

.delete-icon-btn {
    margin-left: 0;
    margin-right: 0;
    padding: 0;
    min-width: 28px;
    min-height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.el-table .el-table__cell {
    padding-top: 8px;
    padding-bottom: 8px;
}

.shortcut-tip {
    color: #999;
    font-size: 12px;
    margin-top: 4px;
    margin-left: 2px;
}
</style>
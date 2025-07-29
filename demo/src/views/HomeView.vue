<script setup>
import { onMounted, ref, toRaw } from "vue";
import LeftTreePanel from "../components/LeftTreePanel.vue";
// @ts-ignore
import MultiTabView from "../components/MultiTabView.vue";

const multiTabRef = ref();

const treeData = ref([]);


const handleNodeSelected = (node) => {
  const id = toRaw(node.id);
  if (multiTabRef.value) {
    multiTabRef.value.refreshData(id);
  }
};

// 统计函数
const getTreeNodeCount = () => {
  const countNodes = (nodes) => {
    let count = 0;
    nodes.forEach((node) => {
      count++;
      if (node.children && node.children.length > 0) {
        count += countNodes(node.children);
      }
    });
    return count;
  };
  return countNodes(treeData.value);
};

const getCurrentTime = () => {
  const now = new Date();
  return now.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
};
</script>

<template>
  <div class="home-layout">
    <div class="home-content">
      <div class="left-panel">
        <LeftTreePanel :tree-data="treeData" @node-selected="handleNodeSelected" />
      </div>
      <div class="resizer"></div>
      <div class="right-panel">
        <MultiTabView ref="multiTabRef" />
      </div>
    </div>

    <!-- 底部统计信息 -->
    <div class="footer-stats">
      <div class="stats-container">
        <span class="stat-item">
          <span class="stat-label">项目节点:</span>
          <span class="stat-value">{{ getTreeNodeCount() }}</span>
        </span>
        <span class="stat-separator">|</span>
        <span class="stat-item">
          <span class="stat-label">活跃用户:</span>
          <span class="stat-value">3</span>
        </span>
        <span class="stat-separator">|</span>
        <span class="stat-item">
          <span class="stat-label">在线状态:</span>
          <span class="stat-value highlight">正常</span>
        </span>
        <span class="stat-separator">|</span>
        <span class="stat-item">
          <span class="stat-label">数据同步:</span>
          <span class="stat-value">已同步</span>
        </span>
        <span class="stat-separator">|</span>
        <span class="stat-item">
          <span class="stat-label">总预算:</span>
          <span class="stat-value highlight">¥15,876,543.20</span>
        </span>
        <span class="stat-separator">|</span>
        <span class="stat-item">
          <span class="stat-label">当前时间:</span>
          <span class="stat-value">{{ getCurrentTime() }}</span>
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.home-layout {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
}

.home-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
  gap: 8px;
  padding: 8px;
}

.left-panel {
  width: 25%;
  background: var(--el-bg-color);
  border-radius: 6px;
  border: 1px solid var(--el-border-color-light);
  box-shadow: var(--el-box-shadow-light);
}

.resizer {
  width: 5px;
  cursor: col-resize;
  background: var(--el-fill-color-light);
  border-radius: 2px;
  margin: 0 2px;
  transition: background 0.2s;
}

.resizer:hover {
  background: var(--el-fill-color);
}

.right-panel {
  width: 70%;
  background: var(--el-bg-color);
  border-radius: 6px;
  border: 1px solid var(--el-border-color-light);
  box-shadow: var(--el-box-shadow-light);
}

/* 底部统计信息 */
.footer-stats {
  height: 32px;
  flex-shrink: 0;
  background: var(--el-bg-color);
  border-top: 1px solid var(--el-border-color-light);
  border-radius: 0 0 6px 6px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  margin: 8px;
  margin-top: 0;
  box-shadow: var(--el-box-shadow-lighter);
}

.stats-container {
  display: flex;
  align-items: center;
  gap: 16px;
  white-space: nowrap;
  overflow: hidden;
}

.stat-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  color: var(--el-text-color-regular);
  font-weight: 500;
}

.stat-value {
  font-size: 12px;
  color: var(--el-text-color-primary);
  font-weight: 600;
}

.stat-value.highlight {
  color: var(--el-color-primary);
  font-weight: 700;
}

.stat-separator {
  color: var(--el-text-color-disabled);
  font-size: 12px;
  margin: 0 4px;
}

/* 暗色主题适配 */
@media (prefers-color-scheme: dark) {
  .home-layout {
    background: #212121;
  }

  .left-panel,
  .right-panel {
    background: #1a1a1a;
    border: 1px solid #2d2d2d;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .footer-stats {
    background: #1a1a1a;
    border-top: 1px solid #2d2d2d;
  }

  .resizer {
    background: #2d2d2d;
  }

  .resizer:hover {
    background: #3d3d3d;
  }
}
</style>

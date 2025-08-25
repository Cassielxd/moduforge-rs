<script setup lang="ts">
import { ref } from "vue";
import ProjectInfoView from "../views/ProjectInfoView.vue";
import FbfxView from "../views/FbfxView.vue";
import CxxmView from "../views/CxxmView.vue";

const activeTabName = ref("project-info"); // 默认显示项目信息

const fbfxRef = ref<{ init: (id: any) => void } | null>(null);
const cxxmRef = ref<{ init: (id: any) => void } | null>(null);
const projectInfoRef = ref<{ init: (id: any) => void } | null>(null);

// 记录当前选中的节点ID
const currentSelectedId = ref<string | number | null>(null);

// 处理标签页切换事件
const handleTabChange = (tabName: string | number) => {
  console.log(
    `MultiTabView: Tab changed to '${tabName}' with current ID:`,
    currentSelectedId.value
  );
  // 当切换到新标签页时，使用当前选中的ID初始化该标签页
  if (tabName === "project-info" && projectInfoRef.value) {
    projectInfoRef.value.init(currentSelectedId.value);
  } else if (tabName === "fbfx" && fbfxRef.value) {
    fbfxRef.value.init(currentSelectedId.value);
  } else if (tabName === "cxxm" && cxxmRef.value) {
    cxxmRef.value.init(currentSelectedId.value);
  }
};

// 暴露给父组件的刷新方法
const refreshData = (id: string | number | null) => {
  console.log(`MultiTabView: Refreshing tab '${activeTabName.value}' with ID:`, id);
  // 保存当前选中的ID
  currentSelectedId.value = id;

  if (activeTabName.value === "project-info" && projectInfoRef.value) {
    projectInfoRef.value.init(id);
  } else if (activeTabName.value === "fbfx" && fbfxRef.value) {
    fbfxRef.value.init(id);
  } else if (activeTabName.value === "cxxm" && cxxmRef.value) {
    cxxmRef.value.init(id);
  }
};

defineExpose({ refreshData });
</script>

<script lang="ts">
export default {
  name: "MultiTabView",
};
</script>

<template>
  <el-tabs v-model="activeTabName" type="border-card" @tab-change="handleTabChange">
    <el-tab-pane label="项目信息" name="project-info">
      <ProjectInfoView ref="projectInfoRef" />
    </el-tab-pane>
    <el-tab-pane label="分部分项" name="fbfx">
      <FbfxView ref="fbfxRef" />
    </el-tab-pane>
    <el-tab-pane label="措施项目" name="cxxm">
      <CxxmView ref="cxxmRef" />
    </el-tab-pane>
  </el-tabs>
</template>

<style scoped>
.el-tabs {
  height: 100%;
  width: 100%;
}
.el-tab-pane {
  padding: 16px;
  height: calc(100% - 40px);
  overflow-y: auto;
}
</style>

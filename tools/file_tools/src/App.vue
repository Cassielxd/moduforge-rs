<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";

type InspectResult = { kind: "mff"; data: MffSummary } | { kind: "zip"; data: ZipSummary };

interface MffSegment {
  index: number;
  kind: string;
  offset: number;
  record_length: number;
  payload_length: number;
  crc32: number;
  preview_json?: string | null;
}

interface MffSummary {
  path: string;
  file_name: string;
  file_size: number;
  logical_len: number;
  segment_count: number;
  directory_flags: number;
  file_hash: string;
  segments: MffSegment[];
}

interface ZipEntry {
  index: number;
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  compressed_size: number;
  compression: string;
  compression_ratio: number;
  crc32: number;
  modified?: string | null;
  preview_json?: string | null;
}

interface ZipSummary {
  path: string;
  file_name: string;
  file_size: number;
  total_entries: number;
  total_uncompressed: number;
  total_compressed: number;
  entries: ZipEntry[];
}

type TreeNodeType = "zip-root" | "zip-folder" | "zip-file" | "mff-root" | "mff-segment";

interface TreeNode {
  id: string;
  label: string;
  fullPath: string;
  type: TreeNodeType;
  meta?: unknown;
  children?: TreeNode[];
}

interface DetailTab {
  id: string;
  title: string;
  type: "segment" | "zip-entry";
  segment?: MffSegment;
  entry?: ZipEntry;
}

const result = ref<InspectResult | null>(null);
const loading = ref(false);
const treeNodes = computed<TreeNode[]>(() => {
  if (!result.value) return [];
  return result.value.kind === "zip"
    ? buildZipTree(result.value.data)
    : buildMffTree(result.value.data);
});
const selectedNode = ref<TreeNode | null>(null);
const currentNodeKey = ref<string>();
const detailTabs = ref<DetailTab[]>([]);
const activeTab = ref<string>("");

watch(
  () => result.value,
  () => {
    selectedNode.value = null;
    currentNodeKey.value = undefined;
    detailTabs.value = [];
    activeTab.value = "";
  }
);

watch(
  () => selectedNode.value,
  (node) => {
    currentNodeKey.value = node?.id;
  }
);

const treeTitle = computed(() => {
  if (!result.value) return "等待打开文件";
  return result.value.kind === "zip" ? "ZIP / YSF 文件树" : "MFF 段结构";
});

const isMff = computed(() => result.value?.kind === "mff");
const isZip = computed(() => result.value?.kind === "zip");
const mffSummary = computed(() => (result.value?.kind === "mff" ? result.value.data : null));
const zipSummary = computed(() => (result.value?.kind === "zip" ? result.value.data : null));

const mffSegments = computed(() => mffSummary.value?.segments ?? []);
const zipEntries = computed(() => zipSummary.value?.entries ?? []);

const headerInfo = computed(() => {
  if (!result.value) return null;
  const common = {
    name: result.value.data.file_name || "未命名",
    path: result.value.data.path,
    size: result.value.data.file_size,
  };
  if (result.value.kind === "mff") {
    return {
      ...common,
      typeLabel: "MFF 文档",
      extra: `${result.value.data.segment_count} 段`
    };
  }
  return {
    ...common,
    typeLabel: "ZIP / YSF 容器",
    extra: `${result.value.data.total_entries} 条目`
  };
});

async function selectFile() {
  try {
    const chosen = await open({
      title: "选择 MFF / YSF / ZIP 文件",
      multiple: false,
      filters: [
        {
          name: "文件",
          extensions: ["mff", "ysf", "zip"]
        }
      ]
    });
    if (typeof chosen !== "string") {
      return;
    }
    loading.value = true;
    const data = await invoke<InspectResult>("inspect_file", { path: chosen });
    result.value = data;
  } catch (error) {
    console.error(error);
    ElMessage.error("加载文件失败");
  } finally {
    loading.value = false;
  }
}

function handleNodeClick(data: TreeNode) {
  selectedNode.value = data;
  if (data.type === "mff-segment" && data.meta) {
    openSegmentTab(data.meta as MffSegment);
  } else if (data.type === "zip-file" && data.meta) {
    openZipEntryTab(data.meta as ZipEntry);
  }
}

const treeProps = {
  children: "children",
  label: "label"
};

function buildMffTree(summary: MffSummary): TreeNode[] {
  const rootId = `mff-root-${summary.path}`;
  const root: TreeNode = {
    id: rootId,
    label: summary.file_name || summary.path,
    fullPath: summary.path,
    type: "mff-root",
    children: summary.segments.map((segment) => ({
      id: `${rootId}-segment-${segment.index}`,
      label: `#${segment.index.toString().padStart(3, "0")} · ${segment.kind}`,
      fullPath: `${summary.path}::${segment.index}`,
      type: "mff-segment",
      meta: segment
    }))
  };
  return [root];
}

function buildZipTree(summary: ZipSummary): TreeNode[] {
  const normalize = (value: string) =>
    value.replace(/\\/g, "/").replace(/\/+$/, "").trim();
  const root: TreeNode = {
    id: `zip-root-${summary.path}`,
    label: summary.file_name || summary.path,
    fullPath: summary.path,
    type: "zip-root",
    children: []
  };
  const dirs = new Map<string, TreeNode>();
  dirs.set("", root);

  const ensureDir = (parts: string[]) => {
    let currentKey = "";
    let node = root;
    parts.forEach((part) => {
      currentKey = currentKey ? `${currentKey}/${part}` : part;
      if (!dirs.has(currentKey)) {
        const dirNode: TreeNode = {
          id: `zip-folder-${currentKey}`,
          label: part,
          fullPath: currentKey,
          type: "zip-folder",
          children: []
        };
        (node.children ??= []).push(dirNode);
        dirs.set(currentKey, dirNode);
      }
      node = dirs.get(currentKey)!;
    });
    return node;
  };

  const sorted = [...summary.entries].sort((a, b) =>
    a.path.localeCompare(b.path)
  );

  sorted.forEach((entry) => {
    const cleanPath = normalize(entry.path);
    const parts = cleanPath ? cleanPath.split("/") : [];
    if (entry.is_dir) {
      const dir = ensureDir(parts);
      dir.meta = entry;
      dir.label = entry.name || dir.label;
      return;
    }
    const parent = ensureDir(parts.slice(0, -1));
    const lastPart = parts.length > 0 ? parts[parts.length - 1] : entry.path;
    const node: TreeNode = {
      id: `zip-entry-${entry.index}`,
      label: entry.name || lastPart,
      fullPath: cleanPath || entry.name,
      type: "zip-file",
      meta: entry
    };
    (parent.children ??= []).push(node);
  });

  return [root];
}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes)) return "-";
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const base = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / 1024 ** base;
  return `${value.toFixed(2)} ${units[base]}`;
}

function percentage(value: number) {
  if (!Number.isFinite(value)) return "-";
  return `${(value * 100).toFixed(2)}%`;
}

function openSegmentTab(segment: MffSegment) {
  const id = `segment-${segment.index}`;
  const title = `段 ${segment.index.toString().padStart(3, "0")}`;
  const existing = detailTabs.value.find((tab) => tab.id === id);
  if (existing) {
    existing.segment = segment;
    activeTab.value = existing.id;
    return;
  }
  detailTabs.value.push({
    id,
    title,
    type: "segment",
    segment
  });
  activeTab.value = id;
}

function openZipEntryTab(entry: ZipEntry) {
  const id = `zip-entry-${entry.index}`;
  const title = entry.name || entry.path;
  const existing = detailTabs.value.find((tab) => tab.id === id);
  if (existing) {
    existing.entry = entry;
    activeTab.value = existing.id;
    return;
  }
  detailTabs.value.push({
    id,
    title,
    type: "zip-entry",
    entry
  });
  activeTab.value = id;
}

function removeTab(targetName: string) {
  const tabs = detailTabs.value;
  let active = activeTab.value;
  if (active === targetName) {
    const idx = tabs.findIndex((tab) => tab.id === targetName);
    if (idx > 0) {
      active = tabs[idx - 1].id;
    } else if (tabs.length > 1) {
      active = tabs[1].id;
    } else {
      active = "";
    }
  }
  detailTabs.value = tabs.filter((tab) => tab.id !== targetName);
  activeTab.value = active;
}

function handleSegmentRowClick(row: MffSegment) {
  openSegmentTab(row);
}

function handleZipRowClick(row: ZipEntry) {
  if (row.is_dir) return;
  openZipEntryTab(row);
}
</script>

<template>
  <el-container class="app-shell">
    <el-header class="app-header">
      <div class="header-left">
        <h1>ModuForge 文件管理</h1>
        <p>查看 .mff / .ysf / .zip 文件的段与条目信息</p>
      </div>
      <div class="header-right">
        <el-button type="primary" :loading="loading" @click="selectFile">
          打开文件
        </el-button>
        <div v-if="headerInfo" class="current-file">
          <el-tag effect="light">{{ headerInfo.typeLabel }}</el-tag>
          <div class="file-meta">
            <strong>{{ headerInfo.name }}</strong>
            <small>{{ headerInfo.path }}</small>
          </div>
          <span class="file-size">
            {{ formatBytes(headerInfo.size) }} · {{ headerInfo.extra }}
          </span>
        </div>
      </div>
    </el-header>

    <el-container class="app-body">
      <el-aside width="280px" class="tree-pane">
        <el-card shadow="never" class="tree-card">
          <template #header>
            <div class="tree-card__title">
              <span>{{ treeTitle }}</span>
              <el-tag size="small" v-if="loading">加载中</el-tag>
            </div>
          </template>
          <div class="tree-wrapper">
            <el-empty v-if="!treeNodes.length" description="尚未选择文件" />
            <el-tree
              v-else
              :data="treeNodes"
              :props="treeProps"
              :expand-on-click-node="false"
              :highlight-current="true"
              node-key="id"
              :current-node-key="currentNodeKey"
              default-expand-all
              @node-click="handleNodeClick"
            />
          </div>
        </el-card>
      </el-aside>
      <el-main class="content-pane">
        <template v-if="result">
          <el-space direction="vertical" :size="16" style="width: 100%">
            <el-card shadow="never">
              <template #header>
                <div class="panel-title">
                  概览与{{ isMff ? "段列表" : "条目列表" }}
                </div>
              </template>
              <div class="overview-block">
                <el-descriptions :column="3" border size="small">
                  <el-descriptions-item label="文件名">
                    {{ result.data.file_name || "未命名" }}
                  </el-descriptions-item>
                  <el-descriptions-item label="路径">
                    {{ result.data.path }}
                  </el-descriptions-item>
                  <el-descriptions-item label="大小">
                    {{ formatBytes(result.data.file_size) }}
                  </el-descriptions-item>
                  <template v-if="isMff && mffSummary">
                    <el-descriptions-item label="段数量">
                      {{ mffSummary.segment_count }}
                    </el-descriptions-item>
                    <el-descriptions-item label="逻辑长度">
                      {{ formatBytes(mffSummary.logical_len) }}
                    </el-descriptions-item>
                    <el-descriptions-item label="BLAKE3 哈希">
                      <code>{{ mffSummary.file_hash }}</code>
                    </el-descriptions-item>
                  </template>
                  <template v-else-if="isZip && zipSummary">
                    <el-descriptions-item label="条目数量">
                      {{ zipSummary.total_entries }}
                    </el-descriptions-item>
                    <el-descriptions-item label="原始体积">
                      {{ formatBytes(zipSummary.total_uncompressed) }}
                    </el-descriptions-item>
                    <el-descriptions-item label="压缩体积">
                      {{ formatBytes(zipSummary.total_compressed) }}
                    </el-descriptions-item>
                  </template>
                </el-descriptions>
              </div>

              <div v-if="isMff" class="table-wrap">
                <el-table
                  :data="mffSegments"
                  border
                  size="small"
                  height="320"
                  empty-text="当前文档没有段"
                  @row-click="handleSegmentRowClick"
                >
                  <el-table-column prop="index" label="#" width="60" />
                  <el-table-column prop="kind" label="类型" min-width="140" />
                  <el-table-column label="偏移" min-width="120">
                    <template #default="{ row }">
                      {{ row.offset }}
                    </template>
                  </el-table-column>
                  <el-table-column label="记录长度" min-width="120">
                    <template #default="{ row }">
                      {{ formatBytes(row.record_length) }}
                    </template>
                  </el-table-column>
                  <el-table-column label="有效载荷" min-width="120">
                    <template #default="{ row }">
                      {{ formatBytes(row.payload_length) }}
                    </template>
                  </el-table-column>
                  <el-table-column prop="crc32" label="CRC32" width="120" />
                </el-table>
              </div>

              <div v-else class="table-wrap">
                <el-table
                  :data="zipEntries"
                  border
                  size="small"
                  height="320"
                  empty-text="归档中没有条目"
                  @row-click="handleZipRowClick"
                >
                  <el-table-column label="路径" min-width="200">
                    <template #default="{ row }">
                      <span v-if="row.is_dir">📁 {{ row.path }}</span>
                      <span v-else>📄 {{ row.path }}</span>
                    </template>
                  </el-table-column>
                  <el-table-column label="原始大小" min-width="120">
                    <template #default="{ row }">
                      {{ formatBytes(row.size) }}
                    </template>
                  </el-table-column>
                  <el-table-column label="压缩后" min-width="120">
                    <template #default="{ row }">
                      {{ formatBytes(row.compressed_size) }}
                    </template>
                  </el-table-column>
                  <el-table-column label="压缩率" min-width="100">
                    <template #default="{ row }">
                      {{ percentage(row.compression_ratio) }}
                    </template>
                  </el-table-column>
                  <el-table-column prop="compression" label="算法" min-width="120" />
                </el-table>
              </div>
            </el-card>

            <el-tabs
              v-if="detailTabs.length"
              v-model="activeTab"
              type="card"
              closable
              @tab-remove="removeTab"
            >
              <el-tab-pane
                v-for="tab in detailTabs"
                :key="tab.id"
                :name="tab.id"
                :label="tab.title"
              >
                <el-card shadow="never">
                  <template #header>
                    <div class="panel-title">
                      {{ tab.type === "segment" ? "段详情" : "条目详情" }}
                    </div>
                  </template>
                  <template v-if="tab.type === 'segment' && tab.segment">
                    <el-descriptions :column="2" border size="small">
                      <el-descriptions-item label="类型">
                        {{ tab.segment.kind }}
                      </el-descriptions-item>
                      <el-descriptions-item label="偏移">
                        {{ tab.segment.offset }}
                      </el-descriptions-item>
                      <el-descriptions-item label="记录长度">
                        {{ formatBytes(tab.segment.record_length) }}
                      </el-descriptions-item>
                      <el-descriptions-item label="有效载荷">
                        {{ formatBytes(tab.segment.payload_length) }}
                      </el-descriptions-item>
                      <el-descriptions-item label="CRC32">
                        {{ tab.segment.crc32 }}
                      </el-descriptions-item>
                    </el-descriptions>
                    <pre
                      v-if="tab.segment.preview_json"
                      class="json-preview"
                    >{{ tab.segment.preview_json }}</pre>
                    <el-empty
                      v-else
                      description="该段不是 JSON 或内容过大，无法预览"
                    />
                  </template>
                  <template v-else-if="tab.type === 'zip-entry' && tab.entry">
                    <el-descriptions :column="2" border size="small">
                      <el-descriptions-item label="路径">
                        {{ tab.entry.path }}
                      </el-descriptions-item>
                      <el-descriptions-item label="修改时间">
                        {{ tab.entry.modified ?? "未知" }}
                      </el-descriptions-item>
                      <el-descriptions-item label="原始大小">
                        {{ formatBytes(tab.entry.size) }}
                      </el-descriptions-item>
                      <el-descriptions-item label="压缩后">
                        {{ formatBytes(tab.entry.compressed_size) }}
                      </el-descriptions-item>
                      <el-descriptions-item label="压缩率">
                        {{ percentage(tab.entry.compression_ratio) }}
                      </el-descriptions-item>
                      <el-descriptions-item label="压缩算法">
                        {{ tab.entry.compression }}
                      </el-descriptions-item>
                      <el-descriptions-item label="CRC32">
                        {{ tab.entry.crc32 }}
                      </el-descriptions-item>
                    </el-descriptions>
                    <pre
                      v-if="tab.entry.preview_json"
                      class="json-preview"
                    >{{ tab.entry.preview_json }}</pre>
                    <el-empty
                      v-else
                      description="该条目不是 JSON 或内容过大，无法预览"
                    />
                  </template>
                </el-card>
              </el-tab-pane>
            </el-tabs>
          </el-space>
        </template>
        <el-empty v-else description="请选择一个文件" />
      </el-main>
    </el-container>
  </el-container>
</template>

<style scoped>
.app-shell {
  height: 100vh;
  background: #f5f6fb;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 24px;
  border-bottom: 1px solid #ebeef5;
  background: #fff;
}

.header-left h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #303133;
}

.header-left p {
  margin: 2px 0 0;
  color: #909399;
  font-size: 13px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.current-file {
  display: flex;
  align-items: center;
  gap: 12px;
  background: #f9fafc;
  border: 1px solid #ebeef5;
  border-radius: 8px;
  padding: 8px 12px;
}

.file-meta {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.file-meta strong {
  font-size: 14px;
  color: #303133;
}

.file-meta small {
  color: #a0a3ad;
  max-width: 280px;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
}

.file-size {
  color: #606266;
  font-size: 13px;
}

.app-body {
  height: calc(100vh - 72px);
}

.tree-pane {
  padding: 16px;
  background: #f5f6fb;
  border-right: 1px solid #ebeef5;
}

.tree-card__title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #303133;
}

.tree-wrapper {
  min-height: calc(100vh - 160px);
  overflow: auto;
}

.content-pane {
  padding: 16px 24px;
  background: #f8f9ff;
}

.panel-title {
  font-weight: 600;
  color: #303133;
  display: flex;
  gap: 8px;
  align-items: baseline;
}

.panel-title small {
  color: #c0c4cc;
  font-weight: 400;
  font-size: 12px;
}

code {
  background: #f2f6fc;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 12px;
}

.json-preview {
  background: #0f172a;
  color: #e2e8f0;
  padding: 12px;
  border-radius: 8px;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 360px;
  overflow: auto;
}

.overview-block {
  margin-bottom: 16px;
}

.table-wrap {
  border: 1px solid #ebeef5;
  border-radius: 6px;
  overflow: hidden;
}
</style>

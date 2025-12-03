<script setup lang="ts">
import {computed, ref, watch} from "vue";
import {open} from "@tauri-apps/plugin-dialog";
import {invoke} from "@tauri-apps/api/core";
import {ElMessage} from "element-plus";

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
  loading?: boolean;
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
const OVERVIEW_TAB_ID = "overview";
const detailTabs = ref<DetailTab[]>([]);
const activeTab = ref<string>(OVERVIEW_TAB_ID);
const segmentDetailCache = new Map<number, MffSegment>();

watch(
    () => result.value,
    () => {
      selectedNode.value = null;
      currentNodeKey.value = undefined;
      detailTabs.value = [];
      activeTab.value = OVERVIEW_TAB_ID;
      segmentDetailCache.clear();
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
    size: result.value.data.file_size
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

const overviewTabLabel = computed(() => {
  if (!result.value) return "概览与列表";
  return `概览与${isMff.value ? "段列表" : "条目列表"}`;
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
    const data = await invoke<InspectResult>("inspect_file", {path: chosen});
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
  if (data.type === "mff-root" || data.type === "zip-root") {
    activeTab.value = OVERVIEW_TAB_ID;
    return;
  }
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
  const title = `段 #${segment.index.toString().padStart(3, "0")}`;
  let tab = detailTabs.value.find((item) => item.id === id);
  const cached = segmentDetailCache.get(segment.index);
  if (!tab) {
    tab = {
      id,
      title,
      type: "segment",
      loading: false,
      segment: cached ?? segment
    };
    detailTabs.value.push(tab);
  }
  activeTab.value = id;
  if (tab.type !== "segment") {
    return;
  }
  if (cached) {
    tab.segment = cached;
    tab.loading = false;
    return;
  }
  tab.loading = true;
  ensureSegmentDetail(id, segment.index);
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
  if (targetName === OVERVIEW_TAB_ID) return;
  const tabs = detailTabs.value;
  let active = activeTab.value;
  if (active === targetName) {
    const idx = tabs.findIndex((tab) => tab.id === targetName);
    if (idx > 0) {
      active = tabs[idx - 1].id;
    } else {
      active = tabs[idx + 1]?.id ?? OVERVIEW_TAB_ID;
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

async function fetchSegmentDetail(index: number) {
  if (segmentDetailCache.has(index)) {
    return segmentDetailCache.get(index)!;
  }
  const summary = mffSummary.value;
  if (!summary) return null;
  try {
    const detail = await invoke<MffSegment>("load_mff_segment", {
      path: summary.path,
      index
    });
    debugger;
    segmentDetailCache.set(index, detail);
    return detail;
  } catch (error) {
    console.error(error);
    ElMessage.error("加载段详情失败");
    return null;
  }
}

async function ensureSegmentDetail(tabId: string, index: number) {
  const detail = await fetchSegmentDetail(index);
  if (!detail) {
    const tab = detailTabs.value.find((t) => t.id === tabId);
    if (tab) {
      tab.loading = false;
    }
    return;
  }
  const tab = detailTabs.value.find((t) => t.id === tabId);
  if (tab && tab.type === "segment") {
    tab.segment = detail;
    tab.loading = false;
  }
  if (selectedNode.value?.type === "mff-segment") {
    const meta = selectedNode.value.meta as MffSegment | undefined;
    if (meta?.index === index) {
      selectedNode.value = {
        ...selectedNode.value,
        meta: detail
      };
    }
  }
}
</script>

<template>
  <div class="navicat-shell">
    <header class="navicat-header">
      <div class="logo-block">
        <div class="logo-mark">MF</div>
        <div class="logo-copy">
          <span class="logo-title">ModuForge Inspector</span>
          <span class="logo-desc">Navicat 风格的段与条目分析工作台</span>
        </div>
      </div>
      <div class="header-actions">
        <div v-if="headerInfo" class="file-pill">
          <el-tag size="small" effect="dark">{{ headerInfo.typeLabel }}</el-tag>
          <div class="file-pill__meta">
            <strong>{{ headerInfo.name }}</strong>
            <small>{{ headerInfo.path }}</small>
            <span>{{ formatBytes(headerInfo.size) }} · {{ headerInfo.extra }}</span>
          </div>
        </div>
        <el-button type="primary" :loading="loading" @click="selectFile">
          打开文件
        </el-button>
      </div>
    </header>

    <div class="navicat-body">
      <aside class="navicat-sidebar">
        <button class="sidebar-icon active" type="button">☰</button>
        <button class="sidebar-icon" type="button">⇪</button>
        <button class="sidebar-icon" type="button">⛃</button>
        <button class="sidebar-icon" type="button">⚙</button>
      </aside>

      <main class="navicat-workbench">
        <section class="tree-pane navicat-pane">
          <header class="pane-header">
            <div>
              <p class="pane-title">{{ treeTitle }}</p>
              <small v-if="headerInfo">{{ headerInfo.path }}</small>
            </div>
            <el-tag v-if="loading" size="small" effect="dark">加载中</el-tag>
          </header>
          <div class="tree-scroll">
            <el-empty v-if="!treeNodes.length" description="尚未选择文件"/>
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
        </section>

        <section class="detail-pane navicat-pane">
          <template v-if="result">
            <el-tabs
                v-model="activeTab"
                type="card"
                class="navicat-tabs"
                @tab-remove="removeTab"
            >
              <el-tab-pane
                  :label="overviewTabLabel"
                  :name="OVERVIEW_TAB_ID"
                  :closable="false"
              >
                <div class="overview-card">
                  <div class="overview-meta">
                    <div>
                      <p class="overview-label">当前文件</p>
                      <h2>{{ result.data.file_name || "未命名" }}</h2>
                      <span>{{ result.data.path }}</span>
                    </div>
                    <div class="overview-size">
                      <p>大小</p>
                      <strong>{{ formatBytes(result.data.file_size) }}</strong>
                    </div>
                  </div>

                  <div class="overview-grid" v-if="isMff && mffSummary">
                    <div class="overview-card__item">
                      <label>段数量</label>
                      <strong>{{ mffSummary.segment_count }}</strong>
                    </div>
                    <div class="overview-card__item">
                      <label>逻辑长度</label>
                      <strong>{{ formatBytes(mffSummary.logical_len) }}</strong>
                    </div>
                    <div class="overview-card__item">
                      <label>BLAKE3 哈希</label>
                      <code>{{ mffSummary.file_hash }}</code>
                    </div>
                  </div>

                  <div class="overview-grid" v-else-if="isZip && zipSummary">
                    <div class="overview-card__item">
                      <label>条目数量</label>
                      <strong>{{ zipSummary.total_entries }}</strong>
                    </div>
                    <div class="overview-card__item">
                      <label>原始体积</label>
                      <strong>{{ formatBytes(zipSummary.total_uncompressed) }}</strong>
                    </div>
                    <div class="overview-card__item">
                      <label>压缩体积</label>
                      <strong>{{ formatBytes(zipSummary.total_compressed) }}</strong>
                    </div>
                  </div>
                </div>

                <div v-if="isMff" class="table-wrap">
                  <el-table
                      :data="mffSegments"
                      border
                      size="small"
                      height="380"
                      empty-text="当前文档没有段"
                      @row-click="handleSegmentRowClick"
                  >
                    <el-table-column prop="index" label="#" width="60"/>
                    <el-table-column prop="kind" label="类型" min-width="140"/>
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
                    <el-table-column prop="crc32" label="CRC32" width="120"/>
                  </el-table>
                </div>

                <div v-else class="table-wrap">
                  <el-table
                      :data="zipEntries"
                      border
                      size="small"
                      height="380"
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
                    <el-table-column label="压缩体积" min-width="120">
                      <template #default="{ row }">
                        {{ formatBytes(row.compressed_size) }}
                      </template>
                    </el-table-column>
                    <el-table-column label="压缩率" min-width="100">
                      <template #default="{ row }">
                        {{ percentage(row.compression_ratio) }}
                      </template>
                    </el-table-column>
                    <el-table-column prop="compression" label="算法" min-width="120"/>
                  </el-table>
                </div>
              </el-tab-pane>

              <el-tab-pane
                  v-for="tab in detailTabs"
                  :key="tab.id"
                  :name="tab.id"
                  :label="tab.title"
                  closable
              >
                <div class="detail-card">
                  <p class="detail-card__title">
                    {{ tab.type === "segment" ? "段详情" : "条目详情" }}
                  </p>
                  <template v-if="tab.type === 'segment'">
                    <el-skeleton
                        v-if="tab.loading || !tab.segment"
                        animated
                        :rows="5"
                    />
                    <template v-else>
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
                      <el-descriptions-item label="压缩体积">
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
                </div>
              </el-tab-pane>
            </el-tabs>
          </template>
          <el-empty v-else description="请选择一个文件"/>
        </section>
      </main>
    </div>
  </div>
</template>

<style scoped>
:global(body) {
  margin: 0;
  font-family: "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
  background: #f3f6fb;
  color: #1f2d3d;
}

.navicat-shell {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.navicat-header {
  height: 64px;
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: linear-gradient(135deg, #ffffff 0%, #eef2f7 100%);
  border-bottom: 1px solid #e6eaf3;
  box-shadow: 0 4px 16px rgba(15, 23, 42, 0.08);
}

.logo-block {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-mark {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: linear-gradient(135deg, #3da6ff 0%, #1d6bff 100%);
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  letter-spacing: 0.3px;
}

.logo-copy {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.logo-title {
  font-size: 16px;
  font-weight: 700;
  color: #1f2937;
}

.logo-desc {
  font-size: 12px;
  color: #5b6478;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 14px;
}

.file-pill {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  border-radius: 12px;
  background: #f6f8fe;
  border: 1px solid #dbe4f6;
  max-width: 480px;
}

.file-pill__meta {
  display: flex;
  flex-direction: column;
  font-size: 12px;
  gap: 2px;
  color: #2f3b52;
}

.file-pill__meta strong {
  font-size: 14px;
  color: #1f2d3d;
}

.file-pill__meta small {
  color: #6b7280;
}

.file-pill__meta span {
  color: #1d6bff;
}

.navicat-body {
  flex: 1;
  display: flex;
  background: linear-gradient(180deg, #f5f7fb 0%, #eef2f7 55%, #e9edf4 100%);
}

.navicat-sidebar {
  width: 64px;
  background: #f6f7fb;
  border-right: 1px solid #e3e7ef;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 0;
  gap: 10px;
}

.sidebar-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: transparent;
  border: none;
  color: #5c6473;
  cursor: pointer;
  font-size: 18px;
  transition: all 0.18s ease;
}

.sidebar-icon.active,
.sidebar-icon:hover {
  background: #e8f0ff;
  color: #1d4ed8;
}

.navicat-workbench {
  flex: 1;
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 16px;
  padding: 16px;
  overflow: hidden;
}

.navicat-pane {
  background: #fff;
  border-radius: 14px;
  border: 1px solid #e6eaf3;
  box-shadow: 0 12px 30px rgba(15, 23, 42, 0.12);
  min-height: 0;
}

.tree-pane {
  display: flex;
  flex-direction: column;
}

.detail-pane {
  display: flex;
  flex-direction: column;
  padding: 12px 14px;
  overflow: hidden;
}

.pane-header {
  padding: 12px 14px 8px;
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.pane-title {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: #1f2d3d;
}

.pane-header small {
  color: #6b7280;
}

.tree-scroll {
  flex: 1;
  padding: 0 12px 12px 12px;
  overflow: auto;
}

:deep(.el-tree) {
  background: transparent;
  color: #2f3b52;
}

:deep(.el-tree-node__content:hover),
:deep(.el-tree-node.is-current > .el-tree-node__content) {
  background: #e8f0ff;
}

.navicat-tabs {
  --el-color-primary: #1d6bff;
}

:deep(.navicat-tabs .el-tabs__header) {
  margin: 0 0 12px;
}

:deep(.navicat-tabs .el-tabs__item) {
  color: #4b5563;
}

:deep(.navicat-tabs .el-tabs__item.is-active) {
  color: #1d6bff;
}

.overview-card {
  background: #f7f9fc;
  border: 1px solid #e6eaf3;
  border-radius: 14px;
  padding: 16px;
  margin-bottom: 12px;
}

.overview-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  margin-bottom: 12px;
}

.overview-label {
  margin: 0;
  font-size: 12px;
  color: #6b7280;
}

.overview-meta h2 {
  margin: 4px 0;
  font-size: 18px;
  color: #111827;
}

.overview-meta span {
  color: #6b7280;
}

.overview-size strong {
  font-size: 24px;
  font-weight: 600;
  color: #1d6bff;
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
  gap: 12px;
}

.overview-card__item {
  padding: 10px 12px;
  border-radius: 12px;
  border: 1px solid #e6eaf3;
  background: #fff;
}

.overview-card__item label {
  display: block;
  color: #6b7280;
  font-size: 12px;
}

.overview-card__item strong {
  font-size: 15px;
  color: #1f2d3d;
}

.table-wrap {
  border: 1px solid #e6eaf3;
  border-radius: 12px;
  overflow: hidden;
}

:deep(.table-wrap .el-table) {
  --el-table-border-color: #e5e7eb;
  --el-table-border: 1px solid #e5e7eb;
  --el-table-text-color: #1f2937;
  --el-table-header-text-color: #4b5563;
  --el-table-bg-color: #fff;
  --el-table-tr-bg-color: #fff;
  --el-table-row-hover-bg-color: #f5f7fb;
}

.detail-card {
  background: #fff;
  border-radius: 12px;
  border: 1px solid #e6eaf3;
  padding: 14px;
}

.detail-card__title {
  margin: 0 0 10px;
  font-weight: 700;
  color: #1f2d3d;
}

.json-preview {
  background: #0f172a;
  color: #e8f2ff;
  padding: 10px;
  border-radius: 10px;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
  margin-top: 10px;
  overflow: auto;
}

:deep(.detail-card .el-descriptions__cell) {
  background: transparent;
  color: #1f2937;
}

:deep(.detail-card .el-empty__description p) {
  color: #6b7280;
}

:deep(.el-empty__description p) {
  color: #6b7280;
}
</style>

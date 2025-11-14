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

watch(
  () => result.value,
  () => {
    selectedNode.value = null;
    currentNodeKey.value = undefined;
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
const selectedSegment = computed<MffSegment | null>(() => {
  if (!isMff.value) return null;
  if (selectedNode.value?.type !== "mff-segment") return null;
  return selectedNode.value.meta as MffSegment;
});

const selectedZipEntry = computed<ZipEntry | null>(() => {
  if (!isZip.value) return null;
  if (
    selectedNode.value?.type !== "zip-file" &&
    selectedNode.value?.type !== "zip-folder"
  ) {
    return null;
  }
  return selectedNode.value.meta as ZipEntry;
});

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
                <div class="panel-title">概览</div>
              </template>
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
            </el-card>

            <el-card v-if="isMff && mffSummary" shadow="never">
              <template #header>
                <div class="panel-title">
                  段列表
                  <small>双击树节点可定位到该段</small>
                </div>
              </template>
              <el-table
                :data="mffSegments"
                border
                size="small"
                height="320"
                empty-text="当前文档没有段"
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
            </el-card>

            <el-card v-if="selectedSegment" shadow="never">
              <template #header>
                <div class="panel-title">
                  段详情 · {{ selectedSegment.kind }} (#{{ selectedSegment.index }})
                </div>
              </template>
              <el-descriptions :column="2" border size="small">
                <el-descriptions-item label="偏移">
                  {{ selectedSegment.offset }}
                </el-descriptions-item>
                <el-descriptions-item label="记录长度">
                  {{ formatBytes(selectedSegment.record_length) }}
                </el-descriptions-item>
                <el-descriptions-item label="有效载荷">
                  {{ formatBytes(selectedSegment.payload_length) }}
                </el-descriptions-item>
                <el-descriptions-item label="CRC32">
                  {{ selectedSegment.crc32 }}
                </el-descriptions-item>
              </el-descriptions>
            </el-card>

            <el-card v-if="isZip && selectedZipEntry" shadow="never">
              <template #header>
                <div class="panel-title">
                  条目详情 · {{ selectedZipEntry.name }}
                </div>
              </template>
              <el-descriptions :column="2" border size="small">
                <el-descriptions-item label="路径">
                  {{ selectedZipEntry.path }}
                </el-descriptions-item>
                <el-descriptions-item label="修改时间">
                  {{ selectedZipEntry.modified ?? "未知" }}
                </el-descriptions-item>
                <el-descriptions-item label="原始大小">
                  {{ formatBytes(selectedZipEntry.size) }}
                </el-descriptions-item>
                <el-descriptions-item label="压缩后">
                  {{ formatBytes(selectedZipEntry.compressed_size) }}
                </el-descriptions-item>
                <el-descriptions-item label="压缩率">
                  {{ percentage(selectedZipEntry.compression_ratio) }}
                </el-descriptions-item>
                <el-descriptions-item label="压缩算法">
                  {{ selectedZipEntry.compression }}
                </el-descriptions-item>
                <el-descriptions-item label="CRC32">
                  {{ selectedZipEntry.crc32 }}
                </el-descriptions-item>
              </el-descriptions>
            </el-card>
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
</style>

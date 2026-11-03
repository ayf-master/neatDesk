<template>
  <div class="history">
    <div class="page-header">
      <h2 class="page-title">整理历史</h2>
      <div class="header-actions" v-if="records.length > 0">
        <button class="btn btn-secondary btn-sm" @click="confirmClearAll">
          <Trash2Icon :size="14" />
          清空全部
        </button>
      </div>
    </div>

    <div v-if="loading" class="empty-state">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>

    <div v-else-if="records.length > 0" class="history-list">
      <div v-for="record in records" :key="record.id" class="card history-card">
        <div class="history-main">
          <div class="history-icon">
            <CheckCircleIcon :size="20" :class="record.undone ? 'icon-undone' : 'icon-done'" />
          </div>
          <div class="history-content">
            <div class="history-header">
              <span class="history-time">{{ record.organized_at }}</span>
              <span class="history-count">{{ record.total_files }} 个文件</span>
            </div>
            <div v-if="record.categories && record.categories.length > 0" class="history-categories">
              <span v-for="cat in record.categories" :key="cat.name" class="cat-tag">
                {{ cat.name }}: {{ cat.count }}
              </span>
            </div>
            <span v-if="record.undone" class="undone-badge">已撤销</span>
          </div>
        </div>
        <div class="history-actions">
          <button v-if="!record.undone" class="btn btn-danger btn-sm" @click="confirmUndo(record)">
            撤销本次整理
          </button>
          <button class="btn btn-secondary btn-sm" @click="confirmDelete(record)">
            <Trash2Icon :size="13" />
            删除记录
          </button>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <ClockIcon :size="40" />
      <p>暂无整理记录</p>
      <p class="empty-hint">点击首页的「立即整理」开始</p>
    </div>

    <!-- 撤销确认 -->
    <ConfirmDialog
      :show="showUndoConfirm"
      title="撤销整理"
      :message="`确定要撤销 ${undoTarget?.organized_at} 的整理操作吗？`"
      :details="undoTarget ? [`该次整理了 ${undoTarget.total_files} 个文件`] : undefined"
      confirm-text="确认撤销"
      variant="danger"
      :loading="undoLoading"
      loading-text="撤销中..."
      @confirm="handleUndo"
      @cancel="showUndoConfirm = false"
    />

    <!-- 删除确认 -->
    <ConfirmDialog
      :show="showDeleteConfirm"
      title="删除记录"
      message="确定要删除这条整理记录吗？这不会影响已整理的文件，仅删除历史记录。"
      confirm-text="删除"
      variant="danger"
      :loading="deleteLoading"
      @confirm="handleDelete"
      @cancel="showDeleteConfirm = false"
    />

    <!-- 清空确认 -->
    <ConfirmDialog
      :show="showClearConfirm"
      title="清空全部历史"
      message="确定要清空所有整理历史记录吗？此操作不可恢复。"
      confirm-text="清空全部"
      variant="danger"
      :loading="clearLoading"
      @confirm="handleClearAll"
      @cancel="showClearConfirm = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { CheckCircleIcon, ClockIcon, Trash2Icon } from "lucide-vue-next";
import { api } from "../lib/api";
import type { HistoryRecord } from "../lib/types";
import ConfirmDialog from "../components/ConfirmDialog.vue";
import { useToast } from "../composables/useToast";

const { success, error } = useToast();

const records = ref<HistoryRecord[]>([]);
const loading = ref(true);

// 撤销
const showUndoConfirm = ref(false);
const undoTarget = ref<HistoryRecord | null>(null);
const undoLoading = ref(false);

// 删除
const showDeleteConfirm = ref(false);
const deleteTarget = ref<HistoryRecord | null>(null);
const deleteLoading = ref(false);

// 清空
const showClearConfirm = ref(false);
const clearLoading = ref(false);

async function loadHistory() {
  loading.value = true;
  try {
    records.value = (await api.getHistory()) as HistoryRecord[];
  } catch (e) {
    error("加载历史记录失败");
    console.error(e);
  } finally {
    loading.value = false;
  }
}

// 撤销
function confirmUndo(record: HistoryRecord) {
  undoTarget.value = record;
  showUndoConfirm.value = true;
}

async function handleUndo() {
  if (!undoTarget.value) return;
  undoLoading.value = true;
  try {
    const msg = (await api.undoOrganize(undoTarget.value.id)) as string;
    const record = records.value.find((r) => r.id === undoTarget.value!.id);
    if (record) record.undone = true;
    success(msg);
    showUndoConfirm.value = false;
  } catch (e: any) {
    error(typeof e === "string" ? e : "撤销失败");
  } finally {
    undoLoading.value = false;
  }
}

// 删除
function confirmDelete(record: HistoryRecord) {
  deleteTarget.value = record;
  showDeleteConfirm.value = true;
}

async function handleDelete() {
  if (!deleteTarget.value) return;
  deleteLoading.value = true;
  try {
    await api.deleteHistoryRecord(deleteTarget.value.id);
    records.value = records.value.filter((r) => r.id !== deleteTarget.value!.id);
    success("记录已删除");
    showDeleteConfirm.value = false;
  } catch (e: any) {
    error(typeof e === "string" ? e : "删除失败");
  } finally {
    deleteLoading.value = false;
  }
}

// 清空
function confirmClearAll() {
  showClearConfirm.value = true;
}

async function handleClearAll() {
  clearLoading.value = true;
  try {
    await api.clearHistory();
    records.value = [];
    success("已清空全部历史记录");
    showClearConfirm.value = false;
  } catch (e: any) {
    error(typeof e === "string" ? e : "清空失败");
  } finally {
    clearLoading.value = false;
  }
}

onMounted(loadHistory);
</script>

<style scoped>
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}

.page-header .page-title {
  margin-bottom: 0;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-card {
  padding: 16px 20px;
}

.history-main {
  display: flex;
  align-items: flex-start;
  gap: 14px;
}

.history-icon {
  flex-shrink: 0;
  margin-top: 2px;
}

.icon-done {
  color: var(--color-success);
}

.icon-undone {
  color: var(--text-secondary);
  opacity: 0.35;
}

.history-content {
  flex: 1;
  min-width: 0;
}

.history-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.history-time {
  font-size: 14px;
  font-weight: 500;
}

.history-count {
  font-size: 12px;
  color: var(--text-secondary);
  background-color: var(--bg-hover);
  padding: 2px 10px;
  border-radius: 999px;
}

.history-categories {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 4px;
}

.cat-tag {
  padding: 2px 8px;
  border-radius: 4px;
  background-color: var(--bg-hover);
  font-size: 12px;
  color: var(--text-secondary);
}

.undone-badge {
  display: inline-block;
  font-size: 11px;
  color: var(--text-secondary);
  background-color: var(--bg-hover);
  padding: 2px 10px;
  border-radius: 999px;
}

.history-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--border-color);
}

.btn-sm {
  padding: 6px 14px;
  font-size: 13px;
}

.empty-state {
  text-align: center;
  padding: 64px 24px;
  color: var(--text-secondary);
}

.empty-state svg {
  margin-bottom: 12px;
  opacity: 0.3;
}

.empty-hint {
  font-size: 13px;
  margin-top: 8px;
  opacity: 0.7;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid var(--border-color);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
  margin: 0 auto 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>

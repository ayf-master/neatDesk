<template>
  <div class="home">
    <h2 class="page-title">首页</h2>

    <div class="summary-cards">
      <div class="card summary-card">
        <div class="summary-icon">
          <MonitorIcon :size="24" />
        </div>
        <div class="summary-info">
          <span class="summary-label">桌面文件</span>
          <span class="summary-value">{{ desktopSummary.total_files }} 个</span>
        </div>
      </div>
      <div class="card summary-card">
        <div class="summary-icon">
          <FolderTreeIcon :size="24" />
        </div>
        <div class="summary-info">
          <span class="summary-label">分类数</span>
          <span class="summary-value">{{ categoryCount }} 类</span>
        </div>
      </div>
      <div class="card summary-card">
        <div class="summary-icon">
          <CalendarIcon :size="24" />
        </div>
        <div class="summary-info">
          <span class="summary-label">上次整理</span>
          <span class="summary-value">{{ lastOrganized || "暂无" }}</span>
        </div>
      </div>
    </div>

    <!-- 桌面文件按分类分布 -->
    <div v-if="desktopSummary.categories && desktopSummary.categories.length > 0" class="card category-dist-card">
      <h3 class="section-title">桌面文件分布</h3>
      <div class="category-bar-list">
        <div v-for="cat in desktopSummary.categories" :key="cat.id" class="category-bar-row">
          <span class="bar-label">{{ cat.name }}</span>
          <div class="bar-track">
            <div
              class="bar-fill"
              :style="{ width: (cat.count / desktopSummary.total_files * 100) + '%' }"
            ></div>
          </div>
          <span class="bar-count">{{ cat.count }} 个</span>
        </div>
      </div>
    </div>

    <div class="action-area">
      <button class="btn btn-primary btn-large" @click="handleOrganize" :disabled="isOrganizing">
        <RefreshCwIcon :size="20" :class="{ spinning: isOrganizing }" />
        {{ isOrganizing ? "整理中..." : "立即整理" }}
      </button>
    </div>

    <div v-if="organizeResult" class="card result-card">
      <div class="result-header">
        <CheckCircleIcon :size="20" class="result-icon" />
        <span>整理完成</span>
      </div>
      <div class="result-stats">
        <span class="stat-success">成功 {{ organizeResult.success_count }} 个</span>
        <span v-if="organizeResult.failed_count > 0" class="stat-failed">
          失败 {{ organizeResult.failed_count }} 个
        </span>
      </div>
      <div class="result-categories">
        <span v-for="cat in organizeResult.categories" :key="cat.name" class="result-tag">
          {{ cat.name }}: {{ cat.count }}
        </span>
      </div>
      <div v-if="organizeResult.errors && organizeResult.errors.length > 0" class="result-errors">
        <p class="error-title">失败详情：</p>
        <div v-for="err in organizeResult.errors" :key="err.file_name" class="error-item">
          <span class="error-file">{{ err.file_name }}</span>
          <span class="error-reason">{{ err.reason }}</span>
        </div>
      </div>
      <div v-if="organizeResult.compact_log" class="compact-log">
        <p class="compact-log-title">桌面图标补齐日志：</p>
        <pre class="compact-log-content">{{ organizeResult.compact_log }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { MonitorIcon, FolderTreeIcon, CalendarIcon, RefreshCwIcon, CheckCircleIcon } from "lucide-vue-next";
import { api } from "../lib/api";
import type { AppConfig, HistoryRecord, OrganizeResult } from "../lib/types";
import { useToast } from "../composables/useToast";

const { success, error } = useToast();

interface CategorySummary {
  id: string;
  name: string;
  icon: string;
  count: number;
}

interface DesktopOverview {
  total_files: number;
  categories: CategorySummary[];
}

const desktopSummary = reactive<DesktopOverview>({
  total_files: 0,
  categories: [],
});

const categoryCount = ref(0);
const lastOrganized = ref("");
const isOrganizing = ref(false);
const organizeResult = ref<OrganizeResult | null>(null);

async function loadData() {
  // 加载配置（获取分类数）
  try {
    const config = (await api.getConfig()) as AppConfig;
    categoryCount.value = config.categories.filter((c) => c.enabled).length;
  } catch (e) {
    console.error("加载配置失败:", e);
  }

  // 加载桌面文件分布（扫描+分类，不移动）
  try {
    const overview = (await api.categorizeDesktop()) as DesktopOverview;
    desktopSummary.total_files = overview.total_files;
    desktopSummary.categories = overview.categories;
  } catch {
    // 安静失败
  }

  // 加载上次整理时间
  try {
    const history = (await api.getHistory()) as HistoryRecord[];
    if (history.length > 0) {
      const last = history.find((r) => !r.undone) || history[0];
      lastOrganized.value = last.organized_at;
    }
  } catch {
    // 安静失败
  }
}

async function handleOrganize() {
  isOrganizing.value = true;
  organizeResult.value = null;
  try {
    const res = await api.organizeDesktop() as OrganizeResult;
    organizeResult.value = res;
    lastOrganized.value = new Date().toLocaleString("zh-CN");
    success(`整理完成：成功 ${res.success_count} 个${res.failed_count > 0 ? `，${res.failed_count} 个失败` : ""}`);
    // 整理后重新加载概况
    await loadData();
  } catch (e: any) {
    error(typeof e === "string" ? e : "整理失败，请重试");
  } finally {
    isOrganizing.value = false;
  }
}

onMounted(loadData);
</script>

<style scoped>
.summary-cards {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.summary-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
}

.summary-icon {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-md);
  background-color: var(--bg-hover);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
  flex-shrink: 0;
}

.summary-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.summary-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.summary-value {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-primary);
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 16px;
}

.category-dist-card {
  margin-bottom: 20px;
}


.category-bar-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.category-bar-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.bar-label {
  width: 60px;
  font-size: 13px;
  color: var(--text-primary);
  flex-shrink: 0;
}

.bar-track {
  flex: 1;
  height: 8px;
  background-color: var(--bg-hover);
  border-radius: 4px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  background-color: var(--color-primary);
  border-radius: 4px;
  transition: width 0.4s ease;
  min-width: 0;
}

.bar-count {
  width: 48px;
  font-size: 12px;
  color: var(--text-secondary);
  text-align: right;
  flex-shrink: 0;
}

.action-area {
  margin-bottom: 20px;
}

.btn-large {
  padding: 12px 32px;
  font-size: 15px;
}

.result-card {
  margin-top: 0;
}

.result-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  font-size: 16px;
  font-weight: 600;
}

.result-icon {
  color: var(--color-success);
}

.result-categories {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 12px;
}

.result-tag {
  padding: 4px 12px;
  border-radius: 999px;
  background-color: var(--bg-hover);
  font-size: 13px;
  color: var(--text-secondary);
}

.result-stats {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  font-size: 14px;
}

.stat-success {
  color: var(--color-success);
  font-weight: 500;
}

.stat-failed {
  color: var(--color-danger);
  font-weight: 500;
}

.result-errors {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.error-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-danger);
  margin-bottom: 8px;
}

.error-item {
  display: flex;
  gap: 8px;
  padding: 4px 0;
  font-size: 13px;
}

.error-file {
  font-family: monospace;
  color: var(--text-primary);
  min-width: 120px;
}

.error-reason {
  color: var(--text-secondary);
}

.compact-log {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.compact-log-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.compact-log-content {
  font-family: monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-secondary);
  background-color: var(--bg-hover);
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>

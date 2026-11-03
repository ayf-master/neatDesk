<template>
  <div class="categories">
    <div class="page-header">
      <h2 class="page-title">分类管理</h2>
      <button class="btn btn-primary" @click="openAddDialog">
        <PlusIcon :size="16" />
        新增分类
      </button>
    </div>

    <p class="page-desc">设置文件扩展名对应的分类。整理时文件将按规则移入对应名称的文件夹。启用的分类才会参与整理。</p>

    <!-- 加载状态 -->
    <div v-if="loading" class="empty-state">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>

    <!-- 分类列表 -->
    <div v-else-if="categories.length > 0" class="categories-list">
      <div v-for="cat in categories" :key="cat.id" class="card category-card">
        <div class="category-main">
          <div class="category-icon-wrapper">
            <span class="category-emoji">{{ getIconEmoji(cat.icon) }}</span>
          </div>
          <div class="category-content">
            <div class="category-header">
              <span class="category-name">{{ cat.name }}</span>
              <span v-if="!cat.enabled" class="disabled-badge">已禁用</span>
              <span v-if="cat.id === 'others'" class="builtin-badge">内置</span>
            </div>
            <div class="category-extensions">
              <span v-if="cat.extensions.length === 0" class="ext-empty">匹配所有未分类文件</span>
              <span v-for="ext in cat.extensions" :key="ext" class="ext-tag">{{ ext }}</span>
            </div>
            <div v-if="cat.name_patterns && cat.name_patterns.length > 0" class="category-patterns">
              <span class="pattern-label">文件名含：</span>
              <span v-for="p in cat.name_patterns" :key="p" class="pattern-tag">{{ p }}</span>
            </div>
            <div class="category-target">
              <span>→</span>
              <span v-if="cat.target_path" class="target-path">{{ cat.target_path }}</span>
              <span v-else class="target-path target-default">桌面</span>
            </div>
          </div>
        </div>
        <div class="category-actions">
          <button class="btn btn-secondary btn-sm" @click="toggleEnabled(cat.id)">
            {{ cat.enabled ? "禁用" : "启用" }}
          </button>
          <button class="btn btn-secondary btn-sm" @click="openEditDialog(cat)">编辑</button>
          <button
            v-if="cat.id !== 'others'"
            class="btn btn-danger btn-sm"
            @click="confirmDelete(cat)"
          >
            删除
          </button>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <p>暂无分类，点击"新增分类"创建</p>
    </div>

    <!-- 编辑对话框 -->
    <CategoryDialog
      :show="showDialog"
      :category="editingCategory"
      @close="showDialog = false"
      @save="handleSave"
    />

    <!-- 确认删除对话框 -->
    <ConfirmDialog
      :show="showDeleteConfirm"
      title="删除分类"
      :message="`确定要删除「${deletingCategory?.name}」分类吗？`"
      confirm-text="删除"
      variant="danger"
      @confirm="handleDelete"
      @cancel="showDeleteConfirm = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { PlusIcon } from "lucide-vue-next";
import { api } from "../lib/api";
import type { Category, AppConfig } from "../lib/types";
import CategoryDialog from "../components/CategoryDialog.vue";
import ConfirmDialog from "../components/ConfirmDialog.vue";
import { useToast } from "../composables/useToast";

const { success, error } = useToast();

const categories = ref<Category[]>([]);
const showDialog = ref(false);
const editingCategory = ref<Category | null>(null);
const showDeleteConfirm = ref(false);
const deletingCategory = ref<Category | null>(null);
const loading = ref(true);

const iconMap: Record<string, string> = {
  Image: "🖼️", FileText: "📄", Video: "🎬",
  Music: "🎵", Archive: "📦", Terminal: "⚙️", FolderOpen: "📁",
};

function getIconEmoji(icon: string): string {
  return iconMap[icon] || "📁";
}

async function loadConfig() {
  loading.value = true;
  try {
    const config = (await api.getConfig()) as AppConfig;
    categories.value = config.categories;
  } catch (e) {
    error("加载配置失败");
    console.error(e);
  } finally {
    loading.value = false;
  }
}

async function saveConfig() {
  try {
    const config = (await api.getConfig()) as AppConfig;
    config.categories = categories.value;
    await api.updateConfig(config);
  } catch (e) {
    error("保存失败");
    console.error(e);
    throw e;
  }
}

function openAddDialog() {
  editingCategory.value = null;
  showDialog.value = true;
}

function openEditDialog(cat: Category) {
  editingCategory.value = { ...cat };
  showDialog.value = true;
}

async function handleSave(catData: Omit<Category, "id"> & { id?: string }) {
  showDialog.value = false;
  const cat: Category = {
    id: catData.id || catData.name.toLowerCase().replace(/\s+/g, "-"),
    name: catData.name,
    icon: catData.icon,
    extensions: catData.extensions,
    name_patterns: catData.name_patterns || [],
    target_path: catData.target_path || "",
    enabled: true,
  };

  try {
    if (editingCategory.value) {
      const idx = categories.value.findIndex((c) => c.id === editingCategory.value!.id);
      if (idx >= 0) categories.value[idx] = cat;
    } else {
      categories.value.push(cat);
    }
    await saveConfig();
    success(editingCategory.value ? "分类已更新" : "分类已添加");
  } catch {
    error("操作失败");
  }
}

async function toggleEnabled(catId: string) {
  const cat = categories.value.find((c) => c.id === catId);
  if (!cat) return;
  cat.enabled = !cat.enabled;
  try {
    await saveConfig();
    success(cat.enabled ? "已启用" : "已禁用");
  } catch {
    cat.enabled = !cat.enabled;
  }
}

function confirmDelete(cat: Category) {
  deletingCategory.value = cat;
  showDeleteConfirm.value = true;
}

async function handleDelete() {
  if (!deletingCategory.value) return;
  const catId = deletingCategory.value.id;
  showDeleteConfirm.value = false;
  categories.value = categories.value.filter((c) => c.id !== catId);
  try {
    await saveConfig();
    success("分类已删除");
  } catch {
    error("删除失败");
  }
}

onMounted(loadConfig);
</script>

<style scoped>
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.page-header .page-title {
  margin-bottom: 0;
}

.page-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 20px;
}

.categories-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.category-card {
  padding: 16px 20px;
}

.category-main {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  margin-bottom: 12px;
}

.category-icon-wrapper {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background-color: var(--bg-hover);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.category-emoji {
  font-size: 20px;
}

.category-content {
  flex: 1;
  min-width: 0;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.category-name {
  font-size: 15px;
  font-weight: 600;
}

.disabled-badge, .builtin-badge {
  font-size: 11px;
  padding: 1px 8px;
  border-radius: 999px;
}

.disabled-badge {
  color: var(--text-secondary);
  background-color: var(--bg-hover);
}

.builtin-badge {
  color: var(--color-primary);
  background-color: color-mix(in srgb, var(--color-primary) 15%, transparent);
}

.category-extensions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.ext-tag {
  padding: 2px 8px;
  border-radius: 4px;
  background-color: var(--bg-hover);
  font-size: 12px;
  font-family: monospace;
  color: var(--text-secondary);
}

.ext-empty {
  font-size: 12px;
  color: var(--text-secondary);
  font-style: italic;
}

.category-patterns {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 4px;
}

.pattern-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-right: 2px;
}

.pattern-tag {
  padding: 1px 8px;
  border-radius: 4px;
  background-color: color-mix(in srgb, var(--color-primary) 12%, transparent);
  font-size: 12px;
  color: var(--color-primary);
}

.category-target {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-secondary);
}

.target-path {
  font-family: monospace;
  padding: 1px 6px;
  border-radius: 3px;
  background-color: var(--bg-hover);
}

.target-default {
  color: var(--text-secondary);
  font-style: italic;
}

.category-actions {
  display: flex;
  gap: 8px;
  padding-top: 10px;
  border-top: 1px solid var(--border-color);
}

.btn-sm {
  padding: 6px 14px;
  font-size: 13px;
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

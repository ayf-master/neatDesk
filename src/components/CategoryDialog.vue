<template>
  <div class="dialog-overlay" v-if="show" @click.self="$emit('close')">
    <div class="dialog-card card">
      <h3 class="dialog-title">{{ isEditing ? "编辑分类" : "新增分类" }}</h3>

      <div class="form-group">
        <label>分类名称</label>
        <input v-model="form.name" type="text" class="input-text" placeholder="如：图片、文档" />
      </div>

      <div class="form-group">
        <label>文件扩展名</label>
        <p class="form-hint">每行一个扩展名，如 .jpg .png</p>
        <textarea v-model="extensionsText" class="input-textarea" rows="4" placeholder=".jpg
.png
.gif"></textarea>
      </div>

      <div class="form-group">
        <label>文件名关键词</label>
        <p class="form-hint">文件名包含这些字的文件归入此类，每行一个关键词</p>
        <textarea v-model="namePatternsText" class="input-textarea" rows="3" placeholder="报告
合同
photo"></textarea>
      </div>

      <div class="form-group">
        <label>目标路径（可选）</label>
        <p class="form-hint">留空则整理到桌面，可指定如 D:\图片</p>
        <input v-model="targetPathText" type="text" class="input-text" placeholder="留空=桌面" />
      </div>

      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="$emit('close')">取消</button>
        <button class="btn btn-primary" @click="handleSave" :disabled="!form.name.trim()">
          {{ isEditing ? "保存修改" : "添加分类" }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import type { Category } from "../lib/types";

const props = defineProps<{
  show: boolean;
  category: Category | null;
}>();

const emit = defineEmits<{
  close: [];
  save: [category: Omit<Category, "id"> & { id?: string }];
}>();

const form = ref({ id: "", name: "", icon: "FolderOpen", extensions: [] as string[], name_patterns: [] as string[], target_path: "" as string, enabled: true });
const extensionsText = ref("");
const namePatternsText = ref("");
const targetPathText = ref("");

const isEditing = ref(false);

watch(
  () => props.show,
  (val) => {
    if (val) {
      if (props.category) {
        isEditing.value = true;
        form.value = { ...props.category };
        extensionsText.value = props.category.extensions.join("\n");
        namePatternsText.value = props.category.name_patterns.join("\n");
        targetPathText.value = props.category.target_path || "";
      } else {
        isEditing.value = false;
        form.value = { id: "", name: "", icon: "FolderOpen", extensions: [], name_patterns: [], target_path: "", enabled: true };
        extensionsText.value = "";
        namePatternsText.value = "";
        targetPathText.value = "";
      }
    }
  }
);

function handleSave() {
  const exts = extensionsText.value
    .split("\n")
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
    .map((s) => (s.startsWith(".") ? s : `.${s}`));

  const patterns = namePatternsText.value
    .split("\n")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const payload = {
    ...form.value,
    extensions: exts,
    name_patterns: patterns,
    target_path: targetPathText.value.trim(),
    // 新增时自动生成 id
    ...(isEditing.value ? {} : { id: form.value.name.toLowerCase().replace(/\s+/g, "-") }),
  };

  emit("save", payload);
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog-card {
  width: 420px;
  padding: 28px;
}

.dialog-title {
  font-size: 18px;
  font-weight: 700;
  margin-bottom: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 6px;
}

.form-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.input-text {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: var(--bg-card);
  color: var(--text-primary);
  box-sizing: border-box;
}

.input-text:focus {
  outline: none;
  border-color: var(--color-primary);
}

.input-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-family: monospace;
  background: var(--bg-card);
  color: var(--text-primary);
  resize: vertical;
  box-sizing: border-box;
}

.input-textarea:focus {
  outline: none;
  border-color: var(--color-primary);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}
</style>

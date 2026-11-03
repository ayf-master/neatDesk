<template>
  <div class="dialog-overlay" v-if="show" @click.self="handleCancel">
    <div class="dialog-card card">
      <h3 class="dialog-title">{{ title }}</h3>
      <p class="dialog-message">{{ message }}</p>

      <div v-if="details" class="dialog-details">
        <p v-for="(line, i) in details" :key="i">{{ line }}</p>
      </div>

      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="handleCancel">{{ cancelText }}</button>
        <button
          class="btn"
          :class="confirmClass"
          @click="handleConfirm"
          :disabled="loading"
        >
          {{ loading ? loadingText : confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(defineProps<{
  show: boolean;
  title: string;
  message: string;
  details?: string[];
  confirmText?: string;
  cancelText?: string;
  variant?: "danger" | "primary";
  loading?: boolean;
  loadingText?: string;
}>(), {
  confirmText: "确认",
  cancelText: "取消",
  variant: "primary",
  loading: false,
  loadingText: "处理中...",
});

const emit = defineEmits<{
  confirm: [];
  cancel: [];
}>();

const confirmClass = computed(() =>
  props.variant === "danger" ? "btn-danger" : "btn-primary"
);

function handleConfirm() {
  emit("confirm");
}

function handleCancel() {
  if (!props.loading) {
    emit("cancel");
  }
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
  animation: fadeIn 0.15s ease;
}

.dialog-card {
  width: 380px;
  padding: 28px;
  animation: slideUp 0.2s ease;
}

.dialog-title {
  font-size: 17px;
  font-weight: 700;
  margin-bottom: 12px;
}

.dialog-message {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.dialog-details {
  margin-top: 12px;
  padding: 12px;
  background-color: var(--bg-hover);
  border-radius: var(--radius-sm);
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.8;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>

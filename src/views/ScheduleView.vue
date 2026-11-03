<template>
  <div class="schedule">
    <h2 class="page-title">定时整理</h2>

    <div class="card">
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">启用定时整理</span>
          <span class="setting-desc">开启后，桌面将按照设定时间自动整理</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="enabled" />
          <span class="toggle-slider"></span>
        </label>
      </div>

      <div v-if="enabled" class="schedule-options">
        <div class="radio-group">
          <label class="radio-item">
            <input type="radio" v-model="scheduleType" value="interval" />
            <span>间隔</span>
          </label>
          <label class="radio-item">
            <input type="radio" v-model="scheduleType" value="daily" />
            <span>每天</span>
          </label>
          <label class="radio-item">
            <input type="radio" v-model="scheduleType" value="weekly" />
            <span>每周</span>
          </label>
        </div>

        <div v-if="scheduleType === 'interval'" class="schedule-detail">
          <label>每</label>
          <input type="number" v-model.number="intervalHours" min="1" class="input-number" />
          <label>小时整理一次</label>
        </div>

        <div v-if="scheduleType === 'daily'" class="schedule-detail">
          <label>每天</label>
          <input type="time" v-model="dailyTime" class="input-time" />
          <label>整理</label>
        </div>

        <div v-if="scheduleType === 'weekly'" class="schedule-detail">
          <label>每周</label>
          <select v-model="weeklyDay" class="input-select">
            <option value="Monday">周一</option>
            <option value="Tuesday">周二</option>
            <option value="Wednesday">周三</option>
            <option value="Thursday">周四</option>
            <option value="Friday">周五</option>
            <option value="Saturday">周六</option>
            <option value="Sunday">周日</option>
          </select>
          <input type="time" v-model="weeklyTime" class="input-time" />
          <label>整理</label>
        </div>
      </div>

      <div class="save-area">
        <button class="btn btn-primary" @click="saveSchedule">保存设置</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { api } from "../lib/api";
import type { AppConfig } from "../lib/types";
import { useToast } from "../composables/useToast";

const { success, error } = useToast();

const enabled = ref(false);
const scheduleType = ref("interval");
const intervalHours = ref(6);
const dailyTime = ref("09:00");
const weeklyDay = ref("Monday");
const weeklyTime = ref("09:00");

async function loadConfig() {
  try {
    const config = (await api.getConfig()) as AppConfig;
    enabled.value = config.schedule.enabled;
    scheduleType.value = config.schedule.type;
    intervalHours.value = config.schedule.interval_hours;
    dailyTime.value = config.schedule.daily_time;
    weeklyDay.value = config.schedule.weekly_day;
    weeklyTime.value = config.schedule.weekly_time;
  } catch (e) {
    console.error("加载定时配置失败:", e);
  }
}

async function saveSchedule() {
  try {
    const config = (await api.getConfig()) as AppConfig;
    config.schedule.enabled = enabled.value;
    config.schedule.type = scheduleType.value as "interval" | "daily" | "weekly";
    config.schedule.interval_hours = intervalHours.value;
    config.schedule.daily_time = dailyTime.value;
    config.schedule.weekly_day = weeklyDay.value;
    config.schedule.weekly_time = weeklyTime.value;
    await api.updateConfig(config);
    success("定时设置已保存");
  } catch (e) {
    error("保存定时配置失败");
    console.error(e);
  }
}

onMounted(loadConfig);
</script>

<style scoped>
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 20px;
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-label {
  font-size: 15px;
  font-weight: 600;
}

.setting-desc {
  font-size: 13px;
  color: var(--text-secondary);
}

.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--border-color);
  border-radius: 24px;
  transition: 0.2s;
}

.toggle-slider::before {
  content: "";
  position: absolute;
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  border-radius: 50%;
  transition: 0.2s;
}

.toggle input:checked + .toggle-slider {
  background-color: var(--color-primary);
}

.toggle input:checked + .toggle-slider::before {
  transform: translateX(20px);
}

.schedule-options {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.radio-group {
  display: flex;
  gap: 16px;
}

.radio-item {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 14px;
}

.schedule-detail {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.input-number {
  width: 80px;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.input-time {
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.input-select {
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.save-area {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
}


</style>

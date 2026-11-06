<template>
  <div class="settings">
    <h2 class="page-title">{{ $t('settings.title') }}</h2>

    <div class="card settings-section">
      <h3 class="section-title">{{ $t('settings.exclusions') }}</h3>
      <p class="section-desc">{{ $t('settings.exclusionsDesc') }}</p>

      <div class="input-row">
        <input
          v-model="newExclude"
          type="text"
          :placeholder="$t('settings.exclusionPlaceholder')"
          class="input-text"
          @keyup.enter="addExclusion"
        />
        <button class="btn btn-primary btn-sm" @click="addExclusion">{{ $t('settings.add') }}</button>
      </div>

      <div class="exclusion-list">
        <div v-for="(item, index) in fileExclusions" :key="index" class="exclusion-item">
          <span class="exclusion-name">{{ item }}</span>
          <button class="btn-icon" @click="removeExclusion(index)" :title="$t('settings.remove')">
            <XIcon :size="14" />
          </button>
        </div>
        <p v-if="fileExclusions.length === 0" class="empty-hint">{{ $t('settings.noExclusions') }}</p>
      </div>
    </div>

    <div class="card settings-section">
      <h3 class="section-title">{{ $t('settings.other') }}</h3>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ $t('settings.compactAuto') }}</span>
          <span class="setting-desc">{{ $t('settings.compactAutoDesc') }}</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="compactAfterOrganize" @change="saveConfig" />
          <span class="toggle-slider"></span>
        </label>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ $t('settings.compactNow') }}</span>
          <span class="setting-desc">{{ $t('settings.compactNowDesc') }}</span>
        </div>
        <button class="btn btn-primary btn-sm" :disabled="compacting" @click="doCompact">
          {{ compacting ? $t('settings.compacting') : $t('settings.compactRun') }}
        </button>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ $t('settings.autoStart') }}</span>
          <span class="setting-desc">{{ $t('settings.autoStartDesc') }}</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="autoStart" @change="saveGeneralSettings" />
          <span class="toggle-slider"></span>
        </label>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ $t('settings.language') }}</span>
          <span class="setting-desc">{{ $t('settings.languageDesc') }}</span>
        </div>
        <select v-model="currentLocale" class="input-select" @change="switchLanguage">
          <option value="zh-CN">中文</option>
          <option value="en">English</option>
        </select>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ $t('settings.theme') }}</span>
          <span class="setting-desc">{{ $t('settings.themeDesc') }}</span>
        </div>
        <select v-model="theme" class="input-select" @change="saveGeneralSettings">
          <option value="system">{{ $t('settings.themeSystem') }}</option>
          <option value="light">{{ $t('settings.themeLight') }}</option>
          <option value="dark">{{ $t('settings.themeDark') }}</option>
        </select>
      </div>
    </div>

    <div class="card shortcuts-section">
      <h3 class="section-title">{{ $t('settings.shortcut') }}</h3>
      <div class="shortcut-row">
        <div class="setting-info">
          <span class="setting-label">{{ $t('settings.oneClickOrganize') }}</span>
          <span class="setting-desc">{{ $t('settings.shortcutDesc') }}</span>
        </div>
        <button v-if="!isRecording" class="shortcut-btn" @click="startRecording">
          <kbd class="shortcut-keys">{{ currentShortcut || 'Ctrl + Shift + C' }}</kbd>
          <span class="shortcut-edit">{{ $t('settings.shortcutModify') }}</span>
        </button>
        <div v-else class="shortcut-recording" @keydown="onKeyDown" tabindex="0" ref="recordingRef">
          <span class="recording-text">{{ $t('settings.shortcutRecording') }}</span>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="section-title">{{ $t('settings.contextMenu') }}</h3>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ $t('settings.contextMenuItem') }}</span>
          <span class="setting-desc">{{ $t('settings.contextMenuDesc') }}</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="contextMenuEnabled" @change="toggleContextMenu" />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="card about-section">
      <h3 class="section-title">{{ $t('settings.about') }}</h3>
      <div class="about-logo">
        <span class="about-icon">ND</span>
      </div>
      <p class="about-name">NeatDesk</p>
      <p class="about-version">v0.1.0</p>
      <p class="about-desc">{{ $t('settings.slogan') }}</p>
      <div class="about-divider"></div>
      <p class="about-copyright">{{ $t('settings.copyright') }}</p>
      <button class="btn btn-secondary btn-sm onboarding-btn" @click="reopenOnboarding">
        {{ $t('settings.replayOnboarding') }}
      </button>
      <button class="btn btn-danger btn-sm onboarding-btn" @click="exitApp">
        {{ $t('settings.exitApp') }}
      </button>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, inject } from "vue";
import { useI18n } from "vue-i18n";
import { XIcon } from "lucide-vue-next";
import { api } from "../lib/api";
import type { AppConfig } from "../lib/types";
import { useToast } from "../composables/useToast";

const { success, error } = useToast();
const { t, locale } = useI18n();

const fileExclusions = ref<string[]>([]);
const newExclude = ref("");
const autoStart = ref(false);
const theme = ref<"system" | "light" | "dark">("system");
const currentLocale = ref("zh-CN");
const compactAfterOrganize = ref(false);
const compacting = ref(false);

async function doCompact() {
  compacting.value = true;
  try {
    await api.compactIcons();
    success(t("settings.compactDone"));
  } catch (e: any) {
    error(typeof e === "string" ? e : t("settings.compactFail"));
  } finally {
    compacting.value = false;
  }
}

// 快捷键
const currentShortcut = ref("");
const isRecording = ref(false);
const recordingRef = ref<HTMLElement | null>(null);
const contextMenuEnabled = ref(false);

async function loadConfig() {
  try {
    const config = (await api.getConfig()) as AppConfig;
    fileExclusions.value = config.exclusions.file_names || [];
    autoStart.value = config.settings.auto_start;
    theme.value = config.settings.theme;
    currentLocale.value = (config.settings.language as string) || "zh-CN";
    currentShortcut.value = config.settings.shortcut || "";
    compactAfterOrganize.value = config.settings.compact_after_organize ?? false;
  } catch (e) {
    console.error("加载配置失败:", e);
  }
}

async function saveConfig() {
  try {
    const config = (await api.getConfig()) as AppConfig;
    config.exclusions.file_names = fileExclusions.value;
    config.settings.auto_start = autoStart.value;
    config.settings.compact_after_organize = compactAfterOrganize.value;
    config.settings.theme = theme.value;
    config.settings.language = currentLocale.value;
    await api.updateConfig(config);
    success(t("common.saved"));
  } catch (e) {
    error(t("settings.saveFail"));
    console.error(e);
  }
}

// 快捷键录制
function startRecording() {
  isRecording.value = true;
  nextTick(() => {
    recordingRef.value?.focus();
  });
}

function onKeyDown(e: KeyboardEvent) {
  e.preventDefault();
  const parts: string[] = [];

  if (e.ctrlKey) parts.push("Ctrl");
  if (e.shiftKey) parts.push("Shift");
  if (e.altKey) parts.push("Alt");

  // 排除单独按修饰键
  const key = e.key;
  if (["Control", "Shift", "Alt", "Meta"].includes(key)) {
    return;
  }

  // 转换键名
  let mainKey = key;
  if (key === " ") mainKey = "Space";
  else if (key.length === 1) mainKey = key.toUpperCase();

  parts.push(mainKey);
  const shortcutStr = parts.join("+");

  currentShortcut.value = shortcutStr;
  isRecording.value = false;

  // 保存到配置并注册到系统
  saveShortcut(shortcutStr);
}

async function saveShortcut(shortcut: string) {
  try {
    // 先更新配置
    const config = (await api.getConfig()) as AppConfig;
    config.settings.shortcut = shortcut;
    await api.updateConfig(config);
    // 再通知后端注册
    await api.updateShortcut(shortcut);
    success(t("settings.shortcutSet", { shortcut }));
  } catch (e: any) {
    error(typeof e === "string" ? e : t("settings.shortcutFail"));
  }
}

const reopenOnboarding = inject("showOnboarding") as (() => void) | undefined;

function switchLanguage() {
  locale.value = currentLocale.value;
  saveGeneralSettings();
  // 同步更新系统托盘菜单语言
  api.setTrayLanguage(currentLocale.value).catch(() => {});
}

async function toggleContextMenu() {
  try {
    if (contextMenuEnabled.value) {
      await api.installContextMenu();
      success(t("settings.contextMenuInstalled"));
    } else {
      await api.uninstallContextMenu();
      success(t("settings.contextMenuUninstalled"));
    }
  } catch (e: any) {
    error(typeof e === "string" ? e : t("settings.opFailed"));
    contextMenuEnabled.value = !contextMenuEnabled.value;
  }
}

function exitApp() {
  api.exitApp();
}

function addExclusion() {
  const val = newExclude.value.trim();
  if (val && !fileExclusions.value.includes(val)) {
    fileExclusions.value.push(val);
    newExclude.value = "";
    saveConfig();
  }
}

function removeExclusion(index: number) {
  fileExclusions.value.splice(index, 1);
  saveConfig();
}

function applyThemeToDOM(t: string) {
  document.documentElement.classList.remove("theme-dark", "theme-light");
  if (t === "dark") document.documentElement.classList.add("theme-dark");
  else if (t === "light") document.documentElement.classList.add("theme-light");
}

async function saveGeneralSettings() {
  // 先即时切换主题
  applyThemeToDOM(theme.value);

  // 保存配置
  await saveConfig();

  // 开机自启单独处理（写注册表）
  try {
    await api.setAutoStart(autoStart.value);
  } catch (e) {
    error(t("settings.autoStartFail"));
    console.error(e);
  }
}

onMounted(loadConfig);
</script>

<style scoped>
.settings-section {
  margin-bottom: 16px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 4px;
}

.section-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 16px;
}

.input-row {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.input-text {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: var(--bg-card);
  color: var(--text-primary);
}

.input-text:focus {
  outline: none;
  border-color: var(--color-primary);
}

.exclusion-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.exclusion-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  background-color: var(--bg-hover);
}

.exclusion-name {
  font-size: 13px;
  font-family: monospace;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.btn-icon:hover {
  background-color: var(--color-danger);
  color: white;
}

.empty-hint {
  font-size: 13px;
  color: var(--text-secondary);
  padding: 8px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
}

.setting-row + .setting-row {
  border-top: 1px solid var(--border-color);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-secondary);
}

.toggle {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
  flex-shrink: 0;
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

.input-select {
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 14px;
  background: var(--bg-card);
  color: var(--text-primary);
}


.btn-sm {
  padding: 6px 14px;
  font-size: 13px;
}

.shortcuts-section {
  margin-bottom: 16px;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  font-size: 14px;
  gap: 16px;
}

.shortcut-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
}

.shortcut-keys {
  display: inline-flex;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 4px;
  background-color: var(--bg-hover);
  border: 1px solid var(--border-color);
  font-family: inherit;
  font-size: 12px;
  color: var(--text-primary);
}

.shortcut-edit {
  font-size: 12px;
  color: var(--color-primary);
  opacity: 0;
  transition: opacity 0.15s;
}

.shortcut-btn:hover .shortcut-edit {
  opacity: 1;
}

.shortcut-recording {
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  background-color: color-mix(in srgb, var(--color-primary) 10%, transparent);
  border: 2px solid var(--color-primary);
  outline: none;
  min-width: 180px;
  text-align: center;
}

.recording-text {
  font-size: 13px;
  color: var(--color-primary);
  font-weight: 500;
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  50% { opacity: 0.3; }
}

.about-section {
  text-align: center;
  padding: 32px 20px;
}

.about-logo {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  background: var(--color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 12px;
}

.about-icon {
  font-size: 22px;
  font-weight: 800;
  color: #fff;
  letter-spacing: -1px;
}

.about-name {
  font-size: 17px;
  font-weight: 700;
  margin-bottom: 2px;
}

.about-version {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.about-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  max-width: 280px;
  margin-left: auto;
  margin-right: auto;
}

.about-divider {
  width: 40px;
  height: 1px;
  background: var(--border-color);
  margin: 16px auto;
}

.about-copyright {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}



.onboarding-btn {
  margin-top: 14px;
  width: 100%;
  justify-content: center;
}

.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 10px 24px;
  border-radius: var(--radius-md);
  font-size: 14px;
  font-weight: 500;
  z-index: 200;
  animation: fadeInUp 0.3s ease;
}

.toast-success {
  background-color: var(--color-success);
  color: white;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}
</style>

<template>
  <div class="settings">
    <h2 class="page-title">设置</h2>

    <div class="card settings-section">
      <h3 class="section-title">排除列表</h3>
      <p class="section-desc">这些文件/文件夹不会被整理</p>

      <div class="input-row">
        <input
          v-model="newExclude"
          type="text"
          placeholder="输入文件名或文件夹名，按回车添加"
          class="input-text"
          @keyup.enter="addExclusion"
        />
        <button class="btn btn-primary btn-sm" @click="addExclusion">添加</button>
      </div>

      <div class="exclusion-list">
        <div v-for="(item, index) in fileExclusions" :key="index" class="exclusion-item">
          <span class="exclusion-name">{{ item }}</span>
          <button class="btn-icon" @click="removeExclusion(index)" title="移除">
            <XIcon :size="14" />
          </button>
        </div>
        <p v-if="fileExclusions.length === 0" class="empty-hint">暂无排除项</p>
      </div>
    </div>

    <div class="card settings-section">
      <h3 class="section-title">其他设置</h3>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">整理后自动补齐</span>
          <span class="setting-desc">移动文件后，剩余文件按顺序向前补齐空位，让桌面更紧凑</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="compactAfterOrganize" @change="saveConfig" />
          <span class="toggle-slider"></span>
        </label>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">立即补齐</span>
          <span class="setting-desc">手动将桌面图标向前紧凑排列，消除空位</span>
        </div>
        <button class="btn btn-primary btn-sm" :disabled="compacting" @click="doCompact">
          {{ compacting ? '补齐中...' : '执行补齐' }}
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
      <h3 class="section-title">快捷键</h3>
      <div class="shortcut-row">
        <div class="setting-info">
          <span class="setting-label">一键整理</span>
          <span class="setting-desc">按下快捷键立即整理桌面文件</span>
        </div>
        <button v-if="!isRecording" class="shortcut-btn" @click="startRecording">
          <kbd class="shortcut-keys">{{ currentShortcut || 'Ctrl + Shift + C' }}</kbd>
          <span class="shortcut-edit">修改</span>
        </button>
        <div v-else class="shortcut-recording" @keydown="onKeyDown" tabindex="0" ref="recordingRef">
          <span class="recording-text">按下新快捷键...</span>
        </div>
      </div>
    </div>

    <div class="card">
      <h3 class="section-title">桌面右键菜单</h3>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">桌面右键「一键整理」</span>
          <span class="setting-desc">在桌面空白处右键可直接整理</span>
        </div>
        <label class="toggle">
          <input type="checkbox" v-model="contextMenuEnabled" @change="toggleContextMenu" />
          <span class="toggle-slider"></span>
        </label>
      </div>
    </div>

    <div class="card about-section">
      <h3 class="section-title">关于</h3>
      <div class="about-logo">
        <span class="about-icon">ND</span>
      </div>
      <p class="about-name">NeatDesk</p>
      <p class="about-version">v0.1.0</p>
      <p class="about-desc">让桌面回归清爽</p>
      <div class="about-divider"></div>
      <p class="about-copyright">© 2026 ayf-master</p>
      <button class="btn btn-secondary btn-sm onboarding-btn" @click="reopenOnboarding">
        重新查看新手引导
      </button>
      <button class="btn btn-danger btn-sm onboarding-btn" @click="exitApp">
        退出应用
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
    const msg = await api.compactIcons();
    success(msg);
  } catch (e: any) {
    error(typeof e === "string" ? e : "补齐失败");
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
    success("设置已保存");
  } catch (e) {
    error("保存设置失败");
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
    success(`快捷键已设置为 ${shortcut}`);
  } catch (e: any) {
    error(typeof e === "string" ? e : "设置快捷键失败");
  }
}

const reopenOnboarding = inject("showOnboarding") as (() => void) | undefined;

function switchLanguage() {
  // @ts-ignore - vue-i18n composable
  const i18n = useI18n();
  i18n.locale.value = currentLocale.value;
  saveGeneralSettings();
}

async function toggleContextMenu() {
  try {
    if (contextMenuEnabled.value) {
      await api.installContextMenu();
      success("右键菜单已安装，刷新桌面后生效");
    } else {
      await api.uninstallContextMenu();
      success("右键菜单已卸载");
    }
  } catch (e: any) {
    error(typeof e === "string" ? e : "操作失败");
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
    error("设置开机自启失败");
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

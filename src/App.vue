<template>
  <div id="app-container" :class="themeClass">
    <aside class="sidebar">
      <div class="sidebar-header">
        <h1 class="app-title">NeatDesk</h1>
        <span class="app-version">v0.2.0</span>
      </div>
      <nav class="sidebar-nav">
        <router-link to="/" class="nav-item">
          <HomeIcon :size="20" />
          <span>{{ $t('nav.home') }}</span>
        </router-link>
        <router-link to="/categories" class="nav-item">
          <FolderIcon :size="20" />
          <span>{{ $t('nav.categories') }}</span>
        </router-link>
        <router-link to="/schedule" class="nav-item">
          <ClockIcon :size="20" />
          <span>{{ $t('nav.schedule') }}</span>
        </router-link>
        <router-link to="/history" class="nav-item">
          <HistoryIcon :size="20" />
          <span>{{ $t('nav.history') }}</span>
        </router-link>
        <router-link to="/settings" class="nav-item">
          <SettingsIcon :size="20" />
          <span>{{ $t('nav.settings') }}</span>
        </router-link>
      </nav>
    </aside>
    <main class="main-content">
      <router-view />
    </main>
    <ToastContainer />

    <!-- 新手引导 -->
    <OnboardingView v-if="showOnboarding" @complete="showOnboarding = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, provide } from "vue";
import { useI18n } from "vue-i18n";
import {
  HomeIcon, FolderIcon, ClockIcon, HistoryIcon, SettingsIcon,
} from "lucide-vue-next";
import ToastContainer from "./components/ToastContainer.vue";
import OnboardingView from "./views/OnboardingView.vue";
import { api } from "./lib/api";
import type { AppConfig } from "./lib/types";

const i18n = useI18n();
const theme = ref<"system" | "light" | "dark">("system");
const showOnboarding = ref(false);

// 提供重新打开新手引导的方法给子组件
provide("showOnboarding", () => { showOnboarding.value = true; });

const themeClass = computed(() => {
  if (theme.value === "dark") return "theme-dark";
  if (theme.value === "light") return "theme-light";
  return "";
});

function applyTheme(configTheme: string) {
  theme.value = (configTheme as "system" | "light" | "dark") || "system";

  if (theme.value === "dark") {
    document.documentElement.classList.add("theme-dark");
    document.documentElement.classList.remove("theme-light");
  } else if (theme.value === "light") {
    document.documentElement.classList.remove("theme-dark");
    document.documentElement.classList.add("theme-light");
  } else {
    // 跟随系统
    document.documentElement.classList.remove("theme-dark", "theme-light");
  }
}

onMounted(async () => {
  try {
    const config = (await api.getConfig()) as AppConfig;
    applyTheme(config.settings.theme);
    // 还原语言设置
    if (config.settings.language) {
      i18n.locale.value = config.settings.language;
    }
    if (!config.settings.onboarding_completed) {
      showOnboarding.value = true;
    }
  } catch {
    // 使用默认设置
  }
});
</script>

<style scoped>
#app-container {
  display: flex;
  height: 100vh;
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

.sidebar {
  width: 220px;
  min-width: 220px;
  background-color: var(--bg-sidebar);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  user-select: none;
}

.sidebar-header {
  padding: 24px 20px 16px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.app-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.3px;
}

.app-version {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.6;
}

.sidebar-nav {
  padding: 12px 8px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 8px;
  text-decoration: none;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  transition: all 0.15s ease;
  position: relative;
}

.nav-item:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.nav-item.router-link-active {
  background-color: var(--color-primary);
  color: #ffffff;
}

.nav-item.router-link-active svg {
  color: #ffffff;
}

.main-content {
  flex: 1;
  padding: 32px;
  overflow-y: auto;
}
</style>

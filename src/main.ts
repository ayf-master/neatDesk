import { createApp } from "vue";
import { createI18n } from "vue-i18n";
import App from "./App.vue";
import router from "./router";
import "./assets/styles/main.css";
import zhCN from "./locales/zh-CN";
import en from "./locales/en";

// 检测系统语言
const systemLang = navigator.language.startsWith("zh") ? "zh-CN" : "en";

const i18n = createI18n({
  locale: systemLang,
  fallbackLocale: "zh-CN",
  messages: { "zh-CN": zhCN, en },
});

const app = createApp(App);

app.config.errorHandler = (err, _instance, info) => {
  console.error("Vue 全局错误:", err, info);
};

app.use(i18n);
app.use(router);
app.mount("#app");

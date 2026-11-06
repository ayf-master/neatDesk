import { invoke } from "@tauri-apps/api/core";

export const api = {
  // 配置
  getConfig: () => invoke("get_config"),
  updateConfig: (config: unknown) => invoke("update_config", { newConfig: config }),

  // 扫描
  scanDesktop: () => invoke("scan_desktop"),

  // 分类预览（扫描 + 分类，不移动文件）
  categorizeDesktop: () => invoke("categorize_desktop"),

  // 整理
  organizeDesktop: () => invoke("organize_desktop"),

  // 撤销
  undoOrganize: (recordId: number) => invoke("undo_organize", { recordId }),
  getHistory: () => invoke("get_history"),

  // 历史管理
  deleteHistoryRecord: (recordId: number) => invoke("delete_history_record", { recordId }),
  clearHistory: () => invoke("clear_history"),

  // 快捷键
  updateShortcut: (shortcut: string) => invoke("update_shortcut", { shortcut }),

  // 右键菜单
  installContextMenu: () => invoke("install_context_menu"),
  uninstallContextMenu: () => invoke("uninstall_context_menu"),

  // 系统
  compactIcons: () => invoke<string>("compact_icons"),
  setAutoStart: (enabled: boolean) => invoke("set_auto_start", { enabled }),
  exitApp: () => invoke("exit_app"),

  // 托盘语言同步
  setTrayLanguage: (language: string) => invoke("set_tray_language", { language }),
};

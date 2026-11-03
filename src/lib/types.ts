// NeatDesk — TypeScript 类型定义（与 Rust 后端结构对应）

export interface Category {
  id: string;
  name: string;
  icon: string;
  extensions: string[];
  name_patterns: string[];
  target_path: string;
  enabled: boolean;
}

export interface ScheduleConfig {
  enabled: boolean;
  type: "interval" | "daily" | "weekly";
  interval_hours: number;
  daily_time: string;
  weekly_day: string;
  weekly_time: string;
}

export interface Exclusions {
  file_names: string[];
  folder_names: string[];
}

export interface AppSettings {
  auto_start: boolean;
  minimize_to_tray: boolean;
  theme: "system" | "light" | "dark";
  shortcut: string;
  organize_target_path: string;
  onboarding_completed: boolean;
  language: string;
  compact_after_organize: boolean;
}

export interface AppConfig {
  version: number;
  categories: Category[];
  schedule: ScheduleConfig;
  exclusions: Exclusions;
  settings: AppSettings;
}

export interface FileInfo {
  name: string;
  extension: string;
  path: string;
  size: number;
  modified: string;
}

export interface OrganizeResult {
  total_files: number;
  success_count: number;
  failed_count: number;
  categories: { name: string; count: number }[];
  errors: { file_name: string; category: string; reason: string }[];
  compact_log?: string;
}

export interface HistoryRecord {
  id: number;
  organized_at: string;
  total_files: number;
  categories: { name: string; count: number }[];
  undone: boolean;
}

// 预设分类图标映射
export const CATEGORY_ICONS: Record<string, string> = {
  images: "Image",
  documents: "FileText",
  videos: "Video",
  music: "Music",
  archives: "Archive",
  executables: "Terminal",
  others: "FolderOpen",
};

// 图标的中文名称（用于显示）
export const CATEGORY_NAMES: Record<string, string> = {
  images: "图片",
  documents: "文档",
  videos: "视频",
  music: "音乐",
  archives: "压缩包",
  executables: "可执行文件",
  others: "其他",
};

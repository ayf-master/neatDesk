# NeatDesk — 技术架构设计

## 整体架构

```
┌─────────────────────────────────────┐
│          Vue 3 前端 (WebView)         │
│  Home | Categories | Schedule | ... │
└──────────────┬──────────────────────┘
       invoke / listen (Tauri IPC)
┌──────────────┴──────────────────────┐
│          Rust 后端 (Tauri 2.0)        │
│  ┌─────┐ ┌──────┐ ┌──────┐ ┌─────┐ │
│  │Cfg  │ │Scan  │ │Class │ │Org  │ │
│  │     │ │      │ │      │ │     │ │
│  └─────┘ └──────┘ └──────┘ └─────┘ │
│  ┌─────┐ ┌──────┐ ┌──────┐ ┌─────┐ │
│  │Undo │ │Sched │ │Tray  │ │Excl │ │
│  └─────┘ └──────┘ └──────┘ └─────┘ │
└─────────────────────────────────────┘
```

## 模块职责

### Rust 后端
| 模块 | 文件 | 职责 |
|------|------|------|
| 入口 | `main.rs` | 应用初始化、整合各模块 |
| 命令 | `commands.rs` | Tauri Command 注册，前端调用入口 |
| 配置 | `config.rs` | JSON 配置文件读写 |
| 扫描 | `scanner.rs` | 桌面文件系统扫描 |
| 分类 | `classifier.rs` | 扩展名 → 分类匹配 |
| 整理 | `organizer.rs` | 文件移动、冲突处理 |
| 撤销 | `undo.rs` | SQLite 记录 + 回滚逻辑 |
| 定时 | `scheduler.rs` | tokio 定时任务调度 |
| 托盘 | `tray.rs` | 系统托盘图标与菜单 |
| 排除 | `exclusions.rs` | 排除列表过滤 |

### Vue 3 前端
| 页面 | 路由 | 功能 |
|------|------|------|
| 首页 | `/` | 桌面概况、立即整理 |
| 分类 | `/categories` | 分类规则管理 |
| 定时 | `/schedule` | 定时整理配置 |
| 历史 | `/history` | 整理历史、撤销 |
| 设置 | `/settings` | 排除列表、主题等 |

## 数据流

### 整理流程
```
用户点击"整理" → commands::organize_desktop
  → scanner::scan(desktop_path) → [FileInfo]
  → exclusions::filter(files, config.exclusions) → [FileInfo]
  → classifier::classify(files, config.categories) → {category: [FileInfo]}
  → undo::save_record(items_before) → record_id
  → organizer::move_files(classified_map) → [MoveResult]
  → 返回结果到前端
```

### 撤销流程
```
用户点击"撤销" → commands::undo_organize(record_id)
  → undo::get_record(id) → {items: [{original, new}]}
  → organizer::move_back(items) → [RestoreResult]
  → undo::mark_undone(id)
  → 返回结果到前端
```

## 数据模型

### 配置 (config.json)
```json
{
  "categories": [{ "id": "images", "name": "图片", "extensions": [".jpg"] }],
  "schedule": { "enabled": true, "type": "interval", "interval_hours": 6 },
  "exclusions": { "file_names": [], "folder_names": [] },
  "settings": { "auto_start": false, "theme": "system" }
}
```

### 整理记录 (SQLite)
```sql
CREATE TABLE organize_records (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  organized_at TEXT NOT NULL,
  total_files INTEGER NOT NULL,
  categories TEXT NOT NULL,
  items TEXT NOT NULL,
  undone INTEGER DEFAULT 0
);
```

## 存储路径
| 数据 | 路径 |
|------|------|
| 配置文件 | `%APPDATA%/desktop-clean/config.json` |
| 历史数据库 | `%APPDATA%/desktop-clean/history.db` |

# 功能 Bug 修复计划 — NeatDesk

> 创建：2026-08-18（代码审查 + 计划制定）
> 范围：功能 Bug 修复（用户已确认），不改前端代码、不改 SQLite 表结构
> 前置：全部 8 个 Step 已完成（见 [04-development-workflow.md](04-development-workflow.md)）

## 一、背景

全面代码审查发现若干"界面有、功能无"的 bug 与系统数据风险。本计划修复 5 个功能级问题，并顺带提取共享整理函数消除 4 处重复流水线。

**约束**：
- 前端对命令的调用契约必须逐字保持：`organize_desktop` 返回 JSON 六字段（`total_files / success_count / failed_count / categories / errors / compact_log`）；`undo_record` 返回消息字符串（可追加明细句，不改变主结构）
- 旧库兼容：已存 history.db 的 items JSON 无新字段 → serde 反序列化为 None → 跳过校验
- 遵循项目规范：`Result<T, String>`、unwrap 注明理由、clippy 干净

**涉及文件**（全部在 src-tauri/src/）：commands.rs、undo.rs、organizer.rs、compact.rs、scheduler.rs、lib.rs、tray.rs、main.rs，新增 pipeline.rs。

---

## 二、修复清单

### Bug 2 — 关闭开机自启永不生效

| 项 | 内容 |
|---|---|
| 位置 | commands.rs `set_auto_start`（142-163 行） |
| 问题 | :156 写入 `"NeatDesk"`，:160 删除 `"DesktopClean"`（旧项目名），自启永远关不掉 |
| 修复 | :160 改为删除 `"NeatDesk"`；顺带尝试删除 `"DesktopClean"` 清理历史残留（`let _ =`，值不存在也 OK） |

### Bug 5 — 整理/撤销后桌面图标不刷新

| 项 | 内容 |
|---|---|
| 位置 | undo.rs:214-250（`refresh_desktop_explorer` 从未被调用） |
| 问题 | SHChangeNotify 刷新函数是死代码，整理/撤销后桌面图标需手动 F5 |
| 修复 | ① 删除 undo.rs 两版 refresh，在 compact.rs 新增 `pub fn refresh_desktop_explorer()`（内容照搬，compact.rs 本就是 cfg(windows) 模块）② commands.rs 的 `organize_desktop` 末尾和 `undo_organize` 成功后调用（`#[cfg(windows)]` 包住，与 compact_desktop_icons 调用方式一致） |

### Bug 4 — 撤销系统数据风险（organizer.rs + undo.rs）

**4a 部分失败被永久标记**
- 问题：undo.rs:116-119 无条件 `UPDATE undone=1`，被跳过的文件永远无法重试
- 修复：仅当 `skipped == 0` 时标记 undone=1；未标记时用户可重复点击撤销重试（现有跳过逻辑幂等）

**4b 撤销可能移错文件**
- 问题：undo.rs:88-91 只查 `source.exists()`，用户放入同名新文件会被误移回
- 修复：
  1. organizer.rs：`OrganizeItem`（25-31 行）增加 `size: Option<u64>`、`modified: Option<String>` 字段；organize() push 时从 `std::fs::metadata(&source)` 采集（失败则 None）
  2. undo.rs：新增 `identity_matches(path, item) -> bool`——metadata 可读且 size 相等且 modified 相等才 true；item 字段为 None（旧记录）时返回 true
  3. 移回前校验，不匹配则 `skipped += 1; mismatch += 1; continue`
  4. 返回消息追加"，N 个跳过（文件身份不匹配，可能已被修改）"

### Bug 3 — 右键"立即整理"读错配置

| 项 | 内容 |
|---|---|
| 位置 | main.rs `quick_organize`（16-50 行） |
| 问题 | `config_dir = app_dir.join("config")` 多一层子目录；主程序 config.json 直接放 `%APPDATA%\com.neatdesk.app`（Tauri 2 Windows 上 app_config_dir == app_data_dir == data_dir.join(identifier)）。右键整理读默认配置而非用户自定义配置 |
| 修复 | ① lib.rs 新增 `pub fn quick_organize()`：路径用 `tauri::generate_context!().config().identifier` 拼 `directories::BaseDirs::data_dir().join(identifier)`（与 Tauri 路径解析同源）；config_dir 与 data_dir 同目录；内部调共享 pipeline ② main.rs 整体删除本地 quick_organize，改调 `desktop_clean_lib::quick_organize()` |

### 横切 — 提取共享整理函数（新模块 pipeline.rs）

整理流水线在 4 处重复（这是 bug 漂移源）：
- commands.rs:69-112 `organize_desktop`
- tray.rs:123-138 `trigger_organize`
- scheduler.rs:156-172（run_scheduler 内）
- main.rs:30-49（quick_organize）

```rust
// pipeline.rs
pub fn perform_organize(config_dir: &Path, data_dir: &Path) -> Result<serde_json::Value, String>
```

- 返回 JSON 与现在 organize_desktop 完全一致
- 内部：load_config → get_desktop_path → scan → exclusions::filter → classifier::classify → organizer::organize → `save_record`（错误记录到日志而非吞掉，不影响返回值）→ compact（`config.settings.compact_after_organize && success_count > 0` 时，cfg(windows)）→ refresh（cfg(windows)）
- 4 个调用点全部改调它

### Bug 1 — 定时整理从未启动 + daily/weekly 重复触发

| 项 | 内容 |
|---|---|
| 位置 | lib.rs（Scheduler 从未实例化）+ scheduler.rs（重复触发缺陷） |
| 问题 | 界面可设置定时规则但永不执行；daily/weekly 分支 `diff <= 5` 窗口内每 30 秒触发一次，一天重复整理约 10 次 |
| 修复 | ① lib.rs setup：`let state = Arc::new(Mutex::new(AppState{...})); app.manage(state.clone());`——所有 9 个带 State 的命令签名改为 `State<'_, Arc<Mutex<AppState>>>`（commands.rs 批量改，内部 `.lock()` 不变；tray.rs 的 try_state 类型同步改）② `let scheduler = Scheduler::new(); scheduler.start(app.handle().clone(), state);` ③ scheduler.rs 去重：新增 `last_trigger_date` 在循环外维护，daily/weekly/interval 触发后记录当天；再触发需 `today != last_trigger_date`；重启兜底——触发前查 `get_history` 当天是否已有记录（has_record_today），有则跳过 ④ run_scheduler 内流水线替换为 `pipeline::perform_organize` ⑤ 锁粒度优化：先取路径再执行整理，不持锁跑完整流程（避免阻塞其他命令） |

---

## 三、修改顺序（每 Phase 后 `cargo check`）

| Phase | 内容 | 验证 |
|---|---|---|
| A | Bug 2（commands.rs 一行）+ Bug 5（compact.rs 加函数 + 两处调用） | `cargo check` |
| B | Bug 4a + 4b（organizer.rs 字段采集 + undo.rs 撤销逻辑） | `cargo check` |
| C | pipeline.rs 新建 + commands::organize_desktop / tray::trigger_organize / main::quick_organize 迁移（含 Bug 3 路径统一） | `cargo check` |
| D | lib.rs Arc 托管 + 9 处命令签名迁移 + Scheduler 启动 + scheduler.rs 去重与 pipeline 接入 | `cargo check` |
| 收尾 | `cargo clippy --all-targets -- -D warnings` + `cargo build` + `npm run build`（前端零改动，最后确认链路） | 三项全过 |

## 四、风险与注意

- **并发**：quick_organize 独立进程与 GUI 可同时整理/写库——rusqlite 无 busy_timeout 时写库可能 SQLITE_BUSY 丢撤销记录（save_record 已容错）；文件移动本身安全（rename 失败进 errors 列表）。本次不引入 OS 级互斥，记录为已知限制
- **消息/JSON 兼容**：前端零改动
- **旧库兼容**：新字段 Option，旧记录跳过校验，行为同旧版
- **行为变化**：daily 去重后"当天手动整理过则定时不再触发"；撤销后未恢复项可重复点击重试（此前会被拒绝）
- **clippy**：Option 用 `unwrap_or_default`；let-else 可用（rust-version 1.77 ≥ 1.65）

## 五、验证

**每 Phase**：`cd src-tauri && cargo check`

**最终质量门**：`cargo clippy --all-targets -- -D warnings` + `cargo build` + `npm run build`

**真机验证清单**（Windows 10/11）：

| Bug | 操作 | 预期 |
|---|---|---|
| Bug 1 | 临时把 scheduler.rs 轮询间隔改 60s；配置 daily_time = 当前+1 分钟、启用定时 → 观察触发；再等 6 分钟 | 到点触发**一次**（`sqlite3 %APPDATA%\com.neatdesk.app\history.db "select count(*) from organize_records where organized_at like '2026-08-18%';"` 当天只有 1 条）；重启应用不重复触发 |
| Bug 2 | 开自启 → `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v NeatDesk` 有值；再关自启 → 查询报错（值已删） |
| Bug 3 | 手写 config.json 到 `%APPDATA%\com.neatdesk.app\config.json`（自定义分类 target_path 如 D:\NeatDeskTest）→ 桌面右键"一键整理桌面" | 文件进 D:\NeatDeskTest；排除项未移动；history.db 出现新记录 |
| Bug 4a | 整理一批 → 占用一个原位置（放同名文件）点撤销 → 释放再点撤销 | 第一次返回"跳过"且 undone=0；第二次补恢复且 undone=1 |
| Bug 4b | 整理 → 分类文件夹放入同名新文件 → 点撤销 | 新文件不被移回，消息含"身份不匹配" |
| Bug 5 | 关闭 compact_after_organize，整理一批文件；再撤销 | 桌面图标即时变化，无需手动 F5 |
| 回归 | 托盘"立即整理"、全局快捷键、历史列表、删除/清空记录 | 与改前一致，无 panic |

---

## 六、可选的后续优化（本次不做）

- **resolve_conflict 时间戳粒度**（organizer.rs:177-200）：同一秒内两个同名文件会撞名导致第二个失败。后续改成循环 `while dest.exists() { 递增序号 }`（如 `_20260617_183000_1`）
- **安全加固**：分类名路径穿越校验（organizer.rs:90 `target_base.join(cat_name)` 需验证 cat_name 不含 `..` 等）、CSP 设置、移除未用 shell 插件、capabilities 补全
- **工程化**：补单元测试（classifier/exclusions/organizer 纯函数）、前端 i18n 补全、类型安全（invoke<T> 泛型）、SettingsView 拆分
- **发布**：MSI 签名（SmartScreen）、重建便携版、journal 补历史、git 版本控制初始化

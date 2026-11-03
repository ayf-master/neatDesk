# NeatDesk — AI 工作指引

## 项目概览
Windows 桌面整理工具，按文件类型自动整理桌面文件。

- **技术栈**：Rust (Tauri 2.0) + Vue 3 + Vite + SQLite
- **前端框架**：Vue 3 (Composition API + `<script setup>`)
- **图标库**：Lucide (lucide-vue-next)

## 文档索引
- [需求文档](docs/01-requirements.md) — 功能需求与分类方案
- [架构设计](docs/02-architecture.md) — 模块划分与数据流
- [UI 设计规范](docs/03-design-standards.md) — 色彩、布局、组件规范
- [开发执行步骤](docs/04-development-workflow.md) — 8 步开发路线图

## 开发日志
- 日志目录：[journal/](journal/)
- 每次会话开始 → 先读取最近一篇日志了解进度
- 每次会话结束 → 更新或创建当天的日志

## 开发规范
- **增量构建**：每个 Step 完成后验证再进入下一个
- **Rust**：遵循 clippy，`unwrap()`/`expect()` 需注明理由
- **前端**：Vue 3 Composition API + `<script setup>`，组件 PascalCase
- **错误处理**：使用 `Result<T, E>`，避免 panic

## 构建命令
```bash
# 开发模式
npm run tauri dev

# 打包
npm run tauri build
```

## 项目状态
| Step | 状态 |
|------|------|
| Step 1：项目初始化与文档体系 | ✅ 完成 |
| Step 2：配置系统（config + 前端联调） | ✅ 完成 |
| Step 3：文件扫描与分类引擎 | ✅ 完成 |
| Step 4：文件整理执行器 | ✅ 完成 |
| Step 5：撤销与历史系统 | ✅ 完成 |
| Step 6：基础 UI 界面 | ✅ 完成 |
| Step 7：定时器与系统托盘 | ✅ 完成 |
| Step 8：打磨与打包 | ✅ 完成 |

## 编译验证
- Rust: `cargo check` ✅ (src-tauri/)
- 前端: `npm run build` ✅ (vue-tsc + vite build)

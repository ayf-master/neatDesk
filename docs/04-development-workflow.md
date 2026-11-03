# NeatDesk — 开发执行步骤与规范

## 开发原则
1. **增量构建**：每个 Step 完成后是可验证的稳定状态
2. **验证先行**：完成功能后必须验证再进入下一步
3. **文档同步**：文档随代码一起更新
4. **日志记录**：每日收工更新开发日志

## 执行计划（8 Steps）

### Step 1：项目初始化
- **输入**：空白项目目录
- **输出**：Tauri + Vue 3 项目跑通，文档体系建立
- **验证**：`npm run tauri dev` 窗口正常显示

### Step 2：配置系统
- **输入**：Tauri 项目骨架
- **输出**：config.rs 读写能力，前后端联调通
- **验证**：前端读写配置，重启后保持

### Step 3：文件扫描与分类
- **输入**：配置系统就绪
- **输出**：scanner.rs + classifier.rs
- **验证**：扫描桌面文件，分类正确

### Step 4：文件整理执行
- **输入**：扫描分类能力
- **输出**：organizer.rs + exclusions.rs
- **验证**：执行整理，文件归入正确分类文件夹

### Step 5：撤销系统
- **输入**：整理执行能力
- **输出**：undo.rs + SQLite
- **验证**：整理后撤销，文件回到原位

### Step 6：基础 UI
- **输入**：全部后端能力
- **输出**：Home / History / Categories / Settings 页面
- **验证**：完整走通 整理→查看历史→撤销 流程

### Step 7：定时器与托盘
- **输入**：UI 界面就绪
- **输出**：scheduler.rs + tray.rs + ScheduleView
- **验证**：设置 1 分钟定时，到期自动整理

### Step 8：打磨与打包
- **输入**：全部功能完成
- **输出**：MSI 安装包
- **验证**：干净 Windows 环境安装运行

## Rust 编码规范
- 遵循 `clippy` 默认规则
- 所有 `unwrap()`/`expect()` 必须注明理由
- 函数需写 doc comment 说明用途
- 错误处理使用 `Result<T, E>`，避免 `panic!`

## Vue 编码规范
- Composition API + `<script setup>` 语法
- 组件名：PascalCase（如 `CategoryCard.vue`）
- 文件命名：camelCase（如 `api.js`）
- CSS 使用 scoped style，类名 kebab-case

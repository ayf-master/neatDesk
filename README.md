# NeatDesk

让 Windows 桌面回归清爽 —— 自动按文件类型整理桌面文件，支持自定义分类规则、定时整理与一键撤销。

NeatDesk 是一个轻量的 Windows 桌面整理工具。它会扫描你的桌面，把杂乱的文件按类型（图片、文档、视频等）自动归位到对应的分类文件夹，还支持自定义规则、定时自动整理，以及随时一键撤销。

## 功能特性

- 按文件类型整理：扫描桌面文件，按扩展名归类到对应的分类文件夹。
- 自定义分类规则：新增 / 编辑 / 删除分类，自由配置扩展名匹配规则。
- 一键手动整理：点击按钮立即执行整理。
- 定时自动整理：完全自定义的时间间隔，后台自动保持桌面整洁。
- 后台驻留：关闭窗口后缩到系统托盘继续运行，定时任务不中断。
- 一键撤销：每次整理都会记录操作日志，可随时回退。
- 智能忽略：自动跳过"此电脑""回收站"等系统桌面项。
- 开机自启：可选是否随系统启动。
- 重名保护：同名文件自动加时间戳后缀，绝不覆盖。
- 排除列表：指定文件 / 文件夹不参与整理。
- 整理统计：查看整理历史（时间、文件数、分类概况）。
- 托盘菜单：右键托盘图标快速操作。
- 深色 / 浅色主题：跟随系统或手动切换。
- 多语言界面：内置中文与英文（基于 vue-i18n）。

## 技术栈

| 层面 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 后端语言 | Rust |
| 前端框架 | Vue 3 + Vite |
| 图标 | Lucide (lucide-vue-next) |
| 本地数据库 | SQLite (rusqlite) |
| 定时调度 | tokio |
| 国际化 | vue-i18n（中文 / English）|

## 安装

### 方式一：下载安装包（推荐给普通用户）

前往 [Releases](https://github.com/ayf-master/neatDesk/releases) 页面下载最新的 `.msi` 安装包，双击安装即可。

> 安装包由 GitHub Actions 在每次打 tag 时**自动构建并发布**（见 [.github/workflows/release.yml](./.github/workflows/release.yml)）。当前未做代码签名，首次运行若被 SmartScreen 拦截，点击"仍要运行"即可继续。

### 方式二：从源码构建

#### 前置依赖

- Node.js 18+ 与 npm
- Rust 工具链（通过 [rustup](https://rustup.rs/) 安装稳定版）
- Windows 构建依赖：Microsoft C++ Build Tools（安装时勾选"使用 C++ 的桌面开发"）与 WebView2 Runtime（Windows 11 已内置）
- Tauri CLI 已作为项目 devDependency 随 `npm install` 安装

#### 构建步骤

```bash
# 1. 克隆仓库
git clone https://github.com/ayf-master/neatDesk.git
cd neatDesk

# 2. 安装前端依赖（同时安装 Tauri CLI）
npm install

# 3. 开发模式（热重载，便于调试）
npm run tauri dev

# 4. 构建发布版安装包（输出到 src-tauri/target/release/bundle/msi/）
npm run tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/msi/` 目录下。

## 使用指南

1. 启动 NeatDesk，应用会自动扫描桌面并展示概况。
2. 在"首页"点击"立即整理"即可手动整理一次。
3. 在"分类"页管理分类规则：可新增分类、编辑扩展名、删除不需要的分类。
4. 在"定时"页设置自动整理间隔，开启后应用会在后台周期性整理桌面。
5. 在"历史"页查看每次整理记录，并可一键撤销某次整理。
6. 在"设置"页配置排除列表、主题、开机自启等。
7. 关闭窗口后应用驻留系统托盘，右键托盘图标可快速整理或退出。

## 项目结构

```
neatDesk/
├── src/                  # Vue 3 前端源码
│   ├── views/            # 页面：首页 / 分类 / 定时 / 历史 / 设置 / 引导
│   ├── components/       # 可复用组件
│   ├── composables/      # 组合式函数（如 Toast 提示）
│   ├── lib/              # 与后端通信的 API 与类型定义
│   └── locales/          # 国际化文案（zh-CN / en）
├── src-tauri/            # Rust 后端（Tauri 2.0）
│   ├── src/
│   │   ├── main.rs        # 应用入口
│   │   ├── commands.rs    # Tauri Command 注册（前端调用入口）
│   │   ├── scanner.rs     # 桌面文件系统扫描
│   │   ├── classifier.rs  # 扩展名 → 分类匹配
│   │   ├── organizer.rs   # 文件移动与冲突处理
│   │   ├── undo.rs        # SQLite 记录与回滚逻辑
│   │   ├── scheduler.rs   # tokio 定时任务调度
│   │   ├── tray.rs        # 系统托盘图标与菜单
│   │   ├── exclusions.rs  # 排除列表过滤
│   │   └── config.rs      # 配置读写
│   └── tauri.conf.json    # Tauri 配置（含应用名、标识符、打包设置）
├── docs/                 # 需求、架构等设计文档
└── package.json
```

## 配置与数据存储

NeatDesk 的所有用户数据保存在：

```
%APPDATA%\com.neatdesk.app\
├── config.json   # 分类规则、定时、排除列表、设置
└── history.db    # 整理历史（SQLite）
```

你可以直接备份这两个文件来迁移或恢复配置。

## 发布新版本（自动构建）

本项目通过 GitHub Actions 自动构建 Windows 安装包，维护者只需打一个 tag 即可触发发布：

```bash
# 在 main 分支上，让版本号与 tag 保持一致（示例为 0.1.0）
git tag v0.1.0
git push origin v0.1.0
```

推送 `v*` 开头的 tag 后，CI 会自动：

1. 检出代码、安装 Node 22 与 Rust 稳定版；
2. 执行 `npm run build` 构建前端；
3. 用 Tauri 打包 Windows `.msi`；
4. 在 [Releases](https://github.com/ayf-master/neatDesk/releases) 创建以 tag 命名的 Release 并上传安装包。

> 如需去除 SmartScreen 拦截，可在工作流中接入 Windows 代码签名证书，并在 `tauri.conf.json` 的 `bundle.windows.certificateThumbprint` 中配置。

## 贡献

欢迎 Issue 与 Pull Request！

1. Fork 本仓库并 clone 到本地。
2. 按上方"从源码构建"步骤安装依赖并启动开发模式。
3. 提交前请确保 `npm run build`（包含 `vue-tsc` 类型检查）通过。
4. 基于 `main` 分支发起 Pull Request，并清晰描述改动内容。

## 许可证

本项目基于 [MIT License](./LICENSE) 开源。

## 致谢

- [Tauri](https://tauri.app/) —— 轻量、安全的桌面应用框架
- [Vue](https://vuejs.org/) —— 渐进式前端框架
- [Lucide](https://lucide.dev/) —— 开源图标库

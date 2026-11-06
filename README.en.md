# NeatDesk

> 🌐 English | [中文](./README.md)

Keep your Windows desktop clean — automatically organize desktop files by type, with custom category rules, scheduled cleaning, and one-click undo.

NeatDesk is a lightweight Windows desktop organizer. It scans your desktop and files clutter into category folders by type (images, documents, videos, etc.), supports custom rules, scheduled auto-cleaning, and undo at any time.

## Features

- **Organize by file type**: scan desktop files and sort them into matching category folders by extension.
- **Custom categories**: add / edit / delete categories and freely configure extension-matching rules.
- **One-click manual organize**: click a button to tidy up immediately.
- **Scheduled auto-organize**: fully customizable intervals keep your desktop clean in the background.
- **Background resident**: minimize to the system tray after closing the window; scheduled tasks keep running.
- **One-click undo**: every organize action is logged so you can roll back anytime.
- **Smart ignore**: automatically skips system desktop items like "This PC" and "Recycle Bin".
- **Auto-start**: optionally launch with Windows.
- **Duplicate protection**: same-name files get a timestamp suffix — never overwritten.
- **Exclusion list**: specify files / folders to skip.
- **Organize history**: view past runs (time, file count, category breakdown).
- **Tray menu**: right-click the tray icon for quick actions.
- **Dark / light theme**: follow the system or toggle manually.
- **Multi-language UI**: built-in Chinese and English (powered by vue-i18n).

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2.0 |
| Backend language | Rust |
| Frontend framework | Vue 3 + Vite |
| Icons | Lucide (lucide-vue-next) |
| Local database | SQLite (rusqlite) |
| Scheduler | tokio |
| i18n | vue-i18n (中文 / English) |

## Installation

### Option 1: Download the installer (recommended for most users)

Go to the [Releases](https://github.com/ayf-master/neatDesk/releases) page and download the latest `.msi` installer, then double-click to install.

> The installer is **built and published automatically** by GitHub Actions on every tag push (see [.github/workflows/release.yml](./.github/workflows/release.yml)). It is currently unsigned; if Windows SmartScreen blocks the first run, click "Run anyway" to continue.

### Option 2: Build from source

#### Prerequisites

- Node.js 18+ and npm
- Rust toolchain (install the stable channel via [rustup](https://rustup.rs/))
- Windows build dependencies: Microsoft C++ Build Tools (select "Desktop development with C++" during install) and WebView2 Runtime (built into Windows 11)
- The Tauri CLI is installed as a project devDependency via `npm install`

#### Build steps

```bash
# 1. Clone the repo
git clone https://github.com/ayf-master/neatDesk.git
cd neatDesk

# 2. Install frontend dependencies (also installs the Tauri CLI)
npm install

# 3. Dev mode (hot reload, good for debugging)
npm run tauri dev

# 4. Build the release installer (output under src-tauri/target/release/bundle/msi/)
npm run tauri build
```

After the build, the installer is located in `src-tauri/target/release/bundle/msi/`.

## Usage Guide

1. Launch NeatDesk — it scans your desktop automatically and shows an overview.
2. On the **Home** page, click **Organize Now** to run a manual cleanup.
3. On the **Categories** page, manage category rules: add categories, edit extensions, delete unwanted ones.
4. On the **Schedule** page, set the auto-organize interval; once enabled, the app tidies your desktop periodically in the background.
5. On the **History** page, review each organize run and undo any of them with one click.
6. On the **Settings** page, configure the exclusion list, theme, auto-start, and more.
7. After closing the window, the app stays in the system tray — right-click the tray icon to organize quickly or quit.

## Project Structure

```
neatDesk/
├── src/                  # Vue 3 frontend source
│   ├── views/            # Pages: Home / Categories / Schedule / History / Settings / Onboarding
│   ├── components/       # Reusable components
│   ├── composables/      # Composition functions (e.g. Toast)
│   ├── lib/              # API and type definitions for backend communication
│   └── locales/          # i18n messages (zh-CN / en)
├── src-tauri/            # Rust backend (Tauri 2.0)
│   ├── src/
│   │   ├── main.rs        # App entry point
│   │   ├── commands.rs    # Tauri Command registration (frontend call entry)
│   │   ├── scanner.rs     # Desktop filesystem scanning
│   │   ├── classifier.rs  # Extension → category matching
│   │   ├── organizer.rs   # File moving and conflict handling
│   │   ├── undo.rs        # SQLite logging and rollback
│   │   ├── scheduler.rs   # tokio scheduled task runner
│   │   ├── tray.rs        # System tray icon and menu
│   │   ├── exclusions.rs  # Exclusion list filtering
│   │   └── config.rs      # Config read/write
│   └── tauri.conf.json    # Tauri config (app name, identifier, bundling)
├── docs/                 # Design docs (requirements, architecture)
└── package.json
```

## Configuration & Data Storage

All NeatDesk user data is stored at:

```
%APPDATA%\com.neatdesk.app\
├── config.json   # Category rules, schedule, exclusions, settings
└── history.db    # Organize history (SQLite)
```

You can back up these two files to migrate or restore your configuration.

## Releasing a New Version (Automated Build)

This project builds the Windows installer via GitHub Actions automatically. Maintainers only need to push a tag to trigger a release:

```bash
# On the main branch, keep the version in sync with the tag (0.2.0 shown as an example)
git tag v0.2.0
git push origin v0.2.0
```

After pushing a `v*` tag, CI automatically:

1. Checks out the code and installs Node 22 and the stable Rust toolchain;
2. Runs `npm run build` to build the frontend;
3. Packages the Windows `.msi` with Tauri;
4. Creates a tag-named Release on [Releases](https://github.com/ayf-master/neatDesk/releases) and uploads the installer.

> To remove the SmartScreen warning, wire a Windows code-signing certificate into the workflow and set `bundle.windows.certificateThumbprint` in `tauri.conf.json`.

## Contributing

Issues and Pull Requests are welcome!

1. Fork the repo and clone it locally.
2. Install dependencies and start dev mode following the "Build from source" steps above.
3. Before committing, make sure `npm run build` (which runs `vue-tsc` type checking) passes.
4. Open a Pull Request against `main` with a clear description of your changes.

## License

This project is open source under the [MIT License](./LICENSE).

## Acknowledgements

- [Tauri](https://tauri.app/) — lightweight, secure desktop app framework
- [Vue](https://vuejs.org/) — progressive frontend framework
- [Lucide](https://lucide.dev/) — open-source icon library

# GrabIt

A fast, simple, cross-platform video downloader for **YouTube, X (Twitter), Instagram, Facebook, TikTok and 1800+ other sites** — with live progress, pause/resume, a download queue, and history.

Downloads go straight to your OS **Downloads** folder (configurable). Works on **Windows, macOS, and Linux** (Fedora, Ubuntu, Arch, or any distro).

## Features

- 🔗 **Paste any link** — GrabIt detects URLs on your clipboard and previews the video (title, thumbnail, duration) before downloading
- 🎚️ **Quality picker** — Best, 4K, 1080p, 720p, 480p, or audio-only (MP3 / M4A)
- 📊 **Live progress** — percentage, speed, ETA, and file size per download
- ⏸️ **Pause / resume / cancel** — resuming continues partial files, no re-downloading
- 📋 **Queue** — download several things at once (concurrency configurable, default 3)
- 🕘 **History** — every download is remembered; open the file, reveal it in your file manager, or download it again
- 📃 **Playlists** — detected automatically with a one-click "download all"
- 🔔 **Native notifications** when downloads finish
- 🌗 **Light/dark** follows your OS theme automatically; native system fonts on every platform
- ⚙️ **Self-contained** — on first run GrabIt downloads its own copies of yt-dlp and ffmpeg (or uses the ones already on your system), and can update them from Settings

## Why this stack? (and what it is)

| Layer | Choice | Why |
|---|---|---|
| App shell | **[Tauri 2](https://v2.tauri.app)** | Rust core + your OS's *native webview* (WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux). Installers are ~10 MB instead of Electron's ~150 MB, RAM usage is a fraction, and it produces native installers for all three OSes from one codebase. |
| Download engine | **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** | The de-facto standard extractor, actively maintained, supports ~1800 sites including YouTube, X, Instagram, Facebook, TikTok, Vimeo, Reddit… GrabIt drives it as a subprocess and parses its machine-readable progress output. |
| Media processing | **[ffmpeg](https://ffmpeg.org)** | Merges the separate video/audio streams modern sites serve and converts audio-only downloads to MP3/M4A. |
| UI | **React 19 + TypeScript + Vite 7** | Mature, fast dev loop, strongly typed against the Rust IPC layer. Plain CSS with design tokens — system font stacks and `prefers-color-scheme` keep it looking native everywhere. |
| Backend logic | **Rust (tokio)** | The queue, process supervision, progress parsing, history and settings persistence all live in Rust — no shell scripts, no race conditions, tiny footprint. |

Engines are **auto-downloaded on first run** rather than bundled: yt-dlp updates frequently (sites change their internals constantly), so GrabIt can refresh it with one click in *Settings → Update downloader* instead of waiting for an app release. If `yt-dlp`/`ffmpeg` are already installed on your system, GrabIt just uses those.

## Running from source

### Prerequisites (all platforms)

1. **Node.js 20+** — <https://nodejs.org>
2. **Rust (stable)** — <https://rustup.rs>:
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Platform-specific prerequisites

**Linux — Fedora / RHEL:**
```sh
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel gcc gcc-c++
```

**Linux — Ubuntu / Debian:**
```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Linux — Arch:**
```sh
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg
```

**macOS:** Xcode Command Line Tools — `xcode-select --install`

**Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (Desktop development with C++) and the [WebView2 runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (preinstalled on Windows 11).

Full official guide: <https://v2.tauri.app/start/prerequisites/>

### Develop

```sh
npm install
npm run tauri dev
```

### Build installers

```sh
npm run tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`:

| OS | Output |
|---|---|
| Windows | `.msi` and `.exe` (NSIS) installers |
| macOS | `.app` bundle and `.dmg` |
| Linux | `.deb`, `.rpm`, and `.AppImage` |

> **Note (Linux/AppImage):** on distros with recent toolchains (Fedora 40+, Ubuntu 24.04+), the AppImage step can fail with `Strip call failed … unknown type [0x13] section '.relr.dyn'` — linuxdeploy's bundled `strip` is too old for modern system libraries. Build with stripping disabled:
> ```sh
> NO_STRIP=true npm run tauri build
> ```

Each OS builds its own installers (you build the Windows installer on Windows, etc.). For all-platform releases, a GitHub Actions matrix with [`tauri-action`](https://github.com/tauri-apps/tauri-action) is the standard approach.

## How it works

```
React UI  ──invoke/events──>  Rust core (tokio)  ──spawn──>  yt-dlp ──> ffmpeg
   │                              │
   │   download-progress events   ├── queue scheduler (max N concurrent)
   │<─────────────────────────────┤── history.json / settings.json (app data dir)
                                  └── engine manager (first-run download, updates)
```

- Progress: yt-dlp runs with `--progress-template '%(progress)j'`, emitting one JSON object per line; Rust parses these and forwards throttled `download-progress` events to the UI.
- Pause = the yt-dlp process is stopped, the `.part` file stays; Resume = re-run with `--continue`, picking up exactly where it stopped.
- Final file paths are captured via `--print after_move:filepath`, so "Open" / "Show in folder" always point at the real file.

## Project layout

```
src/                  React UI (components, IPC wrappers, styles)
src-tauri/src/
  lib.rs              app wiring, command registration
  engine.rs           yt-dlp/ffmpeg discovery, first-run download, updates
  downloader.rs       process spawning, progress parsing, queue, pause/resume
  history.rs          persistent download history
  settings.rs         persistent settings + Downloads-folder resolution
  models.rs           shared types
```

## Legal

GrabIt is a tool. Only download content you have the right to download — your own uploads, openly licensed media, or content where the platform's terms permit it.

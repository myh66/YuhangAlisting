# YuhangAlisting

本地 AList 桌面管理器 · Rclone 挂载控制台 · Tauri 2 · Vue 3 · Rust

[![CI](https://github.com/myh66/YuhangAlisting/actions/workflows/alist-desktop.yml/badge.svg)](https://github.com/myh66/YuhangAlisting/actions/workflows/alist-desktop.yml)
![GitHub stars](https://img.shields.io/github/stars/myh66/YuhangAlisting?style=flat-square)
![GitHub forks](https://img.shields.io/github/forks/myh66/YuhangAlisting?style=flat-square)
![GitHub release](https://img.shields.io/github/v/release/myh66/YuhangAlisting?include_prereleases&style=flat-square)

### 简体中文 | [English](README-en.md)

YuhangAlisting 是一个用来管理本地 AList 和 Rclone 挂载的桌面应用。它把 AList 服务启动、健康检查、管理员密码、Rclone WebDAV 挂载、实时日志、系统托盘、WinFsp 检测与安装入口放到一个原生风格的桌面界面里。

## 功能特性

- **AList 服务管理**：启动、停止、重启、健康检查、进程崩溃自动重启。
- **管理员密码工具**：支持重置和设置 AList `admin` 密码。
- **Rclone 挂载管理**：挂载配置增删改查、本地 JSON 持久化、自动挂载。
- **Windows 盘符挂载**：支持 `Z:` 这类盘符目标，并检测 WinFsp。
- **macOS / Linux 目录挂载**：支持 `/Volumes/...`、`/mnt/...` 等目录挂载点。
- **实时日志查看**：捕获 AList 与 Rclone 子进程输出，前端过滤、搜索、自动滚动。
- **桌面体验**：托盘菜单、关闭最小化到托盘、开机自启、深色/浅色主题。
- **中英文切换**：README 与应用界面都提供中文/英文切换入口。
- **自动化发布**：GitHub Actions 构建 Windows、macOS、Linux 安装包。
- **社区文件齐全**：行为准则、贡献指南、安全策略和 Apache-2.0 许可证已补齐。

## 工作原理

```text
Vue 3 前端
  │  Tauri commands / events
  ▼
Rust 后端
  ├─ AListManager   → 启动 AList、写入配置、轮询 /ping
  ├─ RcloneManager  → 保存挂载配置、执行 rclone mount
  ├─ LogBuffer      → 捕获子进程 stdout/stderr
  └─ WinFsp helper  → Windows 下检测驱动并拉起随包 MSI
        │
        ├─ binaries/alist(.exe)
        ├─ binaries/rclone(.exe)
        └─ binaries/winfsp.msi  Windows 构建时由 prebuild 生成
```

## 快速开始

```bash
git clone https://github.com/myh66/YuhangAlisting.git
cd YuhangAlisting/alist-desktop
corepack enable
yarn install --immutable
yarn prebuild
yarn tauri dev
```

`yarn prebuild` 会根据当前平台下载 AList 与 Rclone 到 `alist-desktop/binaries/`。在 Windows 上，它还会下载官方 WinFsp MSI，并保存为 `binaries/winfsp.msi`。

## 本地构建

```bash
cd alist-desktop
yarn build
cargo check --manifest-path src-tauri/Cargo.toml
yarn tauri build --ci
```

构建产物位于：

```text
alist-desktop/src-tauri/target/release/bundle/
```

## GitHub 发行版

正常发行版通过 tag 触发：

```bash
git tag v0.1.0
git push origin v0.1.0
```

Actions 会构建并上传：

- Windows：NSIS `.exe` 和 MSI `.msi`
- macOS：DMG `.dmg`
- Linux：AppImage 和 Debian `.deb`

详细流程见 [docs/RELEASING.md](docs/RELEASING.md)。

## 项目结构

```text
.
├─ .github/workflows/          # CI 和发行版构建
├─ docs/                       # 发行说明等仓库文档
├─ alist-desktop/
│  ├─ binaries/                # 生成的 sidecar，仅 .gitkeep 入库
│  ├─ docs/                    # 用户文档
│  ├─ scripts/prebuild.js      # 下载 AList、Rclone、WinFsp MSI
│  ├─ src/                     # Vue 3 前端
│  │  ├─ components/
│  │  ├─ router/
│  │  ├─ stores/
│  │  ├─ utils/
│  │  └─ views/
│  └─ src-tauri/               # Rust 后端与 Tauri 配置
│     ├─ src/commands/
│     ├─ src/config/
│     ├─ src/services/
│     └─ src/tray/
├─ README.md
└─ README-en.md
```

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=myh66/YuhangAlisting&type=Date)](https://www.star-history.com/#myh66/YuhangAlisting&Date)

## 说明

- AList、Rclone、WinFsp MSI 不提交到仓库，构建前由 `scripts/prebuild.js` 获取。
- WinFsp 不能像普通 DLL 一样完全免安装内置，因为它包含 Windows 文件系统驱动；应用会随包携带官方 MSI，并在缺失时拉起 UAC 安装。
- 当前 UI 参考 OpenList Desktop 的信息架构，视觉上采用紧凑的原生 Apple 风格。

## 社区与贡献

- [行为准则](CODE_OF_CONDUCT.md)
- [贡献指南](CONTRIBUTING.md)
- [安全策略](SECURITY.md)
- [发行流程](docs/RELEASING.md)

## License

本项目基于 [Apache License 2.0](LICENSE) 开源。

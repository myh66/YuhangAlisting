# YuhangAlisting Wiki

YuhangAlisting 是一个跨平台桌面管理器，用于在本机启动和维护 AList 服务，并通过 Rclone 管理 WebDAV 挂载。它基于 Tauri 2、Vue 3 和 Rust 构建，适用于 Windows、macOS 和 Linux。

## 快速入口

- 仓库主页：https://github.com/myh66/YuhangAlisting
- 发行版下载：https://github.com/myh66/YuhangAlisting/releases
- 问题反馈：https://github.com/myh66/YuhangAlisting/issues
- CI 构建：https://github.com/myh66/YuhangAlisting/actions/workflows/alist-desktop.yml

## 默认登录

首次启动时，应用会生成本地 AList 配置并设置默认管理员账号：

```text
账号：admin
密码：root
```

登录后建议立即修改密码。用户改过密码后，应用后续启动不会再覆盖为默认密码。

## 安装与启动

1. 打开 GitHub Releases。
2. 下载对应系统的安装包。
3. 安装并启动 YuhangAlisting。
4. 在 Dashboard 页面点击 Start 启动 AList。
5. 点击 Open Web 打开 AList Web 界面。
6. 使用 `admin / root` 登录，完成后修改密码。

默认 Web 地址：

```text
http://127.0.0.1:5244
```

## 平台说明

### Windows

- 推荐使用 `.exe` 或 `.msi` 安装包。
- Rclone 盘符挂载需要 WinFsp。
- 应用会检测 WinFsp 是否安装，并在缺失时提供随包 MSI 安装入口。
- 盘符挂载点可使用 `Z:`、`Y:` 等格式。

### macOS

- 推荐使用 `.dmg` 或 `.app.zip`。
- 如果系统提示来自未知开发者，可在系统设置的隐私与安全中允许打开。
- Rclone 目录挂载通常建议使用 `/Volumes/名称`。
- 当前构建使用本地签名/CI ad-hoc 签名，尚未做 Apple notarization。

### Linux

- 推荐使用 AppImage 或 Debian `.deb`。
- 挂载点通常可使用 `/mnt/名称` 或用户目录下的目录。
- 需要系统允许 FUSE 或相应挂载能力。

## 主要功能

- 启动、停止、重启本地 AList 服务。
- 自动写入 AList HTTP 端口配置。
- 健康检查 `/ping`，并显示服务状态。
- 首次启动默认 `admin / root`。
- 支持设置和重置 AList 管理员密码。
- 创建、编辑、删除 Rclone WebDAV 挂载配置。
- 支持自动挂载和手动挂载/卸载。
- 实时查看 AList 与 Rclone 日志。
- 支持系统托盘、关闭最小化到托盘、开机自启。
- 支持中文/英文界面切换。
- GitHub Actions 自动构建 Windows、macOS、Linux 包。

## Rclone 挂载流程

1. 确认 AList 已启动。
2. 打开 Mount 页面。
3. 新增挂载配置。
4. 填写 WebDAV 地址、用户名、密码和本地挂载点。
5. 保存配置。
6. 点击 Mount 开始挂载。
7. 在文件管理器中打开本地挂载点检查内容。

如果挂载失败，请先查看 Logs 页面。常见原因包括：

- AList 未启动。
- WebDAV 地址、账号或密码错误。
- 本地挂载点不存在或权限不足。
- Windows 未安装 WinFsp。
- macOS/Linux 未启用 FUSE 或挂载权限不足。

## 日志说明

日志页面会显示 AList、Rclone 和系统操作日志。

常见日志含义：

- `start HTTP server @ 0.0.0.0:5244`：AList 已启动。
- `GET "/ping" 200`：健康检查正常。
- `not enable search`：AList 搜索未启用，一般不影响使用。
- `qBittorrent / Transmission / aria2 failed`：本机未启动对应下载器服务，一般可忽略。

## 开发运行

```bash
git clone https://github.com/myh66/YuhangAlisting.git
cd YuhangAlisting/alist-desktop
corepack enable
yarn install --immutable
yarn prebuild
yarn tauri dev
```

`yarn prebuild` 会下载当前平台需要的 AList、Rclone sidecar。Windows 构建还会下载 WinFsp MSI。

## 本地检查

```bash
cd alist-desktop
yarn build
cargo check --manifest-path src-tauri/Cargo.toml
```

## 本地构建

```bash
cd alist-desktop
yarn prebuild --force
yarn tauri build --ci
```

构建产物位置：

```text
alist-desktop/src-tauri/target/release/bundle/
```

## 发行流程

普通发行版通过 tag 触发：

```bash
git tag v0.1.0
git push origin main
git push origin v0.1.0
```

GitHub Actions 会创建 draft release，并上传：

- Windows：NSIS `.exe`、MSI `.msi`
- macOS：DMG `.dmg`、`.app.zip`
- Linux：AppImage、Debian `.deb`

如果只想测试构建，可以在 Actions 页面手动运行 workflow。手动运行只上传 workflow artifacts，不创建 GitHub Release。

## 常见问题

### 默认密码是什么？

首次启动默认是 `admin / root`。

### 改过密码后会被重置吗？

不会。默认密码只在首次初始化 AList 配置时写入。

### 为什么 Windows 挂载需要 WinFsp？

Rclone 在 Windows 上挂载盘符需要文件系统驱动支持，WinFsp 提供这个能力。它不能像普通 DLL 一样完全内置免安装，所以应用提供检测和 MSI 安装入口。

### 为什么 macOS 打开会有安全提示？

当前包没有 Apple notarization。可以在系统设置中允许打开，或从源码自行构建。

### AList 日志里下载器初始化失败是 bug 吗？

通常不是。AList 会尝试初始化 qBittorrent、Transmission、aria2 等工具，如果本机没有运行这些服务，会显示连接失败，但不影响 AList 主服务启动。

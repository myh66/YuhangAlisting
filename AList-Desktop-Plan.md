# AList Desktop 开发计划

## 项目概述

借鉴 OpenList Desktop 的架构和设计，为 AList 打造一个跨平台桌面客户端。使用 **Tauri 2 + Vue 3 + Rust** 技术栈，实现 AList 服务的本地化管理、云存储挂载、以及图形化配置。

---

## 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| 前端 UI | Vue 3.5 + TypeScript + Vite | 响应式界面 |
| UI 组件库 | Element Plus / Naive UI | 开箱即用的组件 |
| 桌面框架 | Tauri 2.x | 原生能力桥接 |
| 原生后端 | Rust | 进程管理、系统集成 |
| 文件挂载 | Rclone (WebDAV) | 将 AList 存储挂载为本地磁盘 |
| 核心服务 | AList 二进制 | 文件管理/聚合存储后端 |

---

## 项目结构

```
alist-desktop/
├── src/                          # Vue 前端
│   ├── App.vue
│   ├── main.ts
│   ├── assets/                   # 静态资源、图标
│   ├── components/               # 通用组件
│   │   ├── StatusIndicator.vue   # 服务状态指示器
│   │   ├── LogViewer.vue         # 实时日志组件
│   │   └── ServiceCard.vue       # 服务卡片
│   ├── views/                    # 页面
│   │   ├── Dashboard.vue         # 仪表盘主页
│   │   ├── Mount.vue             # 挂载管理
│   │   ├── Logs.vue              # 日志查看
│   │   └── Settings.vue          # 设置页面
│   ├── stores/                   # Pinia 状态管理
│   │   ├── service.ts            # 服务状态
│   │   └── settings.ts           # 应用设置
│   ├── utils/                    # 工具函数
│   │   └── tauri.ts              # Tauri IPC 封装
│   └── router/
│       └── index.ts              # Vue Router
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Tauri 配置
│   ├── capabilities/             # Tauri 2 权限声明
│   ├── icons/                    # 应用图标
│   └── src/
│       ├── main.rs               # 入口
│       ├── lib.rs                # 库入口
│       ├── commands/             # Tauri 命令
│       │   ├── mod.rs
│       │   ├── alist.rs          # AList 进程管理命令
│       │   ├── rclone.rs         # Rclone 挂载命令
│       │   └── system.rs         # 系统级命令
│       ├── services/             # 服务管理逻辑
│       │   ├── mod.rs
│       │   ├── alist_manager.rs  # AList 进程生命周期
│       │   ├── rclone_manager.rs # Rclone 进程生命周期
│       │   └── health_check.rs   # 健康检查
│       ├── config/               # 配置管理
│       │   └── mod.rs
│       └── tray/                 # 系统托盘
│           └── mod.rs
├── scripts/                      # 构建脚本
│   ├── prebuild.js               # 下载 AList/Rclone 二进制
│   └── download-sidecar.js       # 平台适配下载
├── public/
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

---

## 功能模块详细设计

### 模块一：AList 服务管理（核心）

**目标**：管理 AList 二进制的完整生命周期

#### Rust 后端实现

```rust
// src-tauri/src/services/alist_manager.rs

pub struct AListManager {
    process: Option<Child>,        // 子进程句柄
    port: u16,                     // 默认 5244
    data_dir: PathBuf,             // AList data 目录
    binary_path: PathBuf,          // AList 二进制路径
    status: ServiceStatus,         // 运行状态
    start_time: Option<Instant>,   // 启动时间
}

pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

impl AListManager {
    pub fn start(&mut self) -> Result<()>;    // 启动 alist server
    pub fn stop(&mut self) -> Result<()>;     // 优雅停止
    pub fn restart(&mut self) -> Result<()>;  // 重启
    pub fn health_check(&self) -> bool;       // GET http://localhost:5244/ping
    pub fn get_admin_password(&self) -> Result<String>; // alist admin random
    pub fn get_status(&self) -> ServiceStatus;
    pub fn get_uptime(&self) -> Duration;
}
```

#### Tauri 命令暴露

```rust
// src-tauri/src/commands/alist.rs

#[tauri::command]
async fn start_alist(state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn stop_alist(state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn restart_alist(state: State<'_, AppState>) -> Result<(), String>;

#[tauri::command]
async fn get_alist_status(state: State<'_, AppState>) -> Result<ServiceInfo, String>;

#[tauri::command]
async fn get_alist_password(state: State<'_, AppState>) -> Result<String, String>;

#[tauri::command]
async fn open_alist_web(state: State<'_, AppState>) -> Result<(), String>;
```

#### 前端调用

```typescript
// src/utils/tauri.ts
import { invoke } from '@tauri-apps/api/core'

export const alistApi = {
  start: () => invoke('start_alist'),
  stop: () => invoke('stop_alist'),
  restart: () => invoke('restart_alist'),
  getStatus: () => invoke<ServiceInfo>('get_alist_status'),
  getPassword: () => invoke<string>('get_alist_password'),
  openWeb: () => invoke('open_alist_web'),
}
```

---

### 模块二：Rclone 挂载管理

**目标**：通过 Rclone WebDAV 将 AList 存储挂载为本地磁盘

#### 挂载配置数据结构

```typescript
interface MountConfig {
  id: string
  name: string              // 显示名称
  remotePath: string        // AList 中的路径，如 /aliyundrive
  localPath: string         // 本地挂载点，如 Z: (Windows) 或 /mnt/alist (Linux)
  autoMount: boolean        // 随应用启动自动挂载
  cacheMode: 'off' | 'minimal' | 'writes' | 'full'
  bufferSize: string        // 如 "256M"
  vfsCacheMaxAge: string    // 如 "1h"
  readOnly: boolean
}
```

#### Rust 实现

```rust
// src-tauri/src/services/rclone_manager.rs

pub struct RcloneManager {
    mounts: HashMap<String, MountInstance>,  // 活跃挂载
    rclone_path: PathBuf,
}

struct MountInstance {
    config: MountConfig,
    process: Child,
    status: MountStatus,
}

impl RcloneManager {
    /// 挂载：rclone mount :webdav:/ Z: --webdav-url=http://localhost:5244/dav
    ///       --webdav-user=admin --webdav-pass=xxx --vfs-cache-mode=full
    pub fn mount(&mut self, config: MountConfig) -> Result<()>;
    pub fn unmount(&mut self, mount_id: &str) -> Result<()>;
    pub fn unmount_all(&mut self) -> Result<()>;
    pub fn get_mounts(&self) -> Vec<MountInfo>;
}
```

#### 前端 — 挂载管理页面

- 挂载列表（名称、远程路径、本地路径、状态）
- 添加挂载对话框
- 一键挂载/卸载按钮
- 缓存模式选择（下拉框）
- 在文件管理器中打开挂载路径

---

### 模块三：Dashboard 仪表盘

**目标**：一目了然的服务状态总览

#### UI 设计

```
┌─────────────────────────────────────────────────────────┐
│  AList Desktop                              _ □ ✕       │
├──────────┬──────────────────────────────────────────────┤
│          │                                              │
│ 📊 仪表盘 │  ┌─── AList 服务 ──────────────────────┐    │
│          │  │ 状态: ● 运行中    端口: 5244          │    │
│ 💿 挂载   │  │ 运行时间: 2h 34m                     │    │
│          │  │ [停止] [重启] [打开网页]              │    │
│ 📋 日志   │  └─────────────────────────────────────┘    │
│          │                                              │
│ ⚙ 设置   │  ┌─── Rclone 挂载 ─────────────────────┐    │
│          │  │ 活跃挂载: 2/3                         │    │
│          │  │ 阿里云盘 → Z:  ● 已挂载              │    │
│          │  │ OneDrive → Y:  ● 已挂载              │    │
│          │  │ 百度网盘 → X:  ○ 未挂载              │    │
│          │  └─────────────────────────────────────┘    │
│          │                                              │
│          │  ┌─── 快捷操作 ─────────────────────────┐    │
│          │  │ [一键启动全部] [一键停止全部]          │    │
│          │  │ [打开 AList 管理页面]                 │    │
│          │  └─────────────────────────────────────┘    │
└──────────┴──────────────────────────────────────────────┘
```

---

### 模块四：日志查看器

**目标**：实时展示 AList 和 Rclone 的 stdout/stderr

#### 实现方式

1. Rust 端通过 `tokio::io::BufReader` 逐行读取子进程输出
2. 通过 Tauri Event 系统实时推送到前端
3. 前端使用虚拟滚动列表渲染大量日志

```rust
// Rust 端推送日志事件
app_handle.emit("alist-log", LogEntry {
    timestamp: Utc::now(),
    level: LogLevel::Info,
    message: line,
    source: "alist",
});
```

```typescript
// 前端监听
import { listen } from '@tauri-apps/api/event'

listen<LogEntry>('alist-log', (event) => {
  logs.value.push(event.payload)
})
```

---

### 模块五：系统托盘

**目标**：最小化到托盘，后台运行

#### 功能

- 双击托盘图标：显示/隐藏主窗口
- 右键菜单：
  - 显示主窗口
  - 启动/停止 AList
  - 挂载全部/卸载全部
  - 退出

#### Rust 实现

```rust
// src-tauri/src/tray/mod.rs
use tauri::{
    tray::{TrayIconBuilder, MouseButton, MouseButtonState},
    menu::{MenuBuilder, MenuItemBuilder},
};

pub fn setup_tray(app: &App) -> Result<()> {
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click { button: MouseButton::Left, .. } => {
                    // 显示/隐藏窗口
                }
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}
```

---

### 模块六：设置页面

| 设置项 | 说明 |
|--------|------|
| AList 端口 | 默认 5244，可自定义 |
| 开机自启 | 随系统启动 |
| 启动时自动启动 AList | 应用打开即启动服务 |
| 启动时自动挂载 | 自动执行所有标记为 auto 的挂载 |
| AList 二进制路径 | 内置/自定义路径 |
| Rclone 二进制路径 | 内置/自定义路径 |
| 自动更新检查 | 检查 AList/Rclone 新版本 |
| 语言 | 中文/English |
| 主题 | 浅色/深色/跟随系统 |

---

## 开发阶段计划

### 第一阶段：项目骨架（1-2 周）

- [ ] 初始化 Tauri 2 + Vue 3 + TypeScript 项目
- [ ] 配置 Vite、ESLint、Prettier
- [ ] 搭建基本页面路由（Dashboard / Mount / Logs / Settings）
- [ ] 实现侧边栏导航布局
- [ ] 引入 UI 组件库（推荐 Naive UI，风格现代）
- [ ] 编写 prebuild 脚本，自动下载 AList 和 Rclone 二进制

### 第二阶段：AList 服务管理（1-2 周）

- [ ] Rust 端实现 AListManager（启动/停止/重启）
- [ ] 实现健康检查（轮询 `http://localhost:{port}/ping`）
- [ ] 暴露 Tauri 命令并在前端调用
- [ ] Dashboard 页面展示服务状态
- [ ] 实现进程崩溃自动重启
- [ ] 实现 `alist admin` 获取/重置密码

### 第三阶段：Rclone 挂载（1-2 周）

- [ ] Rust 端实现 RcloneManager
- [ ] 挂载配置的增删改查（持久化到本地 JSON）
- [ ] 前端挂载管理页面
- [ ] 支持 Windows 盘符挂载和 Linux/macOS 目录挂载
- [ ] 挂载状态实时监控

### 第四阶段：日志与托盘（1 周）

- [ ] Rust 端捕获子进程 stdout/stderr
- [ ] 通过 Tauri Event 推送日志到前端
- [ ] 前端日志查看器（过滤、搜索、自动滚动）
- [ ] 系统托盘实现（右键菜单、双击显示）
- [ ] 关闭窗口时最小化到托盘

### 第五阶段：设置与体验优化（1 周）

- [ ] 设置页面实现
- [ ] 开机自启配置（Windows 注册表 / macOS LoginItems / Linux autostart）
- [ ] 深色/浅色主题切换
- [ ] 国际化（i18n）
- [ ] 自动更新检查（对比 GitHub Release）

### 第六阶段：打包发布（1 周）

- [ ] Windows 安装包（NSIS / MSI）
- [ ] macOS DMG
- [ ] Linux AppImage / deb
- [ ] GitHub Actions CI/CD 自动构建
- [ ] 编写用户文档

---

## 关键技术细节

### 1. Sidecar 二进制管理

Tauri 2 的 sidecar 功能可以将 AList 和 Rclone 二进制打包进应用：

```json
// tauri.conf.json
{
  "bundle": {
    "externalBin": [
      "binaries/alist",
      "binaries/rclone"
    ]
  }
}
```

prebuild 脚本根据当前平台下载对应架构的二进制：
- Windows: `alist-windows-amd64.exe`, `rclone-windows-amd64.exe`
- macOS: `alist-darwin-arm64`, `rclone-darwin-arm64`
- Linux: `alist-linux-amd64`, `rclone-linux-amd64`

### 2. AList 与 Rclone 的连接

```
┌──────────────┐     spawn      ┌──────────────┐
│  Rust 后端    │ ──────────────► │  AList 进程   │
│  (Tauri)     │                │  :5244       │
└──────┬───────┘                └──────┬───────┘
       │                               │
       │ spawn                         │ WebDAV /dav
       ▼                               ▼
┌──────────────┐    webdav-url   ┌──────────────┐
│  Rclone 进程  │ ◄──────────────│  连接 AList   │
│  mount Z:    │                │  WebDAV      │
└──────────────┘                └──────────────┘
```

Rclone 挂载命令模板：
```bash
rclone mount :webdav:/{remote_path} {local_path} \
  --webdav-url http://localhost:5244/dav \
  --webdav-user admin \
  --webdav-pass {password} \
  --vfs-cache-mode full \
  --buffer-size 256M \
  --dir-cache-time 5m
```

### 3. 进程管理策略

- 使用 Rust 的 `tokio::process::Command` 异步管理子进程
- stdout/stderr 通过 `tokio::io::Lines` 异步逐行读取
- 健康检查：每 5 秒 GET `/ping`，连续 3 次失败标记为异常
- 优雅停止：先发 SIGTERM（Windows 用 `taskkill`），超时 10s 后 SIGKILL
- 崩溃重启：检测到进程退出码非 0 时自动重启，最多重试 3 次

### 4. 数据持久化

应用配置存储在 Tauri 的 app data 目录下：
- Windows: `%APPDATA%/alist-desktop/`
- macOS: `~/Library/Application Support/alist-desktop/`
- Linux: `~/.config/alist-desktop/`

```
app-data/
├── config.json          # 应用设置
├── mounts.json          # 挂载配置列表
└── alist-data/          # AList 的 data 目录（含 config.json、data.db）
```

---

## 开发环境要求

| 工具 | 版本 | 用途 |
|------|------|------|
| Node.js | 22+ | 前端构建 |
| Yarn | 4.x | 包管理 |
| Rust | nightly | Tauri 后端 |
| Tauri CLI | 2.x | `cargo install tauri-cli` |

### 初始化命令

```bash
# 创建项目
yarn create tauri-app alist-desktop --template vue-ts

# 安装前端依赖
cd alist-desktop
yarn add vue-router@4 pinia naive-ui @vueuse/core
yarn add -D @tauri-apps/cli@next

# Rust 依赖（在 src-tauri/Cargo.toml 中添加）
# tokio, serde, serde_json, reqwest, tauri-plugin-shell
```

---

## 与 OpenList Desktop 的对比

| 特性 | OpenList Desktop | AList Desktop（本项目） |
|------|------------------|------------------------|
| 核心服务 | OpenList | AList |
| 框架 | Tauri 2 + Vue 3 | Tauri 2 + Vue 3 |
| 挂载工具 | Rclone | Rclone |
| UI 组件库 | 未知 | Naive UI |
| 许可证 | GPL-3.0 | MIT（建议） |
| 差异化 | 社区治理 | 原版 AList 桌面化 |

---

## 风险与注意事项

1. **AList 版本兼容**：需跟踪 AList Release，及时更新内置二进制
2. **Windows WinFsp**：Rclone 在 Windows 挂载需要 WinFsp 驱动，需引导用户安装
3. **权限问题**：macOS 可能需要 Full Disk Access；Linux 需要 FUSE 权限
4. **杀毒软件误报**：内置二进制可能被 Windows Defender 拦截，需签名
5. **许可证**：AList 是 AGPL-3.0，仅调用其二进制（不修改源码）不受 AGPL 传染

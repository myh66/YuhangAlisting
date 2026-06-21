import { invoke } from "@tauri-apps/api/core";

export type ServiceStatusKind = "stopped" | "starting" | "running" | "error";
export type MountStatus = "mounted" | "mounting" | "unmounted" | "error";
export type CacheMode = "off" | "minimal" | "writes" | "full";
export type ThemeMode = "light" | "dark" | "system";
export type Language = "zh-CN" | "en-US";
export type CloseAction = "ask" | "minimize" | "exit";

export interface ServiceInfo {
  status: ServiceStatusKind;
  port: number;
  uptime_seconds: number;
  web_url: string;
  data_dir: string;
  binary_path: string;
  error?: string | null;
  restart_attempts: number;
}

export interface MountConfig {
  id: string;
  name: string;
  remotePath: string;
  localPath: string;
  autoMount: boolean;
  cacheMode: CacheMode;
  bufferSize: string;
  vfsCacheMaxAge: string;
  readOnly: boolean;
}

export interface MountInfo extends MountConfig {
  status: MountStatus;
  error?: string | null;
}

export interface LogEntry {
  timestamp: number;
  source: "alist" | "rclone" | "system" | string;
  level: "info" | "warn" | "error" | string;
  message: string;
}

export interface AppConfig {
  alistPort: number;
  autoStartAlist: boolean;
  autoMount: boolean;
  theme: ThemeMode;
  language: Language;
  alistBinaryPath?: string | null;
  rcloneBinaryPath?: string | null;
  checkUpdates: boolean;
  startMinimized: boolean;
  closeAction: CloseAction;
}

export interface ReleaseInfo {
  name: string;
  tagName: string;
  htmlUrl: string;
}

export interface UpdateCheckResult {
  appVersion: string;
  alist: ReleaseInfo;
  rclone: ReleaseInfo;
}

export interface PlatformInfo {
  os: string;
  arch: string;
  family: string;
  defaultMountHint: string;
}

export interface RuntimeReadiness {
  platform: PlatformInfo;
  alistBinaryPath: string;
  alistBinaryReady: boolean;
  rcloneBinaryPath: string;
  rcloneBinaryReady: boolean;
  winfspRequired: boolean;
  winfspInstalled: boolean;
  winfspInstallerPath?: string | null;
}

export interface WinFspStatus {
  required: boolean;
  installed: boolean;
  installerPath?: string | null;
  downloadUrl: string;
}

export const alistApi = {
  start: () => invoke<void>("start_alist"),
  stop: () => invoke<void>("stop_alist"),
  restart: () => invoke<void>("restart_alist"),
  getStatus: () => invoke<ServiceInfo>("get_alist_status"),
  getPassword: () => invoke<string>("get_alist_password"),
  resetPassword: () => invoke<string>("reset_alist_password"),
  setPassword: (password: string) => invoke<string>("set_alist_password", { password }),
  openWeb: () => invoke<void>("open_alist_web"),
};

export const mountApi = {
  list: () => invoke<MountInfo[]>("list_mounts"),
  create: (config: MountConfig) => invoke<MountInfo>("create_mount", { config }),
  update: (config: MountConfig) => invoke<MountInfo>("update_mount", { config }),
  delete: (id: string) => invoke<MountInfo[]>("delete_mount", { id }),
  mount: (id: string, password?: string) => invoke<MountInfo[]>("mount_rclone", { id, password }),
  unmount: (id: string) => invoke<MountInfo[]>("unmount_rclone", { id }),
  unmountAll: () => invoke<MountInfo[]>("unmount_all_rclone"),
  mountAuto: (password?: string) => invoke<MountInfo[]>("mount_auto_rclone", { password }),
  openPath: (id: string) => invoke<void>("open_mount_path", { id }),
};

export const logApi = {
  list: () => invoke<LogEntry[]>("list_logs"),
  clear: () => invoke<void>("clear_logs"),
};

export const settingsApi = {
  get: () => invoke<AppConfig>("get_app_config"),
  save: (config: AppConfig) => invoke<AppConfig>("save_app_config", { config }),
  setAutostart: (enabled: boolean) => invoke<AppConfig>("set_autostart", { enabled }),
  getAutostart: () => invoke<boolean>("get_autostart_enabled"),
  checkUpdates: () => invoke<UpdateCheckResult>("check_updates"),
};

export const systemApi = {
  platform: () => invoke<PlatformInfo>("get_platform_info"),
  readiness: () => invoke<RuntimeReadiness>("get_runtime_readiness"),
  winfspStatus: () => invoke<WinFspStatus>("get_winfsp_status"),
  installWinFsp: () => invoke<void>("install_winfsp"),
  hideMainWindow: () => invoke<void>("hide_main_window"),
  exitApp: () => invoke<void>("exit_app"),
  refreshFileExplorer: () => invoke<void>("refresh_file_explorer"),
};

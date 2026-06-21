import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { darkTheme, type GlobalTheme } from "naive-ui";
import {
  settingsApi,
  type AppConfig,
  type Language,
  type ThemeMode,
  type UpdateCheckResult,
} from "../utils/tauri";

const defaultConfig: AppConfig = {
  alistPort: 5244,
  autoStartAlist: false,
  autoMount: false,
  theme: "system",
  language: "zh-CN",
  alistBinaryPath: null,
  rcloneBinaryPath: null,
  checkUpdates: true,
  startMinimized: false,
};

const messages: Record<Language, Record<string, string>> = {
  "zh-CN": {
    dashboard: "仪表盘",
    mount: "挂载",
    logs: "日志",
    settings: "设置",
    "status.running": "运行中",
    "status.stopped": "已停止",
    "status.starting": "启动中",
    "status.error": "异常",
    "common.save": "保存设置",
    "common.cancel": "取消",
    "common.refresh": "刷新",
    "common.edit": "编辑",
    "common.create": "新建",
    "common.clear": "清空",
    "common.delete": "删除",
    "common.install": "安装",
    "common.searchLogs": "搜索日志内容",
    "settings.theme.system": "跟随系统 / System",
    "settings.theme.light": "浅色 / Light",
    "settings.theme.dark": "深色 / Dark",
    "settings.language.zh-CN": "中文",
    "settings.language.en-US": "English",
    "settings.saved": "设置已保存。",
    "settings.autostart.enabled": "开机自启已启用。",
    "settings.autostart.disabled": "开机自启已关闭。",
    "settings.password.required": "请输入新密码。",
    "settings.password.updated": "管理员密码已设置。",
    "settings.password.copied": "新密码已复制。",
    "settings.updates.refreshed": "更新信息已刷新。",
    "settings.app.title": "应用设置",
    "settings.app.alistPort": "AList 端口",
    "settings.app.autoStartAlist": "打开应用后启动 AList",
    "settings.app.autoMount": "打开应用后自动挂载",
    "settings.app.autostart": "系统登录时启动应用",
    "settings.app.theme": "主题",
    "settings.app.language": "语言",
    "settings.app.startMinimized": "启动时最小化",
    "settings.app.checkUpdates": "自动检查更新",
    "settings.app.alistBinaryPath": "AList 二进制路径",
    "settings.app.rcloneBinaryPath": "Rclone 二进制路径",
    "settings.app.alistBinaryPlaceholder": "留空使用内置 binaries/alist",
    "settings.app.rcloneBinaryPlaceholder": "留空使用内置 binaries/rclone",
    "settings.winfsp.title": "Windows 挂载驱动",
    "settings.winfsp.required": "是否需要",
    "settings.winfsp.requiredDescription": "Windows 盘符挂载需要 WinFsp",
    "settings.winfsp.installed": "安装状态",
    "settings.winfsp.installedYes": "已安装",
    "settings.winfsp.installedNo": "未安装",
    "settings.winfsp.installer": "随包安装器",
    "settings.winfsp.installerMissing": "未找到 winfsp.msi",
    "settings.winfsp.refresh": "刷新状态",
    "settings.winfsp.installing": "WinFsp 安装器已启动，请在系统提示中确认。",
    "settings.winfsp.install": "安装 WinFsp",
    "settings.admin.title": "AList 管理员",
    "settings.admin.placeholder": "输入新的管理员密码",
    "settings.admin.setPassword": "设置密码",
    "settings.admin.resetPassword": "随机重置并复制",
    "settings.updates.title": "更新检查",
    "settings.updates.check": "检查 AList / Rclone 最新版本",
    "dashboard.readiness.withWinfsp": "确认 AList、Rclone、WinFsp 状态",
    "dashboard.readiness.withoutWinfsp": "确认 AList、Rclone 状态",
    "dashboard.winfsp.installing": "WinFsp 安装器已启动，请在系统提示中确认。",
    "dashboard.password.copied": "新管理员密码已复制。",
    "dashboard.password.required": "请输入 AList admin 密码。",
    "dashboard.mountAuto.success": "自动挂载项已执行。",
    "dashboard.service.title": "AList 服务",
    "dashboard.action.start": "启动",
    "dashboard.action.restart": "重启",
    "dashboard.action.stop": "停止",
    "dashboard.action.openWeb": "打开网页",
    "dashboard.action.resetPassword": "重置密码",
    "dashboard.quick.title": "快速流程",
    "dashboard.step.prepare": "准备环境",
    "dashboard.step.start": "启动 AList",
    "dashboard.step.startDescription": "运行本地 WebDAV 服务并健康检查",
    "dashboard.step.createMount": "创建挂载",
    "dashboard.step.createMountDescription": "配置 AList 路径和本地盘符或目录",
    "dashboard.step.mount": "挂载到本地",
    "dashboard.step.mountDescription": "输入 admin 密码后交给 Rclone 挂载",
    "dashboard.action.recheck": "重新检查环境",
    "dashboard.action.mountAuto": "挂载全部自动项",
    "dashboard.readiness.title": "运行环境",
    "dashboard.readiness.alistBinary": "AList 二进制",
    "dashboard.readiness.rcloneBinary": "Rclone 二进制",
    "dashboard.readiness.winfsp": "Windows 挂载驱动",
    "dashboard.readiness.ready": "已就绪",
    "dashboard.readiness.missing": "未找到",
    "dashboard.readiness.winfspInstalled": "WinFsp 已安装",
    "dashboard.readiness.winfspNeeded": "需要安装 WinFsp",
    "dashboard.readiness.winfspHint": "随安装包提供或运行 prebuild 获取",
    "dashboard.readiness.overall": "整体状态",
    "dashboard.readiness.overallReady": "可以使用",
    "dashboard.readiness.overallPending": "需要处理",
    "dashboard.readiness.overallHint": "处理缺失项后重新检查",
    "dashboard.mountSummary.title": "挂载概览",
    "dashboard.mountSummary.active": "活跃挂载",
    "dashboard.mountSummary.auto": "自动挂载",
    "dashboard.mountSummary.defaultPath": "默认挂载点",
    "dashboard.mountSummary.empty": "还没有挂载配置",
    "dashboard.mountSummary.add": "添加挂载",
    "dashboard.mountSummary.name": "名称",
    "dashboard.mountSummary.remotePath": "AList 路径",
    "dashboard.mountSummary.localPath": "本地路径",
    "dashboard.mountSummary.status": "状态",
    "dashboard.passwordModal.title": "输入 AList admin 密码",
    "dashboard.passwordModal.placeholder": "AList admin 密码",
    "dashboard.passwordModal.hint": "AList 不能反查当前密码；忘记时请先重置管理员密码。",
    "dashboard.passwordModal.confirm": "开始挂载",
    "mount.saved": "挂载配置已保存。",
    "mount.updated": "挂载配置已更新。",
    "mount.deleted": "挂载配置已删除。",
    "mount.password.required": "请输入 AList admin 密码。",
    "mount.delete.title": "删除挂载",
    "mount.delete.confirm": "确认删除 {name}？正在运行的挂载会先卸载。",
    "mount.action.auto": "自动项",
    "mount.action.unmountAll": "全部卸载",
    "mount.title": "挂载配置",
    "mount.column.name": "名称",
    "mount.column.remotePath": "AList 路径",
    "mount.column.localPath": "本地路径",
    "mount.column.cache": "缓存",
    "mount.column.auto": "自动",
    "mount.column.status": "状态",
    "mount.column.actions": "操作",
    "mount.empty": "暂无挂载配置",
    "mount.auto.yes": "是",
    "mount.auto.no": "否",
    "mount.action.mount": "挂载",
    "mount.action.unmount": "卸载",
    "mount.form.editTitle": "编辑挂载",
    "mount.form.createTitle": "新增挂载",
    "mount.form.name": "显示名称",
    "mount.form.namePlaceholder": "例如 阿里云盘",
    "mount.form.remotePath": "远程路径",
    "mount.form.localPath": "本地路径",
    "mount.form.cacheMode": "缓存模式",
    "mount.form.cacheMaxAge": "缓存有效期",
    "mount.form.autoMount": "自动挂载",
    "mount.form.readOnly": "只读",
    "mount.form.saveEdit": "保存修改",
    "mount.form.saveCreate": "保存配置",
    "mount.passwordModal.title": "输入 AList admin 密码",
    "mount.passwordModal.placeholder": "AList admin 密码",
    "mount.passwordModal.hint": "密码只用于本次 Rclone 挂载，应用会自动转换为 Rclone 需要的 obscure 格式。",
    "mount.passwordModal.confirmSingle": "挂载此项",
    "mount.passwordModal.confirmAuto": "挂载自动项",
    "logs.cleared": "日志已清空。",
    "logs.title": "实时日志",
    "logs.autoScroll.on": "自动滚动",
    "logs.autoScroll.off": "手动查看",
    "logs.source.all": "全部来源",
    "logs.source.system": "系统",
    "logs.level.all": "全部级别",
    "logs.empty": "暂无日志输出",
    "service.port": "端口",
    "service.uptime": "运行时间",
    "service.url": "访问地址",
  },
  "en-US": {
    dashboard: "Dashboard",
    mount: "Mounts",
    logs: "Logs",
    settings: "Settings",
    "status.running": "Running",
    "status.stopped": "Stopped",
    "status.starting": "Starting",
    "status.error": "Error",
    "common.save": "Save settings",
    "common.cancel": "Cancel",
    "common.refresh": "Refresh",
    "common.edit": "Edit",
    "common.create": "New",
    "common.clear": "Clear",
    "common.delete": "Delete",
    "common.install": "Install",
    "common.searchLogs": "Search logs",
    "settings.theme.system": "System",
    "settings.theme.light": "Light",
    "settings.theme.dark": "Dark",
    "settings.language.zh-CN": "Chinese",
    "settings.language.en-US": "English",
    "settings.saved": "Settings saved.",
    "settings.autostart.enabled": "Launch at login enabled.",
    "settings.autostart.disabled": "Launch at login disabled.",
    "settings.password.required": "Enter a new password.",
    "settings.password.updated": "Admin password updated.",
    "settings.password.copied": "New password copied.",
    "settings.updates.refreshed": "Update info refreshed.",
    "settings.app.title": "App settings",
    "settings.app.alistPort": "AList port",
    "settings.app.autoStartAlist": "Start AList when the app opens",
    "settings.app.autoMount": "Mount automatically when the app opens",
    "settings.app.autostart": "Launch app at system login",
    "settings.app.theme": "Theme",
    "settings.app.language": "Language",
    "settings.app.startMinimized": "Start minimized",
    "settings.app.checkUpdates": "Check updates automatically",
    "settings.app.alistBinaryPath": "AList binary path",
    "settings.app.rcloneBinaryPath": "Rclone binary path",
    "settings.app.alistBinaryPlaceholder": "Leave empty to use bundled binaries/alist",
    "settings.app.rcloneBinaryPlaceholder": "Leave empty to use bundled binaries/rclone",
    "settings.winfsp.title": "Windows mount driver",
    "settings.winfsp.required": "Required",
    "settings.winfsp.requiredDescription": "WinFsp is required for Windows drive-letter mounts",
    "settings.winfsp.installed": "Install status",
    "settings.winfsp.installedYes": "Installed",
    "settings.winfsp.installedNo": "Not installed",
    "settings.winfsp.installer": "Bundled installer",
    "settings.winfsp.installerMissing": "winfsp.msi not found",
    "settings.winfsp.refresh": "Refresh status",
    "settings.winfsp.installing": "WinFsp installer launched. Confirm the system prompt to continue.",
    "settings.winfsp.install": "Install WinFsp",
    "settings.admin.title": "AList admin",
    "settings.admin.placeholder": "Enter a new admin password",
    "settings.admin.setPassword": "Set password",
    "settings.admin.resetPassword": "Reset randomly and copy",
    "settings.updates.title": "Update check",
    "settings.updates.check": "Check latest AList / Rclone versions",
    "dashboard.readiness.withWinfsp": "Confirm AList, Rclone, and WinFsp status",
    "dashboard.readiness.withoutWinfsp": "Confirm AList and Rclone status",
    "dashboard.winfsp.installing": "WinFsp installer launched. Confirm the system prompt to continue.",
    "dashboard.password.copied": "New admin password copied.",
    "dashboard.password.required": "Enter the AList admin password.",
    "dashboard.mountAuto.success": "Auto-mount items started.",
    "dashboard.service.title": "AList service",
    "dashboard.action.start": "Start",
    "dashboard.action.restart": "Restart",
    "dashboard.action.stop": "Stop",
    "dashboard.action.openWeb": "Open web",
    "dashboard.action.resetPassword": "Reset password",
    "dashboard.quick.title": "Quick flow",
    "dashboard.step.prepare": "Prepare environment",
    "dashboard.step.start": "Start AList",
    "dashboard.step.startDescription": "Run local WebDAV service and verify health",
    "dashboard.step.createMount": "Create mount",
    "dashboard.step.createMountDescription": "Configure an AList path and local drive or folder",
    "dashboard.step.mount": "Mount locally",
    "dashboard.step.mountDescription": "Enter the admin password and let Rclone mount it",
    "dashboard.action.recheck": "Recheck environment",
    "dashboard.action.mountAuto": "Mount all auto items",
    "dashboard.readiness.title": "Runtime environment",
    "dashboard.readiness.alistBinary": "AList binary",
    "dashboard.readiness.rcloneBinary": "Rclone binary",
    "dashboard.readiness.winfsp": "Windows mount driver",
    "dashboard.readiness.ready": "Ready",
    "dashboard.readiness.missing": "Missing",
    "dashboard.readiness.winfspInstalled": "WinFsp installed",
    "dashboard.readiness.winfspNeeded": "WinFsp required",
    "dashboard.readiness.winfspHint": "Bundled with the app or fetched during prebuild",
    "dashboard.readiness.overall": "Overall status",
    "dashboard.readiness.overallReady": "Ready to use",
    "dashboard.readiness.overallPending": "Needs attention",
    "dashboard.readiness.overallHint": "Recheck after fixing missing items",
    "dashboard.mountSummary.title": "Mount overview",
    "dashboard.mountSummary.active": "Active mounts",
    "dashboard.mountSummary.auto": "Auto mounts",
    "dashboard.mountSummary.defaultPath": "Default mount point",
    "dashboard.mountSummary.empty": "No mount configuration yet",
    "dashboard.mountSummary.add": "Add mount",
    "dashboard.mountSummary.name": "Name",
    "dashboard.mountSummary.remotePath": "AList path",
    "dashboard.mountSummary.localPath": "Local path",
    "dashboard.mountSummary.status": "Status",
    "dashboard.passwordModal.title": "Enter AList admin password",
    "dashboard.passwordModal.placeholder": "AList admin password",
    "dashboard.passwordModal.hint": "AList cannot reveal the current password. Reset the admin password first if you forgot it.",
    "dashboard.passwordModal.confirm": "Start mounting",
    "mount.saved": "Mount configuration saved.",
    "mount.updated": "Mount configuration updated.",
    "mount.deleted": "Mount configuration deleted.",
    "mount.password.required": "Enter the AList admin password.",
    "mount.delete.title": "Delete mount",
    "mount.delete.confirm": "Delete {name}? Running mounts will be unmounted first.",
    "mount.action.auto": "Auto items",
    "mount.action.unmountAll": "Unmount all",
    "mount.title": "Mount configuration",
    "mount.column.name": "Name",
    "mount.column.remotePath": "AList path",
    "mount.column.localPath": "Local path",
    "mount.column.cache": "Cache",
    "mount.column.auto": "Auto",
    "mount.column.status": "Status",
    "mount.column.actions": "Actions",
    "mount.empty": "No mount configuration",
    "mount.auto.yes": "Yes",
    "mount.auto.no": "No",
    "mount.action.mount": "Mount",
    "mount.action.unmount": "Unmount",
    "mount.form.editTitle": "Edit mount",
    "mount.form.createTitle": "New mount",
    "mount.form.name": "Display name",
    "mount.form.namePlaceholder": "For example, Aliyun Drive",
    "mount.form.remotePath": "Remote path",
    "mount.form.localPath": "Local path",
    "mount.form.cacheMode": "Cache mode",
    "mount.form.cacheMaxAge": "Cache max age",
    "mount.form.autoMount": "Auto mount",
    "mount.form.readOnly": "Read only",
    "mount.form.saveEdit": "Save changes",
    "mount.form.saveCreate": "Save configuration",
    "mount.passwordModal.title": "Enter AList admin password",
    "mount.passwordModal.placeholder": "AList admin password",
    "mount.passwordModal.hint": "The password is used only for this Rclone mount and will be converted to Rclone's obscure format automatically.",
    "mount.passwordModal.confirmSingle": "Mount this item",
    "mount.passwordModal.confirmAuto": "Mount auto items",
    "logs.cleared": "Logs cleared.",
    "logs.title": "Live logs",
    "logs.autoScroll.on": "Auto scroll",
    "logs.autoScroll.off": "Manual view",
    "logs.source.all": "All sources",
    "logs.source.system": "System",
    "logs.level.all": "All levels",
    "logs.empty": "No logs yet",
    "service.port": "Port",
    "service.uptime": "Uptime",
    "service.url": "URL",
  },
};

export const useSettingsStore = defineStore("settings", () => {
  const config = ref<AppConfig>({ ...defaultConfig });
  const loading = ref(false);
  const error = ref<string | null>(null);
  const autostartEnabled = ref(false);
  const updateResult = ref<UpdateCheckResult | null>(null);

  const themeMode = computed<ThemeMode>(() => config.value.theme);
  const naiveTheme = computed<GlobalTheme | null>(() => {
    if (themeMode.value === "dark") {
      return darkTheme;
    }

    if (
      themeMode.value === "system" &&
      window.matchMedia?.("(prefers-color-scheme: dark)").matches
    ) {
      return darkTheme;
    }

    return null;
  });

  function applyDocumentTheme() {
    const dark = naiveTheme.value === darkTheme;
    document.documentElement.lang = config.value.language;
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }

  function t(key: string) {
    return messages[config.value.language]?.[key] ?? key;
  }

  function format(key: string, params: Record<string, string | number>) {
    return t(key).replace(/\{(\w+)\}/g, (_, token) => String(params[token] ?? ""));
  }

  async function load() {
    loading.value = true;
    error.value = null;

    try {
      const [nextConfig, nextAutostart] = await Promise.all([
        settingsApi.get(),
        settingsApi.getAutostart(),
      ]);
      config.value = { ...defaultConfig, ...nextConfig };
      autostartEnabled.value = nextAutostart;
      applyDocumentTheme();
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  }

  async function save() {
    loading.value = true;
    error.value = null;

    try {
      config.value = await settingsApi.save(config.value);
      applyDocumentTheme();
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function setAutostart(enabled: boolean) {
    loading.value = true;
    error.value = null;

    try {
      config.value = await settingsApi.setAutostart(enabled);
      autostartEnabled.value = enabled;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function checkUpdates() {
    loading.value = true;
    error.value = null;

    try {
      updateResult.value = await settingsApi.checkUpdates();
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  return {
    config,
    loading,
    error,
    autostartEnabled,
    updateResult,
    naiveTheme,
    t,
    format,
    load,
    save,
    setAutostart,
    checkUpdates,
    applyDocumentTheme,
  };
});

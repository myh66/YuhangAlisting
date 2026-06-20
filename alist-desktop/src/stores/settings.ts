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
    running: "运行中",
    stopped: "已停止",
    starting: "启动中",
    error: "异常",
  },
  "en-US": {
    dashboard: "Dashboard",
    mount: "Mounts",
    logs: "Logs",
    settings: "Settings",
    running: "Running",
    stopped: "Stopped",
    starting: "Starting",
    error: "Error",
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
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }

  function t(key: string) {
    return messages[config.value.language]?.[key] ?? key;
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
    load,
    save,
    setAutostart,
    checkUpdates,
    applyDocumentTheme,
  };
});

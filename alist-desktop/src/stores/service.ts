import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { alistApi, type ServiceInfo, type ServiceStatusKind } from "../utils/tauri";

const defaultInfo: ServiceInfo = {
  status: "stopped",
  port: 5244,
  uptime_seconds: 0,
  web_url: "http://localhost:5244",
  data_dir: "",
  binary_path: "",
  error: null,
  restart_attempts: 0,
};

const statusLabels: Record<ServiceStatusKind, string> = {
  stopped: "已停止",
  starting: "启动中",
  running: "运行中",
  error: "异常",
};

function formatUptime(totalSeconds: number) {
  if (totalSeconds <= 0) {
    return "0m";
  }

  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

export const useServiceStore = defineStore("service", () => {
  const info = ref<ServiceInfo>({ ...defaultInfo });
  const loading = ref(false);
  const error = ref<string | null>(null);
  const unlistenStatus = ref<UnlistenFn | null>(null);

  const statusKind = computed(() => info.value.status);
  const serviceStatusLabel = computed(() => statusLabels[info.value.status]);
  const uptimeText = computed(() => formatUptime(info.value.uptime_seconds));
  const isRunning = computed(() => info.value.status === "running");
  const canStart = computed(() => info.value.status === "stopped" || info.value.status === "error");

  async function refresh() {
    try {
      info.value = await alistApi.getStatus();
      error.value = info.value.error ?? null;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    }
  }

  async function runServiceAction(action: () => Promise<void>) {
    loading.value = true;
    error.value = null;

    try {
      await action();
      await refresh();
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      await refresh();
    } finally {
      loading.value = false;
    }
  }

  const start = () => runServiceAction(alistApi.start);
  const stop = () => runServiceAction(alistApi.stop);
  const restart = () => runServiceAction(alistApi.restart);

  async function openWeb() {
    await alistApi.openWeb();
  }

  async function getPassword() {
    return alistApi.getPassword();
  }

  async function resetPassword() {
    return alistApi.resetPassword();
  }

  async function setPassword(password: string) {
    return alistApi.setPassword(password);
  }

  async function startStatusListener() {
    if (unlistenStatus.value) {
      return;
    }

    unlistenStatus.value = await listen<ServiceInfo>("alist-status", (event) => {
      info.value = event.payload;
      error.value = event.payload.error ?? null;
    });
  }

  return {
    info,
    loading,
    error,
    statusKind,
    serviceStatusLabel,
    uptimeText,
    isRunning,
    canStart,
    refresh,
    start,
    stop,
    restart,
    openWeb,
    getPassword,
    resetPassword,
    setPassword,
    startStatusListener,
  };
});

import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { logApi, type LogEntry } from "../utils/tauri";

export const useLogStore = defineStore("logs", () => {
  const logs = ref<LogEntry[]>([]);
  const source = ref<"all" | "alist" | "rclone" | "system">("all");
  const level = ref<"all" | "info" | "warn" | "error">("all");
  const query = ref("");
  const autoScroll = ref(true);
  const unlisten = ref<UnlistenFn | null>(null);

  const filteredLogs = computed(() => {
    const normalizedQuery = query.value.trim().toLowerCase();

    return logs.value.filter((entry) => {
      const matchesSource = source.value === "all" || entry.source === source.value;
      const matchesLevel = level.value === "all" || entry.level === level.value;
      const matchesQuery =
        normalizedQuery.length === 0 ||
        entry.message.toLowerCase().includes(normalizedQuery) ||
        entry.source.toLowerCase().includes(normalizedQuery);

      return matchesSource && matchesLevel && matchesQuery;
    });
  });

  async function load() {
    logs.value = await logApi.list();
  }

  async function startListener() {
    if (unlisten.value) {
      return;
    }

    unlisten.value = await listen<LogEntry>("service-log", (event) => {
      logs.value.push(event.payload);

      if (logs.value.length > 2_000) {
        logs.value.splice(0, logs.value.length - 2_000);
      }
    });
  }

  async function clear() {
    await logApi.clear();
    logs.value = [];
  }

  return {
    logs,
    source,
    level,
    query,
    autoScroll,
    filteredLogs,
    load,
    startListener,
    clear,
  };
});

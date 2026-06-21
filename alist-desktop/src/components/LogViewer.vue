<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import type { LogEntry } from "../utils/tauri";

const props = defineProps<{
  logs: LogEntry[];
  autoScroll?: boolean;
  emptyText?: string;
  locale?: string;
}>();

const root = ref<HTMLElement | null>(null);

watch(
  () => [props.logs.length, props.autoScroll],
  async () => {
    if (!props.autoScroll) {
      return;
    }

    await nextTick();
    root.value?.scrollTo({ top: root.value.scrollHeight });
  },
);

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat(props.locale ?? "zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(new Date(timestamp));
}
</script>

<template>
  <div ref="root" class="log-viewer">
    <div v-if="logs.length === 0" class="muted">{{ emptyText ?? "暂无日志输出" }}</div>
    <div
      v-for="(log, index) in logs"
      :key="`${log.timestamp}-${index}`"
      class="log-line"
      :class="log.level"
    >
      <span>{{ formatTime(log.timestamp) }}</span>
      <span>{{ log.source }}</span>
      <span>{{ log.level }}</span>
      <span>{{ log.message }}</span>
    </div>
  </div>
</template>

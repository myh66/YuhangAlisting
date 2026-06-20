<script setup lang="ts">
import { onMounted } from "vue";
import { useMessage } from "naive-ui";
import { Trash2 } from "lucide-vue-next";
import LogViewer from "../components/LogViewer.vue";
import { useLogStore } from "../stores/logs";

const logStore = useLogStore();
const message = useMessage();

const sourceOptions = [
  { label: "全部来源", value: "all" },
  { label: "AList", value: "alist" },
  { label: "Rclone", value: "rclone" },
  { label: "系统", value: "system" },
];

const levelOptions = [
  { label: "全部级别", value: "all" },
  { label: "info", value: "info" },
  { label: "warn", value: "warn" },
  { label: "error", value: "error" },
];

onMounted(async () => {
  await logStore.load();
  await logStore.startListener();
});

async function clearLogs() {
  await logStore.clear();
  message.success("日志已清空。");
}
</script>

<template>
  <div class="page-stack">
    <n-card title="实时日志" :bordered="true">
      <template #header-extra>
        <n-space>
          <n-switch v-model:value="logStore.autoScroll">
            <template #checked>自动滚动</template>
            <template #unchecked>手动查看</template>
          </n-switch>
          <n-button secondary @click="clearLogs">
            <template #icon><Trash2 :size="16" /></template>
            清空
          </n-button>
        </n-space>
      </template>

      <div class="log-toolbar">
        <n-select v-model:value="logStore.source" :options="sourceOptions" />
        <n-select v-model:value="logStore.level" :options="levelOptions" />
        <n-input v-model:value="logStore.query" clearable placeholder="搜索日志内容" />
      </div>

      <LogViewer :logs="logStore.filteredLogs" :auto-scroll="logStore.autoScroll" />
    </n-card>
  </div>
</template>

<style scoped>
.log-toolbar {
  display: grid;
  grid-template-columns: 160px 150px minmax(220px, 1fr);
  gap: 12px;
  margin-bottom: 14px;
}
</style>

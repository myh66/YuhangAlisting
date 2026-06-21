<script setup lang="ts">
import { onMounted } from "vue";
import { useMessage } from "naive-ui";
import { Trash2 } from "lucide-vue-next";
import LogViewer from "../components/LogViewer.vue";
import { useLogStore } from "../stores/logs";
import { useSettingsStore } from "../stores/settings";

const logStore = useLogStore();
const settingsStore = useSettingsStore();
const message = useMessage();

const sourceOptions = () => [
  { label: settingsStore.t("logs.source.all"), value: "all" },
  { label: "AList", value: "alist" },
  { label: "Rclone", value: "rclone" },
  { label: settingsStore.t("logs.source.system"), value: "system" },
];

const levelOptions = () => [
  { label: settingsStore.t("logs.level.all"), value: "all" },
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
  message.success(settingsStore.t("logs.cleared"));
}
</script>

<template>
  <div class="page-stack">
    <n-card :title="settingsStore.t('logs.title')" :bordered="true">
      <template #header-extra>
        <n-space>
          <n-switch v-model:value="logStore.autoScroll">
            <template #checked>{{ settingsStore.t("logs.autoScroll.on") }}</template>
            <template #unchecked>{{ settingsStore.t("logs.autoScroll.off") }}</template>
          </n-switch>
          <n-button secondary @click="clearLogs">
            <template #icon><Trash2 :size="16" /></template>
            {{ settingsStore.t("common.clear") }}
          </n-button>
        </n-space>
      </template>

      <div class="log-toolbar">
        <n-select v-model:value="logStore.source" :options="sourceOptions()" />
        <n-select v-model:value="logStore.level" :options="levelOptions()" />
        <n-input v-model:value="logStore.query" clearable :placeholder="settingsStore.t('common.searchLogs')" />
      </div>

      <LogViewer
        :logs="logStore.filteredLogs"
        :auto-scroll="logStore.autoScroll"
        :empty-text="settingsStore.t('logs.empty')"
        :locale="settingsStore.config.language"
      />
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

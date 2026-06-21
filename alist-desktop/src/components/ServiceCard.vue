<script setup lang="ts">
import StatusIndicator from "./StatusIndicator.vue";
import type { ServiceInfo } from "../utils/tauri";

defineProps<{
  title: string;
  statusLabel: string;
  service: ServiceInfo;
  uptimeText: string;
  portLabel: string;
  uptimeLabel: string;
  urlLabel: string;
}>();
</script>

<template>
  <n-card :bordered="true" size="medium" class="service-card">
    <template #header>
      <div class="service-card-header">
        <span>{{ title }}</span>
        <StatusIndicator :status="service.status" :label="statusLabel" />
      </div>
    </template>

    <div class="stat-row">
      <div class="stat-box">
        <span>{{ portLabel }}</span>
        <strong>{{ service.port }}</strong>
      </div>
      <div class="stat-box">
        <span>{{ uptimeLabel }}</span>
        <strong>{{ uptimeText }}</strong>
      </div>
      <div class="stat-box">
        <span>{{ urlLabel }}</span>
        <strong>{{ service.web_url }}</strong>
      </div>
    </div>

    <template #footer>
      <slot name="actions"></slot>
    </template>
  </n-card>
</template>

<style scoped>
.service-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
</style>

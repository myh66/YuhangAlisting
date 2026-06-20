<script setup lang="ts">
import type { ServiceStatusKind } from "../utils/tauri";

defineProps<{
  status: ServiceStatusKind;
  label: string;
}>();
</script>

<template>
  <span class="inline-status">
    <span class="status-dot" :class="status"></span>
    <span>{{ label }}</span>
  </span>
</template>

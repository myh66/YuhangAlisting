<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import { useIntervalFn } from "@vueuse/core";
import { useMessage } from "naive-ui";
import { ExternalLink, KeyRound, Play, RefreshCcw, RotateCw, Square, Wrench } from "lucide-vue-next";
import ServiceCard from "../components/ServiceCard.vue";
import { useMountStore } from "../stores/mounts";
import { useServiceStore } from "../stores/service";
import { useSettingsStore } from "../stores/settings";
import { getCachedAdminPassword, setCachedAdminPassword } from "../utils/passwordCache";
import { systemApi, type RuntimeReadiness } from "../utils/tauri";

const serviceStore = useServiceStore();
const mountStore = useMountStore();
const settingsStore = useSettingsStore();
const message = useMessage();

const readiness = ref<RuntimeReadiness | null>(null);
const passwordModalVisible = ref(false);
const passwordValue = ref("");
const passwordInputRef = ref<HTMLInputElement | null>(null);
const serviceStatusLabel = computed(() =>
  settingsStore.t(`status.${serviceStore.statusKind}`),
);

const allReady = computed(
  () =>
    readiness.value?.alistBinaryReady &&
    readiness.value?.rcloneBinaryReady &&
    (!readiness.value?.winfspRequired || readiness.value?.winfspInstalled),
);

const readinessStepDescription = computed(() =>
  readiness.value?.winfspRequired
    ? settingsStore.t("dashboard.readiness.withWinfsp")
    : settingsStore.t("dashboard.readiness.withoutWinfsp"),
);

onMounted(async () => {
  await Promise.all([serviceStore.refresh(), mountStore.refresh(), refreshReadiness()]);
});

useIntervalFn(() => {
  void serviceStore.refresh();
  void mountStore.refresh();
}, 5000);

async function refreshReadiness() {
  try {
    readiness.value = await systemApi.readiness();
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function installWinFsp() {
  try {
    await systemApi.installWinFsp();
    message.info(settingsStore.t("dashboard.winfsp.installing"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function resetAdminPassword() {
  try {
    const password = await serviceStore.resetPassword();
    await navigator.clipboard.writeText(password);
    message.success(settingsStore.t("dashboard.password.copied"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function confirmAutoMount() {
  if (!passwordValue.value.trim()) {
    message.warning(settingsStore.t("dashboard.password.required"));
    return;
  }

  try {
    setCachedAdminPassword(passwordValue.value);
    await mountStore.mountAuto(passwordValue.value);
    passwordModalVisible.value = false;
    passwordValue.value = "";
    message.success(settingsStore.t("dashboard.mountAuto.success"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

function openAutoMountPassword() {
  passwordValue.value = getCachedAdminPassword();
  passwordModalVisible.value = true;
  focusPasswordInput();
}

async function focusPasswordInput() {
  await nextTick();
  passwordInputRef.value?.focus();
  passwordInputRef.value?.select();
}

function mountStatusLabel(status: string) {
  return settingsStore.t(`status.${status}`);
}
</script>

<template>
  <div class="page-stack">
    <n-alert v-if="serviceStore.error || mountStore.error" type="error" :show-icon="false">
      {{ serviceStore.error || mountStore.error }}
    </n-alert>

    <div class="panel-grid">
      <ServiceCard
        :title="settingsStore.t('dashboard.service.title')"
        :service="serviceStore.info"
        :status-label="serviceStatusLabel"
        :uptime-text="serviceStore.uptimeText"
        :port-label="settingsStore.t('service.port')"
        :uptime-label="settingsStore.t('service.uptime')"
        :url-label="settingsStore.t('service.url')"
      >
        <template #actions>
          <div class="service-actions">
            <n-button type="primary" :loading="serviceStore.loading" :disabled="!serviceStore.canStart" @click="serviceStore.start">
              <template #icon><Play :size="16" /></template>
              {{ settingsStore.t("dashboard.action.start") }}
            </n-button>
            <n-button secondary :loading="serviceStore.loading" :disabled="!serviceStore.isRunning" @click="serviceStore.restart">
              <template #icon><RotateCw :size="16" /></template>
              {{ settingsStore.t("dashboard.action.restart") }}
            </n-button>
            <n-button secondary type="error" :loading="serviceStore.loading" :disabled="!serviceStore.isRunning" @click="serviceStore.stop">
              <template #icon><Square :size="16" /></template>
              {{ settingsStore.t("dashboard.action.stop") }}
            </n-button>
            <n-button secondary :disabled="!serviceStore.isRunning" @click="serviceStore.openWeb">
              <template #icon><ExternalLink :size="16" /></template>
              {{ settingsStore.t("dashboard.action.openWeb") }}
            </n-button>
            <n-button secondary :disabled="!serviceStore.isRunning" @click="resetAdminPassword">
              <template #icon><KeyRound :size="16" /></template>
              {{ settingsStore.t("dashboard.action.resetPassword") }}
            </n-button>
          </div>
        </template>
      </ServiceCard>

      <n-card :title="settingsStore.t('dashboard.quick.title')" :bordered="true">
        <n-steps vertical size="small" :current="serviceStore.isRunning ? 2 : 1">
          <n-step :title="settingsStore.t('dashboard.step.prepare')" :description="readinessStepDescription" />
          <n-step :title="settingsStore.t('dashboard.step.start')" :description="settingsStore.t('dashboard.step.startDescription')" />
          <n-step :title="settingsStore.t('dashboard.step.createMount')" :description="settingsStore.t('dashboard.step.createMountDescription')" />
          <n-step :title="settingsStore.t('dashboard.step.mount')" :description="settingsStore.t('dashboard.step.mountDescription')" />
        </n-steps>

        <n-space vertical :size="10" class="quick-buttons">
          <n-button block secondary @click="refreshReadiness">
            <template #icon><RefreshCcw :size="16" /></template>
            {{ settingsStore.t("dashboard.action.recheck") }}
          </n-button>
          <n-button
            v-if="readiness?.winfspRequired && !readiness?.winfspInstalled"
            block
            type="warning"
            secondary
            @click="installWinFsp"
          >
            <template #icon><Wrench :size="16" /></template>
            {{ settingsStore.t("settings.winfsp.install") }}
          </n-button>
          <n-button block type="primary" :disabled="serviceStore.isRunning" @click="serviceStore.start">
            <template #icon><Play :size="16" /></template>
            {{ settingsStore.t("dashboard.action.start") }}
          </n-button>
          <n-button block secondary :disabled="!serviceStore.isRunning" @click="openAutoMountPassword">
            {{ settingsStore.t("dashboard.action.mountAuto") }}
          </n-button>
        </n-space>
      </n-card>
    </div>

    <n-card :title="settingsStore.t('dashboard.readiness.title')" :bordered="true">
      <div class="readiness-grid">
        <div class="check-item" :class="{ ok: readiness?.alistBinaryReady }">
          <strong>{{ settingsStore.t("dashboard.readiness.alistBinary") }}</strong>
          <span>{{ readiness?.alistBinaryReady ? settingsStore.t("dashboard.readiness.ready") : settingsStore.t("dashboard.readiness.missing") }}</span>
          <small>{{ readiness?.alistBinaryPath }}</small>
        </div>
        <div class="check-item" :class="{ ok: readiness?.rcloneBinaryReady }">
          <strong>{{ settingsStore.t("dashboard.readiness.rcloneBinary") }}</strong>
          <span>{{ readiness?.rcloneBinaryReady ? settingsStore.t("dashboard.readiness.ready") : settingsStore.t("dashboard.readiness.missing") }}</span>
          <small>{{ readiness?.rcloneBinaryPath }}</small>
        </div>
        <div
          v-if="readiness?.winfspRequired"
          class="check-item"
          :class="{ ok: readiness?.winfspInstalled }"
        >
          <strong>{{ settingsStore.t("dashboard.readiness.winfsp") }}</strong>
          <span>
            {{ readiness?.winfspInstalled ? settingsStore.t("dashboard.readiness.winfspInstalled") : settingsStore.t("dashboard.readiness.winfspNeeded") }}
          </span>
          <small>{{ readiness?.winfspInstallerPath ?? settingsStore.t("dashboard.readiness.winfspHint") }}</small>
        </div>
        <div class="check-item" :class="{ ok: allReady }">
          <strong>{{ settingsStore.t("dashboard.readiness.overall") }}</strong>
          <span>{{ allReady ? settingsStore.t("dashboard.readiness.overallReady") : settingsStore.t("dashboard.readiness.overallPending") }}</span>
          <small>{{ settingsStore.t("dashboard.readiness.overallHint") }}</small>
        </div>
      </div>
    </n-card>

    <n-card :title="settingsStore.t('dashboard.mountSummary.title')" :bordered="true">
      <div class="dashboard-summary">
        <div class="stat-box">
          <span>{{ settingsStore.t("dashboard.mountSummary.active") }}</span>
          <strong>{{ mountStore.activeCount }}/{{ mountStore.totalCount }}</strong>
        </div>
        <div class="stat-box">
          <span>{{ settingsStore.t("dashboard.mountSummary.auto") }}</span>
          <strong>{{ mountStore.mounts.filter((item) => item.autoMount).length }}</strong>
        </div>
        <div class="stat-box">
          <span>{{ settingsStore.t("dashboard.mountSummary.defaultPath") }}</span>
          <strong>{{ mountStore.platform?.defaultMountHint ?? "Z:" }}</strong>
        </div>
      </div>

      <n-empty v-if="mountStore.mounts.length === 0" :description="settingsStore.t('dashboard.mountSummary.empty')">
        <template #extra>
          <n-button type="primary" @click="$router.push('/mount')">{{ settingsStore.t("dashboard.mountSummary.add") }}</n-button>
        </template>
      </n-empty>

      <n-table v-else :bordered="false" size="small" class="mount-table">
        <thead>
          <tr>
            <th>{{ settingsStore.t("dashboard.mountSummary.name") }}</th>
            <th>{{ settingsStore.t("dashboard.mountSummary.remotePath") }}</th>
            <th>{{ settingsStore.t("dashboard.mountSummary.localPath") }}</th>
            <th>{{ settingsStore.t("dashboard.mountSummary.status") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="mount in mountStore.mounts" :key="mount.id">
            <td>{{ mount.name }}</td>
            <td>{{ mount.remotePath }}</td>
            <td>{{ mount.localPath }}</td>
            <td>{{ mountStatusLabel(mount.status) }}</td>
          </tr>
        </tbody>
      </n-table>
    </n-card>

    <n-modal v-model:show="passwordModalVisible" :mask-closable="false">
      <section class="app-dialog wide">
        <header class="app-dialog-header">
          <h2>{{ settingsStore.t("dashboard.passwordModal.title") }}</h2>
          <n-button quaternary circle @click="passwordModalVisible = false">×</n-button>
        </header>
        <div class="app-dialog-body">
          <input
            ref="passwordInputRef"
            v-model="passwordValue"
            class="app-password-input"
            type="password"
            autofocus
            :placeholder="settingsStore.t('dashboard.passwordModal.placeholder')"
            @keyup.enter="confirmAutoMount"
          >
          <n-alert type="info" :show-icon="false">
            {{ settingsStore.t("dashboard.passwordModal.hint") }}
          </n-alert>
        </div>
        <footer class="app-dialog-actions">
          <n-button secondary @click="passwordModalVisible = false">{{ settingsStore.t("common.cancel") }}</n-button>
          <n-button type="primary" @click="confirmAutoMount">{{ settingsStore.t("dashboard.passwordModal.confirm") }}</n-button>
        </footer>
      </section>
    </n-modal>
  </div>
</template>

<style scoped>
.quick-buttons {
  margin-top: 16px;
}

</style>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useMessage } from "naive-ui";
import { Download, KeyRound, Save, Wrench } from "lucide-vue-next";
import { useServiceStore } from "../stores/service";
import { useSettingsStore } from "../stores/settings";
import { systemApi, type WinFspStatus } from "../utils/tauri";

const settingsStore = useSettingsStore();
const serviceStore = useServiceStore();
const message = useMessage();
const newPassword = ref("");
const winfsp = ref<WinFspStatus | null>(null);

const themeOptions = computed(() => [
  { label: settingsStore.t("settings.theme.system"), value: "system" },
  { label: settingsStore.t("settings.theme.light"), value: "light" },
  { label: settingsStore.t("settings.theme.dark"), value: "dark" },
]);

const languageOptions = computed(() => [
  { label: settingsStore.t("settings.language.zh-CN"), value: "zh-CN" },
  { label: settingsStore.t("settings.language.en-US"), value: "en-US" },
]);

const closeActionOptions = computed(() => [
  { label: settingsStore.t("settings.close.ask"), value: "ask" },
  { label: settingsStore.t("settings.close.minimize"), value: "minimize" },
  { label: settingsStore.t("settings.close.exit"), value: "exit" },
]);

onMounted(async () => {
  await settingsStore.load();
  await refreshWinFsp();
});

async function refreshWinFsp() {
  try {
    winfsp.value = await systemApi.winfspStatus();
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function installWinFsp() {
  try {
    await systemApi.installWinFsp();
    message.info(settingsStore.t("settings.winfsp.installing"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function saveSettings() {
  try {
    await settingsStore.save();
    message.success(settingsStore.t("settings.saved"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function saveToggle() {
  try {
    await settingsStore.save();
    message.success(settingsStore.t("settings.toggle.saved"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function updateAutostart(enabled: boolean) {
  try {
    await settingsStore.setAutostart(enabled);
    message.success(
      enabled
        ? settingsStore.t("settings.autostart.enabled")
        : settingsStore.t("settings.autostart.disabled"),
    );
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function setPassword() {
  if (!newPassword.value.trim()) {
    message.warning(settingsStore.t("settings.password.required"));
    return;
  }

  try {
    await serviceStore.setPassword(newPassword.value);
    newPassword.value = "";
    message.success(settingsStore.t("settings.password.updated"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function resetPassword() {
  try {
    const password = await serviceStore.resetPassword();
    await navigator.clipboard.writeText(password);
    message.success(settingsStore.t("settings.password.copied"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function checkUpdates() {
  try {
    await settingsStore.checkUpdates();
    message.success(settingsStore.t("settings.updates.refreshed"));
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}
</script>

<template>
  <div class="page-stack route-placeholder">
    <n-alert v-if="settingsStore.error" type="error" :show-icon="false">
      {{ settingsStore.error }}
    </n-alert>

    <n-card :title="settingsStore.t('settings.app.title')" :bordered="true">
      <n-form label-placement="left" label-width="172">
        <n-form-item :label="settingsStore.t('settings.app.alistPort')">
          <n-input-number v-model:value="settingsStore.config.alistPort" :min="1" :max="65535" />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.autoStartAlist')">
          <n-switch
            v-model:value="settingsStore.config.autoStartAlist"
            :loading="settingsStore.loading"
            @update:value="saveToggle"
          />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.autoMount')">
          <n-switch
            v-model:value="settingsStore.config.autoMount"
            :loading="settingsStore.loading"
            @update:value="saveToggle"
          />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.autostart')">
          <n-switch
            :value="settingsStore.autostartEnabled"
            :loading="settingsStore.loading"
            @update:value="updateAutostart"
          />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.theme')">
          <n-select v-model:value="settingsStore.config.theme" :options="themeOptions" />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.language')">
          <n-select v-model:value="settingsStore.config.language" :options="languageOptions" />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.startMinimized')">
          <n-switch
            v-model:value="settingsStore.config.startMinimized"
            :loading="settingsStore.loading"
            @update:value="saveToggle"
          />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.checkUpdates')">
          <n-switch v-model:value="settingsStore.config.checkUpdates" />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.closeAction')">
          <n-select v-model:value="settingsStore.config.closeAction" :options="closeActionOptions" />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.alistBinaryPath')">
          <n-input
            v-model:value="settingsStore.config.alistBinaryPath"
            clearable
            :placeholder="settingsStore.t('settings.app.alistBinaryPlaceholder')"
          />
        </n-form-item>
        <n-form-item :label="settingsStore.t('settings.app.rcloneBinaryPath')">
          <n-input
            v-model:value="settingsStore.config.rcloneBinaryPath"
            clearable
            :placeholder="settingsStore.t('settings.app.rcloneBinaryPlaceholder')"
          />
        </n-form-item>
      </n-form>

      <n-space>
        <n-button type="primary" :loading="settingsStore.loading" @click="saveSettings">
          <template #icon><Save :size="16" /></template>
          {{ settingsStore.t("common.save") }}
        </n-button>
      </n-space>
    </n-card>

    <n-card v-if="winfsp?.required" :title="settingsStore.t('settings.winfsp.title')" :bordered="true">
      <n-descriptions :column="1" bordered size="small">
        <n-descriptions-item :label="settingsStore.t('settings.winfsp.required')">
          {{ settingsStore.t("settings.winfsp.requiredDescription") }}
        </n-descriptions-item>
        <n-descriptions-item :label="settingsStore.t('settings.winfsp.installed')">
          {{ winfsp?.installed ? settingsStore.t("settings.winfsp.installedYes") : settingsStore.t("settings.winfsp.installedNo") }}
        </n-descriptions-item>
        <n-descriptions-item :label="settingsStore.t('settings.winfsp.installer')">
          {{ winfsp?.installerPath ?? settingsStore.t("settings.winfsp.installerMissing") }}
        </n-descriptions-item>
      </n-descriptions>
      <n-space class="settings-actions">
        <n-button secondary @click="refreshWinFsp">
          {{ settingsStore.t("settings.winfsp.refresh") }}
        </n-button>
        <n-button v-if="winfsp?.required && !winfsp?.installed" type="warning" secondary @click="installWinFsp">
          <template #icon><Wrench :size="16" /></template>
          {{ settingsStore.t("settings.winfsp.install") }}
        </n-button>
      </n-space>
    </n-card>

    <n-card :title="settingsStore.t('settings.admin.title')" :bordered="true">
      <n-space vertical>
        <n-input
          v-model:value="newPassword"
          type="password"
          show-password-on="click"
          :placeholder="settingsStore.t('settings.admin.placeholder')"
        />
        <n-space>
          <n-button type="primary" @click="setPassword">
            <template #icon><KeyRound :size="16" /></template>
            {{ settingsStore.t("settings.admin.setPassword") }}
          </n-button>
          <n-button secondary @click="resetPassword">{{ settingsStore.t("settings.admin.resetPassword") }}</n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-card :title="settingsStore.t('settings.updates.title')" :bordered="true">
      <n-space vertical>
        <n-button :loading="settingsStore.loading" @click="checkUpdates">
          <template #icon><Download :size="16" /></template>
          {{ settingsStore.t("settings.updates.check") }}
        </n-button>

        <n-descriptions v-if="settingsStore.updateResult" :column="1" bordered size="small">
          <n-descriptions-item label="AList Desktop">
            {{ settingsStore.updateResult.appVersion }}
          </n-descriptions-item>
          <n-descriptions-item label="AList">
            <a :href="settingsStore.updateResult.alist.htmlUrl" target="_blank">
              {{ settingsStore.updateResult.alist.tagName }}
            </a>
            - {{ settingsStore.updateResult.alist.name }}
          </n-descriptions-item>
          <n-descriptions-item label="Rclone">
            <a :href="settingsStore.updateResult.rclone.htmlUrl" target="_blank">
              {{ settingsStore.updateResult.rclone.tagName }}
            </a>
            - {{ settingsStore.updateResult.rclone.name }}
          </n-descriptions-item>
        </n-descriptions>
      </n-space>
    </n-card>
  </div>
</template>

<style scoped>
.settings-actions {
  margin-top: 14px;
}
</style>

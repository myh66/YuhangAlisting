<script setup lang="ts">
import { onMounted, ref } from "vue";
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

const themeOptions = [
  { label: "跟随系统 / System", value: "system" },
  { label: "浅色 / Light", value: "light" },
  { label: "深色 / Dark", value: "dark" },
];

const languageOptions = [
  { label: "中文", value: "zh-CN" },
  { label: "English", value: "en-US" },
];

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
    message.info("WinFsp 安装器已启动，请在系统提示中确认。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function saveSettings() {
  try {
    await settingsStore.save();
    message.success("设置已保存。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function updateAutostart(enabled: boolean) {
  try {
    await settingsStore.setAutostart(enabled);
    message.success(enabled ? "开机自启已启用。" : "开机自启已关闭。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function setPassword() {
  if (!newPassword.value.trim()) {
    message.warning("请输入新密码。");
    return;
  }

  try {
    await serviceStore.setPassword(newPassword.value);
    newPassword.value = "";
    message.success("管理员密码已设置。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function resetPassword() {
  try {
    const password = await serviceStore.resetPassword();
    await navigator.clipboard.writeText(password);
    message.success("新密码已复制。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function checkUpdates() {
  try {
    await settingsStore.checkUpdates();
    message.success("更新信息已刷新。");
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

    <n-card title="应用设置" :bordered="true">
      <n-form label-placement="left" label-width="172">
        <n-form-item label="AList 端口">
          <n-input-number v-model:value="settingsStore.config.alistPort" :min="1" :max="65535" />
        </n-form-item>
        <n-form-item label="打开应用后启动 AList">
          <n-switch v-model:value="settingsStore.config.autoStartAlist" />
        </n-form-item>
        <n-form-item label="打开应用后自动挂载">
          <n-switch v-model:value="settingsStore.config.autoMount" />
        </n-form-item>
        <n-form-item label="系统登录时启动应用">
          <n-switch
            :value="settingsStore.autostartEnabled"
            :loading="settingsStore.loading"
            @update:value="updateAutostart"
          />
        </n-form-item>
        <n-form-item label="主题">
          <n-select v-model:value="settingsStore.config.theme" :options="themeOptions" />
        </n-form-item>
        <n-form-item label="语言">
          <n-select v-model:value="settingsStore.config.language" :options="languageOptions" />
        </n-form-item>
        <n-form-item label="启动时最小化">
          <n-switch v-model:value="settingsStore.config.startMinimized" />
        </n-form-item>
        <n-form-item label="自动检查更新">
          <n-switch v-model:value="settingsStore.config.checkUpdates" />
        </n-form-item>
        <n-form-item label="AList 二进制路径">
          <n-input v-model:value="settingsStore.config.alistBinaryPath" clearable placeholder="留空使用内置 binaries/alist" />
        </n-form-item>
        <n-form-item label="Rclone 二进制路径">
          <n-input v-model:value="settingsStore.config.rcloneBinaryPath" clearable placeholder="留空使用内置 binaries/rclone" />
        </n-form-item>
      </n-form>

      <n-space>
        <n-button type="primary" :loading="settingsStore.loading" @click="saveSettings">
          <template #icon><Save :size="16" /></template>
          保存设置
        </n-button>
      </n-space>
    </n-card>

    <n-card title="Windows 挂载驱动" :bordered="true">
      <n-descriptions :column="1" bordered size="small">
        <n-descriptions-item label="是否需要">
          {{ winfsp?.required ? "Windows 盘符挂载需要 WinFsp" : "当前系统不需要" }}
        </n-descriptions-item>
        <n-descriptions-item label="安装状态">
          {{ winfsp?.installed ? "已安装" : "未安装" }}
        </n-descriptions-item>
        <n-descriptions-item label="随包安装器">
          {{ winfsp?.installerPath ?? "未找到 winfsp.msi" }}
        </n-descriptions-item>
      </n-descriptions>
      <n-space class="settings-actions">
        <n-button secondary @click="refreshWinFsp">
          刷新状态
        </n-button>
        <n-button v-if="winfsp?.required && !winfsp?.installed" type="warning" secondary @click="installWinFsp">
          <template #icon><Wrench :size="16" /></template>
          安装 WinFsp
        </n-button>
      </n-space>
    </n-card>

    <n-card title="AList 管理员" :bordered="true">
      <n-space vertical>
        <n-input
          v-model:value="newPassword"
          type="password"
          show-password-on="click"
          placeholder="输入新的管理员密码"
        />
        <n-space>
          <n-button type="primary" @click="setPassword">
            <template #icon><KeyRound :size="16" /></template>
            设置密码
          </n-button>
          <n-button secondary @click="resetPassword">随机重置并复制</n-button>
        </n-space>
      </n-space>
    </n-card>

    <n-card title="更新检查" :bordered="true">
      <n-space vertical>
        <n-button :loading="settingsStore.loading" @click="checkUpdates">
          <template #icon><Download :size="16" /></template>
          检查 AList / Rclone 最新版本
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

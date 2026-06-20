<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useIntervalFn } from "@vueuse/core";
import { useMessage } from "naive-ui";
import { ExternalLink, KeyRound, Play, RefreshCcw, RotateCw, Square, Wrench } from "lucide-vue-next";
import ServiceCard from "../components/ServiceCard.vue";
import { useMountStore } from "../stores/mounts";
import { useServiceStore } from "../stores/service";
import { systemApi, type RuntimeReadiness } from "../utils/tauri";

const serviceStore = useServiceStore();
const mountStore = useMountStore();
const message = useMessage();

const readiness = ref<RuntimeReadiness | null>(null);
const passwordModalVisible = ref(false);
const passwordValue = ref("");

const allReady = computed(
  () =>
    readiness.value?.alistBinaryReady &&
    readiness.value?.rcloneBinaryReady &&
    (!readiness.value?.winfspRequired || readiness.value?.winfspInstalled),
);

const readinessStepDescription = computed(() =>
  readiness.value?.winfspRequired
    ? "确认 AList、Rclone、WinFsp 状态"
    : "确认 AList、Rclone 状态",
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
    message.info("WinFsp 安装器已启动，请在系统提示中确认。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function resetAdminPassword() {
  try {
    const password = await serviceStore.resetPassword();
    await navigator.clipboard.writeText(password);
    message.success("新管理员密码已复制。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function confirmAutoMount() {
  if (!passwordValue.value.trim()) {
    message.warning("请输入 AList admin 密码。");
    return;
  }

  try {
    await mountStore.mountAuto(passwordValue.value);
    passwordModalVisible.value = false;
    passwordValue.value = "";
    message.success("自动挂载项已执行。");
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}
</script>

<template>
  <div class="page-stack">
    <n-alert v-if="serviceStore.error || mountStore.error" type="error" :show-icon="false">
      {{ serviceStore.error || mountStore.error }}
    </n-alert>

    <div class="panel-grid">
      <ServiceCard
        title="AList 服务"
        :service="serviceStore.info"
        :status-label="serviceStore.serviceStatusLabel"
        :uptime-text="serviceStore.uptimeText"
      >
        <template #actions>
          <div class="service-actions">
            <n-button type="primary" :loading="serviceStore.loading" :disabled="!serviceStore.canStart" @click="serviceStore.start">
              <template #icon><Play :size="16" /></template>
              启动
            </n-button>
            <n-button secondary :loading="serviceStore.loading" :disabled="!serviceStore.isRunning" @click="serviceStore.restart">
              <template #icon><RotateCw :size="16" /></template>
              重启
            </n-button>
            <n-button secondary type="error" :loading="serviceStore.loading" :disabled="!serviceStore.isRunning" @click="serviceStore.stop">
              <template #icon><Square :size="16" /></template>
              停止
            </n-button>
            <n-button secondary :disabled="!serviceStore.isRunning" @click="serviceStore.openWeb">
              <template #icon><ExternalLink :size="16" /></template>
              打开网页
            </n-button>
            <n-button secondary :disabled="!serviceStore.isRunning" @click="resetAdminPassword">
              <template #icon><KeyRound :size="16" /></template>
              重置密码
            </n-button>
          </div>
        </template>
      </ServiceCard>

      <n-card title="快速流程" :bordered="true">
        <n-steps vertical size="small" :current="serviceStore.isRunning ? 2 : 1">
          <n-step title="准备环境" :description="readinessStepDescription" />
          <n-step title="启动 AList" description="运行本地 WebDAV 服务并健康检查" />
          <n-step title="创建挂载" description="配置 AList 路径和本地盘符或目录" />
          <n-step title="挂载到本地" description="输入 admin 密码后交给 Rclone 挂载" />
        </n-steps>

        <n-space vertical :size="10" class="quick-buttons">
          <n-button block secondary @click="refreshReadiness">
            <template #icon><RefreshCcw :size="16" /></template>
            重新检查环境
          </n-button>
          <n-button
            v-if="readiness?.winfspRequired && !readiness?.winfspInstalled"
            block
            type="warning"
            secondary
            @click="installWinFsp"
          >
            <template #icon><Wrench :size="16" /></template>
            安装 WinFsp
          </n-button>
          <n-button block type="primary" :disabled="serviceStore.isRunning" @click="serviceStore.start">
            <template #icon><Play :size="16" /></template>
            启动 AList
          </n-button>
          <n-button block secondary :disabled="!serviceStore.isRunning" @click="passwordModalVisible = true">
            挂载全部自动项
          </n-button>
        </n-space>
      </n-card>
    </div>

    <n-card title="运行环境" :bordered="true">
      <div class="readiness-grid">
        <div class="check-item" :class="{ ok: readiness?.alistBinaryReady }">
          <strong>AList 二进制</strong>
          <span>{{ readiness?.alistBinaryReady ? "已就绪" : "未找到" }}</span>
          <small>{{ readiness?.alistBinaryPath }}</small>
        </div>
        <div class="check-item" :class="{ ok: readiness?.rcloneBinaryReady }">
          <strong>Rclone 二进制</strong>
          <span>{{ readiness?.rcloneBinaryReady ? "已就绪" : "未找到" }}</span>
          <small>{{ readiness?.rcloneBinaryPath }}</small>
        </div>
        <div
          v-if="readiness?.winfspRequired"
          class="check-item"
          :class="{ ok: readiness?.winfspInstalled }"
        >
          <strong>Windows 挂载驱动</strong>
          <span>
            {{ readiness?.winfspInstalled ? "WinFsp 已安装" : "需要安装 WinFsp" }}
          </span>
          <small>{{ readiness?.winfspInstallerPath ?? "随安装包提供或运行 prebuild 获取" }}</small>
        </div>
        <div class="check-item" :class="{ ok: allReady }">
          <strong>整体状态</strong>
          <span>{{ allReady ? "可以使用" : "需要处理" }}</span>
          <small>处理缺失项后重新检查</small>
        </div>
      </div>
    </n-card>

    <n-card title="挂载概览" :bordered="true">
      <div class="dashboard-summary">
        <div class="stat-box">
          <span>活跃挂载</span>
          <strong>{{ mountStore.activeCount }}/{{ mountStore.totalCount }}</strong>
        </div>
        <div class="stat-box">
          <span>自动挂载</span>
          <strong>{{ mountStore.mounts.filter((item) => item.autoMount).length }}</strong>
        </div>
        <div class="stat-box">
          <span>默认挂载点</span>
          <strong>{{ mountStore.platform?.defaultMountHint ?? "Z:" }}</strong>
        </div>
      </div>

      <n-empty v-if="mountStore.mounts.length === 0" description="还没有挂载配置">
        <template #extra>
          <n-button type="primary" @click="$router.push('/mount')">添加挂载</n-button>
        </template>
      </n-empty>

      <n-table v-else :bordered="false" size="small" class="mount-table">
        <thead>
          <tr>
            <th>名称</th>
            <th>AList 路径</th>
            <th>本地路径</th>
            <th>状态</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="mount in mountStore.mounts" :key="mount.id">
            <td>{{ mount.name }}</td>
            <td>{{ mount.remotePath }}</td>
            <td>{{ mount.localPath }}</td>
            <td>{{ mount.status }}</td>
          </tr>
        </tbody>
      </n-table>
    </n-card>

    <n-modal v-model:show="passwordModalVisible" preset="card" title="输入 AList admin 密码" class="password-modal">
      <n-space vertical>
        <n-input
          v-model:value="passwordValue"
          type="password"
          show-password-on="click"
          placeholder="AList admin 密码"
          @keyup.enter="confirmAutoMount"
        />
        <n-alert type="info" :show-icon="false">
          AList 不能反查当前密码；忘记时请先重置管理员密码。
        </n-alert>
        <n-space justify="end">
          <n-button secondary @click="passwordModalVisible = false">取消</n-button>
          <n-button type="primary" @click="confirmAutoMount">开始挂载</n-button>
        </n-space>
      </n-space>
    </n-modal>
  </div>
</template>

<style scoped>
.quick-buttons {
  margin-top: 16px;
}

.password-modal {
  width: min(460px, calc(100vw - 32px));
}
</style>

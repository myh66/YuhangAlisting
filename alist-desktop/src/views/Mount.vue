<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useDialog, useMessage } from "naive-ui";
import { FolderOpen, Plus, RefreshCcw, Save, Trash2 } from "lucide-vue-next";
import {
  cacheModeOptions,
  createEmptyMount,
  useMountStore,
} from "../stores/mounts";
import type { MountConfig, MountInfo } from "../utils/tauri";

const mountStore = useMountStore();
const message = useMessage();
const dialog = useDialog();

const form = ref<MountConfig>(createEmptyMount());
const editing = computed(() => form.value.id.length > 0);
const passwordModalVisible = ref(false);
const passwordValue = ref("");
const pendingMountId = ref<string | null>(null);

onMounted(async () => {
  await mountStore.refresh();
  form.value = createEmptyMount(mountStore.platform?.defaultMountHint ?? "Z:");
});

function editMount(mount: MountInfo) {
  form.value = {
    id: mount.id,
    name: mount.name,
    remotePath: mount.remotePath,
    localPath: mount.localPath,
    autoMount: mount.autoMount,
    cacheMode: mount.cacheMode,
    bufferSize: mount.bufferSize,
    vfsCacheMaxAge: mount.vfsCacheMaxAge,
    readOnly: mount.readOnly,
  };
}

function resetForm() {
  form.value = createEmptyMount(mountStore.platform?.defaultMountHint ?? "Z:");
}

async function saveMount() {
  try {
    if (editing.value) {
      await mountStore.update(form.value);
      message.success("挂载配置已更新。");
    } else {
      await mountStore.create(form.value);
      message.success("挂载配置已保存。");
    }

    resetForm();
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function removeMount(mount: MountInfo) {
  dialog.warning({
    title: "删除挂载",
    content: `确认删除 ${mount.name}？正在运行的挂载会先卸载。`,
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        await mountStore.remove(mount.id);
        message.success("挂载配置已删除。");
      } catch (err) {
        message.error(err instanceof Error ? err.message : String(err));
      }
    },
  });
}

async function runMountAction(action: () => Promise<void>) {
  try {
    await action();
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

function mountWithPassword(id: string) {
  pendingMountId.value = id;
  passwordValue.value = "";
  passwordModalVisible.value = true;
}

function mountAutoWithPassword() {
  pendingMountId.value = null;
  passwordValue.value = "";
  passwordModalVisible.value = true;
}

async function confirmMountPassword() {
  if (!passwordValue.value.trim()) {
    message.warning("请输入 AList admin 密码。");
    return;
  }

  await runMountAction(async () => {
    if (pendingMountId.value) {
      await mountStore.mount(pendingMountId.value, passwordValue.value);
    } else {
      await mountStore.mountAuto(passwordValue.value);
    }

    passwordModalVisible.value = false;
  });
}

function statusType(status: MountInfo["status"]) {
  if (status === "mounted") return "success";
  if (status === "mounting") return "warning";
  if (status === "error") return "error";
  return "default";
}
</script>

<template>
  <div class="page-stack">
    <n-alert v-if="mountStore.error" type="error" :show-icon="false">
      {{ mountStore.error }}
    </n-alert>

    <n-card title="挂载配置" :bordered="true">
      <template #header-extra>
        <n-space>
          <n-button secondary :loading="mountStore.loading" @click="mountStore.refresh">
            <template #icon><RefreshCcw :size="16" /></template>
            刷新
          </n-button>
          <n-button secondary :disabled="mountStore.mounts.length === 0" @click="mountAutoWithPassword">
            自动项
          </n-button>
          <n-button secondary :disabled="mountStore.activeCount === 0" @click="mountStore.unmountAll">
            全部卸载
          </n-button>
        </n-space>
      </template>

      <n-table :bordered="false" size="small">
        <thead>
          <tr>
            <th>名称</th>
            <th>AList 路径</th>
            <th>本地路径</th>
            <th>缓存</th>
            <th>自动</th>
            <th>状态</th>
            <th class="actions-col">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="mountStore.mounts.length === 0">
            <td colspan="7" class="muted">暂无挂载配置</td>
          </tr>
          <tr v-for="mount in mountStore.mounts" :key="mount.id">
            <td>{{ mount.name }}</td>
            <td>{{ mount.remotePath }}</td>
            <td>{{ mount.localPath }}</td>
            <td>{{ mount.cacheMode }}</td>
            <td>{{ mount.autoMount ? "是" : "否" }}</td>
            <td>
              <n-tag :type="statusType(mount.status)" size="small">
                {{ mount.status }}
              </n-tag>
            </td>
            <td>
              <n-space :size="6">
                <n-button size="tiny" secondary :disabled="mount.status === 'mounted' || mount.status === 'mounting'" @click="mountWithPassword(mount.id)">
                  挂载
                </n-button>
                <n-button size="tiny" secondary :disabled="mount.status !== 'mounted'" @click="runMountAction(() => mountStore.unmount(mount.id).then(() => undefined))">
                  卸载
                </n-button>
                <n-button size="tiny" secondary @click="mountStore.openPath(mount.id)">
                  <template #icon><FolderOpen :size="14" /></template>
                </n-button>
                <n-button size="tiny" secondary @click="editMount(mount)">编辑</n-button>
                <n-button size="tiny" secondary type="error" @click="removeMount(mount)">
                  <template #icon><Trash2 :size="14" /></template>
                </n-button>
              </n-space>
              <div v-if="mount.error" class="table-error">{{ mount.error }}</div>
            </td>
          </tr>
        </tbody>
      </n-table>
    </n-card>

    <n-card :title="editing ? '编辑挂载' : '新增挂载'" :bordered="true">
      <n-form label-placement="left" label-width="116">
        <n-grid :cols="2" :x-gap="16" responsive="screen">
          <n-form-item-gi label="显示名称">
            <n-input v-model:value="form.name" placeholder="例如 阿里云盘" />
          </n-form-item-gi>
          <n-form-item-gi label="远程路径">
            <n-input v-model:value="form.remotePath" placeholder="/aliyundrive" />
          </n-form-item-gi>
          <n-form-item-gi label="本地路径">
            <n-input v-model:value="form.localPath" :placeholder="mountStore.platform?.defaultMountHint ?? 'Z:'" />
          </n-form-item-gi>
          <n-form-item-gi label="缓存模式">
            <n-select v-model:value="form.cacheMode" :options="cacheModeOptions" />
          </n-form-item-gi>
          <n-form-item-gi label="Buffer size">
            <n-input v-model:value="form.bufferSize" placeholder="256M" />
          </n-form-item-gi>
          <n-form-item-gi label="缓存有效期">
            <n-input v-model:value="form.vfsCacheMaxAge" placeholder="1h" />
          </n-form-item-gi>
          <n-form-item-gi label="自动挂载">
            <n-switch v-model:value="form.autoMount" />
          </n-form-item-gi>
          <n-form-item-gi label="只读">
            <n-switch v-model:value="form.readOnly" />
          </n-form-item-gi>
        </n-grid>

        <n-space>
          <n-button type="primary" :loading="mountStore.loading" @click="saveMount">
            <template #icon><Save :size="16" /></template>
            {{ editing ? "保存修改" : "保存配置" }}
          </n-button>
          <n-button secondary @click="resetForm">
            <template #icon><Plus :size="16" /></template>
            新建
          </n-button>
        </n-space>
      </n-form>
    </n-card>

    <n-modal v-model:show="passwordModalVisible" preset="card" title="输入 AList admin 密码" class="password-modal">
      <n-space vertical>
        <n-input
          v-model:value="passwordValue"
          type="password"
          show-password-on="click"
          placeholder="AList admin 密码"
          @keyup.enter="confirmMountPassword"
        />
        <n-alert type="info" :show-icon="false">
          密码只用于本次 Rclone 挂载，应用会自动转换为 Rclone 需要的 obscure 格式。
        </n-alert>
        <n-space justify="end">
          <n-button secondary @click="passwordModalVisible = false">取消</n-button>
          <n-button type="primary" @click="confirmMountPassword">
            {{ pendingMountId ? "挂载此项" : "挂载自动项" }}
          </n-button>
        </n-space>
      </n-space>
    </n-modal>
  </div>
</template>

<style scoped>
.actions-col {
  width: 340px;
}

.table-error {
  margin-top: 6px;
  color: var(--danger);
  font-size: 12px;
}

.password-modal {
  width: min(460px, calc(100vw - 32px));
}
</style>

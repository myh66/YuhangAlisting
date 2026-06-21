<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useDialog, useMessage } from "naive-ui";
import { FolderOpen, Plus, RefreshCcw, Save, Trash2 } from "lucide-vue-next";
import {
  cacheModeOptions,
  createEmptyMount,
  useMountStore,
} from "../stores/mounts";
import { useSettingsStore } from "../stores/settings";
import type { MountConfig, MountInfo } from "../utils/tauri";

const mountStore = useMountStore();
const settingsStore = useSettingsStore();
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
      const currentMount = mountStore.mounts.find((mount) => mount.id === form.value.id);
      const needsRemount =
        currentMount?.status === "mounted" &&
        (currentMount.name !== form.value.name ||
          currentMount.remotePath !== form.value.remotePath ||
          currentMount.localPath !== form.value.localPath ||
          currentMount.cacheMode !== form.value.cacheMode ||
          currentMount.bufferSize !== form.value.bufferSize ||
          currentMount.vfsCacheMaxAge !== form.value.vfsCacheMaxAge ||
          currentMount.readOnly !== form.value.readOnly);

      await mountStore.update(form.value);
      if (needsRemount) {
        message.warning(settingsStore.t("mount.updatedNeedsRemount"));
      } else {
        message.success(settingsStore.t("mount.updated"));
      }
    } else {
      await mountStore.create(form.value);
      message.success(settingsStore.t("mount.saved"));
    }

    resetForm();
  } catch (err) {
    message.error(err instanceof Error ? err.message : String(err));
  }
}

async function removeMount(mount: MountInfo) {
  dialog.warning({
    title: settingsStore.t("mount.delete.title"),
    content: settingsStore.format("mount.delete.confirm", { name: mount.name }),
    positiveText: settingsStore.t("common.delete"),
    negativeText: settingsStore.t("common.cancel"),
    onPositiveClick: async () => {
      try {
        await mountStore.remove(mount.id);
        message.success(settingsStore.t("mount.deleted"));
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
    message.warning(settingsStore.t("mount.password.required"));
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

function statusLabel(status: MountInfo["status"]) {
  return settingsStore.t(`status.${status}`);
}
</script>

<template>
  <div class="page-stack">
    <n-alert v-if="mountStore.error" type="error" :show-icon="false">
      {{ mountStore.error }}
    </n-alert>

    <n-card :title="settingsStore.t('mount.title')" :bordered="true">
      <template #header-extra>
        <n-space>
          <n-button secondary :loading="mountStore.loading" @click="mountStore.refresh">
            <template #icon><RefreshCcw :size="16" /></template>
            {{ settingsStore.t("common.refresh") }}
          </n-button>
          <n-button secondary :disabled="mountStore.mounts.length === 0" @click="mountAutoWithPassword">
            {{ settingsStore.t("mount.action.auto") }}
          </n-button>
          <n-button secondary :disabled="mountStore.activeCount === 0" @click="mountStore.unmountAll">
            {{ settingsStore.t("mount.action.unmountAll") }}
          </n-button>
        </n-space>
      </template>

      <n-table :bordered="false" size="small">
        <thead>
          <tr>
            <th>{{ settingsStore.t("mount.column.name") }}</th>
            <th>{{ settingsStore.t("mount.column.remotePath") }}</th>
            <th>{{ settingsStore.t("mount.column.localPath") }}</th>
            <th>{{ settingsStore.t("mount.column.cache") }}</th>
            <th>{{ settingsStore.t("mount.column.auto") }}</th>
            <th>{{ settingsStore.t("mount.column.status") }}</th>
            <th class="actions-col">{{ settingsStore.t("mount.column.actions") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="mountStore.mounts.length === 0">
            <td colspan="7" class="muted">{{ settingsStore.t("mount.empty") }}</td>
          </tr>
          <tr v-for="mount in mountStore.mounts" :key="mount.id">
            <td>{{ mount.name }}</td>
            <td>{{ mount.remotePath }}</td>
            <td>{{ mount.localPath }}</td>
            <td>{{ mount.cacheMode }}</td>
            <td>{{ mount.autoMount ? settingsStore.t("mount.auto.yes") : settingsStore.t("mount.auto.no") }}</td>
            <td>
              <n-tag :type="statusType(mount.status)" size="small">
                {{ statusLabel(mount.status) }}
              </n-tag>
            </td>
            <td>
              <n-space :size="6">
                <n-button size="tiny" secondary :disabled="mount.status === 'mounted' || mount.status === 'mounting'" @click="mountWithPassword(mount.id)">
                  {{ settingsStore.t("mount.action.mount") }}
                </n-button>
                <n-button size="tiny" secondary :disabled="mount.status !== 'mounted'" @click="runMountAction(() => mountStore.unmount(mount.id).then(() => undefined))">
                  {{ settingsStore.t("mount.action.unmount") }}
                </n-button>
                <n-button size="tiny" secondary @click="mountStore.openPath(mount.id)">
                  <template #icon><FolderOpen :size="14" /></template>
                </n-button>
                <n-button size="tiny" secondary @click="editMount(mount)">{{ settingsStore.t("common.edit") }}</n-button>
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

    <n-card :title="editing ? settingsStore.t('mount.form.editTitle') : settingsStore.t('mount.form.createTitle')" :bordered="true">
      <n-form label-placement="left" label-width="116">
        <n-grid :cols="2" :x-gap="16" responsive="screen">
          <n-form-item-gi :label="settingsStore.t('mount.form.name')">
            <n-input v-model:value="form.name" :placeholder="settingsStore.t('mount.form.namePlaceholder')" />
          </n-form-item-gi>
          <n-form-item-gi :label="settingsStore.t('mount.form.remotePath')">
            <n-input v-model:value="form.remotePath" placeholder="/" />
          </n-form-item-gi>
          <n-form-item-gi :label="settingsStore.t('mount.form.localPath')">
            <n-input v-model:value="form.localPath" :placeholder="mountStore.platform?.defaultMountHint ?? 'Z:'" />
          </n-form-item-gi>
          <n-form-item-gi :label="settingsStore.t('mount.form.cacheMode')">
            <n-select v-model:value="form.cacheMode" :options="cacheModeOptions" />
          </n-form-item-gi>
          <n-form-item-gi label="Buffer size">
            <n-input v-model:value="form.bufferSize" placeholder="256M" />
          </n-form-item-gi>
          <n-form-item-gi :label="settingsStore.t('mount.form.cacheMaxAge')">
            <n-input v-model:value="form.vfsCacheMaxAge" placeholder="1h" />
          </n-form-item-gi>
          <n-form-item-gi :label="settingsStore.t('mount.form.autoMount')">
            <n-switch v-model:value="form.autoMount" />
          </n-form-item-gi>
          <n-form-item-gi :label="settingsStore.t('mount.form.readOnly')">
            <n-switch v-model:value="form.readOnly" />
          </n-form-item-gi>
        </n-grid>

        <n-space>
          <n-button type="primary" :loading="mountStore.loading" @click="saveMount">
            <template #icon><Save :size="16" /></template>
            {{ editing ? settingsStore.t("mount.form.saveEdit") : settingsStore.t("mount.form.saveCreate") }}
          </n-button>
          <n-button secondary @click="resetForm">
            <template #icon><Plus :size="16" /></template>
            {{ settingsStore.t("common.create") }}
          </n-button>
        </n-space>
      </n-form>
    </n-card>

    <n-modal v-model:show="passwordModalVisible" preset="card" :title="settingsStore.t('mount.passwordModal.title')" class="password-modal">
      <n-space vertical>
        <n-input
          v-model:value="passwordValue"
          type="password"
          show-password-on="click"
          :placeholder="settingsStore.t('mount.passwordModal.placeholder')"
          @keyup.enter="confirmMountPassword"
        />
        <n-alert type="info" :show-icon="false">
          {{ settingsStore.t("mount.passwordModal.hint") }}
        </n-alert>
        <n-space justify="end">
          <n-button secondary @click="passwordModalVisible = false">{{ settingsStore.t("common.cancel") }}</n-button>
          <n-button type="primary" @click="confirmMountPassword">
            {{ pendingMountId ? settingsStore.t("mount.passwordModal.confirmSingle") : settingsStore.t("mount.passwordModal.confirmAuto") }}
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

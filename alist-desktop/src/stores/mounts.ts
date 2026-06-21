import { defineStore } from "pinia";
import { computed, ref } from "vue";
import {
  mountApi,
  systemApi,
  type CacheMode,
  type MountConfig,
  type MountInfo,
  type PlatformInfo,
} from "../utils/tauri";

const LAST_LOCAL_PATH_KEY = "yuhang-alisting:last-mount-local-path";

export function getLastLocalPath(defaultLocalPath = "Z:") {
  return localStorage.getItem(LAST_LOCAL_PATH_KEY) || defaultLocalPath;
}

export function rememberLocalPath(localPath: string) {
  const value = localPath.trim();

  if (value) {
    localStorage.setItem(LAST_LOCAL_PATH_KEY, value);
  }
}

export function createEmptyMount(defaultLocalPath = "Z:"): MountConfig {
  return {
    id: "",
    name: "",
    remotePath: "",
    localPath: defaultLocalPath,
    autoMount: false,
    cacheMode: "full",
    bufferSize: "256M",
    vfsCacheMaxAge: "1h",
    readOnly: false,
  };
}

export const cacheModeOptions: Array<{ label: string; value: CacheMode }> = [
  { label: "off", value: "off" },
  { label: "minimal", value: "minimal" },
  { label: "writes", value: "writes" },
  { label: "full", value: "full" },
];

export const useMountStore = defineStore("mounts", () => {
  const mounts = ref<MountInfo[]>([]);
  const platform = ref<PlatformInfo | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const activeCount = computed(
    () => mounts.value.filter((mount) => mount.status === "mounted").length,
  );

  const totalCount = computed(() => mounts.value.length);

  async function refresh() {
    loading.value = true;
    error.value = null;

    try {
      const [mountList, platformInfo] = await Promise.all([
        mountApi.list(),
        platform.value ? Promise.resolve(platform.value) : systemApi.platform(),
      ]);
      mounts.value = mountList;
      platform.value = platformInfo;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  }

  async function create(config: MountConfig) {
    loading.value = true;
    error.value = null;

    try {
      rememberLocalPath(config.localPath);
      await mountApi.create(config);
      mounts.value = await mountApi.list();
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function update(config: MountConfig) {
    loading.value = true;
    error.value = null;

    try {
      rememberLocalPath(config.localPath);
      await mountApi.update(config);
      mounts.value = await mountApi.list();
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function remove(id: string) {
    mounts.value = await mountApi.delete(id);
  }

  async function mount(id: string, password?: string) {
    mounts.value = await mountApi.mount(id, password);
  }

  async function unmount(id: string) {
    mounts.value = await mountApi.unmount(id);
  }

  async function unmountAll() {
    mounts.value = await mountApi.unmountAll();
  }

  async function mountAuto(password?: string) {
    mounts.value = await mountApi.mountAuto(password);
  }

  async function openPath(id: string) {
    await mountApi.openPath(id);
  }

  return {
    mounts,
    platform,
    loading,
    error,
    activeCount,
    totalCount,
    refresh,
    create,
    update,
    remove,
    mount,
    unmount,
    unmountAll,
    mountAuto,
    openPath,
  };
});

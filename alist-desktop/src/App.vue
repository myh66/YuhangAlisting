<script setup lang="ts">
import { computed, h, onMounted, onUnmounted, ref, watch } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { RouterLink, RouterView, useRoute } from "vue-router";
import type { MenuOption } from "naive-ui";
import { Activity, FileText, HardDrive, Settings } from "lucide-vue-next";
import appIcon from "./assets/app-icon.png";
import { useServiceStore } from "./stores/service";
import { useSettingsStore } from "./stores/settings";
import { systemApi, type CloseAction, type Language } from "./utils/tauri";

const route = useRoute();
const serviceStore = useServiceStore();
const settingsStore = useSettingsStore();
const colorScheme = window.matchMedia?.("(prefers-color-scheme: dark)");
const closeModalVisible = ref(false);
const rememberCloseChoice = ref(false);
let unlistenCloseRequested: UnlistenFn | null = null;

const navItems = computed(() => [
  { label: settingsStore.t("dashboard"), key: "dashboard", path: "/dashboard", icon: Activity },
  { label: settingsStore.t("mount"), key: "mount", path: "/mount", icon: HardDrive },
  { label: settingsStore.t("logs"), key: "logs", path: "/logs", icon: FileText },
  { label: settingsStore.t("settings"), key: "settings", path: "/settings", icon: Settings },
]);

const menuOptions = computed<MenuOption[]>(() =>
  navItems.value.map((item) => ({
    key: item.key,
    label: () =>
      h(
        RouterLink,
        { to: item.path, class: "nav-link" },
        {
          default: () => [
            h(item.icon, { size: 17, strokeWidth: 2 }),
            h("span", item.label),
          ],
        },
      ),
  })),
);

const activeKey = computed(() => route.name?.toString() ?? "dashboard");
const pageTitle = computed(() => settingsStore.t(activeKey.value));
const serviceStatusLabel = computed(() =>
  settingsStore.t(`status.${serviceStore.statusKind}`),
);

async function setLanguage(language: Language) {
  const previousLanguage = settingsStore.config.language;

  if (previousLanguage === language) {
    return;
  }

  try {
    settingsStore.config.language = language;
    await settingsStore.save();
  } catch (err) {
    settingsStore.config.language = previousLanguage;
    settingsStore.error = err instanceof Error ? err.message : String(err);
  }
}

async function handleCloseAction(action: Extract<CloseAction, "minimize" | "exit">) {
  if (rememberCloseChoice.value) {
    settingsStore.config.closeAction = action;
    try {
      await settingsStore.save();
    } catch {
      settingsStore.config.closeAction = "ask";
    }
  }

  closeModalVisible.value = false;
  rememberCloseChoice.value = false;

  if (action === "minimize") {
    await systemApi.hideMainWindow();
  } else {
    await systemApi.exitApp();
  }
}

onMounted(async () => {
  await settingsStore.load();
  await serviceStore.startStatusListener();
  await serviceStore.refresh();
  unlistenCloseRequested = await listen("app-close-requested", () => {
    rememberCloseChoice.value = false;
    closeModalVisible.value = true;
  });
  colorScheme?.addEventListener("change", settingsStore.applyDocumentTheme);
});

onUnmounted(() => {
  colorScheme?.removeEventListener("change", settingsStore.applyDocumentTheme);
  unlistenCloseRequested?.();
});

watch(
  () => settingsStore.config.theme,
  () => settingsStore.applyDocumentTheme(),
);
</script>

<template>
  <n-config-provider :theme="settingsStore.naiveTheme">
    <n-message-provider>
      <n-dialog-provider>
        <div class="app-shell">
          <aside class="app-sidebar">
            <div class="brand">
              <div class="brand-mark">
                <img :src="appIcon" alt="" />
              </div>
              <div>
                <strong>YuhangAlisting</strong>
                <span>Local drive console</span>
              </div>
            </div>

            <n-menu :options="menuOptions" :value="activeKey" class="nav-menu" />

            <div class="sidebar-status">
              <span class="status-dot" :class="serviceStore.statusKind"></span>
              <div>
                <strong>{{ serviceStatusLabel }}</strong>
                <span>{{ serviceStore.info.web_url }}</span>
              </div>
            </div>
          </aside>

          <main class="app-main">
            <header class="app-toolbar">
              <div>
                <p class="eyebrow">AList service manager</p>
                <h1>{{ pageTitle }}</h1>
              </div>
              <div class="toolbar-actions">
                <n-button-group size="small">
                  <n-button
                    :type="settingsStore.config.language === 'zh-CN' ? 'primary' : 'default'"
                    secondary
                    @click="setLanguage('zh-CN')"
                  >
                    中
                  </n-button>
                  <n-button
                    :type="settingsStore.config.language === 'en-US' ? 'primary' : 'default'"
                    secondary
                    @click="setLanguage('en-US')"
                  >
                    EN
                  </n-button>
                </n-button-group>
                <div class="toolbar-status">
                  <span class="status-dot" :class="serviceStore.statusKind"></span>
                  <span>{{ serviceStatusLabel }}</span>
                </div>
              </div>
            </header>

            <section class="app-content">
              <RouterView />
            </section>
          </main>
        </div>

        <n-modal v-model:show="closeModalVisible" :mask-closable="false">
          <section class="close-modal">
            <header class="close-modal-header">
              <h2>{{ settingsStore.t("app.close.title") }}</h2>
              <n-button quaternary circle @click="closeModalVisible = false">×</n-button>
            </header>
            <p class="close-copy">{{ settingsStore.t("app.close.content") }}</p>
            <n-checkbox v-model:checked="rememberCloseChoice">
              {{ settingsStore.t("app.close.remember") }}
            </n-checkbox>
            <footer class="close-actions">
              <n-button secondary @click="handleCloseAction('minimize')">
                {{ settingsStore.t("app.close.minimize") }}
              </n-button>
              <n-button type="primary" @click="handleCloseAction('exit')">
                {{ settingsStore.t("app.close.exit") }}
              </n-button>
            </footer>
          </section>
        </n-modal>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style scoped>
.close-modal {
  width: min(420px, calc(100vw - 32px));
  padding: 24px;
  border: 1px solid var(--line);
  border-radius: 16px;
  background: var(--panel);
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.34);
}

.close-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.close-modal-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 750;
}

.close-copy {
  margin: 0;
  color: var(--muted);
  line-height: 1.6;
}

.close-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 22px;
}
</style>

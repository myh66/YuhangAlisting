<script setup lang="ts">
import { computed, h, onMounted, onUnmounted, watch } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import type { MenuOption } from "naive-ui";
import { Activity, FileText, HardDrive, Settings } from "lucide-vue-next";
import appIcon from "./assets/app-icon.png";
import { useServiceStore } from "./stores/service";
import { useSettingsStore } from "./stores/settings";
import type { Language } from "./utils/tauri";

const route = useRoute();
const serviceStore = useServiceStore();
const settingsStore = useSettingsStore();
const colorScheme = window.matchMedia?.("(prefers-color-scheme: dark)");

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

onMounted(async () => {
  await settingsStore.load();
  await serviceStore.startStatusListener();
  await serviceStore.refresh();
  colorScheme?.addEventListener("change", settingsStore.applyDocumentTheme);
});

onUnmounted(() => {
  colorScheme?.removeEventListener("change", settingsStore.applyDocumentTheme);
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
                <strong>{{ serviceStore.serviceStatusLabel }}</strong>
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
                  <span>{{ serviceStore.serviceStatusLabel }}</span>
                </div>
              </div>
            </header>

            <section class="app-content">
              <RouterView />
            </section>
          </main>
        </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

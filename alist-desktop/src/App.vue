<script setup lang="ts">
import { computed, h, onMounted, watch } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import type { MenuOption } from "naive-ui";
import { Activity, Database, FileText, HardDrive, Settings } from "lucide-vue-next";
import { useServiceStore } from "./stores/service";
import { useSettingsStore } from "./stores/settings";

const route = useRoute();
const serviceStore = useServiceStore();
const settingsStore = useSettingsStore();

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

onMounted(async () => {
  await settingsStore.load();
  await serviceStore.startStatusListener();
  await serviceStore.refresh();
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
            <div class="traffic-lights" aria-hidden="true">
              <span class="traffic red"></span>
              <span class="traffic yellow"></span>
              <span class="traffic green"></span>
            </div>

            <div class="brand">
              <div class="brand-mark">
                <Database :size="18" />
              </div>
              <div>
                <strong>AList Desktop</strong>
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
              <div class="toolbar-status">
                <span class="status-dot" :class="serviceStore.statusKind"></span>
                <span>{{ serviceStore.serviceStatusLabel }}</span>
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

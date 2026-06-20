import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      redirect: "/dashboard",
    },
    {
      path: "/dashboard",
      name: "dashboard",
      component: () => import("../views/Dashboard.vue"),
      meta: { title: "仪表盘" },
    },
    {
      path: "/mount",
      name: "mount",
      component: () => import("../views/Mount.vue"),
      meta: { title: "挂载管理" },
    },
    {
      path: "/logs",
      name: "logs",
      component: () => import("../views/Logs.vue"),
      meta: { title: "日志查看" },
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("../views/Settings.vue"),
      meta: { title: "设置" },
    },
  ],
});

export default router;

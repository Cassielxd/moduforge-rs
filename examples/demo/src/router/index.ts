import { createRouter, createWebHistory } from "vue-router";

// 动态导入组件
const MainLayout = () => import("../components/MainLayout.vue");
const TrayMenuLayout = () => import("../components/TrayMenuLayout.vue");

const HomeView = () => import("../views/HomeView.vue");
const CxxmView = () => import("../views/CxxmView.vue");
const FbfxView = () => import("../views/FbfxView.vue");
const ProjectInfoView = () => import("../views/ProjectInfoView.vue");

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      redirect: "/home/dashboard",
    },
    {
      path: "/tray-menu",
      name: "TrayMenu",
      component: TrayMenuLayout,
    },
    {
      path: "/home",
      component: MainLayout,
      children: [
        {
          path: "dashboard",
          name: "Home",
          component: HomeView,
        },
        {
          path: "cxxm",
          name: "Cxxm",
          component: CxxmView,
        },
        {
          path: "fbfx",
          name: "Fbfx",
          component: FbfxView,
        },
        {
          path: "project-info",
          name: "ProjectInfo",
          component: ProjectInfoView,
        },
      ],
    },
  ],
});

export default router;

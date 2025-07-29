import "./assets/main.css";
import { createApp } from "vue";
import { createPinia } from "pinia";
import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
import * as ElementPlusIconsVue from "@element-plus/icons-vue";
import App from "./App.vue";
import router from "./router";
import { useUserStore } from "./stores/user";

const app = createApp(App);
const pinia = createPinia();

// 注册所有图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component);
}

app.use(pinia);
app.use(ElementPlus);
app.use(router);

// 初始化用户状态
const userStore = useUserStore();
userStore.initUserFromStorage();

console.log("应用初始化 - 用户登录状态:", userStore.isLoggedIn);
console.log("应用初始化 - 用户信息:", userStore.userInfo);

// 确保主窗口显示时用户状态正确
if (typeof window !== "undefined") {
  // 监听窗口焦点事件，确保状态同步
  window.addEventListener("focus", () => {
    console.log("窗口获得焦点，当前登录状态:", userStore.isLoggedIn);

    // 只有在用户状态丢失时才重新初始化
    if (!userStore.isLoggedIn) {
      const storedUser = localStorage.getItem("user-info");
      const storedToken = localStorage.getItem("auth-token");

      if (storedUser && storedToken) {
        console.log("检测到存储的用户信息，重新初始化状态");
        userStore.initUserFromStorage();
        console.log("重新初始化后 - 用户登录状态:", userStore.isLoggedIn);
      } else {
        console.log("没有存储的用户信息");
      }
    } else {
      console.log("用户状态正常，无需重新初始化");
    }
  });
}

// 监听存储变化，用于多窗口状态同步
window.addEventListener("storage", (e) => {
  console.log("存储变化事件:", e.key, e.newValue);

  if (e.key === "user-info" || e.key === "auth-token") {
    // 重新初始化用户状态
    userStore.initUserFromStorage();
    console.log("存储变化后 - 用户登录状态:", userStore.isLoggedIn);

    // 如果用户已登录且当前在登录页面，跳转到主页
    if (userStore.isLoggedIn && router.currentRoute.value.name === "Login") {
      console.log("用户已登录，从登录页跳转到主页");
      router.push("/home/dashboard");
    }
  }
});

// 路由变化时检查用户状态
router.afterEach((to, from) => {
  console.log("路由变化:", from.path, "->", to.path);
  console.log("当前用户登录状态:", userStore.isLoggedIn);
});

app.mount("#app");

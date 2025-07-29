import { defineStore } from "pinia";
import { ref, computed } from "vue";

export const useUserStore = defineStore("user", () => {
  // 状态
  const isLoggedIn = ref(false);
  const userInfo = ref(null);
  const token = ref(null);

  // 计算属性
  const userName = computed(
    () => userInfo.value?.nickname || userInfo.value?.username || "未知用户"
  );
  const userRole = computed(() => userInfo.value?.role || "guest");
  const hasPermission = computed(() => (permission) => {
    return userInfo.value?.permissions.includes(permission) || false;
  });

  // 方法
  const login = (user, authToken) => {
    isLoggedIn.value = true;
    userInfo.value = user;
    token.value = authToken;

    // 保存到localStorage
    localStorage.setItem("user-info", JSON.stringify(user));
    localStorage.setItem("auth-token", authToken);
  };

  const logout = () => {
    isLoggedIn.value = false;
    userInfo.value = null;
    token.value = null;

    // 清除localStorage
    localStorage.removeItem("user-info");
    localStorage.removeItem("auth-token");
  };

  const initUserFromStorage = () => {
    const storedUser = localStorage.getItem("user-info");
    const storedToken = localStorage.getItem("auth-token");

    if (storedUser && storedToken) {
      try {
        userInfo.value = JSON.parse(storedUser);
        token.value = storedToken;
        isLoggedIn.value = true;
      } catch (error) {
        console.error("解析用户信息失败:", error);
        logout();
      }
    }
  };

  const updateUserInfo = (updates) => {
    if (userInfo.value) {
      userInfo.value = { ...userInfo.value, ...updates };
      localStorage.setItem("user-info", JSON.stringify(userInfo.value));
    }
  };

  return {
    // 状态
    isLoggedIn,
    userInfo,
    token,

    // 计算属性
    userName,
    userRole,
    hasPermission,

    // 方法
    login,
    logout,
    initUserFromStorage,
    updateUserInfo,
  };
});

import { defineStore } from "pinia";
import { ref, computed } from "vue";

interface UserInfo {
  id: string;
  username: string;
  nickname: string;
  email?: string;
  avatar?: string;
  role: string;
  permissions: string[];
}

export const useUserStore = defineStore("user", () => {
  // 状态
  const isLoggedIn = ref(false);
  const userInfo = ref<UserInfo | null>(null);
  const token = ref<string | null>(null);

  // 计算属性
  const userName = computed(
    () => userInfo.value?.nickname || userInfo.value?.username || "未知用户"
  );
  const userRole = computed(() => userInfo.value?.role || "guest");
  const hasPermission = computed(() => (permission: string) => {
    return userInfo.value?.permissions.includes(permission) || false;
  });

  // 方法
  const login = (user: UserInfo, authToken: string) => {
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

  const updateUserInfo = (updates: Partial<UserInfo>) => {
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

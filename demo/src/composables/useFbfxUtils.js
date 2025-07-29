// 工具函数组合式函数

export function useFbfxUtils(tableTreeData) {
  // 统计文件夹数量
  const getFolderCount = () => {
    const countFolders = (items) => {
      let count = 0;
      items.forEach((item) => {
        if (item.type === "folder") {
          count++;
        }
        if (item.children && item.children.length > 0) {
          count += countFolders(item.children);
        }
      });
      return count;
    };
    return countFolders(tableTreeData.value);
  };

  // 统计文件数量
  const getFileCount = () => {
    const countFiles = (items) => {
      let count = 0;
      items.forEach((item) => {
        if (item.type === "file") {
          count++;
        }
        if (item.children && item.children.length > 0) {
          count += countFiles(item.children);
        }
      });
      return count;
    };
    return countFiles(tableTreeData.value);
  };

  // 获取当前时间
  const getCurrentTime = () => {
    const now = new Date();
    return now.toLocaleString("zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  return {
    getFolderCount,
    getFileCount,
    getCurrentTime,
  };
}

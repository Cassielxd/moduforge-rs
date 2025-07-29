import { invoke, InvokeArgs } from "@tauri-apps/api/core";


/**
 * 默认 IPC 请求选项
 */
const DEFAULT_OPTIONS = {
  timeout: 5000,
  retries: 1,
};

/**
 * 向 Rust 后端发送 IPC 请求
 * @param cmd 要调用的命令名称
 * @param args 传递给命令的参数
 * @param options 请求选项
 * @returns 返回响应结果的 Promise
 */
export async function ipcRequest(
  cmd,
  args,
  options = {}
){
  const mergedOptions = { ...DEFAULT_OPTIONS, ...options };
  let retries = mergedOptions.retries || 0;

  while (retries >= 0) {
    try {
      const response = await invoke<T>(cmd, args);
      return {
        success: true,
        data: response,
      };
    } catch (error) {
      if (retries === 0) {
        return {
          success: false,
          error: error instanceof Error ? error.message : String(error),
        };
      }
      retries--;
      // 重试前等待一段时间
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
  }

  return {
    success: false,
    error: "超过最大重试次数",
  };
}

/**
 * 使用示例：
 *
 * // 基础请求
 * const result = await ipcRequest<{ message: string }>('greet', { name: 'World' });
 *
 * // 使用自定义选项
 * const result = await ipcRequest<{ data: any[] }>(
 *   'fetch_data',
 *   { limit: 10 },
 *   { timeout: 10000, retries: 3 }
 * );
 */

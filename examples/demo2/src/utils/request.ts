import axios, { AxiosInstance, InternalAxiosRequestConfig, AxiosResponse } from 'axios';
import { ElMessage } from 'element-plus';

// 创建 axios 实例
const service: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api', // 从环境变量获取基础URL
  timeout: 15000, // 请求超时时间
  headers: {
    'Content-Type': 'application/json;charset=utf-8',
  },
});

// 请求拦截器
service.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    // 在发送请求之前做些什么
    // 例如：添加 token
    const token = localStorage.getItem('token');
    if (token && config.headers) {
      config.headers['Authorization'] = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    // 对请求错误做些什么
    console.error('Request error:', error);
    return Promise.reject(error);
  }
);

// 响应拦截器
service.interceptors.response.use(
  (response: AxiosResponse) => {
    const res = response.data;
    
    // 这里可以根据后端的响应结构定制
    // 假设后端返回格式为 { code: number, data: any, message: string }
    if (res.code !== 200) {
      ElMessage({
        message: res.message || 'Error',
        type: 'error',
        duration: 5 * 1000,
      });

      // 处理特定的错误码
      if (res.code === 401) {
        // 未授权，跳转到登录页
        // router.push('/login');
      }
      
      return Promise.reject(new Error(res.message || 'Error'));
    } else {
      return res.data;
    }
  },
  (error) => {
    console.error('Response error:', error);
    
    // 处理 HTTP 错误状态
    let message = '请求失败';
    if (error.response) {
      switch (error.response.status) {
        case 401:
          message = '未授权，请重新登录';
          // router.push('/login');
          break;
        case 403:
          message = '拒绝访问';
          break;
        case 404:
          message = '请求错误，未找到该资源';
          break;
        case 500:
          message = '服务器错误';
          break;
        default:
          message = `连接错误 ${error.response.status}`;
      }
    } else {
      if (error.message.includes('timeout')) {
        message = '请求超时';
      } else {
        message = '网络连接异常';
      }
    }

    ElMessage({
      message,
      type: 'error',
      duration: 5 * 1000,
    });

    return Promise.reject(error);
  }
);

// 封装 GET 请求
export function get<T>(url: string, params?: any): Promise<T> {
  return service.get(url, { params });
}

// 封装 POST 请求
export function post<T>(url: string, data?: any): Promise<T> {
  return service.post(url, data);
}

// 封装 PUT 请求
export function put<T>(url: string, data?: any): Promise<T> {
  return service.put(url, data);
}

// 封装 DELETE 请求
export function del<T>(url: string, params?: any): Promise<T> {
  return service.delete(url, { params });
}

export default service; 
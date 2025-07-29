import { ipcRequest } from "@/utils/ipc";
//新增人材机
export const addRcj = async (data) => {
  const result = await ipcRequest("add_rcj", data);
  return result;
};

//删除人材机
export const deleteRcj = async (id) => {
  const result = await ipcRequest("delete_rcj", { id });
  return result;
};
// 编辑人材机
export const editRcj = async (data) => {
  const result = await ipcRequest("edit_rcj", data);
  return result;
};

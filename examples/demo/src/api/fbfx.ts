import { ipcRequest } from "@/utils/ipc";
//新增分部分项行
export const addFbfxRow = async (data: any) => {
  const result = await ipcRequest("add_fbfx_row", data);
  return result;
};

//删除分部分项行
export const deleteFbfxRow = async (id: string) => {
  const result = await ipcRequest("delete_fbfx_row", { id });
  return result;
};
// 编辑分部分项行
export const editFbfxRow = async (data: any) => {
  const result = await ipcRequest("edit_fbfx_row", data);
  return result;
};

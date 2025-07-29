import { get, post } from "@/utils/request";

//ipc 根据工程项目id获取工程项目树
export const getGcxmTree = async (id) => {
  const result = await get("/gcxm/get_gcxm_tree/" + id);
  return result;
};

//ipc 新增 树节点
export const addGcxmTree = async (data) => {
  const result = await post("/gcxm/insert_child", data);
  return result;
};

//ipc 新增 根节点 工程项目
export const addRootTree = async (data) => {
  const result = await post("/gcxm", data);
  return result;
};

//ipc 添加脚注
export const addFootNote = async (data) => {
  const result = await post("/gcxm/add_footnote", data);
  return result;
};

//ipc 删除 树节点

export const deleteGcxmTree = async (data) => {
  const result = await post("/gcxm/delete_gcxm", data);
  return result;
};

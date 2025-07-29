import { post } from "@/utils/request";

export const getHistory = async (data) => {
  const result = await post("/gcxm/get_history", data);
  return result;
};

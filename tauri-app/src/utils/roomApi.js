import { defaultApiInstance as api } from "./api";

// 단일 방 조회: id로 조회, 서버 get_room은 리스트를 반환하므로 배열을 돌려받음
export async function getRoom(id) {
  try {
    const res = await api.get(`/room`, { params: { id } });
    return res.data;
  } catch (e) {
    return [];
  }
}

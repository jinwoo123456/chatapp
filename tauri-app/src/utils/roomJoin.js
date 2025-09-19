import { defaultApiInstance as api } from "./api";

// 상대 username과 내 username으로 방을 찾거나 생성하고 room id를 반환
export async function findOrCreateDmRoom(me, other) {
  const participants = [me, other].filter(Boolean);
  if (participants.length < 1) return null;
  const res = await api.post("/room/find", { participants });
  if (res && res.data && res.data.id) return res.data.id;
  // 일부 응답은 data가 없이 바로 모델을 반환할 수 있어 보정
  if (res && res.id) return res.id;
  return null;
}

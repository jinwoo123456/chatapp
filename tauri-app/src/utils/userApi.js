import axios from "axios";

// 존재하는 유저 검색: username 정확히 일치하는 첫 사용자 반환
export async function findUserByName(username) {
  try {
    const res = await axios.get("http://localhost:3100/api/user", { params: { username } });
    const list = Array.isArray(res.data) ? res.data : [];
    // 서버는 부분 일치 필터지만, 프론트에서 정확히 일치하는 항목만 선택
    return list.find(u => u.username === username) || null;
  } catch (e) {
    return null;
  }
}

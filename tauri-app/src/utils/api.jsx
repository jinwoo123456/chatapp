import axios from "axios";

// JWT 토큰 자동 첨부 axios 인스턴스
const api = axios.create({
    baseURL: "http://localhost:3100/api",
    validateStatus: () => true,
});

api.interceptors.request.use((config) => {
    const token = localStorage.getItem("token");
    if (token) {
        config.headers["Authorization"] = `Bearer ${token}`;
    }
    return config;
});

// 문자열을 재귀적으로 trim하여 전처리
function sanitize(value) {
    if (typeof value === "string") return value.trim();
    if (Array.isArray(value)) return value.map(sanitize);
    if (value && typeof value === "object") {
        const out = {};
        for (const k of Object.keys(value)) {
            out[k] = sanitize(value[k]);
        }
        return out;
    }
    return value;
}

export default async function postJson(path, data) {
    try {
        const response = await api.post(path, sanitize(data));
        return response.data;
    } catch (error) {
        console.error("에러 발생: ", error);
        return { success: 0, error: error?.message ?? "network error" };
    }
}

// 공용 인스턴스도 export (다른 API 유틸에서 사용)
export { api as defaultApiInstance };
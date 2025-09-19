import { defaultApiInstance as api } from "./api";

export async function getProfile(username) {
    try {
        const res = await api.get(`/profile`, { params: { username } });
        return res.data;
    } catch (e) {
        return { success: 0, error: e?.message || "network error" };
    }
}

export async function updateProfile(profile) {
    try {
        const res = await api.put(`/profile`, profile);
        return res.data;
    } catch (e) {
        return { success: 0, error: e?.message || "network error" };
    }
}

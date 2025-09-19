import axios from "axios";
import { defaultApiInstance as api } from "./api";

export async function getFriends(userId) {
    try {
        const res = await api.get(`/friend`, { params: { user_id: userId } });
        return res.data;
    } catch (e) {
        return { success: 0, error: e?.message || "network error" };
    }
}

export async function addFriend(friend) {
    try {
        const res = await api.post(`/friend`, friend);
        return res.data;
    } catch (e) {
        return { success: 0, error: e?.message || "network error" };
    }
}

export async function deleteFriend(id) {
    try {
        const res = await api.delete(`/friend`, { params: { id } });
        return res.data;
    } catch (e) {
        return { success: 0, error: e?.message || "network error" };
    }
}

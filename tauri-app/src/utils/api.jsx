import axios from "axios";

export default async function postJson(path, data) {
    try {
        const response = await axios.post(
            `http://localhost:3000/api${path}`,
            data,
            {
                validateStatus: () => true,
            }
        );
        return response.data;
    } catch (error) {
        console.error("에러 발생: ", error);
        return { success: 0, error: error?.message ?? "network error" };
    }
}
import axios from "axios";

export default async function postJson(path, data) {
    try {
        const response = await axios.post(`http://localhost:3000/api${path}`, data);
        return response.data;
    } catch (error) {
        console.error("에러 발생: ", error);
        alert("서버와의 통신 중 오류가 발생했습니다. 다시 시도해주세요.");
        throw error;
    }
}
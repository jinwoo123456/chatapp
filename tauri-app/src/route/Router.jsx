
import { Routes, Route } from "react-router-dom";
import Chat from "@/pages/Chat";
import Chats from "@/pages/Chats";
import Friends from "@/pages/Friends";
import Login from "@/pages/Login";
import Signup from "@/pages/Signup";
import MyPage from "@/pages/MyPage";

export default function AppRouter() {
	return (
		<Routes>
			<Route path="/chats" element={<Chats />} />
			<Route path="/Friends" element={<Friends />} />
			<Route path="/friends" element={<Friends />} />
			<Route path="/signup" element={<Signup />} />
			<Route path="/" element={<Login />} />
            <Route path="/login" element={<Login />} />
			<Route path="/chat/:friendId" element={<Chat />} />
			<Route path="/mypage" element={<MyPage />} />
		</Routes>
	);
}


import { Routes, Route } from "react-router-dom";
import Chat from "@/components/Chat";
import Friends from "@/pages/Friends";
import Login from "@/pages/Login";
import Signup from "@/pages/Signup";

export default function AppRouter() {
	return (
		<Routes>
			<Route path="/Friends" element={<Friends />} />
			<Route path="/signup" element={<Signup />} />
			<Route path="/" element={<Login />} />
            <Route path="/login" element={<Login />} />
			<Route path="/chat/:friendId" element={<Chat />} />
		</Routes>
	);
}

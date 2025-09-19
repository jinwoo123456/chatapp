import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import "@/styles/login.css";
import  postJson from "@/utils/api";
import { findUserByName } from "@/utils/userApi";
export default function Login() {
  const [id, setId] = useState("");
  const [pw, setPw] = useState("");
  const navigate = useNavigate();

  const handleLogin = async (e) => {
    e.preventDefault();
    // 프론트 검증: 아이디/비밀번호 필수, 비밀번호 4자 이상
    const trimmedId = id.trim();
    const p = pw.trim();
    if (!trimmedId) {
      alert("아이디를 입력하세요.");
      return;
    }
    if (p.length < 4) {
      alert("비밀번호는 4자 이상이어야 합니다.");
      return;
    }
  // 이전 로그인 정보 제거(계정 전환 시 누수 방지)
  localStorage.removeItem("token");
  localStorage.removeItem("username");
  localStorage.removeItem("user_id");
  localStorage.removeItem("friends");

  let response = await postJson("/login", { userid: trimmedId, password: p });
    console.log("response",response);
  if (response.success != 1 || response.error) {
    alert("로그인에 실패했습니다. 아이디와 비밀번호를 확인해주세요.");
    setId("");
    setPw("");
    return;
  }
  // JWT 토큰 저장 및 사용자 정보 캐시(완료 후 이동)
  if (response.token) {
    localStorage.setItem("token", response.token);
    localStorage.setItem("username", trimmedId);
    try {
      const me = await findUserByName(trimmedId);
      if (me && me.id) {
        localStorage.setItem("user_id", String(me.id));
      }
    } catch {}
  }
  navigate("/chats");

    
  };

  return (
    <div className="login-root">
      <div className="login-title">로그인</div>
      <form className="login-form" onSubmit={handleLogin}>
        <input className="login-input" type="text" placeholder="아이디" value={id} onChange={e=>setId(e.target.value)} required minLength={1} />
        <input className="login-input" type="password" placeholder="비밀번호" value={pw} onChange={e=>setPw(e.target.value)} required minLength={4} />
        <button className="login-btn" type="submit">로그인</button>
      </form>
      <div className="login-bottom">
        <span>계정이 없으신가요?</span>
        <button className="login-link" onClick={()=>navigate("/signup")}>회원가입</button>
      </div>
    </div>
  );
}

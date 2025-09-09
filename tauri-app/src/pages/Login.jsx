import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import "@/styles/login.css";

export default function Login() {
  const [id, setId] = useState("");
  const [pw, setPw] = useState("");
  const navigate = useNavigate();

  const handleLogin = (e) => {
    e.preventDefault();
    // 실제 인증 로직은 서버 연동 필요
    if (id && pw) {
      // 임시: 로그인 성공 시 친구목록으로 이동
      navigate("/");
    }
  };

  return (
    <div className="login-root">
      <div className="login-title">로그인</div>
      <form className="login-form" onSubmit={handleLogin}>
        <input className="login-input" type="text" placeholder="아이디" value={id} onChange={e=>setId(e.target.value)} />
        <input className="login-input" type="password" placeholder="비밀번호" value={pw} onChange={e=>setPw(e.target.value)} />
        <button className="login-btn" type="submit">로그인</button>
      </form>
      <div className="login-bottom">
        <span>계정이 없으신가요?</span>
        <button className="login-link" onClick={()=>navigate("/signup")}>회원가입</button>
      </div>
    </div>
  );
}

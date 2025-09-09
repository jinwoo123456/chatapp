import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import "@/styles/login.css";
import  postJson from "@/utils/api";
export default function Login() {
  const [id, setId] = useState("");
  const [pw, setPw] = useState("");
  const navigate = useNavigate();

  const handleLogin = async (e) => {
    e.preventDefault();
    
    let response = await postJson("/login", { userid: id, password: pw });
    console.log("response",response);
    if (response.success != 1 || response.error) {
        alert("로그인에 실패했습니다. 아이디와 비밀번호를 확인해주세요.");
        setId("");
        setPw("");
        return;
    }
    else {
        navigate("/friends");
    }

    
  };

  return (
    <div className="login-root">
      <div className="login-title">로그인</div>
      <form className="login-form" onSubmit={handleLogin}>
        <input className="login-input" type="text" placeholder="아이디" value={id} onChange={e=>setId(e.target.value)} required />
        <input className="login-input" type="password" placeholder="비밀번호" value={pw} onChange={e=>setPw(e.target.value)} required />
        <button className="login-btn" type="submit">로그인</button>
      </form>
      <div className="login-bottom">
        <span>계정이 없으신가요?</span>
        <button className="login-link" onClick={()=>navigate("/signup")}>회원가입</button>
      </div>
    </div>
  );
}

import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import "@/styles/login.css";
import  postJson from "@/utils/api";
export default function Signup() {
  const [id, setId] = useState("");
  const [pw, setPw] = useState("");
  const [pw2, setPw2] = useState("");
  const navigate = useNavigate();

  const handleSignup = async (e) => {
    e.preventDefault();
    // 프론트 검증: 아이디 3자 이상, 비밀번호 4자 이상, 비밀번호 확인 일치
    const trimmedId = id.trim();
    const p1 = pw.trim();
    const p2 = pw2.trim();
    if (trimmedId.length < 3) {
      alert("아이디는 3자 이상이어야 합니다.");
      return;
    }
    if (p1.length < 4) {
      alert("비밀번호는 4자 이상이어야 합니다.");
      return;
    }
    if (p1 !== p2) {
      alert("비밀번호가 일치하지 않습니다.");
      setPw2("");
      return;
    }
    let response = await postJson("/signup", { userid: trimmedId, password: p1 });
    console.log("response",response);
    if (response.success === 1) {
      alert("회원가입이 완료되었습니다. 로그인 페이지로 이동합니다.");
      navigate("/login");
    } else {
        alert("회원가입에 실패했습니다. 다시 시도해주세요.");
        setId("");
        setPw("");
        setPw2("");
        return;
    }
  };

  return (
    <div className="login-root">
      <div className="login-title">회원가입</div>
      <form className="login-form" onSubmit={handleSignup}>
  <input className="login-input" type="text" placeholder="아이디" value={id} onChange={e=>setId(e.target.value)} required minLength={3} />
  <input id="pw1" className="login-input" type="password" placeholder="비밀번호" value={pw} onChange={e=>setPw(e.target.value)} required minLength={4} />
  <input id="pw2" className="login-input" type="password" placeholder="비밀번호 확인" value={pw2} onChange={e=>setPw2(e.target.value)} required minLength={4} />
        <button className="login-btn" type="submit">회원가입</button>
      </form>
      <div className="login-bottom">
        <span>이미 계정이 있으신가요?</span>
        <button className="login-link" onClick={()=>navigate("/login")}>로그인</button>
      </div>
    </div>
  );
}

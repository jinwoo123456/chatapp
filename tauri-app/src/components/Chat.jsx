


import React, { useState, useRef, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";

const myAvatar = "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-6.webp";

function getFriends() {
  // Friends.jsx에서 localStorage에 저장된 친구목록을 불러오거나, 없으면 기본값 반환
  const saved = localStorage.getItem("friends");
  if (saved) return JSON.parse(saved);
  return [
    { id: 1, name: "홍길동", avatar: "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-1.webp", status: "상태메시지 예시" },
    { id: 2, name: "김철수", avatar: "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-2.webp", status: "오늘도 화이팅!" }
  ];
}

function getInitialMessages() {
  // 친구별 메시지 localStorage에서 불러오기
  const saved = localStorage.getItem("messages");
  if (saved) return JSON.parse(saved);
  return {
    1: [ { from: "other", text: "안녕!" }, { from: "me", text: "안녕! 반가워~" } ],
    2: [ { from: "other", text: "오늘 뭐해?" }, { from: "me", text: "공부중이야!" } ]
  };
}

export default function Chat() {
  const { friendId } = useParams();
  const navigate = useNavigate();
  const friends = getFriends();
  const friend = friends.find(f => String(f.id) === String(friendId));
  const [messages, setMessages] = useState(getInitialMessages());
  const [input, setInput] = useState("");
  const messagesEndRef = useRef(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, friendId]);

  const handleSend = () => {
    if (!input.trim() || !friend) return;
    setMessages(prev => {
      const updated = {
        ...prev,
        [friend.id]: [...(prev[friend.id] || []), { from: "me", text: input }]
      };
      localStorage.setItem("messages", JSON.stringify(updated));
      return updated;
    });
    setInput("");
  };

  if (!friend) {
    return (
      <div style={{maxWidth:480,margin:"0 auto",height:"100vh",display:"flex",flexDirection:"column",justifyContent:"center",alignItems:"center",background:"#fffbe7"}}>
        <div style={{fontSize:18,marginBottom:16}}>존재하지 않는 친구입니다.</div>
        <button onClick={()=>navigate("/")} style={{background:"#ffe100",border:"none",borderRadius:8,padding:"10px 24px",fontWeight:700,cursor:"pointer"}}>친구목록으로</button>
      </div>
    );
  }

  return (
    <div style={{
      maxWidth: 480,
      margin: "0 auto",
      height: "100vh",
      display: "flex",
      flexDirection: "column",
      background: "#fef01b"
    }}>
      {/* 상단 앱바 */}
      <div style={{
        background: "#fef01b",
        borderBottom: "1px solid #e5e5e5",
        padding: "16px 0 12px 0",
        textAlign: "center",
        fontWeight: 700,
        fontSize: 20,
        letterSpacing: 1,
        position: "sticky",
        top: 0,
        zIndex: 10,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between"
      }}>
        <button onClick={()=>navigate("/")} style={{background:"none",border:"none",fontSize:22,marginLeft:12,cursor:"pointer",color:"#3c1e1e"}}>&lt;</button>
        <span style={{color: "#3c1e1e", flex:1, textAlign:"center"}}>{friend.name}</span>
        <div style={{width:32}}></div>
      </div>
      {/* 채팅 메시지 영역 */}
      <div style={{
        flex: 1,
        overflowY: "auto",
        background: "#fffbe7",
        padding: "16px 8px 80px 8px"
      }}>
        {(messages[friend.id]||[]).map((msg, idx) => (
          <div key={idx} style={{
            display: "flex",
            flexDirection: msg.from === "me" ? "row-reverse" : "row",
            alignItems: "flex-end",
            marginBottom: 12
          }}>
            <img
              src={msg.from === "me" ? myAvatar : friend.avatar}
              alt="avatar"
              style={{width: 36, height: 36, borderRadius: "50%", margin: msg.from === "me" ? "0 0 0 8px" : "0 8px 0 0"}}
            />
            <div style={{
              background: msg.from === "me" ? "#ffe100" : "#fff",
              color: "#222",
              borderRadius: 18,
              padding: "10px 16px",
              fontSize: 16,
              maxWidth: "70%",
              boxShadow: "0 1px 2px rgba(0,0,0,0.04)",
              marginLeft: msg.from === "me" ? 0 : 4,
              marginRight: msg.from === "me" ? 4 : 0,
              border: msg.from === "me" ? "1px solid #ffe066" : "1px solid #eee"
            }}>
              {msg.text}
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>
      {/* 하단 입력창 */}
      <div style={{
        position: "fixed",
        left: 0,
        bottom: 0,
        width: "100%",
        maxWidth: 480,
        background: "#fffbe7",
        borderTop: "1px solid #e5e5e5",
        padding: "8px 8px 12px 8px",
        display: "flex",
        alignItems: "center"
      }}>
        <input
          type="text"
          placeholder={friend.name + "에게 메시지 보내기"}
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={e => { if (e.key === "Enter") handleSend(); }}
          style={{
            flex: 1,
            border: "none",
            borderRadius: 20,
            padding: "10px 16px",
            fontSize: 16,
            background: "#fff",
            marginRight: 8,
            outline: "none",
            boxShadow: "0 1px 2px rgba(0,0,0,0.04)"
          }}
        />
        <button
          onClick={handleSend}
          style={{
            background: "#ffe100",
            border: "none",
            borderRadius: "50%",
            width: 40,
            height: 40,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            boxShadow: "0 1px 2px rgba(0,0,0,0.04)",
            cursor: "pointer"
          }}
        >
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#3c1e1e" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="22" y1="2" x2="11" y2="13"></line><polygon points="22 2 15 22 11 13 2 9 22 2"></polygon></svg>
        </button>
      </div>
    </div>
  );
}
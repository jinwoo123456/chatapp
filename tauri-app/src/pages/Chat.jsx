


import React, { useState, useRef, useEffect } from "react";
import "@/styles/chat.css";
import { useParams, useNavigate } from "react-router-dom";
import postJson, { defaultApiInstance as api } from "@/utils/api";
import { subscribeChat } from "@/utils/chatSse";
import { getProfile } from "@/utils/profileApi";
import { getRoom } from "@/utils/roomApi";
import { findOrCreateDmRoom } from "@/utils/roomJoin";

const fallbackMyAvatar = "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-6.webp";

// room 정보를 기반으로 상대 사용자 표시 정보를 구성
async function buildFriendFromRoom(room, meName) {
  try {
    const participants = Array.isArray(room.participants) ? room.participants : room.participants ? JSON.parse(room.participants) : [];
    const otherName = participants.find((p) => p !== meName) || participants[0] || "";
    let avatar = "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-1.webp";
    let displayName = otherName;
    if (otherName) {
      const prof = await getProfile(otherName);
      if (prof && prof.data) {
        if (prof.data.avatar) avatar = prof.data.avatar;
        if (prof.data.display_name) displayName = prof.data.display_name;
      }
    }
    return { id: room.id, name: displayName || otherName || `Room ${room.id}`, avatar, status: "" };
  } catch {
    return { id: room.id, name: `Room ${room.id}`, avatar: "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-1.webp", status: "" };
  }
}

function Chat() {
  const { friendId } = useParams();
  const navigate = useNavigate();
  const [meName, setMeName] = useState(LOCAL_getUsername());
  const [myAvatar, setMyAvatar] = useState(fallbackMyAvatar);
  const [friend, setFriend] = useState(null);
  const [messages, setMessages] = useState([]);
  const [input, setInput] = useState("");
  const [roomId, setRoomId] = useState(null);
  const messagesEndRef = useRef(null);
  const eventSourceRef = useRef(null);

  function LOCAL_getUsername() {
    return localStorage.getItem("username") || "";
  }

  // 방 정보를 서버에서 불러와 상대 사용자 표시를 구성 (친구추가 여부와 무관)
  useEffect(() => {
    let cancelled = false;
    async function loadFriend() {
      const username = LOCAL_getUsername();
      setMeName(username);
      const uidStr = localStorage.getItem("user_id");
      let userId = uidStr ? parseInt(uidStr, 10) : null;
      // 서버에 userId 저장이 없다면 필요 시 username으로 조회해 userId 캐시 (옵션)
      if (!userId && username) {
        // 여기에 서버에서 username으로 id를 얻는 로직을 추가할 수 있음
      }
      // 내 프로필 아바타 로드(가능할 때)
      try {
        const prof = await getProfile(username);
        if (!cancelled && prof && prof.data && prof.data.avatar) {
          setMyAvatar(prof.data.avatar);
        }
      } catch {}
      // 방 정보 불러오기
      const roomRes = await getRoom(Number(friendId));
      if (cancelled) return;
      if (roomRes && Array.isArray(roomRes) && roomRes.length > 0) {
        const room = roomRes[0];
        const f = await buildFriendFromRoom(room, username);
        if (!cancelled) setFriend(f);
      } else {
        // 방을 찾지 못한 경우: URL 파라미터를 상대 username으로 간주해 DM 방을 찾아 이동
        if (username) {
          const dmId = await findOrCreateDmRoom(username, String(friendId));
          if (!cancelled && dmId) {
            navigate(`/chat/${dmId}`);
            return;
          }
        }
        setFriend(null);
      }
    }
    loadFriend();
    return () => { cancelled = true; };
  }, [friendId]);

  // 방 보장(존재하면 사용, 없으면 생성) 및 roomId 설정
  useEffect(() => {
    let cancelled = false;
    async function ensureRoom() {
      if (!friend || !meName) return;
      const participants = [meName, friend.name].sort();
      try {
        const res = await api.get("/room");
        if (cancelled) return;
        if (Array.isArray(res.data)) {
          const found = res.data.find(r => {
            const ps = (r.participants || []).slice().sort();
            return ps.length === participants.length && ps.every((v, i) => v === participants[i]);
          });
          if (found && found.id) {
            setRoomId(found.id);
            return;
          }
        }
      } catch {}
      // 없으면 생성
      try {
        const create = await api.post("/room", { participants });
        if (!cancelled && create && create.data && create.data.id) {
          setRoomId(create.data.id);
        }
      } catch {}
    }
    ensureRoom();
    return () => { cancelled = true; };
  }, [friend, meName]);

  // 채팅방 메시지 불러오기 (최초)
  useEffect(() => {
    let ignore = false;
    async function fetchHistory() {
      if (!roomId || !friend) return;
      try {
        const res = await api.get("/chat", { params: { room_id: roomId } });
        if (!ignore && res && Array.isArray(res.data)) {
          const msgs = res.data.map(msg => ({ ...msg, from: msg.sender === meName ? "me" : "other", text: msg.message }));
          setMessages(msgs);
          
          // 가장 최신 메시지의 ID로 읽음 상태 업데이트
          if (msgs.length > 0) {
            const lastMessageId = msgs[msgs.length - 1].id;
            try {
              await api.post(`/room/read/${roomId}` , {
                username: meName,
                last_read_id: lastMessageId
              });
            } catch (error) {
              console.error("Failed to mark as read:", error);
            }
          }
        }
      } catch {}
    }
    fetchHistory();
    return () => { ignore = true; };
  }, [roomId, friend?.name, meName]);

  // SSE 실시간 메시지 구독
  useEffect(() => {
    if (!roomId || !friend) return;
    if (eventSourceRef.current) eventSourceRef.current.close();
    eventSourceRef.current = subscribeChat(Number(roomId), async (msg) => {
      const newMessage = { ...msg, from: msg.sender === meName ? "me" : "other", text: msg.message };
      setMessages(prev => ([...prev, newMessage]));
      
      // 새 메시지 도착시 읽음 상태 업데이트 (본인이 보낸 메시지가 아닌 경우에도 읽음 처리)
      try {
        await api.post(`/room/read/${roomId}`, {
          username: meName,
          last_read_id: msg.id
        });
      } catch (error) {
        console.error("Failed to mark new message as read:", error);
      }
    });
    return () => { if (eventSourceRef.current) eventSourceRef.current.close(); };
  }, [roomId, friend?.name, meName]);

  // 새 메시지 도착 시 자동 스크롤
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const handleSend = async () => {
    const text = input.trim();
    if (!text || !friend || !roomId) return;
    const res = await postJson("/chat/send", {
      sender: meName,
      message: text,
      room_id: Number(roomId)
    });
    if (res.success !== 1) {
      alert(res.error || "메시지 전송 실패");
      return;
    }
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
    <div className="chat-root">
      {/* 상단 앱바 */}
      <div className="chat-appbar">
        <button onClick={()=>navigate("/chats")} className="chat-back-btn">&lt;</button>
        <span className="chat-title">{friend.name}</span>
        <div className="chat-title-gap"></div>
      </div>
      {/* 채팅 메시지 영역 */}
      <div className="chat-messages">
        {messages.map((msg, idx) => (
          <div key={idx} className={`chat-message-row ${msg.from === "me" ? "me" : "other"}`}>
            <img
              src={msg.from === "me" ? myAvatar : friend.avatar}
              alt="avatar"
              className={`chat-message-avatar ${msg.from === "me" ? "me" : "other"}`}
            />
            <div className={`chat-bubble ${msg.from === "me" ? "me" : "other"}`}>
              {msg.text}
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>
      {/* 하단 입력창 */}
      <div className="chat-input-bar">
        <input
          type="text"
          placeholder={friend.name + "에게 메시지 보내기"}
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={e => { if (e.key === "Enter") handleSend(); }}
          className="chat-input"
        />
        <button onClick={handleSend} className="chat-send-btn">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#3c1e1e" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="22" y1="2" x2="11" y2="13"></line><polygon points="22 2 15 22 11 13 2 9 22 2"></polygon></svg>
        </button>
      </div>
    </div>
  );
}

export default Chat;
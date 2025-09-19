import React, { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import "@/styles/chats.css";
import { defaultApiInstance as api } from "@/utils/api";
import { getProfile } from "@/utils/profileApi";

function Chats() {
  const [rooms, setRooms] = useState([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();
  const [myAvatar, setMyAvatar] = useState("https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-6.webp");
  const [profileMap, setProfileMap] = useState({}); // { username: { display_name, avatar, status } }
  
  // 현재 로그인한 사용자명
  const currentUser = localStorage.getItem("username") || "";

  useEffect(() => {
    if (!currentUser) {
      navigate("/login");
      return;
    }
    // 내 프로필 아바타 동기화
    (async () => {
      const res = await getProfile(currentUser);
      if (res && res.success === 1 && res.data) {
        if (res.data.avatar) setMyAvatar(res.data.avatar);
      }
    })();
    loadRooms();
  }, [currentUser]);

  const loadRooms = async () => {
    try {
      setLoading(true);
      // 읽지 않은 메시지 개수와 함께 방 목록 가져오기
      const response = await api.get('/room/list', {
        params: { username: currentUser }
      });
      
      if (response.data && Array.isArray(response.data)) {
        const data = response.data;
        setRooms(data);
        // 참여자 중 상대 프로필을 병렬로 로드
        const others = Array.from(new Set(data.map(r => {
          const parts = Array.isArray(r.participants) ? r.participants : [];
          const o = parts.filter(p => p !== currentUser);
          return o.length > 0 ? o[0] : null;
        }).filter(Boolean)));
        if (others.length > 0) {
          const results = await Promise.all(others.map(async (u) => {
            const pr = await getProfile(u);
            return [u, (pr && pr.success === 1 && pr.data) ? pr.data : null];
          }));
          const map = {};
          for (const [u, d] of results) {
            if (d) map[u] = d;
          }
          setProfileMap(map);
        }
      }
    } catch (error) {
      console.error("Error loading rooms:", error);
    } finally {
      setLoading(false);
    }
  };

  // 1:1 채팅에서 상대방 이름 추출
  const getOtherParticipants = (participants) => {
    const others = participants.filter(p => p !== currentUser);
    return others.length > 0 ? others[0] : "알 수 없는 사용자";
  };

  // 마지막 메시지 시간 포맷팅
  const formatTime = (timestamp) => {
    if (!timestamp) return "";
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffDays > 0) {
      return `${diffDays}일 전`;
    } else if (diffHours > 0) {
      return `${diffHours}시간 전`;
    } else {
      const diffMinutes = Math.floor(diffMs / (1000 * 60));
      return diffMinutes > 0 ? `${diffMinutes}분 전` : "방금";
    }
  };

  const handleRoomClick = (roomId) => {
    navigate(`/chat/${roomId}`);
  };

  const NavButtons = useMemo(() => (
    <div style={{ display: "flex", gap: 12, marginRight: 16 }}>
      <button
        onClick={() => navigate("/friends")}
        style={{ background: "none", border: "none", fontSize: 16, color: "#3c1e1e", cursor: "pointer" }}
      >친구</button>
      <button
        onClick={() => navigate("/chats")}
        style={{ background: "#ffe9ec", border: "1px solid #ffccd2", borderRadius: 8, padding: "2px 8px", fontSize: 14, color: "#c63a46", cursor: "pointer" }}
      >채팅</button>
    </div>
  ), [navigate]);

  if (loading) {
    return (
      <div className="chats-root">
        <div className="chats-appbar">
          <img
            src={myAvatar}
            alt="me"
            className="chats-myavatar"
            onClick={() => navigate("/mypage")}
          />
          <span className="chats-title">채팅</span>
          {NavButtons}
        </div>
        <div className="chats-loading">채팅방을 불러오는 중...</div>
      </div>
    );
  }

  return (
    <div className="chats-root">
      <div className="chats-appbar">
        <img
          src={myAvatar}
          alt="me"
          className="chats-myavatar"
          onClick={() => navigate("/mypage")}
        />
        <span className="chats-title">채팅</span>
        {NavButtons}
      </div>
      
      <div className="chats-list">
        {rooms.length === 0 ? (
          <div className="chats-empty">
            <div className="chats-empty-title">아직 채팅방이 없습니다</div>
            <div className="chats-empty-subtitle">친구에게 메시지를 보내거나<br/>누군가가 메시지를 보내면 채팅방이 생성됩니다</div>
          </div>
        ) : (
          rooms.map((room, index) => (
            <div 
              key={room.id} 
              className="chats-item"
              onClick={() => handleRoomClick(room.id)}
            >
              {(() => {
                const other = getOtherParticipants(room.participants);
                const p = profileMap[other];
                const avatar = p?.avatar || `https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-${(index%6)+1}.webp`;
                return (
                  <img src={avatar} alt="room avatar" className="chats-avatar" />
                );
              })()}
              <div style={{flex: 1}}>
                <div className="chats-header-row">
                  <div className="chats-name">
                    {(() => {
                      const other = getOtherParticipants(room.participants);
                      const p = profileMap[other];
                      return p?.display_name || other;
                    })()}
                  </div>
                  {room.unread_count > 0 && (
                    <div className="chats-unread-badge">
                      {room.unread_count > 99 ? "99+" : room.unread_count}
                    </div>
                  )}
                </div>
                <div className="chats-status chats-subrow">
                  {(() => {
                    const other = getOtherParticipants(room.participants);
                    const p = profileMap[other];
                    const status = p?.status || "";
                    return (
                      <span className="chats-ellipsis">
                        {status || (room.last_message || "메시지를 시작해보세요")}
                      </span>
                    );
                  })()}
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

export default Chats;
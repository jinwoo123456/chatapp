import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import "@/styles/friends.css";

const defaultFriends = [
  {
    id: 1,
    name: "홍길동",
    avatar: "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-1.webp",
    status: "상태메시지 예시"
  },
  {
    id: 2,
    name: "김철수",
    avatar: "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-2.webp",
    status: "오늘도 화이팅!"
  }
];

export default function Friends() {
  const [friends, setFriends] = useState(defaultFriends);
  const [showAdd, setShowAdd] = useState(false);
  const [newFriendName, setNewFriendName] = useState("");
  const [newFriendStatus, setNewFriendStatus] = useState("");
  const navigate = useNavigate();

  const handleAddFriend = () => {
    if (!newFriendName.trim()) return;
    const newId = friends.length ? Math.max(...friends.map(f=>f.id)) + 1 : 1;
    const newFriend = {
      id: newId,
      name: newFriendName,
      avatar: `https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-${(newId%6)+1}.webp`,
      status: newFriendStatus || "상태메시지 없음"
    };
    setFriends(prev => [...prev, newFriend]);
    setNewFriendName("");
    setNewFriendStatus("");
    setShowAdd(false);
  };
  
  return (
    <div className="friends-root">
      <div className="friends-appbar">
        <span style={{color: "#3c1e1e", flex:1, textAlign:"center"}}>친구</span>
        <div onClick={()=>setShowAdd(true)} style={{background:"none", border:"none", fontSize:26, color:"#3c1e1e", marginRight:16, cursor:"pointer"}} aria-label="친구추가">+</div>
      </div>
      <div className="friends-list">
        {friends.map(friend => (
          <div key={friend.id} className="friends-item" onClick={()=>navigate(`/chat/${friend.id}`)}>
            <img src={friend.avatar} alt={friend.name} className="friends-avatar" />
            <div>
              <div className="friends-name">{friend.name}</div>
              <div className="friends-status">{friend.status}</div>
            </div>
          </div>
        ))}
      </div>
      {showAdd && (
        <div className="friends-add-modal-bg">
          <div className="friends-add-modal">
            <div className="friends-add-title">친구 추가</div>
            <input type="text" placeholder="이름" value={newFriendName} onChange={e=>setNewFriendName(e.target.value)} className="friends-add-input" />
            <input type="text" placeholder="상태메시지 (선택)" value={newFriendStatus} onChange={e=>setNewFriendStatus(e.target.value)} className="friends-add-input" style={{marginBottom:12}} />
            <div className="friends-add-btns">
              <button onClick={handleAddFriend} className="friends-add-btn">추가</button>
              <button onClick={()=>setShowAdd(false)} className="friends-add-btn friends-cancel-btn">취소</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

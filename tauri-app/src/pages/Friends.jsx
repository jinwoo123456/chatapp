
import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import "@/styles/friends.css";
import { getFriends as apiGetFriends, addFriend as apiAddFriend, deleteFriend as apiDeleteFriend } from "@/utils/friendApi";
import { findUserByName } from "@/utils/userApi";
import { findOrCreateDmRoom } from "@/utils/roomJoin";


function Friends() {
  const [friends, setFriends] = useState([]);
  const [showAdd, setShowAdd] = useState(false);
  const [newFriendName, setNewFriendName] = useState("");
  const [newFriendStatus, setNewFriendStatus] = useState("");
  const navigate = useNavigate();
  const [myAvatar, setMyAvatar] = useState("https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-6.webp");

  // 로그인한 유저 ID (localStorage에서 불러오고, 없으면 username으로 조회)
  const [userId, setUserId] = useState(null);

  useEffect(() => {
    let cancelled = false;
    async function resolveUserId() {
      const uidStr = localStorage.getItem("user_id");
      if (uidStr) {
        const id = parseInt(uidStr, 10);
        if (!Number.isNaN(id)) {
          if (!cancelled) setUserId(id);
          return;
        }
      }
      const username = localStorage.getItem("username");
      if (username) {
        const me = await findUserByName(username);
        if (!cancelled && me && me.id) {
          localStorage.setItem("user_id", String(me.id));
          setUserId(me.id);
        }
      }
    }
    resolveUserId();
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    if (!userId) return;
    let cancelled = false;
    apiGetFriends(userId).then(res => {
      if (!cancelled && res.success === 1 && Array.isArray(res.data)) {
        setFriends(res.data);
      }
    });
    return () => { cancelled = true; };
  }, [userId]);

  const handleAddFriend = async () => {
    const name = newFriendName.trim();
    if (!name) return;
    // 존재하는 유저인지 먼저 조회
    const target = await findUserByName(name);
    if (!target) {
      alert("존재하지 않는 아이디입니다.");
      return;
    }
    const newFriend = {
      user_id: userId,
      friend_id: target.id,
      friend_name: target.username,
      friend_avatar: `https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-${(friends.length%6)+1}.webp`,
      friend_status: newFriendStatus || "상태메시지 없음"
    };
    const res = await apiAddFriend(newFriend);
    if (res.success === 1 && res.data) {
      setFriends(prev => [...prev, res.data]);
      setNewFriendName("");
      setNewFriendStatus("");
      setShowAdd(false);
    } else {
      alert(res.error || "친구 추가 실패");
    }
  };
  
  const handleDeleteFriend = async (id) => {
    if (!window.confirm("정말 삭제하시겠습니까?")) return;
    const res = await apiDeleteFriend(id);
    if (res.success === 1) {
      setFriends(prev => prev.filter(f => f.id !== id));
    } else {
      alert(res.error || "삭제 실패");
    }
  };

  return (
    <div className="friends-root">
      <div className="friends-appbar">
        <img
          src={myAvatar}
          alt="me"
          className="friends-myavatar"
          onClick={() => navigate("/mypage")}
        />
        <span className="friends-title">친구</span>
        <div className="friends-appbar-right">
          <button className="friends-to-chats-btn" onClick={() => navigate("/chats")}>채팅</button>
          <button className="friends-add-icon" onClick={()=>setShowAdd(true)} aria-label="친구추가">친구추가</button>
        </div>
      </div>
      <div className="friends-list">
        {friends.map(friend => (
          <div key={friend.id} className="friends-item">
            <div
              className="friends-item-main"
              onClick={async ()=>{
                const me = localStorage.getItem("username") || "";
                const roomId = await findOrCreateDmRoom(me, friend.friend_name);
                if (roomId) navigate(`/chat/${roomId}`);
              }}
            >
              <img src={friend.friend_avatar} alt={friend.friend_name} className="friends-avatar" />
              <div>
                <div className="friends-name">{friend.friend_name}</div>
                <div className="friends-status">{friend.friend_status}</div>
              </div>
            </div>
            <button className="friends-delete-btn" onClick={()=>handleDeleteFriend(friend.id)}>삭제</button>
          </div>
        ))}
      </div>
      {showAdd && (
        <div className="friends-add-modal-bg">
          <div className="friends-add-modal">
            <div className="friends-add-title">친구 추가</div>
            <input type="text" placeholder="이름" value={newFriendName} onChange={e=>setNewFriendName(e.target.value)} className="friends-add-input" />
            <input type="text" placeholder="상태메시지 (선택)" value={newFriendStatus} onChange={e=>setNewFriendStatus(e.target.value)} className="friends-add-input" />
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

export default Friends;

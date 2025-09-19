import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  MDBContainer,
  MDBRow,
  MDBCol,
  MDBCard,
  MDBCardBody,
  MDBBtn,
  MDBInput,
  MDBIcon,
  MDBTypography,
} from "mdb-react-ui-kit";
import { getProfile, updateProfile } from "@/utils/profileApi";

const defaultProfile = {
  username: "",
  display_name: "",
  status: "",
  avatar: "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-6.webp",
};

function MyPage() {
  const navigate = useNavigate();
  const [profile, setProfile] = useState(defaultProfile);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    const me = localStorage.getItem("username") || "";
    if (!me) return;
    getProfile(me).then(res => {
      if (res.success === 1 && res.data) setProfile(res.data);
      else setProfile(p => ({ ...p, username: me, display_name: me }));
    });
  }, []);

  const onChange = (key) => (e) => {
    setProfile((p) => ({ ...p, [key]: e.target.value }));
  };

  const handleSave = async () => {
    setSaving(true);
    const res = await updateProfile(profile);
    setTimeout(() => setSaving(false), 500);
    if (res.success !== 1) alert(res.error || "저장 실패");
  };

  // 이미지 URL을 직접 입력받도록 변경

  const handleLogout = () => {
    // 모든 로컬 캐시/세션 데이터를 초기화해서 계정 전환 시 잔존 데이터 제거
    try {
      localStorage.removeItem("token");
      localStorage.removeItem("username");
      localStorage.removeItem("user_id");
      localStorage.removeItem("friends");
      localStorage.removeItem("chat_drafts");
      localStorage.removeItem("rooms_cache");
      sessionStorage.clear();
    } catch {}
    navigate("/login");
  };

  const handleClearLocalData = () => {
    try {
      localStorage.clear();
      sessionStorage.clear();
      alert("이 기기의 앱 데이터가 초기화되었습니다. 다시 로그인해 주세요.");
    } catch {
      alert("초기화 중 오류가 발생했습니다.");
    }
    navigate("/login");
  };

  return (
    <div
      style={{
        maxWidth: 480,
        margin: "0 auto",
        height: "100vh",
        display: "flex",
        flexDirection: "column",
        background: "#fffbe7",
      }}
    >
      {/* 상단 앱바 */}
      <div
        style={{
          background: "#fef01b",
          borderBottom: "1px solid #e5e5e5",
          padding: "16px 0 12px 0",
          position: "sticky",
          top: 0,
          zIndex: 10,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <button
          onClick={() => navigate("/chats")}
          style={{
            background: "none",
            border: "none",
            fontSize: 22,
            marginLeft: 12,
            cursor: "pointer",
            color: "#3c1e1e",
          }}
          aria-label="뒤로"
        >
          &lt;
        </button>
        <span style={{ color: "#3c1e1e", fontWeight: 700, fontSize: 18 }}>
          마이페이지
        </span>
        <div style={{ width: 32 }} />
      </div>

      {/* 콘텐츠 */}
      <MDBContainer className="py-4" style={{ flex: 1, width: "100%" }}>
        <MDBRow className="justify-content-center">
          <MDBCol md="10" lg="10" xl="9">
            <MDBCard style={{ borderRadius: 16, overflow: "hidden" }}>
              <MDBCardBody className="p-4">
                <div style={{ display: "flex", gap: 16, alignItems: "center" }}>
                  <img
                    src={profile.avatar}
                    alt="avatar"
                    style={{ width: 72, height: 72, borderRadius: "50%" }}
                  />
                  <div style={{ flex: 1 }}>
                    <MDBTypography tag="h5" className="mb-1" style={{ color: "#3c1e1e" }}>
                      {profile.username}
                    </MDBTypography>
                    <div style={{ color: "#7b6f6f", fontSize: 14 }}>{profile.status}</div>
                  </div>
                  <MDBBtn color="warning" onClick={handleLogout}>
                    <MDBIcon fas icon="sign-out-alt" className="me-2" /> 로그아웃
                  </MDBBtn>
                </div>

                <hr className="my-4" />

                <MDBRow className="g-3">
                  <MDBCol md="6">
                    <MDBInput
                      label="닉네임"
                      value={profile.display_name}
                      onChange={onChange("display_name")}
                    />
                  </MDBCol>
                  <MDBCol md="6">
                    <MDBInput
                      label="상태메시지"
                      value={profile.status}
                      onChange={onChange("status")}
                    />
                  </MDBCol>
                  <MDBCol md="12">
                    <MDBInput
                      label="아바타 이미지 URL"
                      value={profile.avatar}
                      onChange={onChange("avatar")}
                    />
                  </MDBCol>
                </MDBRow>

                <div className="d-flex justify-content-end mt-4" style={{ gap: 8 }}>
                  <MDBBtn color="danger" outline onClick={handleClearLocalData}>
                    <MDBIcon fas icon="trash" className="me-2" /> 기기 데이터 초기화
                  </MDBBtn>
                  <MDBBtn color="warning" onClick={handleSave} disabled={saving}>
                    <MDBIcon fas icon={saving ? "spinner" : "save"} className="me-2" />
                    {saving ? "저장중..." : "저장"}
                  </MDBBtn>
                </div>
              </MDBCardBody>
            </MDBCard>
          </MDBCol>
        </MDBRow>
      </MDBContainer>
    </div>
  );
}

export default MyPage;
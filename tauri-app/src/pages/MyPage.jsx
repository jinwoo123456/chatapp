import React, { useEffect, useState, useRef } from "react";
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

const defaultProfile = {
  nickname: "나",
  status: "상태메시지를 입력해보세요",
  avatar: "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-6.webp",
};

function loadProfile() {
  try {
    const saved = localStorage.getItem("profile");
    return saved ? JSON.parse(saved) : defaultProfile;
  } catch {
    return defaultProfile;
  }
}

function saveProfile(profile) {
  localStorage.setItem("profile", JSON.stringify(profile));
}

export default function MyPage() {
  const navigate = useNavigate();
  const [profile, setProfile] = useState(defaultProfile);
  const [saving, setSaving] = useState(false);
  const fileInputRef = useRef(null);

  useEffect(() => {
    setProfile(loadProfile());
  }, []);

  const onChange = (key) => (e) => {
    setProfile((p) => ({ ...p, [key]: e.target.value }));
  };

  const handleSave = () => {
    setSaving(true);
    saveProfile(profile);
    setTimeout(() => setSaving(false), 500);
  };

  const handlePickImage = () => fileInputRef.current?.click();

  const handleFileChange = (e) => {
    const file = e.target.files?.[0];
    if (!file) return;
    if (!file.type.startsWith("image/")) {
      alert("이미지 파일을 선택해주세요.");
      return;
    }
    if (file.size > 2 * 1024 * 1024) {
      alert("이미지는 2MB 이하를 권장합니다.");
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result;
      setProfile((p) => ({ ...p, avatar: dataUrl }));
    };
    reader.readAsDataURL(file);
  };

  const handleLogout = () => {
    // 필요 시 토큰/스토어 초기화 추가 가능
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
          onClick={() => navigate("/friends")}
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
                      {profile.nickname}
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
                      value={profile.nickname}
                      onChange={onChange("nickname")}
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
                    {/* 이미지 등록: 버튼 기반 UI */}
                    <input
                      ref={fileInputRef}
                      type="file"
                      accept="image/*"
                      hidden
                      onChange={handleFileChange}
                    />
                    <div className="d-flex align-items-center gap-2">
                      <MDBBtn color="warning" outline onClick={handlePickImage}>
                        <MDBIcon fas icon="image" className="me-2" /> 이미지 선택
                      </MDBBtn>
                      <MDBBtn
                        color="light"
                        onClick={() => setProfile((p) => ({ ...p, avatar: defaultProfile.avatar }))}
                      >
                        기본으로
                      </MDBBtn>
                    </div>
                    <div style={{ fontSize: 12, color: "#7b6f6f", marginTop: 6 }}>
                      JPG/PNG 권장, 2MB 이하
                    </div>
                  </MDBCol>
                </MDBRow>

                <div className="d-flex justify-content-end mt-4">
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
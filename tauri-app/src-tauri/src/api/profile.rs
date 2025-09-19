use axum::{Json, extract::{State, Query}};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, ActiveValue, ColumnTrait, QueryFilter};
use crate::entities::users::{Entity as UsersEntity, ActiveModel, Column};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub id: Option<i32>,
    pub username: String,       // 로그인 계정명
    pub display_name: String,   // 노출용 닉네임 (없으면 username)
    pub avatar: String,         // 프로필 이미지 URL
    pub status: String,         // 상태 메시지
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: i32,
    pub error: Option<String>,
    pub data: Option<T>,
}

pub async fn get_profile(State(conn): State<DatabaseConnection>, Query(params): Query<HashMap<String, String>>) -> Json<ApiResponse<Profile>> {
    let username = match params.get("username") {
        Some(u) if !u.trim().is_empty() => u,
        _ => return Json(ApiResponse { success: 0, error: Some("username 필요".to_string()), data: None }),
    };
    let user = UsersEntity::find().filter(Column::Username.eq(username)).one(&conn).await;
    match user {
        Ok(Some(u)) => {
            let display_name = u.display_name.clone().unwrap_or_else(|| u.username.clone());
            let status = u.status.clone().unwrap_or_else(|| "".to_string());
            let avatar = u.avatar.clone().unwrap_or_else(|| "https://mdbcdn.b-cdn.net/img/Photos/Avatars/avatar-6.webp".to_string());
            Json(ApiResponse { success: 1, error: None, data: Some(Profile {
                id: Some(u.id),
                username: u.username,
                display_name,
                avatar,
                status,
            }) })
        },
        Ok(None) => Json(ApiResponse { success: 0, error: Some("존재하지 않는 유저".to_string()), data: None }),
        Err(e) => Json(ApiResponse { success: 0, error: Some(format!("DB 오류: {}", e)), data: None }),
    }
}

pub async fn update_profile(State(conn): State<DatabaseConnection>, Json(profile): Json<Profile>) -> Json<ApiResponse<Profile>> {
    // 입력값 검증
    if profile.username.trim().is_empty() {
        return Json(ApiResponse { success: 0, error: Some("username 필요".to_string()), data: None });
    }
    // username으로 조회 후 업데이트 (id는 옵션)
    let user = UsersEntity::find().filter(Column::Username.eq(profile.username.clone())).one(&conn).await;
    match user {
        Ok(Some(u)) => {
            let updated = ActiveModel {
                id: ActiveValue::Set(u.id),
                username: ActiveValue::Set(u.username.clone()),
                password: ActiveValue::Set(u.password),
                display_name: ActiveValue::Set(Some(profile.display_name.clone())),
                status: ActiveValue::Set(Some(profile.status.clone())),
                avatar: ActiveValue::Set(Some(profile.avatar.clone())),
            };
            match updated.update(&conn).await {
                Ok(_m) => Json(ApiResponse { success: 1, error: None, data: Some(profile) }),
                Err(e) => Json(ApiResponse { success: 0, error: Some(format!("업데이트 실패: {}", e)), data: None }),
            }
        },
        Ok(None) => Json(ApiResponse { success: 0, error: Some("존재하지 않는 유저".to_string()), data: None }),
        Err(e) => Json(ApiResponse { success: 0, error: Some(format!("DB 오류: {}", e)), data: None }),
    }
}

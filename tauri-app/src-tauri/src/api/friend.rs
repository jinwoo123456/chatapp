use axum::{Json, extract::{State, Query}};
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, ActiveValue, ColumnTrait, QueryFilter};
use crate::entities::users::{Entity as UsersEntity, Column as UsersColumn};
use crate::entities::friends::{Entity as FriendsEntity, ActiveModel, Model as FriendModel, Column};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Friend {
    pub id: Option<i32>,
    pub user_id: i32,
    pub friend_id: i32,
    pub friend_name: String,
    pub friend_avatar: String,
    pub friend_status: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: i32,
    pub error: Option<String>,
    pub data: Option<T>,
}

pub async fn get_friends(State(conn): State<DatabaseConnection>, Query(params): Query<HashMap<String, String>>) -> Json<ApiResponse<Vec<FriendModel>>> {
    let user_id = match params.get("user_id").and_then(|v| v.parse::<i32>().ok()) {
        Some(id) => id,
        None => return Json(ApiResponse { success: 0, error: Some("user_id 필요".to_string()), data: None }),
    };
    let friends = FriendsEntity::find().filter(Column::UserId.eq(user_id)).all(&conn).await;
    match friends {
        Ok(list) => Json(ApiResponse { success: 1, error: None, data: Some(list) }),
        Err(e) => Json(ApiResponse { success: 0, error: Some(format!("DB 오류: {}", e)), data: None }),
    }
}

pub async fn add_friend(State(conn): State<DatabaseConnection>, Json(friend): Json<Friend>) -> Json<ApiResponse<FriendModel>> {
    // 입력값 검증
    if friend.user_id == 0 || friend.friend_id == 0 {
        return Json(ApiResponse { success: 0, error: Some("user_id와 friend_id가 필요합니다.".to_string()), data: None });
    }
    if friend.friend_name.trim().is_empty() {
        return Json(ApiResponse { success: 0, error: Some("친구 이름을 입력하세요.".to_string()), data: None });
    }
    if friend.user_id == friend.friend_id {
        return Json(ApiResponse { success: 0, error: Some("자기 자신은 친구로 추가할 수 없습니다.".to_string()), data: None });
    }
    // 존재하는 유저인지 확인 (프론트가 검증하더라도 백엔드에서 핸들링)
    let user_exists = UsersEntity::find().filter(UsersColumn::Id.eq(friend.user_id)).one(&conn).await.ok().flatten().is_some();
    let target_exists = UsersEntity::find().filter(UsersColumn::Id.eq(friend.friend_id)).one(&conn).await.ok().flatten().is_some();
    if !user_exists || !target_exists {
        return Json(ApiResponse { success: 0, error: Some("존재하지 않는 사용자입니다.".to_string()), data: None });
    }
    // 중복 친구 방지
    if let Ok(existing) = FriendsEntity::find()
        .filter(Column::UserId.eq(friend.user_id))
        .filter(Column::FriendId.eq(friend.friend_id))
        .one(&conn)
        .await
    {
        if existing.is_some() {
            return Json(ApiResponse { success: 0, error: Some("이미 친구로 추가되어 있습니다.".to_string()), data: None });
        }
    }
        let new_friend = ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(friend.user_id),
            friend_id: ActiveValue::Set(friend.friend_id),
            friend_name: ActiveValue::Set(friend.friend_name.clone()),
            friend_avatar: ActiveValue::Set(friend.friend_avatar.clone()),
            friend_status: ActiveValue::Set(friend.friend_status.clone()),
        };
        match new_friend.insert(&conn).await {
            Ok(model) => Json(ApiResponse { success: 1, error: None, data: Some(model) }),
            Err(e) => Json(ApiResponse { success: 0, error: Some(format!("DB 오류: {}", e)), data: None }),
        }
}

pub async fn delete_friend(State(conn): State<DatabaseConnection>, Query(params): Query<HashMap<String, String>>) -> Json<ApiResponse<()>> {
    let id = match params.get("id").and_then(|v| v.parse::<i32>().ok()) {
            Some(id) if id > 0 => id,
            _ => return Json(ApiResponse { success: 0, error: Some("id 필요 (양수)".to_string()), data: None }),
    };
    let friend = FriendsEntity::find_by_id(id).one(&conn).await;
    match friend {
        Ok(Some(model)) => {
            let active: ActiveModel = model.into();
            let _ = active.delete(&conn).await;
            Json(ApiResponse { success: 1, error: None, data: None })
        },
        Ok(None) => Json(ApiResponse { success: 0, error: Some("존재하지 않는 친구입니다.".to_string()), data: None }),
        Err(e) => Json(ApiResponse { success: 0, error: Some(format!("DB 오류: {}", e)), data: None }),
    }
}

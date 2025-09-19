use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    Json,
};

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    ModelTrait, QueryFilter,
};

use crate::entities::users::{ActiveModel, Column, Entity as UsersEntity, Model};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use argon2::password_hash::SaltString;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

pub async fn get_user(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<Model>> {
    let mut condition = Condition::all();

    if let Some(id) = params.get("id") {
        condition = condition.add(Column::Id.eq(id.parse::<i32>().unwrap()));
    }

    if let Some(username) = params.get("username") {
        condition = condition.add(Column::Username.contains(username));
    }

    Json(
        UsersEntity::find()
            .filter(condition)
            .all(&conn)
            .await
            .unwrap(),
    )
}

#[derive(serde::Deserialize)]
pub struct UpsertModel {
    id: Option<i32>,
    username: Option<String>,
    password: Option<String>,
}


#[derive(Deserialize)]
pub struct SignupRequest {
    pub userid: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub success: i32,
    pub error: Option<String>,
}

pub async fn signup(
    State(conn): State<DatabaseConnection>,
    Json(req): Json<SignupRequest>,
) -> Json<ApiResponse> {
    if req.userid.trim().is_empty() || req.password.trim().is_empty() {
        return Json(ApiResponse { success: 0, error: Some("아이디와 비밀번호를 모두 입력하세요.".to_string()) });
    }
    if req.userid.len() < 3 {
        return Json(ApiResponse { success: 0, error: Some("아이디는 3자 이상이어야 합니다.".to_string()) });
    }
    if req.password.len() < 4 {
        return Json(ApiResponse { success: 0, error: Some("비밀번호는 4자 이상이어야 합니다.".to_string()) });
    }
    let exists = UsersEntity::find()
        .filter(Column::Username.eq(&req.userid))
        .one(&conn)
        .await
        .unwrap()
        .is_some();
    if exists {
        return Json(ApiResponse { success: 0, error: Some("이미 존재하는 아이디입니다.".to_string()) });
    }
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(req.password.as_bytes(), &salt).unwrap().to_string();
    let new_user = ActiveModel {
        id: ActiveValue::NotSet,
        username: ActiveValue::Set(req.userid),
        password: ActiveValue::Set(password_hash),
        display_name: ActiveValue::Set(None),
        status: ActiveValue::Set(None),
        avatar: ActiveValue::Set(None),
    };
    let _ = new_user.insert(&conn).await;
    Json(ApiResponse { success: 1, error: None })
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub userid: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: i32,
    pub token: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn login(
    State(conn): State<DatabaseConnection>,
    Json(req): Json<LoginRequest>,
) -> Json<LoginResponse> {
    if req.userid.trim().is_empty() || req.password.trim().is_empty() {
        return Json(LoginResponse { success: 0, token: None, error: Some("아이디와 비밀번호를 모두 입력하세요.".to_string()) });
    }
    let user = UsersEntity::find()
        .filter(Column::Username.eq(&req.userid))
        .one(&conn)
        .await
        .unwrap();
    if let Some(user) = user {
        let parsed_hash = PasswordHash::new(&user.password).unwrap();
        let argon2 = Argon2::default();
        if argon2.verify_password(req.password.as_bytes(), &parsed_hash).is_ok() {
            // JWT 발급
            let claims = Claims {
                sub: user.username.clone(),
                exp: (chrono::Utc::now().timestamp() + 60 * 60) as usize,
            };
            let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(b"secret"))
                .unwrap();
            return Json(LoginResponse { success: 1, token: Some(token), error: None });
        }
    }
    Json(LoginResponse { success: 0, token: None, error: Some("아이디 또는 비밀번호가 올바르지 않습니다.".to_string()) })
}

pub async fn put_user(State(conn): State<DatabaseConnection>, Json(user): Json<UpsertModel>) -> Json<Model> {
    let result = UsersEntity::find_by_id(user.id.unwrap())
        .one(&conn)
        .await
        .unwrap()
        .unwrap();

    let new_user = ActiveModel {
        id: ActiveValue::Set(result.id),
        username: ActiveValue::Set(user.username.unwrap_or(result.username)),
        password: ActiveValue::Set(user.password.unwrap_or(result.password)),
        display_name: ActiveValue::Set(result.display_name),
        status: ActiveValue::Set(result.status),
        avatar: ActiveValue::Set(result.avatar),
    };

    Json(new_user.update(&conn).await.unwrap())
}

pub async fn delete_user(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<&'static str> {
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    let mut condition = Condition::any();

    if let Some(id) = params.get("id") {
        condition = condition.add(Column::Id.eq(id.parse::<i32>().unwrap()));
    }

    if let Some(username) = params.get("username") {
        condition = condition.add(Column::Username.contains(username));
    }

    let user = UsersEntity::find()
        .filter(condition)
        .one(&conn)
        .await
        .unwrap()
        .unwrap();

    user.delete(&conn).await.unwrap();

    Json("Deleted")
}

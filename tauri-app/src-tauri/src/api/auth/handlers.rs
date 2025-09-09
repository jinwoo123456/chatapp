use axum::{extract::State, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::{Statement, DatabaseBackend, ConnectionTrait};
use crate::AppState;
use super::dto;
use serde_json::json;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core},
    Argon2,
};
use rand_core::OsRng; // 안전한 랜덤 생성기
pub async fn signup_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::SignupReq>,
) -> impl IntoResponse {
    let db = app_state.db.clone();

    let dto::SignupReq { userid, password } = payload;
    println!("[signup] request userid={}", userid);
    let uid_for_log = userid.clone();

    let sql = "
    INSERT INTO users 
    (login_id, password_hash)
    VALUES ($1, $2)".to_owned();
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        sql,
        vec![userid.into(), hash_password(&password).into()],
    );
    // 성공시 1 실패시 9 반환
    match db.execute(stmt).await {
        Ok(_) => {
            println!("[signup] success userid={}", uid_for_log);
            let body = json!({ "success": 1, "userid": uid_for_log });
            (StatusCode::CREATED, Json(body))
        }
        Err(e) => {
            eprintln!("[signup] DB error: {e}");
            let body = json!({ "success": 0, "error": e.to_string() });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(body))
        }
    }
}


/// 비밀번호 해쉬화
pub fn hash_password(plain: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Argon2id, 기본 파라미터
    let hash = argon2.hash_password(plain.as_bytes(), &salt).unwrap().to_string();
    hash
}

/// 로그인 비번 해쉬코드 검증
pub fn verify_password(hash_str: &str, input: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed = PasswordHash::new(hash_str)?;
    let ok = Argon2::default().verify_password(input.as_bytes(), &parsed).is_ok();
    Ok(ok)
}
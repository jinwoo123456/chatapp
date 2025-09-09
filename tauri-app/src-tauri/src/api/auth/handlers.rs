use axum::{extract::State, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TryGetable};

use crate::AppState;
use super::dto;
use super::jwt;
use argon2::{
    password_hash::{PasswordHash, SaltString, rand_core::OsRng},
    Argon2, PasswordHasher, PasswordVerifier,
};

pub async fn signup_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::SignupReq>,
) -> impl IntoResponse {
    let db = app_state.db.clone();

    let dto::SignupReq { userid, password } = payload;
    println!("[signup] request userid={}", userid);

    // Argon2 해시 생성
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(e) => {
            eprintln!("[signup] hash error: {e}");
            let body = json!({ "success": 0, "error": e.to_string() });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(body));
        }
    };

    let insert_sql = r#"
        INSERT INTO users 
        (login_id, password_hash)
        VALUES ($1, $2,)
    "#.to_owned();

    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        insert_sql,
        vec![userid.clone().into(), hashed.into()],
    );

    match db.execute(stmt).await {
        Ok(_) => {
            println!("[signup] success userid={}", userid);
            let body = json!({ "success": 1, "userid": userid });
            (StatusCode::CREATED, Json(body))
        }
        Err(e) => {
            eprintln!("[signup] DB error: {e}");
            let body = json!({ "success": 0, "error": e.to_string() });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(body))
        }
    }
}

pub async fn login_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<dto::LoginReq>,
) -> impl IntoResponse {
    let db = app_state.db.clone();

    let dto::LoginReq { userid, password } = payload;
    println!("[login] request userid={}", userid);


    let select_sql = r#"
        SELECT password_hash
        FROM users
        WHERE login_id = $1
        LIMIT 1
    "#.to_owned();

    let select_stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        select_sql,
        vec![userid.clone().into()],
    );

    match db.query_one(select_stmt).await {
        Ok(Some(row)) => {
            let stored_hash: String = match row.try_get("", "password_hash") {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[login] row parse error: {e}");
                    let body = json!({ "success": 9, "error": "invalid credentials" });
                    return (StatusCode::UNAUTHORIZED, Json(body));
                }
            };

            let parsed = match PasswordHash::new(&stored_hash) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[login] invalid stored hash: {e}");
                    let body = json!({ "success": 9, "error": "invalid credentials" });
                    return (StatusCode::UNAUTHORIZED, Json(body));
                }
            };

            let argon2 = Argon2::default();
            match argon2.verify_password(password.as_bytes(), &parsed) {
                Ok(_) => {
                    println!("[login] success userid={}", userid);
                    let token = match jwt::create_jwt(&userid) {
                        Ok(t) => {
                            let body = json!({ "success": 1, "userid": userid, "token": t });
                            (StatusCode::OK, Json(body))
                        },
                        Err(e) => {
                            eprintln!("[login] JWT creation error: {e}");
                            let body = json!({ "success": 9, "error": e.to_string() });
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(body));
                        }
                    };
                    //let body = json!({ "success": 1, "userid": userid, "token": token });
                    (StatusCode::OK, Json(body))
                }
                Err(_) => {
                    eprintln!("[login] verify failed userid={}", userid);
                    let body = json!({ "success": 9, "error": "invalid credentials" });
                    (StatusCode::UNAUTHORIZED, Json(body))
                }
            }
        }
        Ok(None) => {
            let body = json!({ "success": 9, "error": "invalid credentials" });
            (StatusCode::UNAUTHORIZED, Json(body))
        }
        Err(e) => {
            eprintln!("[login] DB error: {e}");
            let body = json!({ "success": 9, "error": e.to_string() });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(body))
        }
    }
}
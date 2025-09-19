// Removed inner attribute; windows_subsystem attribute stays in main.rs as required by Tauri

mod api;
mod db;
// mod migration; // temporarily disabled migrations
mod entities;

use axum::{Router, routing::{get, post, put, delete}, extract::{Path, State, Query}};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use crate::db::init::init_db;
// use migration::Migrator; // temporarily disabled migrations
use tokio::sync::broadcast;
use api::state::AppState;

fn build_axum(state: api::state::AppState) -> Router {
    // 단일 Router<AppState>로 구성하고, 핸들러 클로저에서 AppState를 분해하여 하위 함수에 전달

    let api_router = Router::new()
        // health check
        .route("/health", get(|State(app): State<AppState>| async move {
            let backend = app.conn.get_database_backend();
            let stmt = Statement::from_string(backend, "SELECT 1");
            let ok = app.conn.execute(stmt).await.is_ok();
            axum::Json(serde_json::json!({"ok": ok}))
        }))
        // auth
        .route("/signup", post(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::user::SignupRequest>| async move {
            api::user::signup(State(app.conn.clone()), axum::Json(payload)).await
        }))
        .route("/login", post(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::user::LoginRequest>| async move {
            api::user::login(State(app.conn.clone()), axum::Json(payload)).await
        }))
        // chat
        .route("/chat", get(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::chat::get_chat(State(app.conn.clone()), Query(params)).await
        }))
        .route("/chat/subscribe", get(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::chat::subscribe(State(app.queue.clone()), Query(params)).await
        }))
        .route("/chat/send", post(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::chat::NewMessage>| async move {
            api::chat::send(State(app.conn.clone()), State(app.queue.clone()), axum::Json(payload)).await
        }))
        // room
        .route("/room", get(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::chat_room::get_room(State(app.conn.clone()), Query(params)).await
        }))
        .route("/room", post(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::chat_room::NewRoom>| async move {
            api::chat_room::post_room(State(app.conn.clone()), axum::Json(payload)).await
        }))
        .route("/room/find", post(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::chat_room::NewRoom>| async move {
            api::chat_room::find_or_create_room(State(app.conn.clone()), axum::Json(payload)).await
        }))
        .route("/room", put(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::chat_room::NewRoom>| async move {
            api::chat_room::put_room(State(app.conn.clone()), axum::Json(payload)).await
        }))
        .route("/room", delete(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::chat_room::delete_room(State(app.conn.clone()), Query(params)).await
        }))
        .route("/room/list", get(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::chat_room::list_rooms_with_unread(Query(params), State(app.conn.clone())).await
        }))
        .route("/room/read/{room_id}", post(|State(app): State<AppState>, Path(room_id): Path<i32>, axum::Json(payload): axum::Json<api::chat_room::ReadUpdate>| async move {
            api::chat_room::mark_read(State(app.conn.clone()), Path(room_id), axum::Json(payload)).await
        }))
        // user
        .route("/user", get(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::user::get_user(State(app.conn.clone()), Query(params)).await
        }))
        .route("/user", put(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::user::UpsertModel>| async move {
            api::user::put_user(State(app.conn.clone()), axum::Json(payload)).await
        }))
        .route("/user", delete(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::user::delete_user(State(app.conn.clone()), Query(params)).await
        }))
        // friend
        .route("/friend", get(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::friend::get_friends(State(app.conn.clone()), Query(params)).await
        }))
        .route("/friend", post(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::friend::Friend>| async move {
            api::friend::add_friend(State(app.conn.clone()), axum::Json(payload)).await
        }))
        .route("/friend", delete(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::friend::delete_friend(State(app.conn.clone()), Query(params)).await
        }))
        // profile
        .route("/profile", get(|State(app): State<AppState>, Query(params): Query<std::collections::HashMap<String, String>>| async move {
            api::profile::get_profile(State(app.conn.clone()), Query(params)).await
        }))
        .route("/profile", put(|State(app): State<AppState>, axum::Json(payload): axum::Json<api::profile::Profile>| async move {
            api::profile::update_profile(State(app.conn.clone()), axum::Json(payload)).await
        }))
        .with_state(state.clone());

    Router::new()
        .nest("/api", api_router)
        .layer(CorsLayer::permissive())
        .fallback_service(
            ServeDir::new("static").not_found_service(ServeFile::new("static/index.html")),
        )
}

async fn run_async() {
    use std::env;
    // Load .env for both src-tauri and project root to support Tauri dev
    let _ = dotenvy::dotenv();
    if env::var("DATABASE_URL").is_err() {
        let _ = dotenvy::from_filename("../.env");
    }
    if env::var("DATABASE_URL").is_err() {
        let _ = dotenvy::from_filename("../../.env");
    }

    // Log which DATABASE_URL will be used (mask credentials), consider compile-time fallback
    let url_for_log = env::var("DATABASE_URL").ok()
        .or_else(|| option_env!("DATABASE_URL").map(|s| s.to_string()));
    if let Some(url) = url_for_log {
        let masked = if let (Some(proto), Some(at)) = (url.find("://"), url.find('@')) {
            let mut s = url.clone();
            let start = proto + 3;
            if at > start { let _ = s.replace_range(start..at, "***"); }
            s
        } else { url };
        eprintln!("DATABASE_URL loaded: {masked}");
    } else {
        eprintln!("DATABASE_URL is missing. Place it in .env (project root or src-tauri)");
    }
    let db: DatabaseConnection = init_db().await;
    // 마이그레이션 실행 (임시 비활성화)
    // {
    //     use sea_orm_migration::MigratorTrait;
    //     Migrator::up(&db, None).await.expect("DB migration failed");
    // }
    let queue = broadcast::channel(10).0;
    let state = AppState {
        conn: db,
        queue,
    };
    tauri::Builder::default()
        .setup(move |_app| {
            let state = state.clone();
            tauri::async_runtime::spawn(async move {
                let router = build_axum(state);
                let listener = tokio::net::TcpListener::bind("127.0.0.1:3100").await.expect("failed to bind 127.0.0.1:3100");
                if let Err(e) = axum::serve(listener, router).await {
                    eprintln!("axum serve error: {e}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::block_on(run_async());
}


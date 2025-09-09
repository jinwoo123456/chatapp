#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api; 
mod db;  

use tauri::Manager;
use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use crate::db::db_connect::db_connect;


#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
fn build_axum() -> Router<AppState> {
    Router::new()
        .merge(crate::api::auth::routes::router()) 
        .layer(CorsLayer::permissive())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    let db = db_connect().await;
    tauri::Builder::default()
        .manage(AppState{
            db: Arc::new(db),
        })
        .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let app_state = app.state::<AppState>().inner().clone();
            tauri::async_runtime::spawn(async move {
                let router = build_axum().with_state(app_state);
                let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
                    .await
                    .expect("failed to bind 127.0.0.1:3000");
                if let Err(e) = axum::serve(listener, router).await {
                    eprintln!("axum serve error: {e}");
                }
            });
            Ok(()) 
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


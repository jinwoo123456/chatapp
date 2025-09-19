use std::env;
use sea_orm::{Database, DatabaseConnection};

pub async fn init_db() -> DatabaseConnection {
    // Prefer runtime env, then compile-time embedded env (from build.rs), else panic
    let url = env::var("DATABASE_URL").ok()
        .or_else(|| option_env!("DATABASE_URL").map(|s| s.to_string()))
        .unwrap_or_default();

    if url.trim().is_empty() {
        panic!("DATABASE_URL not set: NotPresent");
    }

    match Database::connect(&url).await {
        Ok(db) => db,
        Err(e) => panic!("Error connecting to database: {}", e),
    }
}

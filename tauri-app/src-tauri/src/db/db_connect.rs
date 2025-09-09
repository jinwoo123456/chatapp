use dotenvy::dotenv;
use axum::{routing::get, Router};
use sea_orm::{Database, DatabaseConnection,ConnectOptions};
use std::env;
use std::time::Duration;

pub async fn db_connect() -> DatabaseConnection {
    dotenv().ok(); // env 로드

    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL 없음");
    // db 연결 옵션
    let mut options = ConnectOptions::new(url);

    options.max_connections(10) // 최대 연결 수: 동시에 10개의 연결 유지
        .min_connections(1) // 최소 연결 수: 항상 1개는 연결 유지
        .sqlx_logging(false) // SQL 실행 로그 출력 여부 (false → 출력 안 함)
        .connect_timeout(Duration::from_secs(5)) // 연결 시도 제한 시간: 5초
        .acquire_timeout(Duration::from_secs(5)); // 연결 풀에서 커넥션 얻는 제한 시간: 5초

    Database::connect(options).await.expect("DB 연결 실패")
}
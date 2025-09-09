use axum::{
    routing::{get, post},
    Router,
};
use crate::AppState;
use super::handlers;




pub fn router() -> Router<AppState> {
    Router::new()
    .route("/api/signup", post(handlers::signup_handler))
    .route("/api/login", post(handlers::login_handler))
}

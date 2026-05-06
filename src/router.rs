use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::handler::{chat_handler, generate_aispl_handler, health_check, login_handler, verify_handler};
use crate::handler::{execute_skill_handler, list_skills_handler};
use crate::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/login", post(login_handler))
        .route("/api/verify", get(verify_handler))
        .route("/api/chat", post(chat_handler))
        .route("/api/generate-aispl", post(generate_aispl_handler))
        .route("/api/health", get(health_check))
        .route("/api/skills", get(list_skills_handler))
        .route("/api/skills/execute", post(execute_skill_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback_service(ServeDir::new("static"))
}

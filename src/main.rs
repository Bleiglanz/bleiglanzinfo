mod auth;
mod db;
mod error;
mod handlers;
mod views;

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::Key;
use std::env;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub key: Key,
    pub cookie_secure: bool,
    pub max_body_bytes: u64,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let session_key_hex = env::var("SESSION_KEY").expect("SESSION_KEY must be set");
    let cookie_secure = env::var("COOKIE_SECURE").unwrap_or_else(|_| "false".to_string()) == "true";
    let max_body_bytes: u64 = env::var("MAX_BODY_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8192);

    let key_bytes = hex::decode(&session_key_hex).expect("SESSION_KEY must be valid hex");
    assert!(
        key_bytes.len() >= 64,
        "SESSION_KEY must be at least 64 bytes (128 hex chars)"
    );
    let key = Key::from(&key_bytes);

    let pool = db::init_pool(&database_url)
        .await
        .expect("Failed to init DB");

    let state = AppState {
        pool,
        key,
        cookie_secure,
        max_body_bytes,
    };

    let app = Router::new()
        .route("/", get(handlers::get_index).post(handlers::post_index))
        .route("/index.html", get(handlers::get_index))
        .route("/login", get(handlers::get_login))
        .route("/login", post(auth::handle_post_login))
        .route("/logout", post(auth::handle_post_logout))
        .route("/{slug}", get(handlers::get_thread))
        .route("/{slug}", post(handlers::post_thread))
        .route("/{slug}/delete", post(handlers::delete_message))
        .route("/{slug}/edit", post(handlers::edit_message))
        .layer(RequestBodyLimitLayer::new(max_body_bytes as usize))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind");
    tracing::info!("listening on {bind_addr}");
    axum::serve(listener, app).await.expect("Server error");
}

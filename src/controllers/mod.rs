use axum::debug_handler;

pub mod pdf;

#[debug_handler]
pub async fn healthcheck() -> &'static str {
    "OK"
}

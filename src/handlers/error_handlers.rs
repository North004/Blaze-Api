use crate::response::JsendResponse;
use axum::response::IntoResponse;
use axum::response::Json;

pub async fn fallback_handler() -> impl IntoResponse {
    let response = JsendResponse::error("incorrect path placeholder message".to_string());
    Json(response)
}

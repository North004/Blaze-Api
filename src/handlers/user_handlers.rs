use crate::{
    model::UserResponse,
    response::{AppError,JsendResponse},
    AppState,
};

use axum::{
    extract::State,
    response::IntoResponse,
Json,
};
use serde_json::json;
use std::sync::Arc;

pub async fn get_all_users(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let users: Vec<UserResponse> = sqlx::query_as!(UserResponse, "SELECT username,email,id,created_at,updated_at FROM users")
        .fetch_all(&data.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let response = JsendResponse::success(Some(json!({"users" : users})));
    Ok(Json(response))
}
use crate::{
    model::{ProfileResponse, UserResponse},
    response::{AppError, JsendResponse},
    AppState,
};

use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use std::sync::Arc;

pub async fn get_all_users(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let users: Vec<ProfileResponse> = sqlx::query_as!(
        ProfileResponse,
        "SELECT profiles.id AS profile_id, profiles.profile_image, username FROM users JOIN profiles on users.id = profiles.user_id"
    )
    .fetch_all(&data.db)
    .await
    .map_err(|_| AppError::InternalServerError)?;

    let response = JsendResponse::success(Some(json!({"users" : users})));
    Ok(Json(response))
}

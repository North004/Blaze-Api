use crate::{
    model::ProfileModel,
    response::{AppError, AppPath, JsendResponse},
    AppState,
};
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;


pub async fn get_profile(
    AppPath(username): AppPath<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Query to find the user by username
    let user_id: Option<uuid::Uuid> =
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_optional(&data.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

    // If user is not found, return 404 Not Found
    let user_id = match user_id {
        Some(id) => id,
        _ => {
            return Err(AppError::JsendError("User not found".to_string()));
        }
    };

    // Query to find the profile by user_id
    let profile: Option<ProfileModel> = sqlx::query_as!(
        ProfileModel,
        "SELECT id, user_id, profile_image, bio, created_at, updated_at FROM profiles WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| AppError::InternalServerError)?;

    // If profile is not found, return 404 Not Found
    let profile = match profile {
        Some(profile) => profile,
        None => {
            return Err(AppError::JsendFail(json!({"profile" : "profile not found"})));
        }
    };

    let response = JsendResponse::success(Some(json!({"profile" : profile})));
    Ok(Json(response))
}
use crate::{
    model::ProfileModel,
    response::{ApiError, GeneralResponse, Status},
    AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;


pub async fn get_profile(
    Path(username): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    // Query to find the user by username
    let user_id: Option<uuid::Uuid> =
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_optional(&data.db)
            .await
            .map_err(|_| ApiError::InternalServerError)?;

    // If user is not found, return 404 Not Found
    let user_id = match user_id {
        Some(id) => id,
        _ => {
            return Err(ApiError::FailMsg("User not found".to_string()));
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
    .map_err(|_| ApiError::InternalServerError)?;

    // If profile is not found, return 404 Not Found
    let profile = match profile {
        Some(profile) => profile,
        None => {
            return Err(ApiError::FailMsg("Profile not found".to_string()));
        }
    };

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: Some(json!(profile)),
    };
    Ok(Json(response))
}
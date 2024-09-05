use crate::{
    model::UserModel,
    response::{ApiError,GeneralResponse, Status},
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
) -> Result<impl IntoResponse, ApiError> {
    let users: Vec<UserModel> = sqlx::query_as!(UserModel, "SELECT * FROM users")
        .fetch_all(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: Some(json!(users)),
    };

    Ok(Json(response))
}
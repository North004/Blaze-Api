use crate::{
    model::{CommentResponse,UserModel},
    response::{ApiError, AppJson, GeneralResponse, Status},
    schema::
        CommentSchemaOptional,
    AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub async fn get_comments_handler(
    Path(postid): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let postid = Uuid::parse_str(&postid).map_err(|_| ApiError::Fail(json!({"post_id" : "not a valid UUID"})))?;
    let comments = sqlx::query_as!(
        CommentResponse,
        "SELECT
            comments.id,
            users.username,
            comments.user_id,
            comments.post_id,
            comments.content,
            comments.created_at,
            comments.updated_at,
            profiles.profile_image
        FROM comments
        JOIN users ON comments.user_id = users.id
        JOIN profiles ON comments.user_id = profiles.user_id
        WHERE comments.post_id = $1
        ORDER BY comments.created_at DESC",
        postid
    )
    .fetch_all(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: Some(json!({
            "comments" : Some(comments)
        })),
    };
    Ok(Json(response))
}

pub async fn create_comment_handler(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(postid): Path<String>,
    AppJson(comment): AppJson<CommentSchemaOptional>,
) -> Result<impl IntoResponse, ApiError> {
    let postid = Uuid::parse_str(&postid).map_err(|_| ApiError::Fail(json!({"post_id" : "not a valid UUID"})))?;
    comment.validate()?;
    let content = comment.content.unwrap();
    sqlx::query!(
        "INSERT INTO comments (content,user_id,post_id) VALUES ($1,$2,$3)",
        content,
        user.id,
        postid
    )
    .execute(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: None,
    };
    Ok(Json(response))
}
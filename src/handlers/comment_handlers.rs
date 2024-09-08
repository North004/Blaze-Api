use crate::{
    model::{CommentResponse,UserModel},
    response::{AppError, AppJson, AppPath, JsendResponse},
    schema::CommentSchema, 
    AppState,
};
use axum::{
    extract::State,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use validator::Validate;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_comments_handler(
    AppPath(postid): AppPath<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let postid = Uuid::parse_str(&postid).map_err(|_| AppError::JsendFail(json!({"post_id" : "not a valid UUID"})))?;
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
    .map_err(|_| AppError::InternalServerError)?;
    let response =  JsendResponse::success(Some(json!({
        "comments" : Some(comments)
    })));
    Ok(Json(response))
}

pub async fn create_comment_handler(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    AppPath(postid): AppPath<String>,
    AppJson(comment): AppJson<CommentSchema>,
) -> Result<impl IntoResponse, AppError> {
    comment.validate()?;
    let postid = Uuid::parse_str(&postid).map_err(|_| AppError::JsendFail(json!({"post_id" : "not a valid UUID"})))?;
    sqlx::query!(
        "INSERT INTO comments (content,user_id,post_id) VALUES ($1,$2,$3)",
        comment.content,
        user.id,
        postid
    )
    .execute(&data.db)
    .await
    .map_err(|_| AppError::InternalServerError)?;
    let response = JsendResponse::success(None);
    Ok(Json(response))
}
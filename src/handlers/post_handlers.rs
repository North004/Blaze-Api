use crate::{
    model::{PostResponse,UserModel},
    response::{ApiError, AppJson, GeneralResponse, Status},
    schema::{CreatePostSchema, LikePostSchema},
    AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use validator::Validate;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_post(
    State(data): State<Arc<AppState>>,
    Path(postid): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let postid = Uuid::parse_str(&postid).map_err(|_| ApiError::Fail(json!({"post_id" : "not a valid UUID"})))?;
    let post :PostResponse = sqlx::query_as!(
        PostResponse,
        "SELECT
            posts.id,
            users.username,
            posts.title,
            posts.content,
            posts.created_at,
            posts.updated_at,
            posts.user_id,
            profiles.profile_image,
            COALESCE(SUM(CASE WHEN reactions.reaction_type = TRUE THEN 1 ELSE 0 END), 0) AS likes,
            COALESCE(SUM(CASE WHEN reactions.reaction_type = FALSE THEN 1 ELSE 0 END), 0) AS dislikes
        FROM posts
        JOIN users ON posts.user_id = users.id
        JOIN profiles ON profiles.user_id = users.id
        LEFT JOIN reactions ON posts.id = reactions.post_id
        WHERE posts.id = $1
        GROUP BY posts.id, users.username, posts.title, posts.content, posts.created_at, posts.updated_at, users.id, profiles.profile_image",postid
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?
    .ok_or( ApiError::Fail(json!({"post" : "post doesnt exist"})))?;
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: Some(json!({
            "post": post
        })),
    };
    Ok(Json(response))
}

pub async fn delete_post(
    Extension(user): Extension<UserModel>,
    State(data): State<Arc<AppState>>,
    Path(postid): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let post_id = Uuid::parse_str(&postid).map_err(|_| ApiError::Fail(json!({"post_id" : "not a valid UUID"})))?;
    let post_uuid = sqlx::query_scalar!("SELECT user_id FROM posts WHERE id = $1", post_id)
        .fetch_one(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    if user.id != Some(post_uuid) {
        return Err(ApiError::FailMsg(
            "not authorized to delete post".to_string(),
        ));
    }
    sqlx::query!("DELETE FROM posts WHERE id = $1", post_id)
        .execute(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: None,
    };

    Ok(Json(response))
}

pub async fn react_to_post(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(postid): Path<String>,
    AppJson(is_like): AppJson<LikePostSchema>,
) -> Result<impl IntoResponse, ApiError> {
    is_like.validate()?;
    let post_id = Uuid::parse_str(&postid).map_err(|_| ApiError::Fail(json!({"post_id" : "not a valid UUID"})))?;

    let existing_reaction = sqlx::query!(
        "SELECT id FROM reactions WHERE post_id = $1 AND user_id = $2",
        post_id,
        user.id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    if let Some(reaction) = existing_reaction {
        // Update the existing reaction
        sqlx::query!(
            "UPDATE reactions SET reaction_type = $1 WHERE id = $2",
            is_like.is_like,
            reaction.id
        )
        .execute(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    } else {
        // Insert a new reaction
        sqlx::query!(
            "INSERT INTO reactions (post_id, user_id,  reaction_type) VALUES ($1, $2, $3)",
            post_id,
            user.id,
            is_like.is_like
        )
        .execute(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    }

    let counts = sqlx::query!(
        "SELECT 
            COALESCE(SUM(CASE WHEN reaction_type = TRUE THEN 1 ELSE 0 END), 0) AS likes,
            COALESCE(SUM(CASE WHEN reaction_type = FALSE THEN 1 ELSE 0 END), 0) AS dislikes
        FROM reactions
        WHERE post_id = $1",
        post_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: Some(json!({
                "post_id": post_id,
                "like_count" : counts.likes,
                "dislike_count" : counts.dislikes,
        })),
    };
    Ok(Json(response))
}

pub async fn get_all_posts(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let posts: Vec<PostResponse> = sqlx::query_as!(
        PostResponse,
        "SELECT
            posts.id,
            users.username,
            posts.title,
            posts.content,
            posts.created_at,
            posts.updated_at,
            posts.user_id,
            profiles.profile_image,
            COALESCE(SUM(CASE WHEN reactions.reaction_type = TRUE THEN 1 ELSE 0 END), 0) AS likes,
            COALESCE(SUM(CASE WHEN reactions.reaction_type = FALSE THEN 1 ELSE 0 END), 0) AS dislikes
        FROM posts
        JOIN users ON posts.user_id = users.id
        JOIN profiles ON profiles.user_id = users.id
        LEFT JOIN reactions ON posts.id = reactions.post_id
        GROUP BY posts.id, users.username, posts.title, posts.content, posts.created_at, posts.updated_at, users.id, profiles.profile_image
        ORDER BY posts.created_at DESC"
    )
    .fetch_all(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: Some(json!({
            "posts" : posts
        })),
    };
    Ok(Json(response))
}

pub async fn create_post(
    Extension(user): Extension<UserModel>,
    State(data): State<Arc<AppState>>,
    AppJson(post): AppJson<CreatePostSchema>,
) -> Result<impl IntoResponse, ApiError> {
    post.validate()?;
    sqlx::query!(
        "INSERT INTO posts (user_id,title,content) VALUES ($1,$2,$3)",
        user.id,
        post.title,
        post.content
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
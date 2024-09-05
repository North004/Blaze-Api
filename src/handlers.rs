use crate::{
    model::{CommentModel, CommentResponse, PostResponse, ProfileModel, UserModel},
    response::{ApiError, AppJson, GeneralResponse, Status},
    schema::{
        CommentSchema, CommentSchemaOptional, CreatePostSchema, CreatePostSchemaOptional,
        LikePostSchema, LikePostSchemaOptional, LoginUserSchema, LoginUserSchemaOptional,
        RegisterUserSchema, RegisterUserSchemaOptional,
    },
    AppState,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

pub async fn login_user_handler(
    session: Session,
    State(data): State<Arc<AppState>>,
    AppJson(body): AppJson<LoginUserSchemaOptional>,
) -> Result<impl IntoResponse, ApiError> {
    body.validate()?;
    let username = body.username.ok_or(ApiError::InternalServerError)?;
    let password = body.password.ok_or(ApiError::InternalServerError)?;
    let body = LoginUserSchema { username, password };

    let user: UserModel = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE username = $1",
        body.username
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?
    .ok_or_else(|| ApiError::Fail(json!({"username" : "user does not exist"})))?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err(ApiError::Fail(
            json!({"password" : "password is incorrect"}),
        ));
    }

    session
        .insert("user_id", user.id)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let response = GeneralResponse {
        status: Status::Success,
        data: Some(json!({
            "username" : body.username
        })),
    };
    Ok(Json(response))
}

pub async fn logout_handler(session: Session) -> Result<impl IntoResponse, ApiError> {
    session
        .delete()
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: None,
    };
    Ok(Json(response))
}

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    AppJson(body): AppJson<RegisterUserSchemaOptional>,
) -> Result<impl IntoResponse, ApiError> {
    body.validate()?;
    let username = body.username.ok_or(ApiError::InternalServerError)?;
    let email = body.email.ok_or(ApiError::InternalServerError)?;
    let password = body.password.ok_or(ApiError::InternalServerError)?;
    let body = RegisterUserSchema {
        username,
        email,
        password,
    };

    let user_exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE username = $1)",
        body.username.to_owned()
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?
    .unwrap_or(false);

    let email_exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)",
        body.email
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?
    .unwrap_or(false);

    let mut fails: HashMap<String, String> = HashMap::new();
    if user_exists {
        fails.insert(
            "username".to_string(),
            "username already exists".to_string(),
        );
    }
    if email_exists {
        fails.insert("email".to_string(), "email already exists".to_string());
    }
    if !fails.is_empty() {
        return Err(ApiError::Fail(json!(fails)));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|_| ApiError::InternalServerError)
        .map(|hash| hash.to_string())?;

    let tx = data
        .db
        .begin()
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let user_id: Uuid = sqlx::query_scalar!(
        "INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id",
        body.username.to_string(),
        body.email.to_string().to_ascii_lowercase(),
        hashed_password
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    sqlx::query!("INSERT INTO profiles (user_id, profile_image, bio) VALUES ($1, $2, $3) RETURNING id, user_id,profile_image,bio,created_at,updated_at",
        user_id,
        "default.jpg".to_string(),
        "".to_string(),
    )
    .fetch_one(&data.db)
    .await.map_err(|_| { ApiError::InternalServerError})?;

    tx.commit()
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: None,
    };
    Ok(Json(response))
}

pub async fn create_post(
    Extension(user): Extension<UserModel>,
    State(data): State<Arc<AppState>>,
    AppJson(post): AppJson<CreatePostSchemaOptional>,
) -> Result<impl IntoResponse, ApiError> {
    post.validate()?;
    let title = post.title.ok_or(ApiError::InternalServerError)?;
    let content = post.content.ok_or(ApiError::InternalServerError)?;
    let post = CreatePostSchema { title, content };

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

pub async fn react_to_post(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(post_id): Path<Uuid>,
    AppJson(is_like): AppJson<LikePostSchemaOptional>,
) -> Result<impl IntoResponse, ApiError> {
    is_like.validate()?;
    let is_like = is_like.like;

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
            is_like,
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
            is_like
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

pub async fn delete_post(
    Extension(user): Extension<UserModel>,
    State(data): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
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

pub async fn is_logged_in(Extension(user): Extension<UserModel>) -> Result<impl IntoResponse, ApiError> {
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        data: Some(json!({
            "is_logged_in": true,
            "username" : user.username,
        })),
    };
    Ok(Json(response))
}

pub async fn get_comments(
    Path(postid): Path<Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
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

pub async fn comment(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<UserModel>,
    Path(postid): Path<Uuid>,
    AppJson(comment): AppJson<CommentSchemaOptional>,
) -> Result<impl IntoResponse, ApiError> {
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

pub async fn get_post(
    State(data): State<Arc<AppState>>,
    Path(postid): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
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

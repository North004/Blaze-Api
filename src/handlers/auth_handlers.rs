use crate::{
    model::UserModel,
    response::{AppError, AppJson, JsendResponse},
    schema::{LoginUserSchema, RegisterUserSchema},
    AppState,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{extract::State, response::IntoResponse, Extension, Json};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

pub async fn login_handler(
    session: Session,
    State(data): State<Arc<AppState>>,
    AppJson(body): AppJson<LoginUserSchema>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;
    let user: UserModel = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE username = $1",
        body.username
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| AppError::InternalServerError)?
    .ok_or_else(|| AppError::JsendFail(json!({"username" : "user does not exist"})))?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err(AppError::JsendFail(
            json!({"password" : "password is incorrect"}),
        ));
    }

    session
        .insert("user_id", user.id)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let response = JsendResponse::success(Some(json!({
        "username" : body.username
    })));
    Ok(Json(response))
}

pub async fn logout_handler(session: Session) -> Result<impl IntoResponse, AppError> {
    session
        .delete()
        .await
        .map_err(|_| AppError::InternalServerError)?;
    let response: JsendResponse = JsendResponse::success(None);
    Ok(Json(response))
}

pub async fn register_handler(
    State(data): State<Arc<AppState>>,
    AppJson(body): AppJson<RegisterUserSchema>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()?;
    let user_exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE username = $1)",
        body.username.to_owned()
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| AppError::InternalServerError)?
    .unwrap_or(false);

    let email_exists: bool = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)",
        body.email
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| AppError::InternalServerError)?
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
        return Err(AppError::JsendFail(json!(fails)));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError)
        .map(|hash| hash.to_string())?;

    let tx = data
        .db
        .begin()
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let user_id: Uuid = sqlx::query_scalar!(
        "INSERT INTO users (username, email, password) VALUES ($1, $2, $3) RETURNING id",
        body.username.to_string(),
        body.email.to_string().to_ascii_lowercase(),
        hashed_password
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| AppError::InternalServerError)?;

    sqlx::query!("INSERT INTO profiles (user_id, profile_image, bio) VALUES ($1, $2, $3) RETURNING id, user_id,profile_image,bio,created_at,updated_at",
        user_id,
        "default.jpg".to_string(),
        "".to_string(),
    )
    .fetch_one(&data.db)
    .await.map_err(|_| { AppError::InternalServerError})?;

    tx.commit()
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let response: JsendResponse = JsendResponse::success(None);
    Ok(Json(response))
}

pub async fn status_handler(
    Extension(user): Extension<UserModel>,
) -> Result<impl IntoResponse, AppError> {
    let response: JsendResponse = JsendResponse::success(Some(json!({
        "is_logged_in": true,
        "username" : user.username,
    })));
    Ok(Json(response))
}

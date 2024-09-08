//file archived session based authentication implemented instead :)
use axum::{
    body::Body,
    extract::State,
    http::{header, Request},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::sync::Arc;

use crate::{
    errors::ApiError,
    model::{TokenClaims, User},
    AppState,
};

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let token = token.ok_or_else(|| ApiError::Unauthorized("Token is invalid".to_owned()))?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| ApiError::BadRequest("Invalid token".to_owned()))?
    .claims;

    let user_id = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::BadRequest("Invalid token".to_owned()))?;

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let user =
        user.ok_or_else(|| return ApiError::BadRequest("User no longer exists".to_owned()))?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

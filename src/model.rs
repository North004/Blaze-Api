use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct UserModel {
    pub id: Option<Uuid>,
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}


#[derive(Debug, Deserialize,FromRow, Serialize, Clone)]
pub struct ProfileModel {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub profile_image: String,
    pub bio: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct PostModel {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct CommentModel {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct ReactionModel {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub reaction_type: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}



//#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
//pub struct AlterdPost {
//  pub id: uuid::Uuid,
// #[serde(rename = "authorId")]
//pub user_id: uuid::Uuid,
// #[serde(rename = "author")]
//pub username: String,
//pub title: String,
//pub content: String,
//#[serde(rename = "likeCount")]
//pub like_count: Option<i64>,
//#[serde(rename = "dislikeCount")]
//pub dislike_count: Option<i64>,
//#[serde(rename = "createdAt")]
//pub created_at: DateTime<Utc>, // added to match SQL schema
//#[serde(rename = "updatedAt")]
//pub updated_at: DateTime<Utc>, // added to match SQL schema
//}

//#[derive(Debug, Serialize, Deserialize)]
//pub struct TokenClaims {
//  pub sub: String,
// pub iat: usize,
//pub exp: usize,
//}

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct Register {
    pub id: Uuid,
    pub password: String,
}

#[derive(Serialize)]
pub struct PostResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub profile_image: String,
    pub title: String,
    pub content: String,
    pub likes: Option<i64>,
    pub dislikes: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct CommentResponse {
    pub id: Option<Uuid>,
    pub username: String,
    pub profile_image: String,
    pub user_id: Uuid,
    pub post_id: Uuid,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserResponse {
    pub id: Option<Uuid>,
    pub username: String,
    pub email: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}
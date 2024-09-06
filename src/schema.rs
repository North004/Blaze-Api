use crate::validation::{validate_email_length,validate_password_length,validate_username_length,validate_title_length,validate_content_length};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserSchema {
    #[serde(default)]
    #[validate(custom(function = "validate_username_length"))]
    pub username: String,
    #[serde(default)]
    #[validate(custom(function = "validate_email_length"))]
    pub email: String,
    #[serde(default)]
    #[validate(custom(function = "validate_password_length"))]
    pub password: String,
}


#[derive(Debug, Deserialize,Validate)]
pub struct LoginUserSchema {
    #[serde(default)]
    #[validate(custom(function = "validate_username_length"))]
    pub username: String,
    #[serde(default)]
    #[validate(custom(function = "validate_password_length"))]
    pub password: String,
}

#[derive(Debug, Deserialize,Validate)]
pub struct CreatePostSchema {
    #[serde(default)]
    #[validate(custom(function = "validate_title_length"))]
    pub title: String,
    #[serde(default)]
    #[validate(custom(function = "validate_content_length"))]
    pub content: String,
}

#[derive(Debug, Deserialize,Validate)]
pub struct CommentSchema {
    #[serde(default)]
    #[validate(custom(function = "validate_content_length"))]
    pub content: String,
}

#[derive(Debug, Deserialize,Validate)]
pub struct LikePostSchema {
    #[serde(default)]
    pub is_like: bool,
}
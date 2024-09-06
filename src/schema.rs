use std::{borrow::Cow, collections::HashMap, hash::Hash};

use serde::Deserialize;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserSchema {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserSchemaOptional {
    #[validate(required(message="username field required"),custom(function = "validate_username_length"))]
    pub username: Option<String>,
    #[validate(required(message="email field required"),custom(function = "validate_email_length"),email(message="not a valid email"))]
    pub email: Option<String>,
    #[validate(required(message="password field required"),custom(function = "validate_password_length"))]
    pub password: Option<String>,
}

fn validate_username_length(username: &str) -> Result<(), ValidationError> {
    // Ensure that username is not empty before checking length
    let len = username.len();
    println!("data {}",len);
    match validate_length(len,1,20,"username too short","username too long","username cannot be empty") {
        Ok(_) => return Ok(()),
        Err(res) => return Err(res),
    };
}

fn validate_email_length(username: &str) -> Result<(), ValidationError> {
    // Ensure that username is not empty before checking length
    let len = username.len();
    match validate_length(len,1,20,"email too short","email too long","email cannot be empty") {
        Ok(_) => return Ok(()),
        Err(res) => return Err(res),
    };
}

fn validate_password_length(username: &str) -> Result<(), ValidationError> {
    // Ensure that username is not empty before checking length
    let len = username.len();
    match validate_length(len,1,20,"password too short","password too long","password cannot be empty") {
        Ok(_) => return Ok(()),
        Err(res) => return Err(res),
    };
}
fn validate_length(
    len: usize,
    min: usize,
    max: usize,
    min_err: &'static str,
    max_err: &'static str,
    empty_err: &'static str
) -> Result<(), ValidationError> {
        if len == 0 {
            let error = ValidationError {
                code: Cow::Borrowed(empty_err),
                message: Some(Cow::Borrowed(empty_err)),
                params: HashMap::new()
            };
            Err(error)
        } else if len < min {
            let error = ValidationError {
                code: Cow::Borrowed(min_err),
                message: Some(Cow::Borrowed(min_err)),
                params: HashMap::new()
            };
            Err(error)
        } else if len > max {
            let error = ValidationError {
                code: Cow::Borrowed(max_err),
                message: Some(Cow::Borrowed(max_err)),
                params: HashMap::new()
            };
            Err(error)
        } else {
            Ok(())
        }
    }



#[derive(Debug, Deserialize,Validate)]
pub struct LoginUserSchemaOptional {
    #[validate(required(message = "username is required"),length(min=1,message="username is required"))]
    pub username: Option<String>,
    #[validate(required(message = "password is required"),length(min=1,message="password is required"))]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize,Validate)]
pub struct CreatePostSchemaOptional {
    #[validate(required(message = "title is required"),length(min=1,message="title is required"))]
    pub title: Option<String>,
    #[validate(required(message = "title is required"),length(min=1,message="title is required"))]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostSchema {
    pub title: String,
    pub content: String,
}

//#[derive(Debug, Deserialize)]
//pub struct LikePostSchema {
//    pub like: bool,
//}

#[derive(Debug, Deserialize,Validate)]
pub struct LikePostSchemaOptional {
    #[validate(required(message= "like status is required"))]
    pub like: Option<bool>,
}


//#[derive(Debug, Deserialize)]
//pub struct CommentSchema {
//    pub content: String,
//}

#[derive(Debug, Deserialize,Validate)]
pub struct CommentSchemaOptional {
    #[validate(required(message= "content is required"),length(min=1,message="title is required"))]
    pub content: Option<String>,
}

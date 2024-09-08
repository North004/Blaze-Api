use std::{borrow::Cow, collections::HashMap};
use validator::ValidationError;

pub fn validate_username_length(username: &str) -> Result<(), ValidationError> {
    let len = username.len();
    validate_length(
        len,
        1,
        20,
        "username too short",
        "username too long",
        "username cannot be empty",
    )
}

pub fn validate_email_length(username: &str) -> Result<(), ValidationError> {
    let len = username.len();
    validate_length(
        len,
        1,
        20,
        "email too short",
        "email too long",
        "email cannot be empty",
    )
}

pub fn validate_password_length(username: &str) -> Result<(), ValidationError> {
    let len = username.len();
    validate_length(
        len,
        8,
        40,
        "password too short",
        "password too long",
        "password cannot be empty",
    )
}

pub fn validate_title_length(username: &str) -> Result<(), ValidationError> {
    let len = username.len();
    validate_length(
        len,
        1,
        20,
        "title too short",
        "title too long",
        "title cannot be empty",
    )
}

pub fn validate_content_length(username: &str) -> Result<(), ValidationError> {
    let len = username.len();
    validate_length(
        len,
        1,
        225,
        "content too short",
        "content too long",
        "content cannot be empty",
    )
}

fn validate_length(
    len: usize,
    min: usize,
    max: usize,
    min_err: &'static str,
    max_err: &'static str,
    empty_err: &'static str,
) -> Result<(), ValidationError> {
    if len == 0 {
        let error = ValidationError {
            code: Cow::Borrowed(empty_err),
            message: Some(Cow::Borrowed(empty_err)),
            params: HashMap::new(),
        };
        Err(error)
    } else if len < min {
        let error = ValidationError {
            code: Cow::Borrowed(min_err),
            message: Some(Cow::Borrowed(min_err)),
            params: HashMap::new(),
        };
        Err(error)
    } else if len > max {
        let error = ValidationError {
            code: Cow::Borrowed(max_err),
            message: Some(Cow::Borrowed(max_err)),
            params: HashMap::new(),
        };
        Err(error)
    } else {
        Ok(())
    }
}

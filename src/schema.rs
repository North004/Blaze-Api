use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserSchema {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserSchemaOptional {
    #[validate(required(message = "username is required"),length(min=1,message="username is required"))]
    pub username: Option<String>,
    #[validate(required(message = "email is required"),length(min=1,message="email is required"))]
    pub email: Option<String>,
    #[validate(required(message = "password is required"),length(min=1,message="password is required"))]
    pub password: Option<String>,
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

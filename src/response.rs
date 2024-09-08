use axum::async_trait;
use axum::body::Body;
use axum::extract::rejection::PathRejection;
use axum::extract::FromRequestParts;
use axum::extract::{rejection::JsonRejection, FromRequest};
use axum::http::request::Parts;
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Serialize, Deserialize, Debug)]
pub enum Status {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize, Deserialize)]
pub struct JsendResponse {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsendResponse {
    pub fn success(data: Option<Value>) -> Self {
        JsendResponse {
            status: Status::Success,
            message: None,
            data,
        }
    }
    pub fn error(message: String) -> Self {
        JsendResponse {
            status: Status::Error,
            message: Some(message),
            data: None,
        }
    }
    pub fn fail(data: Value) -> Self {
        JsendResponse {
            status: Status::Fail,
            message: None,
            data: Some(data),
        }
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("internal server error")]
    InternalServerError,
    #[error("invalid json error")]
    JsonRejection(JsonRejection),
    #[error("validation error")]
    ValidationError(#[from] ValidationErrors),
    #[error("invalid path")]
    PathRejection(PathRejection),
    #[error("jsend fail")]
    JsendFail(Value),
    #[error("jsend error")]
    JsendError(String),
}

//fn serialize_option_value<S>(option: &Option<Value>, serializer: S) -> Result<S::Ok, S::Error>
//where
//    S: Serializer,
//{
//    match option {
//        Some(value) => serializer.serialize_some(value),
//        None => serializer.serialize_some(&Value::Array(vec![])),
//    }
//}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let (status_code, response) = match self {
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                JsendResponse::error("internal server error".to_string()),
            ),
            AppError::JsonRejection(_) => (
                StatusCode::BAD_REQUEST,
                JsendResponse::error("invalid json data".to_string()),
            ),
            AppError::ValidationError(err) => (
                StatusCode::OK,
                JsendResponse::fail(valiation_error_to_hashmap(err)),
            ),
            AppError::PathRejection(_) => (
                StatusCode::OK,
                JsendResponse::error("invalid path data".to_string()),
            ),
            AppError::JsendError(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                JsendResponse::error(message),
            ),
            AppError::JsendFail(data) => {
                (StatusCode::INTERNAL_SERVER_ERROR, JsendResponse::fail(data))
            }
        };

        Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(Body::from(json!(response).to_string()))
            .unwrap()
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);
impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

pub struct AppPath<T>(pub T);
#[async_trait]
impl<S, T> FromRequestParts<S> for AppPath<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => Err(AppError::PathRejection(rejection)),
        }
    }
}

fn valiation_error_to_hashmap(err: ValidationErrors) -> Value {
    let mut error_map: HashMap<String, String> = HashMap::new();

    for (field, errors) in err.field_errors() {
        let messages: Vec<String> = errors
            .iter()
            .map(|e| {
                e.message
                    .clone()
                    .unwrap_or_else(|| Cow::Borrowed("invalid value"))
                    .into_owned()
            })
            .collect();

        if let Some(message) = messages.first() {
            error_map.insert(field.to_string(), message.clone());
        }
    }
    json!(error_map)
}

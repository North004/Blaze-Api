use axum::async_trait;
use axum::body::Body;
use axum::extract::{rejection::JsonRejection, FromRequest};
use axum::response::Response;
use axum::{
    extract::{path::ErrorKind, rejection::PathRejection, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use serde::Serializer;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use thiserror::Error;
use validator::ValidationErrors;

#[derive(Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralResponse {
    pub status: Status,
    #[serde(serialize_with = "serialize_option_value")]
    pub data: Option<Value>,
}

fn serialize_option_value<S>(option: &Option<Value>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match option {
        Some(value) => serializer.serialize_some(value),
        None => serializer.serialize_some(&Value::Array(vec![])),
    }
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("random")]
    Fail(Value),
    #[error("random")]
    FailMsg(String),
    #[error("random")]
    InternalServerError,
    #[error("missing field or invalid request format")]
    JsonRejection(JsonRejection),
    #[error("validation error")]
    ValidationError(#[from] ValidationErrors),
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        let (value, status) = match self {
            ApiError::Fail(err) => (
                json!({"status" : "fail",
                        "data" : err}),
                StatusCode::OK,
            ),
            ApiError::InternalServerError => (
                json!({"status" : "error",
                        "message": "internal server error"}),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            ApiError::JsonRejection(data) => (
                json!({"status" : "error",
                        "message" : data.body_text()}),
                StatusCode::BAD_REQUEST,
            ),
            ApiError::ValidationError(err) => {
                let mut error_map: HashMap<String, String> = HashMap::new();

                // Populate the error map with field names and their corresponding error messages
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
                (
                    json!({
                        "status" : "fail",
                        "data" : error_map
                    }),
                    StatusCode::OK,
                )
            }
            ApiError::FailMsg(err) => (
                json!({"status" : "fail", "data" : { "message" : err}, }),
                StatusCode::OK,
            ),
        };

        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(json!(value).to_string()))
            .unwrap()
    }
}

pub struct AppPath<T>(T);

#[async_trait]
impl<S, T> FromRequestParts<S> for AppPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<PathError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let mut status = StatusCode::BAD_REQUEST;

                        let kind = inner.into_kind();
                        let body = match &kind {
                            ErrorKind::WrongNumberOfParameters { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            ErrorKind::ParseErrorAtKey { key, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            ErrorKind::ParseErrorAtIndex { index, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(index.to_string()),
                            },

                            ErrorKind::ParseError { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            ErrorKind::InvalidUtf8InPathParam { key } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            ErrorKind::UnsupportedType { .. } => {
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                PathError {
                                    message: kind.to_string(),
                                    location: None,
                                }
                            }

                            ErrorKind::Message(msg) => PathError {
                                message: msg.clone(),
                                location: None,
                            },

                            _ => PathError {
                                message: format!("Unhandled deserialization error: {kind}"),
                                location: None,
                            },
                        };

                        (status, body)
                    }
                    PathRejection::MissingPathParams(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        PathError {
                            message: error.to_string(),
                            location: None,
                        },
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        PathError {
                            message: format!("Unhandled path rejection: {rejection}"),
                            location: None,
                        },
                    ),
                };

                Err((status, axum::Json(body)))
            }
        }
    }
}

#[derive(Serialize)]
pub struct PathError {
    message: String,
    location: Option<String>,
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

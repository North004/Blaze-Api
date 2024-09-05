use axum::body::Body;
use axum::extract::{rejection::JsonRejection, FromRequest};
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize, Serializer};
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

//impl From<ValidationErrors> for ApiError {
//    fn from(errors: ValidationErrors) -> Self {
//       ApiError::ValidationError(errors)
//  }
//}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        let (value,status)  = match self {
            ApiError::Fail(err) => 
                (json!({"status" : "fail",
                        "data" : err}),StatusCode::OK),
            ApiError::InternalServerError => 
                (json!({"status" : "error",
                        "message": "internal server error"}),StatusCode::INTERNAL_SERVER_ERROR),
            ApiError::JsonRejection(_) => 
                (json!({"status" : "error",
                        "message" : "invalid json data"}),StatusCode::BAD_REQUEST),
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
                (json!({
                    "status" : "fail",
                    "data" : error_map
                }),
                StatusCode::OK)
            }
            ApiError::FailMsg(err) => (json!({"status" : "fail", "data" : { "message" : err}, }),StatusCode::OK)
        };

        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(json!(value).to_string()))
            .unwrap()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

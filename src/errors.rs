use crate::response::{GeneralResponse, Status};
use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde_json::json;

pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    InternalServerError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        let (status, message) = match self {
            ApiError::NotFound(err) => (StatusCode::NOT_FOUND, err),
            ApiError::BadRequest(err) => (StatusCode::BAD_REQUEST, err),
            ApiError::Unauthorized(err) => (StatusCode::UNAUTHORIZED, err),
            ApiError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let response = GeneralResponse::<()> {
            status: Status::Success,
            message,
            data: None,
        };

        let json_response = json!(response);
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(json_response.to_string()))
            .unwrap()
    }
}

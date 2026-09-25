use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, msg) = match self {
            ApiError::BadRequest(m) => (actix_web::http::StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Upstream(m) => (actix_web::http::StatusCode::BAD_GATEWAY, m.clone()),
            ApiError::Other(e) => (
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
            ),
        };
        HttpResponse::build(status).json(ErrorBody { error: msg })
    }
}

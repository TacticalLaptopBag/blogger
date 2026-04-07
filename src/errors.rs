use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid username or password")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token has expired")]
    ExpiredToken,
    #[error("Token has been revoked")]
    BlacklistedToken,
    #[error("No authentication token provided")]
    MissingToken,
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        let body = ErrorBody {
            error: self.to_string(),
        };
        match self {
            Self::InvalidCredentials => HttpResponse::Unauthorized().json(body),
            Self::InvalidToken | Self::ExpiredToken | Self::BlacklistedToken => {
                HttpResponse::Unauthorized().json(body)
            }
            Self::MissingToken => HttpResponse::Unauthorized().json(body),
            Self::InternalError(_) => HttpResponse::InternalServerError().json(body),
        }
    }
}

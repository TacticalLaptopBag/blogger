use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    InvalidToken,
    ExpiredToken,
    BlacklistedToken,
    MissingToken,
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid username or password"),
            Self::InvalidToken => write!(f, "Invalid token"),
            Self::ExpiredToken => write!(f, "Token has expired"),
            Self::BlacklistedToken => write!(f, "Token has been revoked"),
            Self::MissingToken => write!(f, "No authentication token provided"),
            Self::InternalError(msg) => write!(f, "Internal error: {msg}"),
        }
    }
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

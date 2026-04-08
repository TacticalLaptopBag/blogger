use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

pub mod auth;
pub mod config;
pub mod db;

#[derive(Debug, Error)]
pub enum BloggerError {
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error(transparent)]
    AuthError(#[from] auth::AuthError),
    #[error(transparent)]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error(transparent)]
    DbError(#[from] db::DbError),
}

impl From<BloggerError> for std::io::Error {
    fn from(e: BloggerError) -> Self {
        std::io::Error::other(e)
    }
}

impl ResponseError for BloggerError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let body = ErrorBody {
            error: self.to_string(),
        };
        match self {
            Self::AuthError(_) => HttpResponse::Unauthorized().json(body),
            _ => HttpResponse::InternalServerError().json(body),
        }
    }
}

pub type BloggerResult<T> = Result<T, BloggerError>;

#[derive(Serialize)]
pub struct ErrorBody {
    error: String,
}

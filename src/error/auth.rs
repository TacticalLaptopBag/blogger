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
    #[error("You aren't allowed to do that!")]
    Unauthorized,
    #[error("Internal error: {0}")]
    InternalError(String),
}

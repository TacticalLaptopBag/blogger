use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error(transparent)]
    R2d2Error(#[from] r2d2::Error),
    #[error(transparent)]
    QueryError(#[from] diesel::result::Error),
    #[error("The desired object is missing")]
    NotFound,
}

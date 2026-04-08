use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Environment variable {0} must be set")]
    Unset(&'static str),
    #[error("Environment variable {0} must be a number")]
    NotANumber(&'static str),
    #[error("Environment variable {0} must be a boolean")]
    NotABoolean(&'static str),
}

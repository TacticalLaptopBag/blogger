use serde::{Deserialize, Serialize};

use crate::models::db::user::User;

pub mod db;

// ── Request / response shapes ────────────────────────────────────────────────

/// Form body for POST /login
#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct BlogPostForm {
    pub title: String,
    pub post_content: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordForm {
    pub old_password: String,
    pub password: String,
}

/// Returned by GET /login
#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

impl UserInfo {
    pub fn from_user(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
        }
    }
}

/// Returned on a successful login or refresh
#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
}

// ── JWT claims ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenKind {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject — user id
    pub sub: String,
    pub username: String,
    /// Expiry (Unix timestamp)
    pub exp: i64,
    /// Issued-at (Unix timestamp)
    pub iat: i64,
    /// Unique token id — used for blacklisting
    pub jti: String,
    pub kind: TokenKind,
}

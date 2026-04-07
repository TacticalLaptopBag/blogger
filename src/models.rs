use serde::{Deserialize, Serialize};

// ── Stored user ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    /// bcrypt hash of the password
    pub password_hash: String,
}

// ── Request / response shapes ────────────────────────────────────────────────

/// Form body for POST /login
#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// Returned by GET /login
#[derive(Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
}

/// Returned on a successful login or refresh
#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
    /// Access token also echoed in the body for non-browser clients.
    /// The HTTP-only cookie is the primary delivery mechanism for browsers.
    pub token_type: String,
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

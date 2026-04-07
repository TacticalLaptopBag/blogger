use crate::{
    errors::AuthError,
    models::{AuthResponse, Claims, LoginForm, TokenKind, UserInfo},
    store::AppState,
};
use actix_web::FromRequest;
use actix_web::{
    HttpRequest, HttpResponse,
    cookie::{Cookie, SameSite, time::Duration},
    web,
};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

// ── Cookie names ─────────────────────────────────────────────────────────────

const ACCESS_COOKIE: &str = "access_token";
const REFRESH_COOKIE: &str = "refresh_token";

// ── JWT helpers ───────────────────────────────────────────────────────────────

fn make_token(
    state: &AppState,
    user_id: &str,
    username: &str,
    kind: TokenKind,
) -> Result<String, AuthError> {
    let now = Utc::now().timestamp();
    let expiry_secs = match kind {
        TokenKind::Access => state.config.jwt_expiry_secs,
        TokenKind::Refresh => state.config.jwt_refresh_expiry_secs,
    };

    let claims = Claims {
        sub: user_id.to_owned(),
        username: username.to_owned(),
        exp: now + expiry_secs,
        iat: now,
        jti: Uuid::new_v4().to_string(),
        kind,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AuthError::InternalError(e.to_string()))
}

fn verify_token(state: &AppState, token: &str) -> Result<Claims, AuthError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        use jsonwebtoken::errors::ErrorKind;
        match e.kind() {
            ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
            _ => AuthError::InvalidToken,
        }
    })
}

/// Extract a cookie value from the request by name.
fn cookie_value<'a>(req: &'a HttpRequest, name: &str) -> Option<String> {
    req.cookie(name).map(|c| c.value().to_owned())
}

/// Build an HTTP-only, Secure, SameSite=Strict cookie.
fn auth_cookie<'c>(
    name: &'c str,
    value: String,
    max_age_secs: i64,
    use_secure: bool,
) -> Cookie<'c> {
    Cookie::build(name, value)
        .path("/")
        .http_only(true)
        .secure(use_secure) // set to false for local http testing
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(max_age_secs))
        .finish()
}

/// Build an expired cookie that clears the named cookie in the browser.
fn clear_cookie(name: &'static str, use_secure: bool) -> Cookie<'static> {
    Cookie::build(name, "")
        .path("/")
        .http_only(true)
        .secure(use_secure)
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(0))
        .finish()
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /login — validate credentials, issue access + refresh tokens as cookies.
pub async fn login_post(
    state: web::Data<AppState>,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse, AuthError> {
    println!("LOGIN_POST");
    // Look up user
    let user = state
        .users
        .get(&form.username)
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    let valid = bcrypt::verify(&form.password, &user.password_hash)
        .map_err(|e| AuthError::InternalError(e.to_string()))?;

    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    let access = make_token(&state, &user.id, &user.username, TokenKind::Access)?;
    let refresh = make_token(&state, &user.id, &user.username, TokenKind::Refresh)?;

    let access_cookie = auth_cookie(
        ACCESS_COOKIE,
        access,
        state.config.jwt_expiry_secs,
        state.config.use_secure_cookies,
    );
    let refresh_cookie = auth_cookie(
        REFRESH_COOKIE,
        refresh,
        state.config.jwt_refresh_expiry_secs,
        state.config.use_secure_cookies,
    );

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(AuthResponse {
            message: "Logged in successfully".into(),
            token_type: "Bearer".into(),
        }))
}

/// GET /login — return information about the currently authenticated user.
pub async fn login_get(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AuthError> {
    let token = cookie_value(&req, ACCESS_COOKIE).ok_or(AuthError::MissingToken)?;
    let claims = verify_token(&state, &token)?;

    if state.is_blacklisted(&claims.jti) {
        return Err(AuthError::BlacklistedToken);
    }
    if claims.kind != TokenKind::Access {
        return Err(AuthError::InvalidToken);
    }

    let user = state
        .users
        .iter()
        .find(|u| u.id == claims.sub)
        .ok_or(AuthError::InvalidToken)?;

    Ok(HttpResponse::Ok().json(UserInfo {
        id: user.id.clone(),
        username: user.username.clone(),
    }))
}

/// POST /refresh — use the refresh token cookie to issue a new access token.
pub async fn refresh_post(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AuthError> {
    let token = cookie_value(&req, REFRESH_COOKIE).ok_or(AuthError::MissingToken)?;
    let claims = verify_token(&state, &token)?;

    if state.is_blacklisted(&claims.jti) {
        return Err(AuthError::BlacklistedToken);
    }
    if claims.kind != TokenKind::Refresh {
        return Err(AuthError::InvalidToken);
    }

    // Blacklist the used refresh token (single-use rotation)
    state.blacklist_token(&claims.jti, claims.exp);

    let new_access = make_token(&state, &claims.sub, &claims.username, TokenKind::Access)?;
    let new_refresh = make_token(&state, &claims.sub, &claims.username, TokenKind::Refresh)?;

    let access_cookie = auth_cookie(
        ACCESS_COOKIE,
        new_access,
        state.config.jwt_expiry_secs,
        state.config.use_secure_cookies,
    );
    let refresh_cookie = auth_cookie(
        REFRESH_COOKIE,
        new_refresh,
        state.config.jwt_refresh_expiry_secs,
        state.config.use_secure_cookies,
    );

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(AuthResponse {
            message: "Token refreshed".into(),
            token_type: "Bearer".into(),
        }))
}

/// POST /logout — blacklist both tokens and clear their cookies.
pub async fn logout_post(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AuthError> {
    // Blacklist the access token if present and valid
    if let Some(token) = cookie_value(&req, ACCESS_COOKIE) {
        if let Ok(claims) = verify_token(&state, &token) {
            state.blacklist_token(&claims.jti, claims.exp);
        }
    }

    // Blacklist the refresh token if present and valid
    if let Some(token) = cookie_value(&req, REFRESH_COOKIE) {
        if let Ok(claims) = verify_token(&state, &token) {
            state.blacklist_token(&claims.jti, claims.exp);
        }
    }

    Ok(HttpResponse::Ok()
        .cookie(clear_cookie(ACCESS_COOKIE, state.config.use_secure_cookies))
        .cookie(clear_cookie(
            REFRESH_COOKIE,
            state.config.use_secure_cookies,
        ))
        .json(serde_json::json!({ "message": "Logged out successfully" })))
}

impl FromRequest for Claims {
    type Error = AuthError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let state = req
                .app_data::<web::Data<AppState>>()
                .ok_or(AuthError::InternalError("Missing state".into()))?;

            let token = cookie_value(&req, ACCESS_COOKIE).ok_or(AuthError::MissingToken)?;

            let claims = verify_token(&state, &token)?;

            if state.is_blacklisted(&claims.jti) {
                return Err(AuthError::BlacklistedToken);
            }
            if claims.kind != TokenKind::Access {
                return Err(AuthError::InvalidToken);
            }

            Ok(claims)
        })
    }
}

pub async fn protected_get(
    user: Claims, // 401s automatically if token is missing/invalid
) -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Hello, {}!", user.username)
    }))
}

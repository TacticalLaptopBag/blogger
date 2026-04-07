use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub jwt_secret: String,
    /// Access token lifetime in seconds (default 15 min)
    pub jwt_expiry_secs: i64,
    /// Refresh token lifetime in seconds (default 7 days)
    pub jwt_refresh_expiry_secs: i64,
    pub use_secure_cookies: bool,
    pub host: String,
    pub port: u16,

    pub database_url: String,
    pub init_user_name: Option<String>,
    pub init_user_pass: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiry_secs: env::var("JWT_EXPIRY_SECONDS")
                .unwrap_or("900".into())
                .parse()
                .expect("JWT_EXPIRY_SECONDS must be a number"),
            jwt_refresh_expiry_secs: env::var("JWT_REFRESH_EXPIRY_SECONDS")
                .unwrap_or("604800".into())
                .parse()
                .expect("JWT_REFRESH_EXPIRY_SECONDS must be a number"),
            use_secure_cookies: env::var("USE_SECURE_COOKIES")
                .unwrap_or("true".into())
                .to_lowercase()
                .parse()
                .expect("USE_SECURE_COOKIES must be a boolean"),
            host: env::var("HOST").unwrap_or("127.0.0.1".into()),
            port: env::var("PORT")
                .unwrap_or("8080".into())
                .parse()
                .expect("PORT must be a number"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            init_user_name: env::var("INIT_USER_NAME").ok(),
            init_user_pass: env::var("INIT_USER_PASS").ok(),
        }
    }
}

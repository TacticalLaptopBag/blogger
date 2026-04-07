use crate::{config::Config, models::User};
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;

/// Shared application state injected via `web::Data`.
pub struct AppState {
    pub config: Config,
    /// In-memory user "database" — keyed by username.
    /// Replace with a real DB in production.
    pub users: Arc<DashMap<String, User>>,
    /// Blacklisted token JTIs mapped to their expiry timestamp.
    /// Entries are pruned lazily when they have already expired.
    pub blacklist: Arc<DashMap<String, i64>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let users: Arc<DashMap<String, User>> = Arc::new(DashMap::new());

        // Seed a demo user — password is "secret"
        let hash =
            bcrypt::hash("secret", bcrypt::DEFAULT_COST).expect("Failed to hash seed password");
        users.insert(
            "alice".into(),
            User {
                id: "usr_01".into(),
                username: "alice".into(),
                password_hash: hash,
            },
        );

        Self {
            config,
            users,
            blacklist: Arc::new(DashMap::new()),
        }
    }

    /// Add a JTI to the blacklist.
    pub fn blacklist_token(&self, jti: &str, exp: i64) {
        self.blacklist.insert(jti.to_owned(), exp);
        self.prune_blacklist();
    }

    /// Returns `true` if the JTI is currently blacklisted.
    pub fn is_blacklisted(&self, jti: &str) -> bool {
        self.blacklist.contains_key(jti)
    }

    /// Remove expired entries from the blacklist.
    fn prune_blacklist(&self) {
        let now = Utc::now().timestamp();
        self.blacklist.retain(|_, exp| *exp > now);
    }
}

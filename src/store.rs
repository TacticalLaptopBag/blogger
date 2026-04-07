use crate::schema::token_blacklist::dsl::*;
use crate::{config::Config, models::db::BlacklistEntry};
use chrono::{NaiveDateTime, Utc};
use diesel::r2d2::ConnectionManager;
use diesel::{ExpressionMethods, RunQueryDsl, SqliteConnection, query_dsl::methods::FilterDsl};
use r2d2::{Pool, PooledConnection};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Shared application state injected via `web::Data`.
pub struct AppState {
    pub config: Config,
    pool: DbPool,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(&config.database_url);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        let mut state = Self { config, pool };
        state.prune_blacklist();
        state
    }

    fn get_conn(&self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool.get().expect("Failed to get DB connection")
    }

    /// Add a JTI to the blacklist.
    pub fn blacklist_token(&mut self, other_jti: &str, exp: NaiveDateTime) {
        let entry = BlacklistEntry {
            jti: other_jti.into(),
            expires_at: exp,
        };

        diesel::insert_into(token_blacklist)
            .values(&entry)
            .execute(&mut self.get_conn());
        self.prune_blacklist();
    }

    /// Returns `true` if the JTI is currently blacklisted.
    pub fn is_blacklisted(&mut self, other_jti: &str) -> bool {
        diesel::select(diesel::dsl::exists(
            token_blacklist.filter(jti.eq(other_jti)),
        ))
        .get_result(&mut self.get_conn())
        .expect("Failed to check token blacklist")
    }

    /// Remove expired entries from the blacklist.
    fn prune_blacklist(&mut self) {
        let now = Utc::now().naive_utc();
        diesel::delete(token_blacklist.filter(expires_at.lt(now))).execute(&mut self.get_conn());
    }
}

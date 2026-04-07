use crate::models::db::User;
use crate::schema::token_blacklist;
use crate::schema::users;
use crate::{config::Config, models::db::BlacklistEntry};
use chrono::{NaiveDateTime, Utc};
use diesel::dsl::exists;
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
        let state = Self { config, pool };
        state.prune_blacklist();
        state
    }

    fn get_conn(&self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool.get().expect("Failed to get DB connection")
    }

    /// Add a JTI to the blacklist.
    pub fn blacklist_token(&self, other_jti: &str, exp: NaiveDateTime) {
        let entry = BlacklistEntry {
            jti: other_jti.into(),
            expires_at: exp,
        };

        diesel::insert_into(token_blacklist::table)
            .values(&entry)
            .execute(&mut self.get_conn())
            .expect("Failed to add token to blacklist");
        self.prune_blacklist();
    }

    /// Returns `true` if the JTI is currently blacklisted.
    pub fn is_blacklisted(&self, jti: &str) -> bool {
        diesel::select(exists(
            token_blacklist::table.filter(token_blacklist::jti.eq(jti)),
        ))
        .get_result(&mut self.get_conn())
        .expect("Failed to check token blacklist")
    }

    /// Remove expired entries from the blacklist.
    fn prune_blacklist(&self) {
        let now = Utc::now().naive_utc();
        diesel::delete(token_blacklist::table.filter(token_blacklist::expires_at.lt(now)))
            .execute(&mut self.get_conn())
            .expect("Failed to prune token blacklist");
    }

    pub fn get_user_by_name(&self, username: &str) -> Option<User> {
        users::table
            .filter(users::username.eq(username))
            .load::<User>(&mut self.get_conn())
            .ok()
            .and_then(|users| users.into_iter().next())
    }
}

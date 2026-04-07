use chrono::NaiveDateTime;
use diesel::{prelude::*, sqlite::Sqlite};

use crate::schema;

// ── Stored user ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(
    table_name = schema::users,
    check_for_backend(Sqlite),
)]
pub struct User {
    pub id: String,
    pub username: String,
    /// bcrypt hash of the password
    pub password_hash: String,
    pub created_at: String,
}

#[derive(Insertable)]
#[diesel(
    table_name = schema::users,
    check_for_backend(Sqlite),
)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub username: &'a str,
    pub password_hash: &'a str,
}

#[derive(Debug, Queryable, Insertable)]
#[diesel(
    table_name = schema::token_blacklist,
    check_for_backend(Sqlite),
)]
pub struct BlacklistEntry {
    pub jti: String,
    pub expires_at: NaiveDateTime,
}

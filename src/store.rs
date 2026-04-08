use crate::config::Config;
use crate::error::BloggerResult;
use crate::error::db::DbError;
use crate::models::db::blog_post::{BlogPost, BlogPostItem, NewBlogPost, UpdateBlogPost};
use crate::models::db::user::{BlacklistEntry, NewUser, User};
use crate::schema::{blog_post, token_blacklist, users};
use bcrypt::BcryptResult;
use chrono::DateTime;
use chrono::Utc;
use diesel::QueryDsl;
use diesel::SelectableHelper;
use diesel::dsl::exists;
use diesel::r2d2::ConnectionManager;
use diesel::{ExpressionMethods, RunQueryDsl, SqliteConnection};
use r2d2::{Pool, PooledConnection};
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Shared application state injected via `web::Data`.
pub struct AppState {
    pub config: Config,
    pool: DbPool,
}

fn hash_password(password: &str) -> BcryptResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

impl AppState {
    pub fn new(config: Config) -> BloggerResult<Self> {
        let manager = ConnectionManager::<SqliteConnection>::new(&config.database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");
        let state = Self { config, pool };
        state.prune_blacklist()?;

        // Initialize new user
        let user_count = state.get_user_count()?;
        if user_count == 0 {
            log::info!("No users exist.");
            if let Some(init_username) = &state.config.init_user_name
                && let Some(init_password) = &state.config.init_user_pass
            {
                log::info!("Creating initial user '{}'", init_username);
                state.create_user(NewUser {
                    id: &Uuid::new_v4().to_string(),
                    username: init_username,
                    password_hash: &hash_password(init_password)?,
                })?;
            } else {
                panic!("No users exist, and INIT_USER_NAME or INIT_USER_PASS is not set!");
            }
        }

        Ok(state)
    }

    fn get_conn(&self) -> BloggerResult<PooledConnection<ConnectionManager<SqliteConnection>>> {
        Ok(self.pool.get().map_err(|e| DbError::R2d2Error(e))?)
    }

    /// Add a JTI to the blacklist.
    pub fn blacklist_token(&self, other_jti: &str, exp: i64) -> BloggerResult<()> {
        let exp_datetime = DateTime::from_timestamp_secs(exp)
            .expect("Expiration time cannot be parsed")
            .naive_utc();
        let entry = BlacklistEntry {
            jti: other_jti.into(),
            expires_at: exp_datetime,
        };

        diesel::insert_into(token_blacklist::table)
            .values(&entry)
            .execute(&mut self.get_conn()?)
            .expect("Failed to add token to blacklist");
        self.prune_blacklist()?;
        Ok(())
    }

    /// Returns `true` if the JTI is currently blacklisted.
    pub fn is_blacklisted(&self, jti: &str) -> BloggerResult<bool> {
        Ok(diesel::select(exists(
            token_blacklist::table.filter(token_blacklist::jti.eq(jti)),
        ))
        .get_result(&mut self.get_conn()?)
        .map_err(|e| DbError::QueryError(e))?)
    }

    /// Remove expired entries from the blacklist.
    fn prune_blacklist(&self) -> BloggerResult<()> {
        let now = Utc::now().naive_utc();
        diesel::delete(token_blacklist::table.filter(token_blacklist::expires_at.lt(now)))
            .execute(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?;
        Ok(())
    }

    pub fn get_user_by_name(&self, username: &str) -> BloggerResult<Option<User>> {
        Ok(users::table
            .filter(users::username.eq(username))
            .load::<User>(&mut self.get_conn()?)
            .ok()
            .and_then(|users| users.into_iter().next()))
    }

    pub fn get_user_by_id(&self, id: &str) -> BloggerResult<Option<User>> {
        Ok(users::table
            .filter(users::id.eq(id))
            .load::<User>(&mut self.get_conn()?)
            .ok()
            .and_then(|users| users.into_iter().next()))
    }

    pub fn get_user_count(&self) -> BloggerResult<i64> {
        Ok(users::table
            .count()
            .get_result(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?)
    }

    pub fn create_user(&self, user: NewUser) -> BloggerResult<()> {
        diesel::insert_into(users::table)
            .values(&user)
            .execute(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?;
        Ok(())
    }

    pub fn update_password(&self, user_id: &str, password: &str) -> BloggerResult<()> {
        let hash = hash_password(password)?;
        diesel::update(users::table)
            .filter(users::id.eq(user_id))
            .set(users::password_hash.eq(hash))
            .execute(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?;
        Ok(())
    }

    pub fn get_post_list(&self) -> BloggerResult<Vec<BlogPostItem>> {
        Ok(blog_post::table
            .select(BlogPostItem::as_select())
            .load::<BlogPostItem>(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?)
    }

    pub fn get_post(&self, id: i32) -> BloggerResult<Option<BlogPost>> {
        Ok(blog_post::table
            .filter(blog_post::id.eq(id))
            .select(BlogPost::as_select())
            .load::<BlogPost>(&mut self.get_conn()?)
            .ok()
            .and_then(|blogs| blogs.into_iter().next()))
    }

    pub fn create_post(&self, blog: NewBlogPost) -> BloggerResult<i32> {
        Ok(diesel::insert_into(blog_post::table)
            .values(&blog)
            .returning(blog_post::id)
            .get_result(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?)
    }

    pub fn update_post(&self, id: i32, title: String, post_content: String) -> BloggerResult<()> {
        let now = Utc::now().naive_utc();
        let post = UpdateBlogPost {
            id,
            title,
            post_content,
            modified_at: now.to_string(),
        };
        diesel::update(blog_post::table)
            .filter(blog_post::id.eq(&post.id))
            .set(&post)
            .execute(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?;
        Ok(())
    }

    pub fn delete_post(&self, id: i32) -> BloggerResult<()> {
        diesel::delete(blog_post::table.filter(blog_post::id.eq(id)))
            .execute(&mut self.get_conn()?)
            .map_err(|e| DbError::QueryError(e))?;
        Ok(())
    }
}

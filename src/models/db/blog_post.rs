use diesel::{prelude::*, sqlite::Sqlite};

use crate::schema;

#[derive(Debug, Queryable, Selectable)]
#[diesel(
    table_name = schema::blog_post,
    check_for_backend(Sqlite),
)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub post_content: String,
    pub created_at: String,
    pub modified_at: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(
    table_name = schema::blog_post,
    check_for_backend(Sqlite),
)]
pub struct NewBlogPost {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub post_content: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(
    table_name = schema::blog_post,
    check_for_backend(Sqlite),
)]
pub struct BlogPostItem {
    pub id: String,
    pub title: String,
    pub author_id: String,
    pub created_at: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(
    table_name = schema::blog_post,
    check_for_backend(Sqlite),
)]
pub struct UpdateBlogPost {
    pub id: String,
    pub title: String,
    pub post_content: String,
    pub modified_at: String,
}

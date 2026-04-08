use actix_web::{HttpResponse, web};
use serde_json::json;

use crate::{
    error::{BloggerError, BloggerResult, auth::AuthError, db::DbError},
    models::{BlogPostForm, Claims, db::blog_post::NewBlogPost},
    store::AppState,
};

pub async fn post_list_get(state: web::Data<AppState>) -> BloggerResult<HttpResponse> {
    let post_list = web::block(move || state.get_post_list()).await??;
    Ok(HttpResponse::Ok().json(json!({
        "posts": post_list,
    })))
}

pub async fn post_get(
    state: web::Data<AppState>,
    id: web::Path<i32>,
) -> BloggerResult<HttpResponse> {
    let post = web::block(move || {
        state
            .get_post(*id)?
            .ok_or(BloggerError::DbError(DbError::NotFound))
    })
    .await??;
    Ok(HttpResponse::Ok().json(json!({
        "post": post,
    })))
}

pub async fn post_post(
    state: web::Data<AppState>,
    form: web::Form<BlogPostForm>,
    claims: Claims,
) -> BloggerResult<HttpResponse> {
    let post = NewBlogPost {
        title: form.title.clone(),
        author_id: claims.sub,
        post_content: form.post_content.clone(),
    };
    let id = web::block(move || state.create_post(post)).await??;

    Ok(HttpResponse::Ok().json(json!({
        "id": id,
        "message": format!("Successfully created blog post \"{}\"", form.title)
    })))
}

pub async fn post_put(
    state: web::Data<AppState>,
    form: web::Form<BlogPostForm>,
    claims: Claims,
    id: web::Path<i32>,
) -> BloggerResult<HttpResponse> {
    let id_into = id;
    let state_into = state.clone();
    let existing_post = web::block(move || {
        state_into
            .get_post(*id_into)?
            .ok_or(BloggerError::DbError(DbError::NotFound))
    })
    .await??;
    if existing_post.author_id == claims.sub {
        web::block(move || {
            state.update_post(
                existing_post.id,
                form.title.clone(),
                form.post_content.clone(),
            )
        })
        .await??;
        return Ok(HttpResponse::Ok().json(json!({
            "message": format!("Updated post \"{}\"", existing_post.title),
        })));
    }

    Err(AuthError::Unauthorized.into())
}

pub async fn post_delete(
    state: web::Data<AppState>,
    claims: Claims,
    id: web::Path<i32>,
) -> BloggerResult<HttpResponse> {
    let id_into = id.clone();
    let state_into = state.clone();
    let existing_post = web::block(move || {
        state_into
            .get_post(id_into)?
            .ok_or(BloggerError::DbError(DbError::NotFound))
    })
    .await??;
    if existing_post.author_id == claims.sub {
        web::block(move || state.delete_post(*id)).await??;
        return Ok(HttpResponse::Ok().json(json!({
            "message": format!("Deleted post \"{}\"", existing_post.title),
        })));
    }

    Err(AuthError::Unauthorized.into())
}

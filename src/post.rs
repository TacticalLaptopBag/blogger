use actix_web::{HttpResponse, web};
use serde_json::json;

use crate::{
    error::{BloggerResult, auth::AuthError},
    models::{BlogPostForm, Claims, db::blog_post::NewBlogPost},
    store::AppState,
};

pub async fn post_list_get(state: web::Data<AppState>) -> BloggerResult<HttpResponse> {
    let post_list = state.get_post_list()?;
    Ok(HttpResponse::Ok().json(json!({
        "posts": post_list,
    })))
}

pub async fn post_get(
    state: web::Data<AppState>,
    id: web::Path<i32>,
) -> BloggerResult<HttpResponse> {
    let post = state.get_post(*id)?;
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
    let id = state.create_post(post)?;

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
    let existing_post = state.get_post(*id)?;
    if let Some(post) = existing_post
        && post.author_id == claims.sub
    {
        state.update_post(post.id, form.title.clone(), form.post_content.clone())?;
        return Ok(HttpResponse::Ok().json(json!({
            "message": format!("Updated post \"{}\"", post.title),
        })));
    }

    Err(AuthError::Unauthorized.into())
}

pub async fn post_delete(
    state: web::Data<AppState>,
    claims: Claims,
    id: web::Path<i32>,
) -> BloggerResult<HttpResponse> {
    let existing_post = state.get_post(*id)?;
    if let Some(post) = existing_post
        && post.author_id == claims.sub
    {
        state.delete_post(*id)?;
        return Ok(HttpResponse::Ok().json(json!({
            "message": format!("Deleted post \"{}\"", post.title),
        })));
    }

    Err(AuthError::Unauthorized.into())
}

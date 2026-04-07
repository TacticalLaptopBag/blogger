use actix_web::{HttpResponse, web};
use serde_json::json;
use uuid::Uuid;

use crate::{
    models::{BlogPostForm, Claims, db::blog_post::NewBlogPost},
    store::AppState,
};

pub async fn post_list_get(state: web::Data<AppState>) -> HttpResponse {
    let post_list = state.get_post_list();
    HttpResponse::Ok().json(json!({
        "posts": post_list,
    }))
}

pub async fn post_get(state: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    let post = state.get_post(&id);
    HttpResponse::Ok().json(json!({
        "post": post,
    }))
}

pub async fn post_post(
    state: web::Data<AppState>,
    form: web::Form<BlogPostForm>,
    claims: Claims,
) -> HttpResponse {
    let id = Uuid::new_v4().to_string();
    let post = NewBlogPost {
        id: id.clone(),
        title: form.title.clone(),
        author_id: claims.sub,
        post_content: form.post_content.clone(),
    };
    state.create_post(post);

    HttpResponse::Ok().json(json!({
        "id": id,
        "message": format!("Successfully created blog post \"{}\"", form.title)
    }))
}

pub async fn post_put(
    state: web::Data<AppState>,
    form: web::Form<BlogPostForm>,
    claims: Claims,
    id: web::Path<String>,
) -> HttpResponse {
    let existing_post = state.get_post(&id);
    if let Some(post) = existing_post
        && post.author_id == claims.sub
    {
        state.update_post(post.id, form.title.clone(), form.post_content.clone());
        return HttpResponse::Ok().json(json!({
            "message": format!("Updated post \"{}\"", post.title),
        }));
    }

    HttpResponse::Unauthorized().json(json!({
        "error": "You do not have permission to edit this post!"
    }))
}

pub async fn post_delete(
    state: web::Data<AppState>,
    claims: Claims,
    id: web::Path<String>,
) -> HttpResponse {
    let existing_post = state.get_post(&id);
    if let Some(post) = existing_post
        && post.author_id == claims.sub
    {
        state.delete_post(&id);
        return HttpResponse::Ok().json(json!({
            "message": format!("Deleted post \"{}\"", post.title),
        }));
    }

    HttpResponse::Unauthorized().json(json!({
        "error": "You do not have permission to edit this post!"
    }))
}

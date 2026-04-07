use actix_web::{HttpResponse, web};

pub async fn post_list_get() -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

pub async fn post_get(id: web::Path<String>) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

pub async fn post_post() -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

pub async fn post_put(id: web::Path<String>) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

pub async fn post_delete(id: web::Path<String>) -> HttpResponse {
    HttpResponse::NotImplemented().finish()
}

mod auth;
mod config;
mod errors;
mod models;
mod schema;
mod store;

use actix_web::{App, HttpServer, middleware::Logger, web};
use store::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cfg = config::Config::from_env();
    if cfg.jwt_secret == "debug-key" {
        log::warn!("===============================================================");
        log::warn!("JWT_SECRET is not configured! DO NOT use this in a deployment!");
        log::warn!("===============================================================");
    }
    let host = cfg.host.clone();
    let port = cfg.port;

    let state = web::Data::new(AppState::new(cfg));

    log::info!("Starting auth-api on {host}:{port}");

    HttpServer::new(move || {
        App::new().service(
            web::scope("/api/v1")
                .app_data(state.clone())
                .wrap(Logger::default())
                // Login: POST submits credentials, GET returns current user info
                .route("/login", web::post().to(auth::login_post))
                .route("/login", web::get().to(auth::login_get))
                .route("/login", web::put().to(auth::login_put))
                // Refresh the access token using the refresh token cookie
                .route("/refresh", web::post().to(auth::refresh_post))
                // Logout: blacklist token and clear cookies
                .route("/logout", web::post().to(auth::logout_post))
                .route("/protected", web::get().to(auth::protected_get)),
        )
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

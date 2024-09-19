use axum::middleware;
use axum::routing::{get, post};
use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse, Router};
use searchdb::Repository;
use spow::pow::Pow;
use std::net::SocketAddr;
use std::time;
// pub mod config
pub mod indexing;
pub mod insertdb;
pub mod pow;
pub mod rest;
pub mod schemas;
pub mod search;
pub mod searchdb;
pub mod services;
pub mod templates;

#[derive(Clone)]
pub struct Repositories {
    pub db: Repository,
    pub hb: handlebars::Handlebars<'static>,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 12)]
async fn main() {
    tracing_subscriber::fmt().json().init();
    let mut hb = handlebars::Handlebars::new();
    hb.register_template_string("post", templates::POST_TPL)
        .unwrap();

    hb.register_template_string("publish", templates::PUBLISH_TPL)
        .unwrap();

    hb.register_template_string("home", templates::HOME_TPL)
        .unwrap();
    hb.register_template_string("list", templates::LIST_TPL)
        .unwrap();
    let repos = Repositories {
        db: Repository::new("redis", "redka").await,
        hb,
    };
    Pow::init_random().unwrap();
    let app = Router::new()
        .route("/:lang/home", get(rest::home))
        .route("/:lang/search", get(rest::search_post))
        .route("/:lang/post", post(rest::post_form))
        .route("/:lang/post/:slug", get(rest::get_post))
        .route("/:lang/post", get(rest::get_challenge_form))
        .layer(middleware::from_fn(log_access))
        .with_state(repos);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8062));
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn log_access(req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let t0 = time::Instant::now();
    let uri = req.uri().to_owned().to_string();

    let res = next.run(req).await;

    let t = (time::Instant::now() - t0).as_millis();
    tracing::info!(uri = uri, time_ms = t);
    Ok(res)
}

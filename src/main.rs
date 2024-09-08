mod config;
mod filters;
mod handlers;
mod model;
mod response;
mod route;
mod schema;
mod session_auth;
mod validation;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use config::Config;
use dotenv::dotenv;
use route::create_router;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use time::Duration;
use tower_http::trace::TraceLayer;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

#[allow(dead_code)]
pub struct AppState {
    db: Pool<Postgres>,
    env: Config,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = Config::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection to db successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let redis_pool = match RedisPool::new(RedisConfig::default(), None, None, None, 6) {
        Ok(pool) => {
            println!("✅ Connection to redis successfull!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to redis: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("✅ Server started successfully");

    let redis_conn = redis_pool.connect();

    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::hours(12)));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = create_router(Arc::new(AppState {
        db: pool.clone(),
        env: config.clone(),
    }))
    .nest_service("/assets", ServeDir::new("./assets"))
    .layer(cors)
    .layer(session_layer)
    .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    redis_conn.await.unwrap().unwrap();
}

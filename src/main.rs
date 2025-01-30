use std::time::Duration;

use axum::Router;

use backend::config::Config;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend::{http, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // This returns an error if the `.env` file doesn't exist, but that's not what we want
    // since we're not going to use a `.env` file if we deploy this application
    dotenv::dotenv().ok();
    let config = Config::parse();

    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .expect("Failed to bind to port");
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await
        .expect("Failed to create db pool");

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let reqwest_client = reqwest::Client::new();
    let state = AppState {
        pool,
        config,
        reqwest_client,
    };

    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .nest("/feed", http::feed::router())
        .with_state(state);

    axum::serve(listener, app).await.unwrap();
}

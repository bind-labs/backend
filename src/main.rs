use axum::Router;

use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod feed;
mod index;
mod items;
mod user;
mod search;

#[derive(Clone)]
struct AppState {
    pub pool: sqlx::PgPool,
}

impl AppState {
    async fn new() -> Result<Self, sqlx::Error> {
        let pool = db::get_db_pool().await?;
        Ok(Self { pool })
    }
}

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

    let state = AppState::new().await.expect("Failed to create app state");
    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .merge(user::router())
        .merge(items::router())
        .merge(feed::router())
        .merge(index::router())
        .merge(search::router());

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

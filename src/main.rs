use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use bind_backend::config::Config;
use bind_backend::feed::daemon::Daemon;
use bind_backend::http::{self, common::ApiContext};

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

    // Ignore the error when `.env` does not exist
    dotenv::dotenv().ok();
    let config = Config::new().expect("Failed to parse config");

    // Initialize database and run migrations
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await
        .expect("Failed to create db pool");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed while running migrations");

    // Start the feed daemon
    let daemon = Daemon::new(pool.clone(), 5);

    // Start the API
    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .nest("/api/v1", http::router())
        .with_state(ApiContext::new(pool.clone(), config.clone()));

    let listener = TcpListener::bind(config.host)
        .await
        .expect("Failed to bind to port");
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(pool, daemon))
        .await
        .unwrap();
}

async fn shutdown_signal(db: PgPool, daemon: Daemon) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutting down feed daemon");
    daemon.cancel().await;
    tracing::info!("Feed daemon shut down");

    tracing::info!("Shutting down postgres connection pool");
    db.close().await;
    tracing::info!("Postgres connection pool shut down");
}

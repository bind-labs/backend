use api::auth::oauth::{OAuth2Client, OAuth2ClientConfig};
use api::http::common::Origins;
use api::smtp::SmtpClient;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use api::config::Config;
use api::feed::daemon::Daemon;
use api::http::{self, common::ApiContext};

#[tokio::main]
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

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

    // Create OAuth clients
    let mut oauth_clients = HashMap::new();
    for (name, oauth_config) in config.oauth.iter() {
        oauth_clients.insert(
            name.clone(),
            OAuth2Client::new(OAuth2ClientConfig::from_config(
                oauth_config,
                name,
                &config.web_origin,
            ))
            .expect("Failed to create OAuth client"),
        );
    }

    // Create SMTP client
    let smtp_client = SmtpClient::new(&config.smtp_uri, &config.smtp_from)
        .await
        .expect("Failed to create SMTP client");

    // Start the API
    let context = ApiContext {
        pool: pool.clone(),
        reqwest_client: reqwest::Client::new(),
        oauth_clients,
        smtp_client,
        origins: Origins {
            web: config.web_origin.clone(),
            android: config.android_origin.clone(),
            ios: config.ios_origin.clone(),
        },
        jwt_secret: config.jwt_secret.clone(),
    };
    let (router, api) = http::router()
        .layer(TraceLayer::new_for_http())
        .with_state(context)
        .split_for_parts();

    let app = router
        .merge(Scalar::with_url("/docs", api))
        .into_make_service();

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

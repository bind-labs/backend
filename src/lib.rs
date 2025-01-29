pub mod config;
pub mod error;
pub mod feed_parsers;
pub mod html;
pub mod http;
pub mod schema;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: config::Config,
    pub reqwest_client: reqwest::Client,
}

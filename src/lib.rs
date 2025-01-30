pub mod config;
pub mod error;
pub mod feed;
pub mod html;
pub mod http;
pub mod schema;
pub mod query; 


use schema::*;
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: config::Config,
    pub reqwest_client: reqwest::Client,
}

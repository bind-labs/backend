pub use super::error::{Error, Result};
pub use axum::{
    extract::{Path, State},
    Json,
};
pub use serde::{Deserialize, Serialize};
pub use validator::Validate;

#[derive(Clone)]
pub struct ApiContext {
    pub pool: sqlx::PgPool,
    pub reqwest_client: reqwest::Client,
}

impl ApiContext {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            reqwest_client: reqwest::Client::new(),
        }
    }
}

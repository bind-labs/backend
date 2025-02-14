use std::collections::HashMap;

use crate::{auth::oauth::OAuth2Client, config::Config, smtp::SmtpClient};

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
    pub oauth_clients: HashMap<String, OAuth2Client>,
    pub smtp_client: SmtpClient,
    pub config: Config,
}

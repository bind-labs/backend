use std::collections::HashMap;

use crate::{
    auth::oauth::{OAuth2Client, OAuth2ClientConfig},
    config::Config,
};

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
    pub config: Config,
}

impl ApiContext {
    pub fn new(pool: sqlx::PgPool, config: Config) -> Self {
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

        Self {
            pool,
            reqwest_client: reqwest::Client::new(),
            config,
            oauth_clients,
        }
    }
}

use std::collections::HashMap;

use crate::{auth::oauth::OAuth2Client, smtp::SmtpClient};

pub use super::error::{Error, Result};
pub use crate::auth::user::AuthUser;

pub use axum::{
    extract::{Path, Query, State},
    http,
    response::IntoResponse,
    Json,
};
pub use ormx::{Delete, Insert, Patch, Table};
use reqwest::Url;
pub use serde::{Deserialize, Serialize};
pub use validator::Validate;

#[derive(Clone)]
pub struct ApiContext {
    pub pool: sqlx::PgPool,
    pub reqwest_client: reqwest::Client,
    pub oauth_clients: HashMap<String, OAuth2Client>,
    pub smtp_client: SmtpClient,
    pub origins: Origins,
    pub jwt_secret: String,
}

#[derive(Clone, Debug)]
pub struct Origins {
    pub web: Url,
    pub android: Url,
    pub ios: Url,
}

#[derive(Deserialize, Debug)]
pub struct Pagination {
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    20
}

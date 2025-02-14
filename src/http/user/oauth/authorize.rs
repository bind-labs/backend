use axum::extract::{Query, State};
use axum::http;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::auth::oauth::{OAuth2Client, OAuth2ClientConfig};
use crate::http::common::{ApiContext, Error, Result};
use crate::sql::OAuthRedirectClient;

#[derive(Deserialize)]
pub struct AuthorizeRequest {
    provider: String,
    client: OAuthRedirectClient,
}

pub async fn authorize(
    State(state): State<ApiContext>,
    Query(query): Query<AuthorizeRequest>,
) -> Result<impl IntoResponse> {
    let provider = state
        .config
        .oauth
        .get(&query.provider)
        .ok_or(Error::BadRequest(format!(
            "Provider {} not found",
            query.provider
        )))?;

    let client = OAuth2Client::new(OAuth2ClientConfig::from_config(
        provider,
        &query.provider,
        &state.config.web_origin,
    ))?;

    let auth_url = client.authorize_url(&state.pool, &query.client).await?;

    Ok(http::Response::builder()
        .status(http::StatusCode::TEMPORARY_REDIRECT)
        .header("Location", auth_url.to_string())
        .body("".to_string())
        .unwrap())
}

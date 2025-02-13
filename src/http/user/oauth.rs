use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{http, Json, Router};
use ormx::Delete;
use serde::Deserialize;

use crate::auth::jwt::BindJwtToken;
use crate::auth::oauth::{OAuth2Client, OAuth2ClientConfig};
use crate::http::common::{ApiContext, Error, Result};
use crate::sql::{OAuthRedirectClient, UserOAuthState};

pub fn router() -> Router<ApiContext> {
    Router::new()
        .route("/providers", get(list_providers))
        .route("/authorize", get(authorize))
        .route("/callback", get(callback))
}

async fn list_providers(State(state): State<ApiContext>) -> Result<Json<Vec<String>>> {
    let providers = state
        .config
        .oauth
        .keys()
        .map(|k| k.to_string())
        .collect::<Vec<_>>();
    Ok(Json(providers))
}

#[derive(Deserialize)]
struct AuthorizeRequest {
    provider: String,
    client: OAuthRedirectClient,
}

async fn authorize(
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

#[derive(Deserialize)]
struct CallbackRequest {
    code: String,
    state: String,
}

async fn callback(
    State(state): State<ApiContext>,
    Query(query): Query<CallbackRequest>,
) -> Result<impl IntoResponse> {
    let oauth_state = UserOAuthState::get_by_csrf_token(&state.pool, &query.state).await?;
    oauth_state.clone().delete(&state.pool).await?;

    let client = state
        .oauth_clients
        .get(&oauth_state.provider)
        .ok_or(Error::BadRequest(format!(
            "Provider {} not found",
            oauth_state.provider
        )))?;

    let external_token = client.exchange_code(&oauth_state, &query.code).await?;

    let user = external_token.create_or_update_user(&state.pool).await?;

    let token = BindJwtToken::user_to_token(&user, &state.config.jwt_secret).unwrap();

    Ok(http::Response::builder()
        .status(http::StatusCode::TEMPORARY_REDIRECT)
        .header(
            "Location",
            oauth_state
                .client
                .to_uri(&state.config)
                .join(&format!("#token={}", token))
                .unwrap()
                .to_string(),
        )
        .body("".to_string())
        .unwrap())
}

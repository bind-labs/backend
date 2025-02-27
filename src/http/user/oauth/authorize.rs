use crate::http::common::*;
use crate::sql::OAuthRedirectClient;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AuthorizeRequest {
    provider: String,
    client: OAuthRedirectClient,
}

/// Redirect to OAuth provider authorization page
#[utoipa::path(
    get,
    path = "/authorize",
    tag = "user:oauth",
    params(
        ("provider" = String, Query, description = "OAuth provider name"),
        ("client" = OAuthRedirectClient, Query, description = "Client type (web, android, ios)")
    ),
    responses(
        (status = 307, description = "Redirect to provider's authorization page"),
        (status = 400, description = "Invalid provider"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn authorize(
    State(state): State<ApiContext>,
    Query(query): Query<AuthorizeRequest>,
) -> Result<impl IntoResponse> {
    let client = state
        .oauth_clients
        .get(&query.provider)
        .ok_or(Error::BadRequest(format!(
            "Provider {} not found",
            query.provider
        )))?;

    let auth_url = client.authorize_url(&state.pool, &query.client).await?;

    Ok(http::Response::builder()
        .status(http::StatusCode::TEMPORARY_REDIRECT)
        .header("Location", auth_url.to_string())
        .body("".to_string())
        .unwrap())
}

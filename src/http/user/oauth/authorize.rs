use crate::http::common::*;
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

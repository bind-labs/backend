use crate::auth::user::AuthUserClaims;
use crate::http::common::*;
use crate::sql::UserOAuthState;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CallbackRequest {
    code: String,
    state: String,
}

/// Handle OAuth callback from authentication providers
///
/// This endpoint handles the OAuth 2.0 callback from external providers (e.g., Google, GitHub).
/// After the user authorizes the application with the provider, they are redirected back to this
/// endpoint with an authorization code and state parameter.
///
/// The endpoint:
/// 1. Validates the state parameter to prevent CSRF attacks
/// 2. Exchanges the authorization code for an access token
/// 3. Fetches user information from the provider
/// 4. Creates or updates the user in the database
/// 5. Issues a JWT token for the user
/// 6. Redirects back to the client application with the token
///
/// The client app should extract the token from the URL fragment and use it for future API requests.
#[utoipa::path(
    get,
    path = "/callback",
    tag = "user:oauth",
    params(
        ("code" = String, Query, description = "Authorization code from provider"),
        ("state" = String, Query, description = "CSRF state token to verify the request")
    ),
    responses(
        (status = 307, description = "Redirect back to app with token in URL fragment (#token=...)"),
        (status = 400, description = "Invalid state (CSRF protection) or authorization code"),
        (status = 500, description = "Internal server error during token exchange or user creation")
    )
)]
pub async fn callback(
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

    let claims: AuthUserClaims = user.into();
    let token = claims.to_jwt(&state.jwt_secret).unwrap();

    Ok(http::Response::builder()
        .status(http::StatusCode::TEMPORARY_REDIRECT)
        .header(
            "Location",
            oauth_state
                .client
                .to_uri(&state.origins)
                .join(&format!("#token={}", token))
                .unwrap()
                .to_string(),
        )
        .body("".to_string())
        .unwrap())
}

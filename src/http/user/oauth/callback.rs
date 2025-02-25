use crate::auth::user::AuthUserClaims;
use crate::http::common::*;
use crate::sql::UserOAuthState;

#[derive(Deserialize)]
pub struct CallbackRequest {
    code: String,
    state: String,
}

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

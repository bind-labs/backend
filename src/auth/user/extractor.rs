use axum::{extract::FromRequestParts, http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use reqwest::StatusCode;

use super::AuthUserClaims;
use crate::http::common::ApiContext;

/// Authenticated user information extracted from a valid JWT token.
///
/// Add this as a parameter to a handler function to require the user to be logged in.
/// Parses a JWT from the `Authorization: Bearer <token>` header.
///
/// This extractor will return 401 Unauthorized if no valid token is provided.
#[derive(Debug, Clone, PartialEq, Eq, utoipa::ToSchema)]
pub struct AuthUser {
    /// Unique identifier for the user
    pub id: i32,
    /// User's email address
    pub email: String,
    /// User's username
    pub username: String,
}

/// Add this as a parameter to a handler function to optionally check if the user is logged in.
///
/// If the `Authorization` header is absent then this will be `Self(None)`, otherwise it will
/// validate the token.
///
/// This is in contrast to directly using `Option<AuthUser>`, which will be `None` if there
/// is *any* error in deserializing, which isn't exactly what we want.
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequestParts<ApiContext> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiContext,
    ) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let auth_header: Option<TypedHeader<Authorization<Bearer>>> = parts
            .extract()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // In tests, mock a valid token when the Authorization header is missing
        #[cfg(test)]
        let auth_header = Some(auth_header.unwrap_or_else(|| {
            TypedHeader(
                Authorization::bearer(
                    &AuthUserClaims::new(1, "test@example.com".to_string(), "test".to_string())
                        .to_jwt(&state.jwt_secret)
                        .unwrap(),
                )
                .unwrap(),
            )
        }));

        let auth_header = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header.0.token();
        let claims = AuthUserClaims::from_jwt(token, &state.jwt_secret).map_err(|e| {
            // TODO: return better errors to the user
            tracing::debug!("failed to parse Authorization header: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

        Ok(claims.into())
    }
}

impl From<AuthUserClaims> for AuthUser {
    fn from(claims: AuthUserClaims) -> Self {
        Self {
            id: claims.id,
            email: claims.email,
            username: claims.username,
        }
    }
}

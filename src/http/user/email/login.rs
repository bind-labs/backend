use crate::auth::password::verify_password;
use crate::auth::user::AuthUserClaims;
use crate::http::common::*;
use crate::sql::User;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UserLoginRequest {
    email: Option<String>,
    username: Option<String>,
    password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserLoginResponse {
    token: String,
}

/// Login with email/username and password
///
/// Authenticates a user using their email/username and password, returning a JWT token
/// that can be used for subsequent API requests. The token should be included in the
/// Authorization header as a Bearer token.
///
/// The token is valid for 7 days by default. After expiration, the user will need to
/// login again to obtain a new token.
#[utoipa::path(
    post,
    path = "/login",
    tag = "user:email",
    request_body = UserLoginRequest,
    responses(
        (status = 200, description = "Login successful", body = UserLoginResponse, 
         example = json!({"token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."})),
        (status = 400, description = "Bad request - missing email or username"),
        (status = 401, description = "Login failed - incorrect credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    State(state): State<ApiContext>,
    Json(info): Json<UserLoginRequest>,
) -> Result<Json<UserLoginResponse>> {
    let user = if let Some(email) = info.email {
        User::get_by_email(&state.pool, &email).await.ok()
    } else if let Some(username) = info.username {
        User::get_by_username(&state.pool, &username).await.ok()
    } else {
        return Err(Error::BadRequest("Email or username required".to_string()));
    };

    if let Some(user) = user
        && let Some(ref password_hash) = user.password_hash
        && verify_password(&info.password, password_hash).expect("Failed to verify password")
    {
        let claims: AuthUserClaims = user.into();
        let token = claims
            .to_jwt(&state.jwt_secret)
            .expect("Failed to create token");
        Ok(Json(UserLoginResponse { token }))
    } else {
        Err(Error::LoginFailed)
    }
}

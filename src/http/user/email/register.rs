use crate::auth::password::hash_password;
use crate::auth::user::AuthUserClaims;
use crate::http::common::*;
use crate::sql::{InsertUser, User, UserEmailVerification};

#[derive(Deserialize)]
pub struct UserRegisterRequest {
    email: String,
    email_code: String,
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserRegisterResponse {
    token: String,
}

pub async fn register(
    State(state): State<ApiContext>,
    Json(info): Json<UserRegisterRequest>,
) -> Result<Json<UserRegisterResponse>> {
    // Check if user already exists
    if User::get_by_email(&state.pool, &info.email).await.is_ok() {
        return Err(Error::Conflict(format!(
            "User with email {} already exists",
            info.email
        )));
    }

    // Check the email code
    let user_email_verification = UserEmailVerification::get_by_code(&state.pool, &info.email_code)
        .await
        .map_err(|_| Error::Forbidden("Email code not found".to_string()))?;
    if user_email_verification.email != info.email {
        return Err(Error::Forbidden("Email code not valid".to_string()));
    }

    // Create the user
    let password_hash = hash_password(&info.password)
        // TODO: should this be a 500?
        .map_err(|_| Error::BadRequest("Failed to hash password".to_string()))?;
    let user = InsertUser {
        email: info.email,
        username: info.username,
        password_hash: Some(password_hash),
        providers: vec![],
    }
    .insert(&state.pool)
    .await?;

    // Send the token
    let claims: AuthUserClaims = user.into();
    let token = claims.to_jwt(&state.jwt_secret).unwrap();
    Ok(Json(UserRegisterResponse { token }))
}

use zxcvbn::Score;

use crate::auth::password::hash_password;
use crate::auth::user::AuthUserClaims;
use crate::http::common::*;
use crate::sql::{InsertUser, User, UserEmailVerification};

use std::sync::LazyLock;

static USERNAME_REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[a-zA-Z0-9_\.-]{2,48}$").unwrap());

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UserRegisterRequest {
    email: String,
    email_code: String,
    username: String,
    password: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserRegisterResponse {
    token: String,
}

/// Register a new user with email and password
#[utoipa::path(
    post,
    path = "/register",
    tag = "user:email",
    request_body = UserRegisterRequest,
    responses(
        (status = 200, description = "User registered successfully", body = UserRegisterResponse),
        (status = 400, description = "Invalid registration data"),
        (status = 403, description = "Invalid email verification code"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn register(
    State(state): State<ApiContext>,
    Json(info): Json<UserRegisterRequest>,
) -> Result<Json<UserRegisterResponse>> {
    if !USERNAME_REGEX.is_match(&info.username) {
        return Err(Error::BadRequest(format!(
            "Username must match regex [a-zA-Z0-9_\\.-]{{2,48}}, got {}",
            info.username
        )));
    }

    let password_estimate = zxcvbn::zxcvbn(&info.password, &[&info.username, &info.email]);
    if password_estimate.score() <= Score::Two {
        if let Some(feedback) = password_estimate.feedback() {
            return Err(Error::BadRequest(format!(
                "{}. {}",
                feedback
                    .warning()
                    .map(|x| x.to_string())
                    .unwrap_or("Password is too weak".to_string()),
                feedback
                    .suggestions()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(". ")
            )));
        } else {
            return Err(Error::BadRequest("Password is too weak".to_string()));
        }
    }
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

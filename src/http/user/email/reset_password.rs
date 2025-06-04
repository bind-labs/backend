use crate::http::common::*;
use crate::{
    auth::password::hash_password,
    sql::{PasswordVerificationCode, User},
};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    email: String,
    code: String,
    new_password: String,
}

/// Sends an email to the user with a verification code
/// which must be used during registration
#[utoipa::path(
    post,
    path = "/reset_password",
    tag = "user:email",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful"),
        (status = 400, description = "Invalid email format"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn reset_password(
    State(state): State<ApiContext>,
    Json(query): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse> {
    // Check the email code
    let password_verification = PasswordVerificationCode::get_by_code(&state.pool, &query.code)
        .await
        .map_err(|_| Error::Forbidden("Password code not found".to_string()))?;
    if password_verification.email != query.email {
        return Err(Error::Forbidden("Password code not valid".to_string()));
    }

    // modify the password for the user
    let password_hash = hash_password(&query.new_password)
        // TODO: should this be a 500?
        .map_err(|_| Error::BadRequest("Failed to hash password".to_string()))?;

    let mut user = User::get_by_email(&state.pool, &query.email).await?;
    user.set_password_hash(&state.pool, Some(password_hash))
        .await?;

    Ok(http::StatusCode::OK)
}

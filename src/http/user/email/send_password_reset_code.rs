use lettre::{
    message::{header::ContentType, Mailbox},
    Address, Message,
};

use crate::sql::{PasswordVerificationCode, User};
use crate::{http::common::*, sql::InsertPasswordVerificationCode};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct PasswordResetRequest {
    email: String,
}

/// Sends an email to the user with a verification code
/// which must be used during registration
#[utoipa::path(
    post,
    path = "/send-password-reset-code",
    tag = "user:email",
    request_body = PasswordResetRequest,
    responses(
        (status = 200, description = "Password resent email sent successfully"),
        (status = 400, description = "Invalid email format"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn send_password_reset_code(
    State(state): State<ApiContext>,
    Json(query): Json<PasswordResetRequest>,
) -> Result<impl IntoResponse> {
    // if a user for the email does not exist, we don't want them to know so we just return 200
    if User::get_by_email(&state.pool, &query.email).await.is_err() {
        return Ok(http::StatusCode::OK);
    }

    // Insert verification code into DB
    let code = PasswordVerificationCode::generate_code();
    InsertPasswordVerificationCode {
        email: query.email.clone(),
        code: code.clone(),
    }
    .insert(&state.pool)
    .await?;
    PasswordVerificationCode::cleanup_expired(&state.pool).await?;

    // Send email to user
    let to_email: Address = query
        .email
        .parse()
        .map_err(|_| Error::BadRequest("Invalid email".to_string()))?;
    let to = Mailbox::new(None, to_email);

    let email = Message::builder()
        .from(state.smtp_client.from.clone())
        .to(to)
        .subject("Bind: Password reset code")
        .header(ContentType::TEXT_PLAIN)
        .body(code)
        .unwrap();

    state.smtp_client.send(&email).await?;

    Ok(http::StatusCode::OK)
}

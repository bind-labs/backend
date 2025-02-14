use axum::{extract::State, http, response::IntoResponse, Json};
use lettre::{
    message::{header::ContentType, Mailbox},
    Address, Message,
};
use ormx::Insert;
use serde::Deserialize;

use crate::{
    http::{
        common::ApiContext,
        error::{Error, Result},
    },
    sql::{InsertUserEmailVerification, User, UserEmailVerification},
};

#[derive(Deserialize)]
pub struct EmailVerificationRequest {
    email: String,
}

/// Sends an email to the user with a verification code
/// which must be used during registration
pub async fn verify(
    State(state): State<ApiContext>,
    Json(query): Json<EmailVerificationRequest>,
) -> Result<impl IntoResponse> {
    if User::get_by_email(&state.pool, &query.email).await.is_ok() {
        return Err(Error::Conflict(format!(
            "User with email {} already exists",
            query.email
        )));
    }

    // Insert verification code into DB
    let code = UserEmailVerification::generate_code();
    InsertUserEmailVerification {
        email: query.email.clone(),
        code: code.clone(),
    }
    .insert(&state.pool)
    .await?;
    UserEmailVerification::cleanup_expired(&state.pool).await?;

    // Send email to user
    let to_email: Address = query
        .email
        .parse()
        .map_err(|_| Error::BadRequest("Invalid email".to_string()))?;
    let to = Mailbox::new(None, to_email);

    let email = Message::builder()
        .from(state.smtp_client.from.clone())
        .to(to)
        .subject("Bind: Email Verification")
        .header(ContentType::TEXT_PLAIN)
        .body(code)
        .unwrap();

    state.smtp_client.send(&email).await?;

    Ok(http::StatusCode::OK)
}

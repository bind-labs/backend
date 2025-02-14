use axum::{extract::State, http, response::IntoResponse, routing::post, Json, Router};
use lettre::{
    message::{header::ContentType, Mailbox},
    Address, Message,
};
use ormx::Insert;
use serde::Deserialize;

use crate::{
    auth::password::hash_password,
    http::{
        common::ApiContext,
        error::{Error, Result},
    },
    sql::{InsertUser, InsertUserEmailVerification, SafeUser, User, UserEmailVerification},
};

mod history;
mod oauth;

pub fn router() -> Router<ApiContext> {
    Router::new()
        .nest("/oauth", oauth::router())
        .nest("/history", history::router())
        .route("/email/verify", post(request_email_verification))
        .route("/email/register", post(register))
}

#[derive(Deserialize)]
struct UserRegisterRequest {
    email: String,
    email_code: String,
    username: String,
    password: String,
}

async fn register(
    State(state): State<ApiContext>,
    Json(info): Json<UserRegisterRequest>,
) -> Result<Json<SafeUser>> {
    if User::get_by_email(&state.pool, &info.email).await.is_ok() {
        return Err(Error::Conflict(format!(
            "User with email {} already exists",
            info.email
        )));
    }

    let user_email_verification = UserEmailVerification::get_by_code(&state.pool, &info.email_code)
        .await
        .map_err(|_| Error::Forbidden("Invalid email code".to_string()))?;
    if user_email_verification.email != info.email {
        return Err(Error::Forbidden("Invalid email code".to_string()));
    }

    let password_hash = hash_password(&info.password)
        .map_err(|_| Error::BadRequest("Invalid password".to_string()))?;

    let user = InsertUser {
        email: info.email,
        username: info.username,
        password_hash: Some(password_hash),
        providers: vec![],
    }
    .insert(&state.pool)
    .await?;

    Ok(Json(SafeUser::from(user)))
}

#[derive(Deserialize)]
struct EmailVerificationRequest {
    email: String,
}

async fn request_email_verification(
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

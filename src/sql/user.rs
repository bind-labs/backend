use rand::{rng, Rng};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http::common::Origins;

/// Represents a user in the database
#[derive(Clone, Debug, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user", id = id, insertable, deletable)]
pub struct User {
    #[ormx(default)]
    pub id: i32,
    #[ormx(get_one = get_by_email)]
    pub email: String,
    #[ormx(get_one = get_by_username)]
    pub username: String,
    #[ormx(by_ref)]
    pub providers: Vec<String>,
    #[ormx(set)]
    pub password_hash: Option<String>,
    #[ormx(default, by_ref)]
    pub passwordless_pub_key: Vec<String>,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default, set)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "user_oauth_client", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum OAuthRedirectClient {
    Web,
    Android,
    IOS,
}

impl OAuthRedirectClient {
    pub fn to_uri(&self, origins: &Origins) -> Url {
        match self {
            OAuthRedirectClient::Web => origins.web.clone(),
            OAuthRedirectClient::Android => origins.android.clone(),
            OAuthRedirectClient::IOS => origins.ios.clone(),
        }
        .join("/oauth/callback")
        .unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user_oauth_state", id = id, insertable, deletable)]
pub struct UserOAuthState {
    #[ormx(default)]
    pub id: i32,
    #[ormx(custom_type)]
    pub client: OAuthRedirectClient,
    pub provider: String,
    #[ormx(get_one = get_by_csrf_token)]
    pub csrf_token: String,
    pub pkce_verifier: String,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl UserOAuthState {
    /// Deletes all user oauth states older than 1 hour
    pub async fn cleanup_expired(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM user_oauth_state WHERE created_at < $1"#,
            chrono::Utc::now() - chrono::Duration::hours(1)
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

fn generate_code() -> String {
    let mut rng = rng();
    (0..5)
        .map(|_| rng.random_range(0..=9).to_string())
        .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user_email_verification", id = id, insertable, deletable)]
pub struct UserEmailVerification {
    #[ormx(default)]
    pub id: i32,
    pub email: String,
    #[ormx(get_one = get_by_code)]
    pub code: String,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl UserEmailVerification {
    /// Deletes all expired email authentications older than 1 hour
    pub async fn cleanup_expired(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM user_email_verification WHERE created_at < $1"#,
            chrono::Utc::now() - chrono::Duration::days(1)
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub fn generate_code() -> String {
        generate_code()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user_password_reset_codes", id = id, insertable, deletable)]
pub struct PasswordVerificationCode {
    #[ormx(default)]
    pub id: i32,
    pub email: String,
    #[ormx(get_one = get_by_code)]
    pub code: String,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PasswordVerificationCode {
    /// Deletes all expired password reset codes older than 1 hour
    pub async fn cleanup_expired(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"DELETE FROM user_password_reset_codes WHERE created_at < $1"#,
            chrono::Utc::now() - chrono::Duration::days(1)
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub fn generate_code() -> String {
        generate_code()
    }
}

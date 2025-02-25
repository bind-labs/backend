use fake::{faker::internet::en::Username, Fake};
use jsonwebtoken::{decode_header, DecodingKey, Validation};
use ormx::{Insert, Table};
use serde::Deserialize;
use sqlx::PgPool;

use crate::auth::oauth::OAuth2Client;
use crate::sql::{InsertUser, User};

#[derive(Debug, Deserialize)]
struct ExternalClaims {
    email: String,
    name: Option<String>,
    given_name: Option<String>,
    preferred_username: Option<String>,
    nickname: Option<String>,
}

pub struct ExternalJwtToken {
    provider: String,
    claims: ExternalClaims,
}

impl ExternalJwtToken {
    pub fn parse(
        provider: &OAuth2Client,
        token: String,
        decoding_key: &DecodingKey,
    ) -> jsonwebtoken::errors::Result<Self> {
        let header = decode_header(&token)?;

        let mut validation = Validation::new(header.alg);
        // TODO: is the audience always going to be the client id?
        validation.set_audience(&[provider.client_id().to_string()]);

        let token_data = jsonwebtoken::decode::<ExternalClaims>(&token, decoding_key, &validation)?;

        Ok(Self {
            provider: provider.name().to_string(),
            claims: token_data.claims,
        })
    }

    fn email(&self) -> String {
        self.claims.email.clone()
    }

    fn username(&self) -> Option<String> {
        let claims = &self.claims;
        let username_from_email = self.email().split('@').next().map(|s| s.to_string());
        claims
            .preferred_username
            .as_ref()
            .or(claims.given_name.as_ref())
            .or(claims.nickname.as_ref())
            .or(claims.name.as_ref())
            .or(username_from_email.as_ref())
            .cloned()
    }

    pub async fn create_or_update_user(&self, pool: &PgPool) -> Result<User, sqlx::Error> {
        let user = User::get_by_email(pool, &self.email()).await.ok();

        // Add the provider to the user
        if let Some(mut user) = user {
            if !user.providers.contains(&self.provider) {
                user.providers.push(self.provider.clone());
                user.update(pool).await?;
            }
            return Ok(user);
        }

        // Pick a username for the new user
        let mut username = self.username().unwrap_or(Username().fake::<String>());
        let mut iteration = 0;
        while User::get_by_username(pool, &username).await.ok().is_some() {
            username = Username().fake::<String>();
            iteration += 1;
            if iteration > 10 {
                panic!("Failed to generate a unique username");
            }
        }

        // Create the user
        let user = InsertUser {
            email: self.email(),
            username,
            providers: vec![self.provider.clone()],
            password_hash: None,
        }
        .insert(pool)
        .await?;

        Ok(user)
    }
}

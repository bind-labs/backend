use fake::{faker::internet::en::Username, Fake};
use jsonwebtoken::{decode_header, DecodingKey, Validation};
use ormx::{Insert, Table};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::sql::{InsertUser, User};

#[derive(Debug, Deserialize, Serialize)]
struct ExternalClaims {
    email: String,
    name: Option<String>,
    given_name: Option<String>,
    preferred_username: Option<String>,
    nickname: Option<String>,
    exp: i64,
}

pub struct ExternalJwtToken {
    provider: String,
    claims: ExternalClaims,
}

impl ExternalJwtToken {
    pub fn parse(
        provider: String,
        client_id: String,
        token: String,
        decoding_key: &DecodingKey,
    ) -> jsonwebtoken::errors::Result<Self> {
        let header = decode_header(&token)?;

        let mut validation = Validation::new(header.alg);
        // TODO: is the audience always going to be the client id?
        validation.set_audience(&[client_id]);

        let token_data = jsonwebtoken::decode::<ExternalClaims>(&token, decoding_key, &validation)?;

        Ok(Self {
            provider,
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

#[cfg(test)]
mod tests {
    use crate::tests::TestContext;

    use super::*;
    use chrono::Utc;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use jsonwebtoken::{Algorithm, EncodingKey, Header};

    #[tokio::test]
    async fn test_parse() {
        let jwt_secret = "secret";

        let claims = ExternalClaims {
            email: SafeEmail().fake(),
            name: Some("Test Name".to_string()),
            given_name: None,
            preferred_username: None,
            nickname: None,
            exp: Utc::now().timestamp() + 1000,
        };
        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .unwrap();

        let token = ExternalJwtToken::parse(
            "test".to_string(),
            "test_client_id".to_string(),
            token,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .unwrap();
        assert_eq!(token.provider, "test");
        assert_eq!(token.claims.email, claims.email);
    }

    #[test]
    fn test_email() {
        let claims = ExternalClaims {
            email: "test@example.com".to_string(),
            name: None,
            given_name: None,
            preferred_username: None,
            nickname: None,
            exp: Utc::now().timestamp() + 1000,
        };
        let token = ExternalJwtToken {
            provider: "test_provider".to_string(),
            claims,
        };
        assert_eq!(token.email(), "test@example.com");
    }

    #[test]
    fn test_username() {
        let claims = ExternalClaims {
            email: "test@example.com".to_string(),
            name: Some("Test Name".to_string()),
            given_name: Some("Test".to_string()),
            preferred_username: Some("testuser".to_string()),
            nickname: None,
            exp: Utc::now().timestamp() + 1000,
        };
        let token = ExternalJwtToken {
            provider: "test_provider".to_string(),
            claims,
        };
        assert_eq!(token.username(), Some("testuser".to_string()));
    }

    #[tokio::test]
    async fn test_create_or_update_user() {
        let pool = TestContext::new().await.pool;

        let claims = ExternalClaims {
            email: SafeEmail().fake(),
            name: Some("Test Name".to_string()),
            given_name: None,
            preferred_username: None,
            nickname: None,
            exp: Utc::now().timestamp() + 1000,
        };
        let token = ExternalJwtToken {
            provider: "test_provider".to_string(),
            claims,
        };

        // Test user creation
        let user = token.create_or_update_user(&pool).await.unwrap();
        assert_eq!(user.email, token.email());
        assert!(user.providers.contains(&token.provider));

        // Test user update
        let updated_user = token.create_or_update_user(&pool).await.unwrap();
        assert_eq!(updated_user.email, token.email());
        assert!(updated_user.providers.contains(&token.provider));
    }
}

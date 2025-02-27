use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::AuthUser;
use crate::sql::User;

const DEFAULT_SESSION_LENGTH: chrono::Duration = chrono::Duration::weeks(1);

#[derive(Debug, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct AuthUserClaims {
    /// Expiration time as UTC timestamp (required for JWT validation)
    pub exp: i64,
    /// Issued at time as UTC timestamp
    pub iat: i64,
    /// User's unique ID
    pub id: i32,
    /// User's email address
    pub email: String,
    /// User's username
    pub username: String,
}

impl AuthUserClaims {
    pub fn new(id: i32, email: String, username: String) -> Self {
        Self {
            exp: (Utc::now() + DEFAULT_SESSION_LENGTH).timestamp(),
            iat: Utc::now().timestamp(),
            id,
            email,
            username,
        }
    }

    pub fn to_jwt(&self, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_ref()),
        )
    }

    pub fn from_jwt(jwt: &str, secret: &str) -> Result<Self, jsonwebtoken::errors::Error> {
        let decoded_token = decode::<AuthUserClaims>(
            jwt,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(jsonwebtoken::Algorithm::HS256),
        )?;
        Ok(decoded_token.claims)
    }
}

impl From<AuthUser> for AuthUserClaims {
    fn from(user: AuthUser) -> Self {
        Self::new(user.id, user.email, user.username)
    }
}

impl From<User> for AuthUserClaims {
    fn from(user: User) -> Self {
        Self::new(user.id, user.email, user.username)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::tests::dates::assert_within_second;

    use super::*;

    #[test]
    fn test_from_user() {
        let user = User {
            id: 1,
            email: "test@example.com".to_string(),
            username: "test".to_string(),
            password_hash: None,
            passwordless_pub_key: vec![],
            providers: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let claims: AuthUserClaims = user.clone().into();

        assert_eq!(claims.id, 1, "Username did not match in JWT claims");
        assert_eq!(
            claims.email, user.email,
            "Email did not match in JWT claims"
        );
        assert_eq!(
            claims.username, user.username,
            "Username did not match in JWT claims"
        );
    }

    #[test]
    fn test_from_auth_user() {
        let user = AuthUser {
            id: 1,
            email: "test@example.com".to_string(),
            username: "test".to_string(),
        };
        let claims: AuthUserClaims = user.clone().into();

        assert_eq!(claims.id, 1, "Username did not match in JWT claims");
        assert_eq!(
            claims.email, user.email,
            "Email did not match in JWT claims"
        );
        assert_eq!(
            claims.username, user.username,
            "Username did not match in JWT claims"
        );
    }

    #[test]
    fn test_to_jwt() {
        let jwt_secret = "secret";

        let claims = AuthUserClaims::new(1, "test@example.com".to_string(), "test".to_string());
        let token = claims.to_jwt(jwt_secret).unwrap();
        let decoded_claims = AuthUserClaims::from_jwt(&token, jwt_secret).unwrap();

        assert_eq!(
            claims.id, decoded_claims.id,
            "Username did not match in JWT claims"
        );
        assert_eq!(
            claims.email, decoded_claims.email,
            "Email did not match in JWT claims"
        );
        assert_eq!(
            claims.username, decoded_claims.username,
            "Username did not match in JWT claims"
        );

        assert_within_second(
            Utc::now() + DEFAULT_SESSION_LENGTH,
            Utc.timestamp_opt(claims.exp, 0).unwrap(),
        );
        assert_within_second(Utc::now(), Utc.timestamp_opt(claims.iat, 0).unwrap());
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "auth_provider", rename_all = "lowercase")]
pub enum AuthProvider {
    Google,
    Github,
    Apple,
}

impl From<&str> for AuthProvider {
    fn from(s: &str) -> Self {
        match s {
            "google" => AuthProvider::Google,
            "github" => AuthProvider::Github,
            "apple" => AuthProvider::Apple,
            _ => AuthProvider::Google,
        }
    }
}

/// Represents a user in the database
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user", id = id, insertable, deletable)]
pub struct User {
    #[ormx(default)]
    pub id: i32,
    pub email: String,
    pub username: String,
    #[ormx(custom_type, by_ref)]
    pub providers: Vec<AuthProvider>,
    pub password_hash: Option<String>,
    pub passwordless_pub_key: Option<String>,
    #[ormx(by_ref)]
    pub refresh_tokens: Vec<String>,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default, set)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

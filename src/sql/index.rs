use super::{Icon, SortOrder};
use serde::{Deserialize, Serialize};

/// Represents an index created by a user.
/// An **Index** can be thought of as a custom search over a certain set of feeds.
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserIndex {
    pub id: i32,
    pub owner: i32,

    pub query: String,
    pub sort: SortOrder,

    pub title: String,
    pub description: Option<String>,
    pub icon: Icon,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

use serde::{Deserialize, Serialize};

use super::Icon;

/// Represent a list created by a user
/// Lists are a way for users to catalogue items from feeds
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserList {
    pub id: i32,
    pub owner: i32,
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<Icon>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a single item in a user's list
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserListItem {
    pub id: i32,
    pub index: i32,
    pub owner: i32,
    /// The id of the list this item belongs to
    pub list: i32,
    /// The id of the feed item this item is referencing
    pub item: u64,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

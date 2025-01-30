use serde::{Deserialize, Serialize};

/// Represent a list created by a user
/// Lists are a way for users to catalogue items from feeds
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct List {
    pub id: u32,
    pub owner: u32,
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a single item in a user's list
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ListItem {
    pub id: u32,
    pub index: u32,
    pub owner: u32,
    /// The id of the list this item belongs to
    pub list: u32,
    /// The id of the feed item this item is referencing
    pub item: u64,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

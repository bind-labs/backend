use serde::{Deserialize, Serialize};

use super::Icon;

/// Represent a list created by a user
/// Lists are a way for users to catalogue items from feeds
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user_list", id = id, insertable, deletable)]
pub struct UserList {
    #[ormx(default)]
    pub id: i32,
    pub owner: i32,
    pub title: String,
    pub description: Option<String>,
    #[ormx(custom_type, by_ref)]
    pub icon: Option<Icon>,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a single item in a user's list
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user_list_item", id = id, insertable, deletable)]
pub struct UserListItem {
    #[ormx(default)]
    pub id: i32,
    pub index: i32,
    pub owner: i32,
    /// The id of the list this item belongs to
    pub list: i32,
    /// The id of the feed item this item is referencing
    pub item: i64,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default, set)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

use serde::{Deserialize, Serialize};

/// Represents a single item in a user's history
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user_history_item", id = id, insertable, deletable)]
pub struct HistoryItem {
    #[ormx(default)]
    pub id: i32,
    #[ormx(get_many = get_items_for_user)]
    pub owner: i32,
    /// The id of the feed item this item is referencing
    pub item: i64,
    /// Progress in the item, 0 - 1
    pub progress: f64,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default, set)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

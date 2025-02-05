use serde::{Deserialize, Serialize};

/// Represents a single item in a user's history
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow, ormx::Table)]
#[ormx(table = "user_history_item", id = id, insertable, deletable)]
pub struct HistoryItem {
    pub id: i32,
    pub owner: i32,
    /// The id of the feed item this item is referencing
    pub item: i64,
    /// Progress in the item, 0 - 1
    pub progress: f64,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

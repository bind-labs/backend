use serde::{Deserialize, Serialize};

/// Represents a single item in a user's history
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HistoryItem {
    pub id: u32,
    pub owner: u32,
    /// The id of the feed item this item is referencing
    pub item: u64,
    /// Progress in the item, 0 - 1
    pub progress: f64,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

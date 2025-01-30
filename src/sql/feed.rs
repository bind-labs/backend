use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "feed_status", rename_all = "lowercase")]
pub enum FeedStatus {
    Active,
    Completed,
    Suspended,
    Broken,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "feed_format", rename_all = "lowercase")]
pub enum FeedFormat {
    Atom,
    Rss,
    Json,
}

/// Represents a single feed in the database.
/// Note: This feed can be an RSS, Atom or JSON feed.
#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Feed {
    pub id: i32,
    pub status: FeedStatus,
    pub format: FeedFormat,
    pub link: String,
    pub domain: String,

    pub title: String,
    pub description: String,
    pub icon: Option<String>,

    pub skip_hours: Vec<i32>,
    pub skip_days_of_week: Vec<i32>,
    pub ttl_in_minutes: i32,
    pub next_update_at: chrono::DateTime<chrono::Utc>,
    pub etag: Option<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents an attach media file in a feed item (e.g. an image or audio file)
#[derive(Clone, Debug, PartialEq, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "feed_item_enclosure")]
pub struct FeedItemEnclosure {
    pub url: String,
    pub length: i32,
    pub mime_type: String,
}

/// Represent a single feed item in the database
#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct FeedItem {
    pub id: u64,
    pub feed_id: u64,
    pub guid: Option<String>,
    pub index_in_feed: u32,

    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub enclosure: Option<FeedItemEnclosure>,
    pub content: Option<String>,
    pub categories: Vec<String>,
    pub comments_link: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "feed_status", rename_all = "lowercase")]
pub enum FeedStatus {
    Active,
    Completed,
    Suspended,
    Broken,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "feed_format", rename_all = "lowercase")]
pub enum FeedFormat {
    Atom,
    Rss,
    Json,
}

impl FeedFormat {
    pub fn from_content_type(content_type: &str) -> Option<Self> {
        match content_type {
            "application/rss+xml" | "application/rss" | "text/xml" | "text/rss+xml" => {
                Some(Self::Rss)
            }
            "application/atom+xml" | "applcation/atom" | "text/atom+xml" | "text/atom" => {
                Some(Self::Atom)
            }
            "application/json" | "text/json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// Represents a single feed in the database.
/// Note: This feed can be an RSS, Atom or JSON feed.
#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize, ormx::Table)]
#[ormx(table = "feed", id = id, insertable, deletable)]
pub struct Feed {
    pub id: i32,
    #[ormx(custom_type)]
    pub status: FeedStatus,
    #[ormx(custom_type)]
    pub format: FeedFormat,
    pub link: String,
    pub domain: Option<String>,

    pub title: String,
    pub description: String,
    pub icon: Option<String>,
    #[ormx(by_ref)]
    pub skip_hours: Vec<i32>,
    #[ormx(by_ref)]
    pub skip_days_of_week: Vec<i32>,
    /// Minimum time to cache the feed for
    pub ttl_in_minutes: Option<i32>,
    /// ETag header from the last update
    pub etag: Option<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Time of the last update to the content
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Time of the last fetch
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    /// Time of the last successful fetch
    pub successful_fetch_at: chrono::DateTime<chrono::Utc>,
    /// Time to fetch the feed next
    pub next_fetch_at: chrono::DateTime<chrono::Utc>,
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
#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize, ormx::Table)]
#[ormx(table = "feed_item", id = id, insertable, deletable)]
pub struct FeedItem {
    #[ormx(default)]
    pub id: i64,
    pub feed_id: i32,
    pub guid: String,
    pub index_in_feed: i32,

    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
    #[ormx(custom_type, by_ref)]
    pub enclosure: Option<FeedItemEnclosure>,
    #[ormx(by_ref)]
    pub categories: Vec<String>,
    pub comments_link: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    pub language: String,
    pub content: Option<String>,
    pub content_type: String,
    pub base_link: Option<String>,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default, set)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

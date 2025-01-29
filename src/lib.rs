use serde::{Deserialize, Serialize};

pub mod config;
pub mod error;
pub mod feed_parsers;
pub mod html;
pub mod http;
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: config::Config,
    pub reqwest_client: reqwest::Client,
}

/// Represents an icon for a feed or a list.c
pub struct Icon {
    pub icon: String,
    pub color: String,
}

/// Represents a single feed in the database.
/// Note: This feed can be an RSS, Atom or JSON feed.
pub struct Feed {
    pub id: i32,
    pub link: String,
    pub domain: String,
    pub title: String,
    pub description: String,
    pub icon: Option<String>,

    pub skip_hours: [i32; 24],
    pub skip_days_of_week: [i32; 7],

    pub ttl_in_minutes: i32,
    pub suspended: bool,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents an attach media file in a feed item (e.g. an image or audio file).
pub struct FeedItemEnclosure {
    pub url: String,
    pub length: i32,
    pub mime_type: String,
}

/// Represent a single feed item in the database.
pub struct FeedItem {
    pub id: i32,
    pub feed_id: i32,
    pub guid: Option<String>,
    pub index_in_feed: i32,

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

pub enum SortOrder {
    RecentlyUpdated,
    AsIs,
}

impl From<&str> for SortOrder {
    fn from(s: &str) -> Self {
        match s {
            "recently_updated" => SortOrder::RecentlyUpdated,
            "as_is" => SortOrder::AsIs,
            _ => SortOrder::RecentlyUpdated,
        }
    }
}

/// Represents an index created by a user.
/// An **Index** can be thought of as a custom search over a certain set of feeds.
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

/// Represent a list created by a user.
/// Lists are a way for users to catalogue items from feeds.
pub struct UserList {
    pub id: i32,
    pub owner: i32,
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a single item in a user's list.
pub struct UserListItem {
    pub id: i32,
    pub owner: i32,
    /// The id of the list this item belongs to.
    pub list: i32,
    /// The id of the feed item this item is referencing.
    pub item: i32,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represents a single item in a user's history.
pub struct UserHistoryItem {
    pub id: i32,
    pub owner: i32,
    /// The id of the feed item this item is referencing.
    pub item: i32,
    /// Progress in the item, 0 - 100
    pub progress: i32,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

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

/// Represents a user in the database.
pub struct User {
    pub id: i32,
    pub email: String,
    pub username: String,
    pub providers: Vec<AuthProvider>,
    pub password_hash: Option<String>,
    pub passwordless_pub_key: Option<String>,
    pub refresh_tokens: Vec<String>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub enum FeedType {
    Atom,
    Rss,
    Json,
}


#[derive(Serialize, Deserialize)]
pub struct FeedInformation {
    pub link: String,
    pub feed_type: FeedType,
}

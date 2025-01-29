use serde::{Deserialize, Serialize};

/// Represents an icon for a feed or a list
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "icon")]
pub struct Icon {
    pub icon: String,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "feed_type", rename_all = "lowercase")]
pub enum FeedType {
    Atom,
    Rss,
    Json,
}

/// Represents a single feed in the database.
/// Note: This feed can be an RSS, Atom or JSON feed.
#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize)]
pub struct Feed {
    pub id: u32,
    #[sqlx(rename = "type")]
    pub type_: FeedType,
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

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sort_order")]
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
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserIndex {
    pub id: u32,
    pub owner: u32,

    pub query: String,
    pub sort: SortOrder,

    pub title: String,
    pub description: Option<String>,
    pub icon: Icon,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Represent a list created by a user
/// Lists are a way for users to catalogue items from feeds
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserList {
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
pub struct UserListItem {
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

/// Represents a single item in a user's history
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserHistoryItem {
    pub id: u32,
    pub owner: u32,
    /// The id of the feed item this item is referencing
    pub item: u64,
    /// Progress in the item, 0 - 1
    pub progress: f64,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "auth_provider", rename_all = "lowercase")]
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

/// Represents a user in the database
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: u32,
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
pub struct FeedInformation {
    pub link: String,
    pub feed_type: FeedType,
}

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use mockito::ServerGuard;

use crate::feed::{
    daemon::FeedUpdate,
    parser::{feed::ParsedFeed, feed_item::ParsedFeedItem},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, utoipa::ToSchema)]
#[sqlx(type_name = "feed_status", rename_all = "lowercase")]
/// Status of a feed
pub enum FeedStatus {
    /// Feed is active and being updated regularly
    Active,
    /// Feed is completed (e.g., a podcast series that has ended)
    Completed,
    /// Feed is temporarily suspended
    Suspended,
    /// Feed has broken links or cannot be fetched
    Broken,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, utoipa::ToSchema)]
#[sqlx(type_name = "feed_format", rename_all = "lowercase")]
/// Format of a feed
pub enum FeedFormat {
    /// Atom feed format
    Atom,
    /// RSS feed format
    Rss,
    /// JSON feed format
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
#[derive(Clone, Debug, sqlx::FromRow, Deserialize, Serialize, ormx::Table, utoipa::ToSchema)]
#[ormx(table = "feed", id = id, insertable, deletable)]
pub struct Feed {
    /// Unique identifier of the feed
    #[ormx(default)]
    pub id: i32,
    /// Current status of the feed
    #[ormx(custom_type)]
    pub status: FeedStatus,
    /// Format of the feed (RSS, Atom, JSON)
    #[ormx(custom_type)]
    pub format: FeedFormat,
    /// URL of the feed
    pub link: String,
    /// Domain of the feed (extracted from the URL)
    pub domain: Option<String>,

    /// Title of the feed
    pub title: String,
    /// Description of the feed
    pub description: String,
    /// URL to the feed's icon
    pub icon: Option<String>,
    /// Language of the feed (e.g., "en-us")
    pub language: Option<String>,

    /// Hours when the feed should not be fetched
    #[ormx(by_ref)]
    #[schema(value_type = Vec<i32>)]
    pub skip_hours: Vec<i32>,
    /// Days of the week when the feed should not be fetched
    #[ormx(by_ref)]
    #[schema(value_type = Vec<i32>)]
    pub skip_days_of_week: Vec<i32>,
    /// Minimum time to cache the feed for
    pub ttl_in_minutes: Option<i32>,
    /// ETag header from the last update
    pub etag: Option<String>,

    /// When the feed was created in the system
    #[schema(format = "date-time")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Time of the last update to the content
    #[schema(format = "date-time")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Time of the last fetch
    #[schema(format = "date-time")]
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    /// Time of the last successful fetch
    #[schema(format = "date-time")]
    pub successful_fetch_at: chrono::DateTime<chrono::Utc>,
    /// Time to fetch the feed next
    #[schema(format = "date-time")]
    pub next_fetch_at: chrono::DateTime<chrono::Utc>,
}

impl Feed {
    pub async fn get_out_of_date(pool: &sqlx::PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"
            SELECT
                id,
                status AS "status: FeedStatus",
                format AS "format: FeedFormat",
                link,
                domain,

                title,
                description,
                icon,
                language,

                skip_hours,
                skip_days_of_week,
                ttl_in_minutes,
                etag,

                created_at,
                updated_at,
                fetched_at,
                successful_fetch_at,
                next_fetch_at
            FROM feed
            WHERE next_fetch_at < NOW() AND status = 'active'"#,
        )
        .fetch_all(pool)
        .await
    }

    pub fn merge_with_update(&mut self, update: &FeedUpdate) {
        self.status = update.status.unwrap_or(self.status);
        self.format = update.format.unwrap_or(self.format);
        self.link = update.link.clone().unwrap_or(self.link.clone());
        self.domain = update.domain.clone().or(self.domain.clone());

        self.title = update.title.clone().unwrap_or(self.title.clone());
        self.description = update
            .description
            .clone()
            .unwrap_or(self.description.clone());
        self.icon = update.icon.clone().or(self.icon.clone());

        self.skip_hours = update.skip_hours.clone().unwrap_or(self.skip_hours.clone());
        self.skip_days_of_week = update
            .skip_days_of_week
            .clone()
            .unwrap_or(self.skip_days_of_week.clone());
        self.ttl_in_minutes = update.ttl_in_minutes.or(self.ttl_in_minutes);
        self.etag = update.etag.clone().or(self.etag.clone());

        self.fetched_at = update.fetched_at.unwrap_or(self.fetched_at);
        self.successful_fetch_at = update
            .successful_fetch_at
            .unwrap_or(self.successful_fetch_at);
        self.next_fetch_at = update.next_fetch_at.unwrap_or(self.next_fetch_at);
    }
}

impl InsertFeed {
    #[cfg(test)]
    pub fn from_mockito(server: &ServerGuard, date: chrono::DateTime<chrono::Utc>) -> Self {
        InsertFeed {
            status: FeedStatus::Active,
            format: FeedFormat::Rss,
            link: server.url().to_string(),
            domain: Some(server.host_with_port().to_string()),
            title: "Hello World".to_string(),
            description: "This is a test feed".to_string(),
            icon: None,
            language: None,

            skip_hours: Vec::new(),
            skip_days_of_week: Vec::new(),
            ttl_in_minutes: None,
            etag: None,

            created_at: date,
            updated_at: date,
            fetched_at: date,
            successful_fetch_at: date,
            next_fetch_at: date,
        }
    }
}

/// Represents an attached media file in a feed item (e.g. an image or audio file)
#[derive(Clone, Debug, PartialEq, sqlx::Type, Deserialize, Serialize, utoipa::ToSchema)]
#[sqlx(type_name = "feed_item_enclosure")]
pub struct FeedItemEnclosure {
    /// URL of the media file
    pub url: String,
    /// Size of the media file in bytes
    pub length: i32,
    /// MIME type of the media file
    pub mime_type: String,
}

/// Represent a single feed item in the database
#[derive(Clone, Debug, PartialEq, sqlx::FromRow, Deserialize, Serialize, ormx::Table, utoipa::ToSchema)]
#[ormx(table = "feed_item", id = id, insertable, deletable)]
pub struct FeedItem {
    /// Unique identifier of the feed item
    #[ormx(default)]
    pub id: i64,
    /// Globally unique identifier of the item (from the feed)
    pub guid: String,
    /// ID of the feed this item belongs to
    #[ormx(get_many = get_by_feed)]
    pub feed_id: i32,
    /// Position of the item in the feed
    pub index_in_feed: i32,

    /// Title of the feed item
    pub title: String,
    /// Link to the full article
    pub link: Option<String>,
    /// Summary or description of the item
    pub description: Option<String>,
    /// Attached media file (if any)
    #[ormx(custom_type, by_ref)]
    pub enclosure: Option<FeedItemEnclosure>,
    /// Categories or tags for the item
    #[ormx(by_ref)]
    #[schema(value_type = Vec<String>)]
    pub categories: Vec<String>,
    /// Link to the comments section
    pub comments_link: Option<String>,
    /// When the item was published
    #[schema(format = "date-time")]
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Full content of the item
    pub content: Option<String>,
    /// MIME type of the content
    pub content_type: Option<String>,
    /// Base URL for relative links in the content
    pub base_link: Option<String>,
    /// When the item was created in the system
    #[schema(format = "date-time")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// When the item was last updated
    #[schema(format = "date-time")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl FeedItem {
    pub fn merge_with_parsed(&mut self, parsed: &ParsedFeedItem) {
        self.title = parsed.title.clone();
        self.link = parsed.link.clone().or(self.link.clone());
        self.description = parsed.description.clone().or(self.description.clone());
        self.enclosure = parsed.enclosure.clone().or(self.enclosure.clone());
        self.categories = parsed.categories.clone();
        self.comments_link = parsed.comments_link.clone().or(self.comments_link.clone());
        self.published_at = parsed.published_at.or(self.published_at);
        self.content = parsed.content.clone();
    }
}

impl InsertFeedItem {
    pub fn from_parsed(
        item: &ParsedFeedItem,
        feed_id: i32,
        index_in_feed: i32,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            feed_id,
            index_in_feed,
            guid: item.guid.clone(),
            title: item.title.clone(),
            link: item.link.clone(),
            description: item.description.clone(),
            enclosure: item.enclosure.clone(),
            categories: item.categories.clone(),
            comments_link: item.comments_link.clone(),
            published_at: item.published_at,
            content: item.content.clone(),
            content_type: None,
            base_link: None,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<ParsedFeed> for InsertFeed {
    fn from(parsed_feed: ParsedFeed) -> Self {
        InsertFeed {
            status: FeedStatus::Active,
            format: parsed_feed.format,
            link: parsed_feed.link,
            domain: parsed_feed.domain,

            title: parsed_feed.title,
            description: parsed_feed.description,
            icon: parsed_feed.icon,
            language: None,

            skip_hours: parsed_feed.skip_hours,
            skip_days_of_week: parsed_feed.skip_days_of_week,
            ttl_in_minutes: Some(parsed_feed.ttl_in_minutes),
            etag: None,

            created_at: Utc::now(),
            updated_at: Utc::now(),
            fetched_at: Utc::now(),
            successful_fetch_at: Utc::now(),
            next_fetch_at: Utc::now(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, sqlx::FromRow, Deserialize, Serialize, ormx::Table, utoipa::ToSchema)]
#[ormx(table = "feed_item_parsed", id = id, insertable, deletable)]
pub struct FeedItemParsed {
    #[ormx(default)]
    pub id: i64,
    #[ormx(get_optional = get_by_feed_item)]
    pub feed_item_id: i64,
    pub content: String,
    pub content_type: String,
    #[ormx(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ormx(default)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
